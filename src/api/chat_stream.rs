use axum::Json;
use axum::extract::State;
use axum::response::sse::{Event, KeepAlive, Sse};
use futures::StreamExt;
use rig_core::agent::MultiTurnStreamItem;
use rig_core::streaming::StreamingPrompt;
use serde::Deserialize;
use std::convert::Infallible;

use crate::AppState;
use crate::error::AppError;

/// 流式对话请求
#[derive(Debug, Deserialize)]
pub struct ChatStreamRequest {
    /// 会话 ID
    pub id: String,
    /// 用户问题
    pub question: String,
}

/// POST /api/chat_stream — SSE 流式对话
pub async fn chat_stream_handler(
    State(state): State<AppState>,
    Json(req): Json<ChatStreamRequest>,
) -> Result<Sse<impl futures::Stream<Item = Result<Event, Infallible>>>, AppError> {
    tracing::info!(session = %req.id, question = %req.question, "处理流式对话请求");

    // stream_prompt 返回 StreamingPromptRequest，.await (IntoFuture) 得到流
    let stream = state
        .chat_agent
        .stream_prompt(&req.question)
        .conversation(&req.id)
        .await;

    // 将流中的错误转换为 SSE error 事件（不中断整个流）
    let sse_stream = stream.map(|item| match item {
        Ok(MultiTurnStreamItem::StreamAssistantItem(
            rig_core::streaming::StreamedAssistantContent::Text(text),
        )) => Ok(Event::default().data(text.text)),
        Ok(MultiTurnStreamItem::StreamAssistantItem(
            rig_core::streaming::StreamedAssistantContent::ToolCall { tool_call, .. },
        )) => {
            let data = serde_json::to_string(&tool_call).unwrap_or_default();
            Ok(Event::default().event("tool_call").data(data))
        }
        Ok(MultiTurnStreamItem::StreamAssistantItem(
            rig_core::streaming::StreamedAssistantContent::Reasoning(reasoning),
        )) => Ok(Event::default()
            .event("reasoning")
            .data(reasoning.display_text())),
        Ok(MultiTurnStreamItem::StreamAssistantItem(
            rig_core::streaming::StreamedAssistantContent::Final(_),
        )) => Ok(Event::default().event("done").data("[DONE]")),
        Ok(MultiTurnStreamItem::FinalResponse(_)) => {
            Ok(Event::default().event("done").data("[DONE]"))
        }
        Ok(_) => Ok(Event::default()),
        Err(e) => Ok(Event::default().event("error").data(e.to_string())),
    });

    Ok(Sse::new(sse_stream).keep_alive(KeepAlive::default()))
}

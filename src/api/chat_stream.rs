use axum::Json;
use axum::extract::State;
use axum::response::sse::{Event, Sse};
use rig_core::streaming::StreamingPrompt;
use serde::Deserialize;
use std::convert::Infallible;

use devops_agent_server::sse::rig_stream_to_sse;

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

    // 使用框架 crate 的通用 SSE 转换（消除与 sse.rs 的重复）
    Ok(rig_stream_to_sse(stream))
}

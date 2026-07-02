//! SSE 转换工具 — rig Stream → axum SSE
//!
//! 提取自 src/sse.rs，消除与 chat_stream.rs 的重复逻辑。

use axum::response::sse::{Event, KeepAlive, Sse};
use futures::stream::Stream;
use futures::StreamExt;
use rig_core::agent::MultiTurnStreamItem;
use rig_core::streaming::StreamedAssistantContent;
use std::convert::Infallible;

/// 将 rig 的流式响应转换为 axum SSE 事件流
///
/// 统一处理所有流式事件类型：
/// - Text → 默认 SSE data 事件
/// - ToolCall → "tool_call" 事件
/// - Reasoning → "reasoning" 事件
/// - Final / FinalResponse → "done" 事件
/// - Error → "error" 事件
pub fn rig_stream_to_sse<S, R>(
    stream: S,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>>
where
    S: Stream<Item = Result<MultiTurnStreamItem<R>, rig_core::agent::StreamingError>>
        + Send
        + 'static,
    R: Clone + Unpin + Send + 'static,
{
    let sse_stream = stream.map(|item| match item {
        Ok(MultiTurnStreamItem::StreamAssistantItem(
            StreamedAssistantContent::Text(text),
        )) => Ok(Event::default().data(text.text)),

        Ok(MultiTurnStreamItem::StreamAssistantItem(
            StreamedAssistantContent::ToolCall { tool_call, .. },
        )) => {
            let data = serde_json::to_string(&tool_call).unwrap_or_default();
            Ok(Event::default().event("tool_call").data(data))
        }

        Ok(MultiTurnStreamItem::StreamAssistantItem(
            StreamedAssistantContent::Reasoning(reasoning),
        )) => Ok(Event::default().event("reasoning").data(reasoning.display_text())),

        Ok(MultiTurnStreamItem::StreamAssistantItem(
            StreamedAssistantContent::Final(_),
        )) => Ok(Event::default().event("done").data("[DONE]")),

        Ok(MultiTurnStreamItem::FinalResponse(_)) => {
            Ok(Event::default().event("done").data("[DONE]"))
        }

        Ok(_) => Ok(Event::default()),

        Err(e) => Ok(Event::default().event("error").data(e.to_string())),
    });

    Sse::new(sse_stream).keep_alive(KeepAlive::default())
}

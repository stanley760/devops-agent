use axum::extract::State;
use axum::Json;
use rig_core::completion::Prompt;
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::AppState;

/// 同步对话请求
#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    /// 会话 ID（用于保持对话历史）
    pub id: String,
    /// 用户问题
    pub question: String,
}

/// 同步对话响应
#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub answer: String,
}

/// POST /api/chat — 同步对话
pub async fn chat_handler(
    State(state): State<AppState>,
    Json(req): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, AppError> {
    tracing::info!(session = %req.id, question = %req.question, "处理同步对话请求");

    let answer = state
        .chat_agent
        .prompt(&req.question)
        .conversation(&req.id)
        .await
        .map_err(AppError::Prompt)?;

    tracing::info!(session = %req.id, "同步对话完成");
    Ok(Json(ChatResponse { answer }))
}

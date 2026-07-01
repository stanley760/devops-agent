use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

/// 统一应用错误类型
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("配置错误: {0}")]
    Config(#[from] anyhow::Error),

    #[error("LLM 调用失败: {0}")]
    Llm(#[from] rig_core::completion::CompletionError),

    #[error("Prompt 执行失败: {0}")]
    Prompt(#[from] rig_core::completion::PromptError),

    #[error("向量存储错误: {0}")]
    VectorStore(#[from] rig_core::vector_store::VectorStoreError),

    #[error("工具执行错误: {0}")]
    Tool(#[from] rig_core::tool::ToolError),

    #[error("HTTP 请求失败: {0}")]
    Http(#[from] reqwest::Error),

    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),

    #[error("序列化错误: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("内部错误: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Config(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::Llm(_)
            | AppError::Prompt(_)
            | AppError::VectorStore(_)
            | AppError::Tool(_) => (StatusCode::BAD_GATEWAY, self.to_string()),
            AppError::Http(_) => (StatusCode::BAD_GATEWAY, self.to_string()),
            AppError::Database(_) => (StatusCode::SERVICE_UNAVAILABLE, self.to_string()),
            AppError::Json(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::Io(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = serde_json::json!({
            "message": "Error",
            "data": {
                "error": message,
            }
        });

        (status, axum::Json(body)).into_response()
    }
}

/// 方便的 Result 类型别名
pub type AppResult<T> = Result<T, AppError>;

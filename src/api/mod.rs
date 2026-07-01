pub mod chat;
pub mod chat_stream;
pub mod upload;
pub mod ai_ops;

use axum::Router;
use axum::routing::post;
use tower_http::cors::{CorsLayer, Any};
use crate::api::chat::chat_handler;
use crate::api::chat_stream::chat_stream_handler;
use crate::api::upload::upload_handler;
use crate::api::ai_ops::ai_ops_handler;
use crate::AppState;

/// 构建 API Router，包含所有端点和中间件
pub fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/api/chat", post(chat_handler))
        .route("/api/chat_stream", post(chat_stream_handler))
        .route("/api/upload", post(upload_handler))
        .route("/api/ai_ops", post(ai_ops_handler))
        .layer(cors)
        .with_state(state)
}

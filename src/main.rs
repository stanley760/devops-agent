mod agent;
mod api;
mod config;
mod error;
mod hooks;
mod mcp;
mod plan_types;
mod tools;
mod vector_store;

use config::AppConfig;
use rig_core::client::{CompletionClient, EmbeddingsClient};
use rig_core::providers;
use rig_milvus::MilvusVectorStore;
use std::sync::Arc;

/// 应用共享状态 — 通过 axum State 在 handler 间共享
#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub chat_agent: rig_core::agent::Agent<providers::deepseek::CompletionModel>,
    pub embedding_model: rig_core::providers::openai::EmbeddingModel,
    pub milvus_store: Arc<MilvusVectorStore<rig_core::providers::openai::EmbeddingModel>>,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // 加载 .env 文件（如果存在）
    dotenvy::dotenv().ok();

    // 初始化 tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "devOpsAgent=info,rig=info".into()),
        )
        .init();

    tracing::info!("devOpsAgent 启动中...");

    // 加载配置
    let config = AppConfig::from_env()?;
    tracing::info!(
        volcengine_url = %config.volcengine_base_url,
        milvus_url = %config.milvus_url,
        prometheus_url = %config.prometheus_url,
        server_port = config.server_port,
        "配置加载完成"
    );

    // 构建 DeepSeek 客户端 (Volcengine)
    let ds_client = providers::deepseek::Client::builder()
        .api_key(&config.volcengine_api_key)
        .base_url(&config.volcengine_base_url)
        .build()?;

    let quick_model = ds_client.completion_model(&config.deepseek_quick_model);

    // 构建 DashScope Embedding 客户端
    let ds_embedding_client = vector_store::embedding::create_embedding_client(
        &config.dashscope_api_key,
        &config.dashscope_base_url,
    );

    let embedding_model = ds_embedding_client.embedding_model(&config.dashscope_embedding_model);

    // 构建 Milvus 向量存储 — 创建两个实例：
    // 一个用于 Chat Agent 的 dynamic_context（被 Agent 消费），
    // 一个用于文件上传的 insert_documents（放在 AppState 中共享）
    let milvus_for_agent = vector_store::milvus_store::create_milvus_store(
        embedding_model.clone(),
        &config.milvus_url,
        &config.milvus_database,
        &config.milvus_collection,
        config.milvus_username.clone(),
        config.milvus_password.clone(),
    );

    let milvus_for_upload = vector_store::milvus_store::create_milvus_store(
        embedding_model.clone(),
        &config.milvus_url,
        &config.milvus_database,
        &config.milvus_collection,
        config.milvus_username.clone(),
        config.milvus_password.clone(),
    );

    // 构建 Chat Agent
    let chat_agent = agent::chat_agent::build_chat_agent(
        quick_model,
        milvus_for_agent,
        config.prometheus_url.clone(),
    );

    // 可选：连接 MCP 服务
    if let Some(ref mcp_url) = config.mcp_url {
        tracing::info!(url = %mcp_url, "尝试连接 MCP 服务...");
        let mcp_client = mcp::cls::ClsMcpClient::new(mcp_url.clone());
        match mcp_client.connect(mcp_url).await {
            Ok(_) => tracing::info!("MCP 服务连接成功"),
            Err(e) => tracing::warn!(error = %e, "MCP 服务连接失败，继续启动"),
        }
    }

    // 构建应用状态
    let state = AppState {
        config: config.clone(),
        chat_agent,
        embedding_model,
        milvus_store: Arc::new(milvus_for_upload),
    };

    // 构建 axum Router
    let app = api::build_router(state);

    // 启动 HTTP 服务器
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], config.server_port));
    tracing::info!(addr = %addr, "devOpsAgent 服务器启动");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("devOpsAgent 已停止");
    Ok(())
}

/// 优雅关闭信号
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("收到 Ctrl+C 信号，开始优雅关闭...");
        },
        _ = terminate => {
            tracing::info!("收到 SIGTERM 信号，开始优雅关闭...");
        },
    }
}

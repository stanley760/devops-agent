use rig_core::providers;

/// 创建 DashScope embedding 模型（OpenAI 兼容 API）
pub fn create_embedding_client(
    api_key: &str,
    base_url: &str,
) -> providers::openai::Client {
    providers::openai::Client::builder()
        .api_key(api_key)
        .base_url(base_url)
        .build()
        .expect("DashScope embedding client 构建失败")
}

use serde::Deserialize;

/// 应用配置，从环境变量或 .env 文件加载
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    // Volcengine DeepSeek
    pub volcengine_api_key: String,
    pub volcengine_base_url: String,
    pub deepseek_think_model: String,
    pub deepseek_quick_model: String,

    // DashScope Embedding
    pub dashscope_api_key: String,
    pub dashscope_base_url: String,
    pub dashscope_embedding_model: String,

    // Milvus
    pub milvus_url: String,
    pub milvus_database: String,
    pub milvus_collection: String,
    pub milvus_username: Option<String>,
    pub milvus_password: Option<String>,

    // Prometheus
    pub prometheus_url: String,

    // MySQL
    pub mysql_dsn: Option<String>,

    // MCP
    pub mcp_url: Option<String>,

    // 文件上传目录
    pub file_dir: String,

    // 服务器
    pub server_port: u16,
}

impl AppConfig {
    /// 从环境变量加载配置，缺失的必填项返回错误
    pub fn from_env() -> Result<Self, anyhow::Error> {
        Ok(Self {
            volcengine_api_key: env_required("VOLCENGINE_API_KEY")?,
            volcengine_base_url: env_optional_or("VOLCENGINE_BASE_URL", "https://ark.cn-beijing.volces.com/api/v3".to_string()),
            deepseek_think_model: env_optional_or("DEEPSEEK_THINK_MODEL", "deepseek-v3-241226".to_string()),
            deepseek_quick_model: env_optional_or("DEEPSEEK_QUICK_MODEL", "deepseek-v3-241226".to_string()),
            dashscope_api_key: env_required("DASHSCOPE_API_KEY")?,
            dashscope_base_url: env_optional_or("DASHSCOPE_BASE_URL", "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string()),
            dashscope_embedding_model: env_optional_or("DASHSCOPE_EMBEDDING_MODEL", "text-embedding-v4".to_string()),
            milvus_url: env_optional_or("MILVUS_URL", "http://localhost:19530".to_string()),
            milvus_database: env_optional_or("MILVUS_DATABASE", "agent".to_string()),
            milvus_collection: env_optional_or("MILVUS_COLLECTION", "biz".to_string()),
            milvus_username: env_optional("MILVUS_USERNAME"),
            milvus_password: env_optional("MILVUS_PASSWORD"),
            prometheus_url: env_optional_or("PROMETHEUS_URL", "http://127.0.0.1:9090".to_string()),
            mysql_dsn: env_optional("MYSQL_DSN"),
            mcp_url: env_optional("MCP_URL"),
            file_dir: env_optional_or("FILE_DIR", "./uploads".to_string()),
            server_port: env_optional_or_parse("SERVER_PORT", 6872),
        })
    }
}

fn env_required(key: &str) -> Result<String, anyhow::Error> {
    std::env::var(key).map_err(|_| anyhow::anyhow!("环境变量 {} 未设置", key))
}

fn env_optional(key: &str) -> Option<String> {
    std::env::var(key).ok()
}

fn env_optional_or(key: &str, default: String) -> String {
    std::env::var(key).unwrap_or(default)
}

fn env_optional_or_parse<T: std::str::FromStr>(key: &str, default: T) -> T {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

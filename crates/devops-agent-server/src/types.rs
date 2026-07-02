//! 通用请求/响应类型
//!
//! 提供常用的 HTTP API 请求和响应结构体，
//! 业务代码可直接使用或扩展。

use serde::{Deserialize, Serialize};

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

/// 流式对话请求
#[derive(Debug, Deserialize)]
pub struct ChatStreamRequest {
    /// 会话 ID
    pub id: String,
    /// 用户问题
    pub question: String,
}

/// Agent 运行请求（如 AIOps）
#[derive(Debug, Deserialize)]
pub struct AgentRunRequest {
    /// 输入内容
    pub input: String,
    /// 最大迭代次数
    #[serde(default)]
    pub max_iterations: Option<usize>,
}

/// Agent 运行响应
#[derive(Debug, Serialize)]
pub struct AgentRunResponse {
    /// 分析结论
    pub result: String,
    /// 详细步骤
    pub details: Vec<String>,
}

/// 文件上传响应
#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub file_name: String,
    pub file_path: String,
    pub file_size: u64,
}

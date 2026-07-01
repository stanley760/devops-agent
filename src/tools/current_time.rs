use rig_core::completion::ToolDefinition;
use rig_core::tool::Tool;
use serde::{Deserialize, Serialize};

/// 当前时间工具参数（无参数）
#[derive(Deserialize)]
pub struct CurrentTimeArgs;

/// 当前时间工具错误
#[derive(Debug, thiserror::Error)]
pub enum CurrentTimeError {
    #[error("时间格式化错误: {0}")]
    Format(#[from] std::fmt::Error),
}

/// 当前时间工具 — 返回多种格式的当前时间
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentTimeTool;

impl Tool for CurrentTimeTool {
    const NAME: &'static str = "get_current_time";
    type Error = CurrentTimeError;
    type Args = CurrentTimeArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "获取当前时间，返回多种时间格式（Unix秒/毫秒/微秒、ISO8601、可读格式）。当需要查询或比较时间时使用此工具。".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        let now = chrono::Utc::now();
        let result = serde_json::json!({
            "unix_seconds": now.timestamp(),
            "unix_milliseconds": now.timestamp_millis(),
            "unix_microseconds": now.timestamp_micros(),
            "iso8601": now.to_rfc3339(),
            "readable": now.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        });
        Ok(serde_json::to_string_pretty(&result).unwrap_or_default())
    }
}

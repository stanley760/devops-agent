//! 腾讯云 CLS MCP 客户端
//!
//! 使用 devops-agent-mcp 框架 crate 的 McpConnector 通用连接器，
//! 业务特定的参数（client_name, version）保留在此。

use devops_agent_mcp::client::McpConnector;

/// 腾讯云 CLS MCP 客户端
pub struct ClsMcpClient {
    connector: McpConnector,
}

impl ClsMcpClient {
    /// 创建新的 MCP 客户端
    pub fn new(_mcp_url: String) -> Self {
        Self {
            connector: McpConnector::new(),
        }
    }

    /// 异步连接 MCP 服务端并注册工具
    pub async fn connect(&self, mcp_url: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.connector
            .connect(mcp_url, "devOpsAgent", "0.1.0")
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        tracing::info!(url = %mcp_url, "MCP 客户端连接成功");
        Ok(())
    }

    /// 获取 ToolServerHandle（可传给 AgentBuilder）
    pub fn handle(&self) -> rig_core::tool::server::ToolServerHandle {
        self.connector.handle()
    }
}

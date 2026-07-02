//! 通用 MCP 连接器
//!
//! 泛化自腾讯 CLS MCP 客户端，提供通用的 MCP 服务连接能力。
//! 不绑定特定 MCP 服务，业务代码负责传入 URL 和客户端信息。

use rig_core::tool::rmcp::McpClientHandler;
use rig_core::tool::server::{ToolServer, ToolServerHandle};
use rmcp::model::{ClientCapabilities, ClientInfo, Implementation};
use rmcp::transport::streamable_http_client::StreamableHttpClientTransportConfig;

/// MCP 连接错误
#[derive(Debug, thiserror::Error)]
pub enum McpError {
    #[error("MCP 连接失败: {0}")]
    Connection(String),

    #[error("MCP 工具发现失败: {0}")]
    ToolDiscovery(String),
}

/// 通用 MCP 连接器
///
/// 通过 rig 的 rmcp 功能连接任意 MCP 服务端，自动发现并注册工具。
///
/// # Example
/// ```no_run
/// use devops_agent_mcp::client::McpConnector;
///
/// let connector = McpConnector::new();
/// connector.connect("http://localhost:8080/mcp", "my-agent", "0.1.0").await?;
/// let handle = connector.handle();
/// // handle 可传给 AgentBuilder.tool_server_handle(handle)
/// ```
pub struct McpConnector {
    tool_server_handle: ToolServerHandle,
}

impl McpConnector {
    /// 创建新的 MCP 连接器
    pub fn new() -> Self {
        let tool_server = ToolServer::new();
        let handle = tool_server.run();
        Self {
            tool_server_handle: handle,
        }
    }

    /// 连接到 MCP 服务端（Streamable HTTP transport）
    ///
    /// # Arguments
    /// * `url` - MCP 服务端 URL
    /// * `client_name` - 客户端名称（标识自己）
    /// * `client_version` - 客户端版本
    pub async fn connect(
        &self,
        url: &str,
        client_name: &str,
        client_version: &str,
    ) -> Result<(), McpError> {
        let client_info = ClientInfo::new(
            ClientCapabilities::default(),
            Implementation::new(client_name, client_version),
        );

        let handler = McpClientHandler::new(client_info, self.tool_server_handle.clone());

        let config = StreamableHttpClientTransportConfig::with_uri(url);
        let http_client = reqwest::Client::new();
        let transport =
            rmcp::transport::StreamableHttpClientTransport::with_client(http_client, config);

        handler
            .connect(transport)
            .await
            .map_err(|e| McpError::Connection(e.to_string()))?;

        tracing::info!(url = %url, "MCP 客户端连接成功");

        Ok(())
    }

    /// 获取 ToolServerHandle（可传给 AgentBuilder）
    pub fn handle(&self) -> ToolServerHandle {
        self.tool_server_handle.clone()
    }
}

impl Default for McpConnector {
    fn default() -> Self {
        Self::new()
    }
}

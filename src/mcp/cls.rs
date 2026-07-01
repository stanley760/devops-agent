/// 腾讯云 CLS MCP 客户端
///
/// 通过 rig 的 rmcp 功能连接腾讯云 CLS MCP 服务端，
/// 自动发现并注册 MCP 工具到 ToolServer。
use rig_core::tool::server::{ToolServer, ToolServerHandle};

/// MCP 客户端配置
pub struct ClsMcpClient {
    pub mcp_url: String,
    pub tool_server_handle: ToolServerHandle,
}

impl ClsMcpClient {
    /// 创建新的 MCP 客户端
    pub fn new(mcp_url: String) -> Self {
        let tool_server = ToolServer::new();
        let handle = tool_server.run();
        Self {
            mcp_url,
            tool_server_handle: handle,
        }
    }

    /// 异步连接 MCP 服务端并注册工具
    pub async fn connect(&self) -> Result<(), Box<dyn std::error::Error>> {
        use rig_core::tool::rmcp::McpClientHandler;
        use rmcp::model::{ClientCapabilities, ClientInfo, Implementation};
        use rmcp::transport::streamable_http_client::StreamableHttpClientTransportConfig;

        let client_info = ClientInfo::new(
            ClientCapabilities::default(),
            Implementation::new("devOpsAgent", "0.1.0"),
        );

        let handler = McpClientHandler::new(client_info, self.tool_server_handle.clone());

        // 使用 Streamable HTTP transport 连接 MCP 服务端
        let config = StreamableHttpClientTransportConfig::with_uri(self.mcp_url.as_str());
        let http_client = reqwest::Client::new();
        let transport = rmcp::transport::StreamableHttpClientTransport::with_client(http_client, config);

        // 启动连接
        let _service = handler.connect(transport).await?;

        tracing::info!(url = %self.mcp_url, "MCP 客户端连接成功");

        Ok(())
    }
}

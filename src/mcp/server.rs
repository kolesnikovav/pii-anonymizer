use tracing::info;

#[derive(Debug, Clone)]
pub struct McpServer;

impl McpServer {
    pub fn new() -> Self {
        info!("MCP Server инициализирован");
        Self
    }

    pub async fn start(&self) {
        info!("MCP Server запущен");
    }
}

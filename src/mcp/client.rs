use tracing::info;

#[derive(Debug, Clone)]
pub struct McpClient;

impl McpClient {
    pub fn new() -> Self {
        info!("MCP Client инициализирован");
        Self
    }
}

use tracing::info;

#[derive(Debug, Clone)]
pub struct McpProxy;

impl McpProxy {
    pub fn new() -> Self {
        info!("MCP Proxy инициализирован");
        Self
    }
}

use std::sync::Arc;
use tokio::sync::RwLock;
use serde_json::{json, Value};
use tracing::{info, error, warn};

use crate::config::UpstreamServer;
use crate::mcp::client::McpClient;

/// Прокси для управления несколькими upstream MCP серверами
pub struct McpProxy {
    clients: Arc<RwLock<Vec<(UpstreamServer, McpClient)>>>,
}

impl McpProxy {
    pub fn new() -> Self {
        info!("🔄 MCP Proxy инициализирован");
        
        Self {
            clients: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Добавление upstream сервера
    pub async fn add_upstream(&self, upstream: UpstreamServer) {
        let client = McpClient::new(upstream.clone());
        let mut clients = self.clients.write().await;
        clients.push((upstream, client));
        info!("➕ Добавлен upstream сервер");
    }

    /// Удаление upstream сервера
    pub async fn remove_upstream(&self, name: &str) {
        let mut clients = self.clients.write().await;
        clients.retain(|(upstream, _)| upstream.name != name);
        info!("➖ Удален upstream сервер: {}", name);
    }

    /// Проксирование вызова инструмента ко всем upstream серверам
    pub async fn proxy_tool_call(&self, tool_name: &str, arguments: Value) -> Vec<(String, Result<Value, String>)> {
        info!("📡 Проксирование вызова {} ко всем upstream серверам", tool_name);
        
        let clients = self.clients.read().await;
        let mut results = Vec::new();

        for (upstream, client) in clients.iter() {
            let result = client.call_tool(tool_name, arguments.clone()).await;
            results.push((upstream.name.clone(), result));
        }

        results
    }

    /// Получение списка всех инструментов от всех upstream серверов
    pub async fn list_all_tools(&self) -> Vec<(String, Result<Value, String>)> {
        let clients = self.clients.read().await;
        let mut results = Vec::new();

        for (upstream, client) in clients.iter() {
            let result = client.list_tools().await;
            results.push((upstream.name.clone(), result));
        }

        results
    }

    /// Инициализация всех upstream соединений
    pub async fn initialize_all(&self) -> Vec<(String, Result<Value, String>)> {
        info!("🔌 Инициализация всех upstream соединений");
        
        let clients = self.clients.read().await;
        let mut results = Vec::new();

        for (upstream, client) in clients.iter() {
            let result = client.initialize().await;
            results.push((upstream.name.clone(), result));
        }

        results
    }

    /// Проверка здоровья всех upstream серверов
    pub async fn health_check_all(&self) -> Vec<(String, bool)> {
        let clients = self.clients.read().await;
        let mut results = Vec::new();

        for (upstream, client) in clients.iter() {
            let is_healthy = client.health_check().await;
            results.push((upstream.name.clone(), is_healthy));
        }

        results
    }

    /// Количество upstream серверов
    pub async fn upstream_count(&self) -> usize {
        self.clients.read().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_proxy_creation() {
        let proxy = McpProxy::new();
        assert_eq!(proxy.upstream_count().await, 0);
    }

    #[tokio::test]
    async fn test_add_upstream() {
        let proxy = McpProxy::new();
        let upstream = UpstreamServer {
            name: "test".to_string(),
            url: "http://localhost:8080".to_string(),
            timeout: 30,
        };
        
        proxy.add_upstream(upstream).await;
        assert_eq!(proxy.upstream_count().await, 1);
    }

    #[tokio::test]
    async fn test_remove_upstream() {
        let proxy = McpProxy::new();
        let upstream = UpstreamServer {
            name: "test".to_string(),
            url: "http://localhost:8080".to_string(),
            timeout: 30,
        };
        
        proxy.add_upstream(upstream.clone()).await;
        assert_eq!(proxy.upstream_count().await, 1);
        
        proxy.remove_upstream("test").await;
        assert_eq!(proxy.upstream_count().await, 0);
    }
}

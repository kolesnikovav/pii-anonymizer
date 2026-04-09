use reqwest::Client;
use serde_json::{json, Value};
use tracing::{info, error, warn};

use crate::config::UpstreamServer;

/// MCP клиент для проксирования вызовов к upstream серверам
#[derive(Clone)]
pub struct McpClient {
    http_client: Client,
    upstream: UpstreamServer,
}

impl McpClient {
    pub fn new(upstream: UpstreamServer) -> Self {
        info!("🔗 MCP Client инициализирован для upstream: {}", upstream.name);
        
        Self {
            http_client: Client::new(),
            upstream,
        }
    }

    /// Вызов инструмента на upstream сервере
    pub async fn call_tool(&self, tool_name: &str, arguments: Value) -> Result<Value, String> {
        info!("📤 Проксирование вызова инструмента: {} -> {}", tool_name, self.upstream.name);

        let request = json!({
            "jsonrpc": "2.0",
            "method": "tools/call",
            "params": {
                "name": tool_name,
                "arguments": arguments
            },
            "id": 1
        });

        let url = format!("{}/tools/call", self.upstream.url);
        
        let response = match self.http_client
            .post(&url)
            .json(&request)
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                error!("❌ Ошибка соединения с upstream {}: {}", self.upstream.name, e);
                return Err(format!("Ошибка соединения: {}", e));
            }
        };

        if !response.status().is_success() {
            let status = response.status();
            warn!("⚠️ Upstream вернул ошибку: {}", status);
            return Err(format!("Upstream error: {}", status));
        }

        match response.json::<Value>().await {
            Ok(result) => {
                info!("✅ Получен ответ от upstream {}", self.upstream.name);
                Ok(result)
            }
            Err(e) => {
                error!("❌ Ошибка парсинга ответа: {}", e);
                Err(format!("Parse error: {}", e))
            }
        }
    }

    /// Получение списка инструментов с upstream сервера
    pub async fn list_tools(&self) -> Result<Value, String> {
        info!("📋 Запрос списка инструментов от upstream: {}", self.upstream.name);

        let request = json!({
            "jsonrpc": "2.0",
            "method": "tools/list",
            "id": 1
        });

        let url = format!("{}/tools/list", self.upstream.url);
        
        let response = self.http_client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Request error: {}", e))?;

        response
            .json::<Value>()
            .await
            .map_err(|e| format!("Parse error: {}", e))
    }

    /// Инициализация соединения с upstream
    pub async fn initialize(&self) -> Result<Value, String> {
        info!("🔌 Инициализация MCP соединения с upstream: {}", self.upstream.name);

        let request = json!({
            "jsonrpc": "2.0",
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "pii-anonymizer-proxy",
                    "version": "0.1.0"
                }
            },
            "id": 1
        });

        let url = format!("{}/initialize", self.upstream.url);
        
        let response = self.http_client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Request error: {}", e))?;

        response
            .json::<Value>()
            .await
            .map_err(|e| format!("Parse error: {}", e))
    }

    /// Проверка доступности upstream сервера
    pub async fn health_check(&self) -> bool {
        let url = format!("{}/health", self.upstream.url);
        
        match self.http_client.get(&url).send().await {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_client_creation() {
        let upstream = UpstreamServer {
            name: "test-upstream".to_string(),
            url: "http://localhost:8080".to_string(),
            timeout: 30,
        };
        
        let client = McpClient::new(upstream);
        assert_eq!(client.upstream.name, "test-upstream");
    }
}

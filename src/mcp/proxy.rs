use crate::anonymizer::AnonymizerEngine;
use crate::mcp::client::McpProxyManager;
use tracing::info;

/// Прокси-менеджер с анонимизацией запросов
pub struct AnonymizingProxy {
    /// Базовый прокси менеджер
    pub proxy: McpProxyManager,
    /// Движок анонимизации
    engine: AnonymizerEngine,
}

impl AnonymizingProxy {
    pub fn new(proxy: McpProxyManager, engine: AnonymizerEngine) -> Self {
        info!("🔒 AnonymizingProxy создан с {} серверами", proxy.server_names().len());
        Self { proxy, engine }
    }

    /// Вызвать инструмент с предварительной анонимизацией
    pub async fn call_tool(&self, tool_name: &str, args: serde_json::Value) -> Result<serde_json::Value, String> {
        // Анонимизируем все строковые значения в аргументах
        let anonymized_args = self.anonymize_json(&args);
        info!("🔒 Аргументы анонимизированы для {}", tool_name);

        // Вызываем прокси
        self.proxy.call_tool(tool_name, anonymized_args).await
    }

    /// Анонимизировать все строковые значения в JSON
    fn anonymize_json(&self, value: &serde_json::Value) -> serde_json::Value {
        match value {
            serde_json::Value::String(s) => {
                let result = self.engine.anonymize(&crate::models::AnonymizeRequest {
                    text: s.clone(),
                    strategy: None,
                });
                serde_json::Value::String(result.anonymized_text)
            }
            serde_json::Value::Array(arr) => {
                serde_json::Value::Array(arr.iter().map(|v| self.anonymize_json(v)).collect())
            }
            serde_json::Value::Object(map) => {
                let mut new_map = serde_json::Map::new();
                for (k, v) in map {
                    new_map.insert(k.clone(), self.anonymize_json(v));
                }
                serde_json::Value::Object(new_map)
            }
            _ => value.clone(),
        }
    }
}

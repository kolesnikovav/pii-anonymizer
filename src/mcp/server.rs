use serde_json::json;
use std::sync::Arc;
use tracing::info;

use crate::anonymizer::AnonymizerEngine;
use crate::models::AnonymizeRequest;

/// MCP сервер для анонимизации PII
#[derive(Clone)]
pub struct McpServer {
    engine: Arc<AnonymizerEngine>,
    pub server_name: String,
    pub server_version: String,
}

impl McpServer {
    pub fn new(engine: AnonymizerEngine, server_name: &str, server_version: &str) -> Self {
        info!("🤖 MCP Server инициализирован: {} v{}", server_name, server_version);
        
        Self {
            engine: Arc::new(engine),
            server_name: server_name.to_string(),
            server_version: server_version.to_string(),
        }
    }

    /// Получение списка инструментов MCP
    pub fn get_tools(&self) -> serde_json::Value {
        json!({
            "tools": [
                {
                    "name": "anonymize",
                    "description": "Анонимизировать текст, удаляя PII данные",
                    "input_schema": {
                        "type": "object",
                        "properties": {
                            "text": {
                                "type": "string",
                                "description": "Текст для анонимизации"
                            },
                            "strategy": {
                                "type": "string",
                                "enum": ["mask", "hash", "replace", "redact"],
                                "description": "Стратегия анонимизации (по умолчанию: mask)"
                            }
                        },
                        "required": ["text"]
                    }
                },
                {
                    "name": "detect_pii",
                    "description": "Обнаружить PII данные в тексте",
                    "input_schema": {
                        "type": "object",
                        "properties": {
                            "text": {
                                "type": "string",
                                "description": "Текст для анализа"
                            }
                        },
                        "required": ["text"]
                    }
                },
                {
                    "name": "batch_anonymize",
                    "description": "Пакетная анонимизация нескольких текстов",
                    "input_schema": {
                        "type": "object",
                        "properties": {
                            "texts": {
                                "type": "array",
                                "items": {"type": "string"},
                                "description": "Список текстов для анонимизации"
                            },
                            "strategy": {
                                "type": "string",
                                "enum": ["mask", "hash", "replace", "redact"],
                                "description": "Стратегия анонимизации"
                            }
                        },
                        "required": ["texts"]
                    }
                }
            ]
        })
    }

    /// Вызов инструмента
    pub async fn call_tool(&self, name: &str, arguments: serde_json::Value) -> serde_json::Value {
        match name {
            "anonymize" => self.tool_anonymize(arguments).await,
            "detect_pii" => self.tool_detect_pii(arguments).await,
            "batch_anonymize" => self.tool_batch_anonymize(arguments).await,
            _ => json!({
                "error": format!("Неизвестный инструмент: {}", name)
            }),
        }
    }

    async fn tool_anonymize(&self, args: serde_json::Value) -> serde_json::Value {
        let text = match args.get("text").and_then(|t| t.as_str()) {
            Some(t) => t.to_string(),
            None => {
                return json!({
                    "error": "Требуется параметр 'text'"
                });
            }
        };

        let strategy = args.get("strategy").and_then(|s| s.as_str()).map(String::from);
        
        let request = AnonymizeRequest {
            text,
            strategy,
        };

        let result = self.engine.anonymize(&request);
        
        json!({
            "anonymized_text": result.anonymized_text,
            "detected_pii_count": result.detected_pii.len(),
            "detected_pii": result.detected_pii.iter().map(|p| {
                json!({
                    "type": format!("{:?}", p.pii_type),
                    "confidence": p.confidence
                })
            }).collect::<Vec<_>>(),
            "strategy": result.strategy
        })
    }

    async fn tool_detect_pii(&self, args: serde_json::Value) -> serde_json::Value {
        let text = match args.get("text").and_then(|t| t.as_str()) {
            Some(t) => t.to_string(),
            None => {
                return json!({
                    "error": "Требуется параметр 'text'"
                });
            }
        };

        let detected = self.engine.detect_pii(&text);
        
        json!({
            "detected_pii": detected.iter().map(|p| {
                json!({
                    "type": format!("{:?}", p.pii_type),
                    "value": p.value,
                    "start": p.start,
                    "end": p.end,
                    "confidence": p.confidence
                })
            }).collect::<Vec<_>>(),
            "total_found": detected.len()
        })
    }

    async fn tool_batch_anonymize(&self, args: serde_json::Value) -> serde_json::Value {
        let texts = match args.get("texts").and_then(|t| t.as_array()) {
            Some(arr) => arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect::<Vec<_>>(),
            None => {
                return json!({
                    "error": "Требуется параметр 'texts' (array)"
                });
            }
        };

        let strategy = args.get("strategy").and_then(|s| s.as_str()).map(String::from);
        
        let requests: Vec<AnonymizeRequest> = texts.iter().map(|text| AnonymizeRequest {
            text: text.clone(),
            strategy: strategy.clone(),
        }).collect();

        let results = self.engine.anonymize_batch(&requests);
        
        json!({
            "results": results.iter().map(|r| {
                json!({
                    "anonymized_text": r.anonymized_text,
                    "pii_count": r.detected_pii.len()
                })
            }).collect::<Vec<_>>(),
            "total_processed": results.len()
        })
    }

    /// Инициализация MCP (информация о сервере)
    pub fn get_server_info(&self) -> serde_json::Value {
        json!({
            "name": self.server_name,
            "version": self.server_version,
            "protocol_version": "2024-11-05",
            "capabilities": ["tools"],
            "instructions": "Сервис для обнаружения и анонимизации PII данных в тексте"
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AnonymizerSettings;

    fn create_test_server() -> McpServer {
        let settings = AnonymizerSettings {
            default_strategy: "mask".to_string(),
            patterns: vec!["email".to_string()],
        };
        let engine = AnonymizerEngine::new(&settings);
        McpServer::new(engine, "test-server", "0.1.0")
    }

    #[test]
    fn test_server_info() {
        let server = create_test_server();
        let info = server.get_server_info();
        
        assert_eq!(info["name"], "test-server");
        assert_eq!(info["version"], "0.1.0");
    }

    #[test]
    fn test_get_tools() {
        let server = create_test_server();
        let tools = server.get_tools();
        
        assert!(tools["tools"].as_array().unwrap().len() >= 3);
    }

    #[tokio::test]
    async fn test_call_anonymize_tool() {
        let server = create_test_server();
        let result = server.call_tool("anonymize", json!({
            "text": "Contact: test@example.com"
        })).await;
        
        assert!(result["anonymized_text"].as_str().is_some());
        assert!(result["detected_pii_count"].as_u64().unwrap() > 0);
    }

    #[tokio::test]
    async fn test_call_detect_pii_tool() {
        let server = create_test_server();
        let result = server.call_tool("detect_pii", json!({
            "text": "Email: user@test.com"
        })).await;
        
        assert!(result["total_found"].as_u64().unwrap() > 0);
    }

    #[tokio::test]
    async fn test_call_unknown_tool() {
        let server = create_test_server();
        let result = server.call_tool("unknown", json!({})).await;
        
        assert!(result["error"].as_str().is_some());
    }
}

use crate::anonymizer::AnonymizerEngine;
use crate::mcp::client::McpProxyManager;
use std::collections::HashMap;
use tracing::info;

/// Anonymization rules for одного upstream сервера
#[derive(Debug, Clone)]
pub struct ServerAnonymizationRules {
    /// tool_name → список полей для анонимизации
    /// Пустой Vec = не анонимизировать вообще
    /// Отсутствие ключа = анонимизировать все строки (fallback)
    pub tool_fields: HashMap<String, Vec<String>>,
}

/// Прокси-менеджер с выборочной анонимизацией запросов
pub struct AnonymizingProxy {
    /// Базовый прокси менеджер
    pub proxy: McpProxyManager,
    /// Движок анонимизации
    engine: AnonymizerEngine,
    /// Правила анонимизации по серверам: server_name → rules
    rules: HashMap<String, ServerAnonymizationRules>,
}

impl AnonymizingProxy {
    pub fn new(proxy: McpProxyManager, engine: AnonymizerEngine) -> Self {
        info!(
            "🔒 AnonymizingProxy создан с {} серверами",
            proxy.server_names().len()
        );
        Self {
            proxy,
            engine,
            rules: HashMap::new(),
        }
    }

    /// Установить правила анонимизации для сервера
    pub fn set_rules(&mut self, server_name: String, rules: ServerAnonymizationRules) {
        info!(
            "📋 Anonymization rules for '{}': {} tools",
            server_name,
            rules.tool_fields.len()
        );
        self.rules.insert(server_name, rules);
    }

    /// Вызвать инструмент с выборочной анонимизацией
    pub async fn call_tool(
        &self,
        tool_name: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let strategy = self.find_rules_for_tool(tool_name);

        let (log_msg, anonymized_args) = match strategy {
            AnonymizeStrategy::All => (
                String::from("все строки анонимизированы"),
                self.anonymize_all_strings(&args),
            ),
            AnonymizeStrategy::None => (String::from("анонимизация отключена"), args.clone()),
            AnonymizeStrategy::Specific(ref fields) => (
                format!("анонимизированы поля: {:?}", fields),
                self.anonymize_fields(&args, fields),
            ),
        };

        info!("🔒 {}: {}", tool_name, log_msg);
        self.proxy.call_tool(tool_name, anonymized_args).await
    }

    /// Найти правила для инструмента
    fn find_rules_for_tool(&self, tool_name: &str) -> AnonymizeStrategy {
        // Ищем правила во всех серверах
        for rules in self.rules.values() {
            if let Some(fields) = rules.tool_fields.get(tool_name) {
                if fields.is_empty() {
                    return AnonymizeStrategy::None;
                }
                return AnonymizeStrategy::Specific(fields.clone());
            }
        }
        // Не нашли правил — анонимизируем всё (обратная совместимость)
        AnonymizeStrategy::All
    }

    /// Анонимизировать только указанные поля верхнего уровня
    fn anonymize_fields(&self, value: &serde_json::Value, fields: &[String]) -> serde_json::Value {
        match value {
            serde_json::Value::Object(map) => {
                let mut new_map = serde_json::Map::new();
                for (k, v) in map {
                    if fields.contains(k) {
                        // Анонимизируем только строковые значения указанных полей
                        if let serde_json::Value::String(s) = v {
                            let result = self.engine.anonymize(&crate::models::AnonymizeRequest {
                                text: s.clone(),
                                strategy: None,
                            });
                            new_map.insert(
                                k.clone(),
                                serde_json::Value::String(result.anonymized_text),
                            );
                        } else {
                            new_map.insert(k.clone(), v.clone());
                        }
                    } else {
                        new_map.insert(k.clone(), v.clone());
                    }
                }
                serde_json::Value::Object(new_map)
            }
            _ => value.clone(),
        }
    }

    /// Анонимизировать все строковые значения рекурсивно (старое поведение)
    fn anonymize_all_strings(&self, value: &serde_json::Value) -> serde_json::Value {
        match value {
            serde_json::Value::String(s) => {
                let result = self.engine.anonymize(&crate::models::AnonymizeRequest {
                    text: s.clone(),
                    strategy: None,
                });
                serde_json::Value::String(result.anonymized_text)
            }
            serde_json::Value::Array(arr) => serde_json::Value::Array(
                arr.iter().map(|v| self.anonymize_all_strings(v)).collect(),
            ),
            serde_json::Value::Object(map) => {
                let mut new_map = serde_json::Map::new();
                for (k, v) in map {
                    new_map.insert(k.clone(), self.anonymize_all_strings(v));
                }
                serde_json::Value::Object(new_map)
            }
            _ => value.clone(),
        }
    }
}

/// Стратегия анонимизации для инструмента
#[derive(Debug)]
enum AnonymizeStrategy {
    /// Анонимизировать все строки (fallback)
    All,
    /// Не анонимизировать вообще
    None,
    /// Анонимизировать только указанные поля
    Specific(Vec<String>),
}

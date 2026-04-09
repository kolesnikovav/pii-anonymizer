use rmcp::{ServerHandler, model::ServerInfo};
use rmcp::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::anonymizer::AnonymizerEngine;

/// Сервис для анонимизации PII — реализует rmcp ServerHandler
#[derive(Debug, Clone)]
pub struct AnonymizerService {
    engine: AnonymizerEngine,
}

// ── Request модели для инструментов ───────────────────────────────────────

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct AnonymizeReq {
    pub text: String,
    pub strategy: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct DetectPiiReq {
    pub text: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct BatchAnonymizeReq {
    pub texts: Vec<String>,
    pub strategy: Option<String>,
}

// ── Реализация инструментов через макросы rmcp ────────────────────────────

impl AnonymizerService {
    pub fn new(engine: AnonymizerEngine) -> Self {
        info!("🤖 AnonymizerService создан");
        Self { engine }
    }

    #[tool(description = "Анонимизировать текст, удаляя PII данные")]
    async fn anonymize(&self, #[tool(aggr)] req: AnonymizeReq) -> Result<String, String> {
        let request = crate::models::AnonymizeRequest {
            text: req.text,
            strategy: req.strategy,
        };

        let result = self.engine.anonymize(&request);

        Ok(serde_json::to_string(&result).unwrap_or_else(|_| "Error serializing result".to_string()))
    }

    #[tool(description = "Обнаружить PII данные в тексте")]
    async fn detect_pii(&self, #[tool(aggr)] req: DetectPiiReq) -> Result<String, String> {
        let detected = self.engine.detect_pii(&req.text);

        let result = serde_json::json!({
            "detected_pii": detected.iter().map(|p| {
                serde_json::json!({
                    "type": format!("{:?}", p.pii_type),
                    "value": p.value,
                    "start": p.start,
                    "end": p.end,
                    "confidence": p.confidence
                })
            }).collect::<Vec<_>>(),
            "total_found": detected.len()
        });

        Ok(serde_json::to_string(&result).unwrap_or_else(|_| "Error serializing result".to_string()))
    }

    #[tool(description = "Пакетная анонимизация нескольких текстов")]
    async fn batch_anonymize(&self, #[tool(aggr)] req: BatchAnonymizeReq) -> Result<String, String> {
        let requests: Vec<crate::models::AnonymizeRequest> = req.texts.iter().map(|text| {
            crate::models::AnonymizeRequest {
                text: text.clone(),
                strategy: req.strategy.clone(),
            }
        }).collect();

        let results = self.engine.anonymize_batch(&requests);

        let result = serde_json::json!({
            "results": results.iter().map(|r| {
                serde_json::json!({
                    "anonymized_text": r.anonymized_text,
                    "pii_count": r.detected_pii.len()
                })
            }).collect::<Vec<_>>(),
            "total_processed": results.len()
        });

        Ok(serde_json::to_string(&result).unwrap_or_else(|_| "Error serializing result".to_string()))
    }
}

// ── tool_box макрос генерирует call_tool и list_tools ─────────────────────

rmcp::tool_box!(AnonymizerService { anonymize, detect_pii, batch_anonymize });

// ── ServerHandler реализация ──────────────────────────────────────────────

impl ServerHandler for AnonymizerService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("Сервис для обнаружения и анонимизации PII данных в тексте. Поддерживает email, телефоны, паспорта РФ, СНИЛС, ИНН, кредитные карты, API ключи, JWT токены, SSH ключи и домены.".into()),
            ..Default::default()
        }
    }
}

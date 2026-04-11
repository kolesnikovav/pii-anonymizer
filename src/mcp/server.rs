use rmcp::tool;
use rmcp::{model::ServerInfo, ServerHandler};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::anonymizer::AnonymizerEngine;

#[derive(Debug, Clone)]
pub struct AnonymizerService {
    engine: AnonymizerEngine,
}

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

impl AnonymizerService {
    pub fn new(engine: AnonymizerEngine) -> Self {
        info!("🤖 AnonymizerService создан");
        Self { engine }
    }

    #[tool(description = "Анонимизировать текст, удаляя PII данные")]
    async fn anonymize(&self, #[tool(aggr)] req: AnonymizeReq) -> Result<String, String> {
        let result = self.engine.anonymize(&crate::models::AnonymizeRequest {
            text: req.text,
            strategy: req.strategy,
        });
        Ok(serde_json::to_string(&result).unwrap_or_else(|_| "Error".to_string()))
    }

    #[tool(description = "Обнаружить PII данные в тексте")]
    async fn detect_pii(&self, #[tool(aggr)] req: DetectPiiReq) -> Result<String, String> {
        let detected = self.engine.detect_pii(&req.text);
        let result = serde_json::json!({ "found": detected.len(), "pii": detected.iter().map(|p| format!("{:?}", p.pii_type)).collect::<Vec<_>>() });
        Ok(serde_json::to_string(&result).unwrap_or_else(|_| "Error".to_string()))
    }

    #[tool(description = "Пакетная анонимизация")]
    async fn batch_anonymize(
        &self,
        #[tool(aggr)] req: BatchAnonymizeReq,
    ) -> Result<String, String> {
        let requests: Vec<_> = req
            .texts
            .iter()
            .map(|t| crate::models::AnonymizeRequest {
                text: t.clone(),
                strategy: req.strategy.clone(),
            })
            .collect();
        let results = self.engine.anonymize_batch(&requests);
        let result = serde_json::json!({ "processed": results.len() });
        Ok(serde_json::to_string(&result).unwrap_or_else(|_| "Error".to_string()))
    }
}

rmcp::tool_box!(AnonymizerService {
    anonymize,
    detect_pii,
    batch_anonymize
});

impl ServerHandler for AnonymizerService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("PII Anonymizer MCP Server".into()),
            ..Default::default()
        }
    }
}

use rmcp::handler::server::ServerHandler;
use rmcp::model::{
    CallToolRequestParam, CallToolResult, Content, PaginatedRequestParamInner, ServerInfo, Tool,
};
use rmcp::service::RequestContext;
use rmcp::RoleServer;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

use crate::anonymizer::AnonymizerEngine;
use crate::mcp::AnonymizingProxy;

/// Объединённый MCP сервис: свои инструменты + проксированные
#[derive(Clone)]
pub struct ProxyMcpService {
    engine: Arc<AnonymizerEngine>,
    proxy: Arc<RwLock<Option<AnonymizingProxy>>>,
}

impl ProxyMcpService {
    pub fn new(engine: AnonymizerEngine) -> Self {
        info!("🤖 ProxyMcpService создан");
        Self {
            engine: Arc::new(engine),
            proxy: Arc::new(RwLock::new(None)),
        }
    }

    pub fn set_proxy(&mut self, proxy: AnonymizingProxy) {
        let count = proxy.proxy.get_tools().len();
        info!("🌐 Proxy установлен: {} внешних инструментов", count);
        self.proxy = Arc::new(RwLock::new(Some(proxy)));
    }

    /// Получить все инструменты (свои + прокси)
    pub async fn all_tools(&self) -> Vec<Tool> {
        let anonymize_schema = std::sync::Arc::new(serde_json::json!({
            "type": "object",
            "properties": {
                "text": { "type": "string", "description": "Текст для анонимизации" },
                "strategy": { "type": "string", "description": "Стратегия: mask, replace, hash", "enum": ["mask", "replace", "hash"] }
            },
            "required": ["text"]
        }).as_object().unwrap().clone());

        let detect_pii_schema = std::sync::Arc::new(
            serde_json::json!({
                "type": "object",
                "properties": {
                    "text": { "type": "string", "description": "Текст для анализа" }
                },
                "required": ["text"]
            })
            .as_object()
            .unwrap()
            .clone(),
        );

        let batch_schema = std::sync::Arc::new(serde_json::json!({
            "type": "object",
            "properties": {
                "texts": { "type": "array", "items": { "type": "string" }, "description": "Список текстов" },
                "strategy": { "type": "string", "description": "Стратегия: mask, replace, hash" }
            },
            "required": ["texts"]
        }).as_object().unwrap().clone());

        let mut tools = vec![
            Tool {
                name: "anonymize".into(),
                description: "Анонимизировать текст, удаляя PII данные".into(),
                input_schema: anonymize_schema,
            },
            Tool {
                name: "detect_pii".into(),
                description: "Обнаружить PII данные в тексте".into(),
                input_schema: detect_pii_schema,
            },
            Tool {
                name: "batch_anonymize".into(),
                description: "Пакетная анонимизация нескольких текстов".into(),
                input_schema: batch_schema,
            },
        ];

        let proxy = self.proxy.read().await;
        if let Some(p) = proxy.as_ref() {
            for tool in p.proxy.get_tools() {
                let schema = if let serde_json::Value::Object(m) = tool.input_schema {
                    // Убедимся что schema содержит "type": "object"
                    let mut m = m.clone();
                    if !m.contains_key("type") {
                        m.insert("type".to_string(), serde_json::json!("object"));
                    }
                    std::sync::Arc::new(m)
                } else {
                    std::sync::Arc::new(
                        serde_json::json!({"type": "object"})
                            .as_object()
                            .unwrap()
                            .clone(),
                    )
                };

                tools.push(Tool {
                    name: tool.name.into(),
                    description: tool.description.unwrap_or_default().into(),
                    input_schema: schema,
                });
            }
        }

        info!("📋 Всего инструментов: {}", tools.len());
        tools
    }

    /// Обработать вызов инструмента
    pub async fn handle_call(
        &self,
        tool_name: &str,
        arguments: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, String> {
        match tool_name {
            "anonymize" => self.call_anonymize(arguments).await,
            "detect_pii" => self.call_detect_pii(arguments).await,
            "batch_anonymize" => self.call_batch_anonymize(arguments).await,
            _ => self.call_proxy(tool_name, arguments).await,
        }
    }

    async fn call_anonymize(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, String> {
        let args = args.ok_or("Missing arguments")?;
        let text = args
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'text' field")?
            .to_string();
        let strategy = args
            .get("strategy")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let result = self
            .engine
            .anonymize(&crate::models::AnonymizeRequest { text, strategy });
        let json = serde_json::to_string(&result).map_err(|e| format!("Serialize: {}", e))?;

        Ok(CallToolResult {
            content: vec![Content::text(json)],
            is_error: None,
        })
    }

    async fn call_detect_pii(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, String> {
        let args = args.ok_or("Missing arguments")?;
        let text = args
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'text' field")?
            .to_string();

        let detected = self.engine.detect_pii(&text);
        let result = serde_json::json!({
            "found": detected.len(),
            "pii": detected.iter().map(|p| serde_json::json!({
                "type": format!("{:?}", p.pii_type),
                "value": p.value,
                "start": p.start,
                "end": p.end,
            })).collect::<Vec<_>>()
        });

        Ok(CallToolResult {
            content: vec![Content::text(
                serde_json::to_string(&result).unwrap_or_default(),
            )],
            is_error: None,
        })
    }

    async fn call_batch_anonymize(
        &self,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, String> {
        let args = args.ok_or("Missing arguments")?;
        let texts = args
            .get("texts")
            .and_then(|v| v.as_array())
            .ok_or("Missing 'texts' field")?
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect::<Vec<_>>();
        let strategy = args
            .get("strategy")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let requests: Vec<_> = texts
            .iter()
            .map(|t| crate::models::AnonymizeRequest {
                text: t.clone(),
                strategy: strategy.clone(),
            })
            .collect();
        let results = self.engine.anonymize_batch(&requests);
        let result = serde_json::json!({ "processed": results.len() });

        Ok(CallToolResult {
            content: vec![Content::text(
                serde_json::to_string(&result).unwrap_or_default(),
            )],
            is_error: None,
        })
    }

    async fn call_proxy(
        &self,
        tool_name: &str,
        arguments: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, String> {
        let proxy = self.proxy.read().await;
        let proxy = proxy
            .as_ref()
            .ok_or_else(|| format!("Unknown tool: {}", tool_name))?;

        let args = serde_json::Value::Object(arguments.unwrap_or_default());
        let result = proxy.call_tool(tool_name, args).await?;

        Ok(CallToolResult {
            content: vec![Content::text(
                serde_json::to_string(&result).unwrap_or_default(),
            )],
            is_error: None,
        })
    }
}

// ═══════════════════════════════════════════════════════════════════
// rmcp ServerHandler реализация
// ═══════════════════════════════════════════════════════════════════

impl ServerHandler for ProxyMcpService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "PII Anonymizer MCP Server с проксированием внешних MCP серверов.".into(),
            ),
            ..Default::default()
        }
    }

    async fn list_tools(
        &self,
        _params: Option<PaginatedRequestParamInner>,
        _context: RequestContext<RoleServer>,
    ) -> Result<rmcp::model::ListToolsResult, rmcp::model::ErrorData> {
        let tools = self.all_tools().await;
        Ok(rmcp::model::ListToolsResult {
            tools,
            next_cursor: None,
        })
    }

    async fn call_tool(
        &self,
        params: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::model::ErrorData> {
        let name = params.name.to_string();
        self.handle_call(&name, params.arguments)
            .await
            .map_err(|e| rmcp::model::ErrorData {
                code: rmcp::model::ErrorCode(-32603),
                message: e.into(),
                data: None,
            })
    }
}

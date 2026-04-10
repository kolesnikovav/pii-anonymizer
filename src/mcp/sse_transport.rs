use axum::{
    extract::{Query, State},
    response::sse::{Event, Sse},
    Json, Router, routing::{get, post},
};
use futures::Stream;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value, Map};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{info, error, warn};
use uuid::Uuid;

use crate::mcp::ProxyMcpService;

// ═══════════════════════════════════════════════════════════════════
/// MCP JSON-RPC сообщения
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<u64>,
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcError {
    code: i64,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcNotification {
    jsonrpc: String,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<Value>,
}

// ═══════════════════════════════════════════════════════════════════
/// Сессия клиента
// ═══════════════════════════════════════════════════════════════════

struct ClientSession {
    /// Канал для отправки SSE событий клиенту
    tx: mpsc::Sender<String>,
    /// Инициализирована ли MCP сессия
    initialized: bool,
}

// ═══════════════════════════════════════════════════════════════════
/// Состояние SSE сервера
// ═══════════════════════════════════════════════════════════════════

#[derive(Clone)]
pub struct SseServerState {
    /// Активные сессии
    sessions: Arc<RwLock<HashMap<String, ClientSession>>>,
    /// MCP сервис для обработки инструментов
    service: Arc<ProxyMcpService>,
}

impl SseServerState {
    pub fn new(service: ProxyMcpService) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            service: Arc::new(service),
        }
    }

    pub fn new_arc(service: Arc<ProxyMcpService>) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            service,
        }
    }

    /// Создать новую сессию
    pub async fn create_session(&self) -> (String, mpsc::Receiver<String>) {
        let session_id = Uuid::new_v4().to_string().replace('-', "");
        let (tx, rx) = mpsc::channel(100);

        self.sessions.write().await.insert(session_id.clone(), ClientSession {
            tx,
            initialized: false,
        });

        info!("📡 Новая SSE сессия: {}", session_id);
        (session_id, rx)
    }

    /// Отправить сообщение клиенту
    pub async fn send_to_client(&self, session_id: &str, message: &str) -> Result<(), String> {
        let sessions = self.sessions.read().await;
        if let Some(session) = sessions.get(session_id) {
            session.tx.send(message.to_string()).await
                .map_err(|_| "Failed to send message".to_string())
        } else {
            Err("Session not found".to_string())
        }
    }

    /// Отправить JSON-RPC response клиенту
    pub async fn send_response(&self, session_id: &str, response: JsonRpcResponse) -> Result<(), String> {
        let data = serde_json::to_string(&response).map_err(|e| format!("Serialize error: {}", e))?;
        self.send_to_client(session_id, &data).await
    }

    /// Обработать входящее сообщение
    pub async fn handle_message(&self, session_id: &str, message: Value) -> Result<(), String> {
        // Пробуем распознать как request
        if let Ok(request) = serde_json::from_value::<JsonRpcRequest>(message.clone()) {
            return self.handle_request(session_id, request).await;
        }

        // Или notification
        if let Ok(notification) = serde_json::from_value::<JsonRpcNotification>(message.clone()) {
            return self.handle_notification(session_id, notification).await;
        }

        warn!("⚠️ Неизвестный формат сообщения от {}", session_id);
        Ok(())
    }

    /// Обработать JSON-RPC request
    async fn handle_request(&self, session_id: &str, request: JsonRpcRequest) -> Result<(), String> {
        info!("📨 {}.{} от {}", request.method, request.id.unwrap_or(0), session_id);

        match request.method.as_str() {
            "initialize" => self.handle_initialize(session_id, request.id, request.params).await,
            "tools/list" => self.handle_tools_list(session_id, request.id).await,
            "tools/call" => self.handle_tools_call(session_id, request.id, request.params).await,
            "ping" => self.handle_ping(session_id, request.id).await,
            _ => {
                warn!("⚠️ Неизвестный метод: {}", request.method);
                self.send_response(session_id, JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32601,
                        message: format!("Method not found: {}", request.method),
                        data: None,
                    }),
                }).await
            }
        }
    }

    /// Обработать notification
    async fn handle_notification(&self, session_id: &str, notification: JsonRpcNotification) -> Result<(), String> {
        match notification.method.as_str() {
            "notifications/initialized" => {
                let mut sessions = self.sessions.write().await;
                if let Some(session) = sessions.get_mut(session_id) {
                    session.initialized = true;
                    info!("✅ Сессия {} инициализирована", session_id);
                }
                Ok(())
            }
            _ => {
                warn!("⚠️ Неизвестная нотификация: {}", notification.method);
                Ok(())
            }
        }
    }

    /// MCP Initialize
    async fn handle_initialize(&self, session_id: &str, id: Option<u64>, _params: Option<Value>) -> Result<(), String> {
        info!("🔧 Initialize от {}", session_id);

        let result = json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {
                    "listChanged": true
                }
            },
            "serverInfo": {
                "name": "PII Anonymizer",
                "version": "0.2.0"
            }
        });

        self.send_response(session_id, JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }).await
    }

    /// Tools List
    async fn handle_tools_list(&self, session_id: &str, id: Option<u64>) -> Result<(), String> {
        info!("📋 Tools List запрос от {}", session_id);

        let tools = self.service.all_tools().await;
        let tools_json: Vec<Value> = tools.iter().map(|t| {
            json!({
                "name": t.name,
                "description": t.description,
                "inputSchema": t.input_schema
            })
        }).collect();

        info!("📋 Возвращаю {} инструментов", tools_json.len());

        self.send_response(session_id, JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({ "tools": tools_json })),
            error: None,
        }).await
    }

    /// Tools Call
    async fn handle_tools_call(&self, session_id: &str, id: Option<u64>, params: Option<Value>) -> Result<(), String> {
        let params = params.ok_or("Missing params")?;
        let tool_name = params.get("name").and_then(|v| v.as_str()).ok_or("Missing tool name")?;
        let arguments = params.get("arguments").cloned();

        info!("🔧 Вызов инструмента: {} от {}", tool_name, session_id);

        // Преобразуем arguments в Map
        let args_map = match arguments {
            Some(Value::Object(m)) => Some(m),
            Some(_) => Some(Map::new()),
            None => None,
        };

        match self.service.handle_call(tool_name, args_map).await {
            Ok(result) => {
                info!("✅ Инструмент {} выполнен", tool_name);
                self.send_response(session_id, JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: Some(json!(result)),
                    error: None,
                }).await
            }
            Err(e) => {
                error!("❌ Ошибка инструмента {}: {}", tool_name, e);
                self.send_response(session_id, JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32603,
                        message: e,
                        data: None,
                    }),
                }).await
            }
        }
    }

    /// Ping
    async fn handle_ping(&self, session_id: &str, id: Option<u64>) -> Result<(), String> {
        self.send_response(session_id, JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({})),
            error: None,
        }).await
    }
}

// ═══════════════════════════════════════════════════════════════════
/// Axum handlers
// ═══════════════════════════════════════════════════════════════════

/// SSE endpoint — возвращает поток событий
async fn sse_endpoint(
    State(state): State<SseServerState>,
) -> Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>> {
    let (session_id, mut rx) = state.create_session().await;

    // Отправляем endpoint URL
    let endpoint_url = format!("/message?sessionId={}", session_id);
    let endpoint_event = Event::default()
        .event("endpoint")
        .data(&endpoint_url)
        .retry(std::time::Duration::from_secs(3000));

    info!("📡 SSE endpoint отправлен: {}", endpoint_url);

    // Поток событий
    let stream = async_stream::stream! {
        yield Ok(endpoint_event);

        while let Some(message) = rx.recv().await {
            yield Ok(Event::default().data(&message));
        }
    };

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(15))
            .text("keep-alive"),
    )
}

/// Message endpoint — принимает JSON-RPC сообщения
#[derive(Debug, Deserialize)]
struct MessageQuery {
    #[serde(rename = "sessionId")]
    session_id: String,
}

async fn message_endpoint(
    State(state): State<SseServerState>,
    Query(query): Query<MessageQuery>,
    Json(message): Json<Value>,
) -> Json<serde_json::Value> {
    match state.handle_message(&query.session_id, message).await {
        Ok(()) => Json(json!({})),
        Err(e) => {
            error!("❌ Ошибка обработки сообщения: {}", e);
            Json(json!({ "error": e }))
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
/// Создание роутера
// ═══════════════════════════════════════════════════════════════════

pub fn create_sse_router(service: ProxyMcpService) -> Router {
    let state = SseServerState::new(service);
    create_router_from_state(state)
}

/// Создание роутера с Arc<ProxyMcpService> (чтобы proxy не терялся при клонировании)
pub fn create_sse_router_arc(service: Arc<ProxyMcpService>) -> Router {
    let state = SseServerState::new_arc(service);
    create_router_from_state(state)
}

fn create_router_from_state(state: SseServerState) -> Router {
    Router::new()
        .route("/sse", get(sse_endpoint))
        .route("/message", post(message_endpoint))
        .with_state(state)
}

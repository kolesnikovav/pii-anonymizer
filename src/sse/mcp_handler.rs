use axum::{
    extract::{Query, State},
    response::sse::{Event, Sse},
    Json,
};
use futures::stream::Stream;
use serde::Deserialize;
use serde_json::json;
use std::convert::Infallible;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tracing::{info, warn};

use crate::mcp::server::McpServer;

/// Состояние для SSE MCP подключений
#[derive(Clone)]
pub struct SseMcpState {
    pub mcp_server: McpServer,
}

/// Query параметры для SSE подключения
#[derive(Debug, Deserialize)]
pub struct SseQueryParams {
    pub session_id: Option<String>,
}

/// Сообщение от клиента
#[derive(Debug, Deserialize)]
pub struct ClientMessage {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    pub method: Option<String>,
    pub params: Option<serde_json::Value>,
    pub result: Option<serde_json::Value>,
    pub error: Option<serde_json::Value>,
}

/// SSE endpoint для MCP
/// Клиент подключается и получает события от сервера
pub async fn sse_mcp_endpoint(
    State(state): State<SseMcpState>,
    Query(params): Query<SseQueryParams>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let session_id = params.session_id.unwrap_or_else(|| {
        uuid::Uuid::new_v4().to_string()
    });

    info!("🔌 SSE MCP подключение: session_id={}", session_id);

    // Создаём канал для отправки сообщений клиенту
    let (tx, rx) = mpsc::channel::<Result<Event, Infallible>>(32);

    // Отправляем приветственное сообщение
    let welcome_event = Ok(Event::default()
        .json_data(json!({
            "jsonrpc": "2.0",
            "method": "notification/initialized",
            "params": {
                "session_id": session_id,
                "server_info": state.mcp_server.get_server_info()
            }
        }))
        .unwrap_or_else(|_| Event::default().data("connected")));

    // Отправляем приветствие сразу
    let _ = tx.send(welcome_event).await;

    // Создаём stream из receiver
    let stream = ReceiverStream::new(rx);

    info!("✅ SSE MCP stream создан для session_id={}", session_id);

    Sse::new(stream)
}

/// POST endpoint для отправки MCP сообщений через SSE
pub async fn send_mcp_message(
    State(state): State<SseMcpState>,
    Json(message): Json<ClientMessage>,
) -> Json<serde_json::Value> {
    info!("📨 Получено MCP сообщение: {:?}", message.method);

    // Обрабатываем JSON-RPC запрос
    let response = handle_mcp_request(&state.mcp_server, &message).await;

    Json(response)
}

/// Обработка MCP запросов
async fn handle_mcp_request(server: &McpServer, message: &ClientMessage) -> serde_json::Value {
    // Initialize запрос
    if message.method.as_deref() == Some("initialize") {
        return handle_initialize(server, &message.params);
    }

    // Initialized notification
    if message.method.as_deref() == Some("notifications/initialized") {
        info!("✅ MCP клиент инициализирован");
        return json!({
            "jsonrpc": "2.0",
            "id": message.id
        });
    }

    // tools/list запрос
    if message.method.as_deref() == Some("tools/list") {
        let tools = server.get_tools();
        return json!({
            "jsonrpc": "2.0",
            "id": message.id,
            "result": tools
        });
    }

    // tools/call запрос
    if message.method.as_deref() == Some("tools/call") {
        return handle_tools_call(server, message).await;
    }

    // Неизвестный метод
    warn!("❓ Неизвестный метод MCP: {:?}", message.method);
    json!({
        "jsonrpc": "2.0",
        "id": message.id,
        "error": {
            "code": -32601,
            "message": format!("Method not found: {}", message.method.as_deref().unwrap_or("unknown"))
        }
    })
}

/// Обработка initialize
fn handle_initialize(server: &McpServer, _params: &Option<serde_json::Value>) -> serde_json::Value {
    let server_info = server.get_server_info();
    
    json!({
        "jsonrpc": "2.0",
        "result": {
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {
                    "listChanged": false
                }
            },
            "serverInfo": {
                "name": server_info["name"],
                "version": server_info["version"]
            }
        }
    })
}

/// Обработка tools/call
async fn handle_tools_call(server: &McpServer, message: &ClientMessage) -> serde_json::Value {
    let params = match &message.params {
        Some(p) => p,
        None => {
            return json!({
                "jsonrpc": "2.0",
                "id": message.id,
                "error": {
                    "code": -32602,
                    "message": "Missing params for tools/call"
                }
            });
        }
    };

    let name = match params.get("name").and_then(|n| n.as_str()) {
        Some(n) => n,
        None => {
            return json!({
                "jsonrpc": "2.0",
                "id": message.id,
                "error": {
                    "code": -32602,
                    "message": "Missing 'name' in params"
                }
            });
        }
    };

    let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

    info!("🔧 Вызов инструмента: {}", name);

    let result: serde_json::Value = server.call_tool(name, arguments).await;

    // Проверяем на ошибку
    if result.get("error").is_some() {
        return json!({
            "jsonrpc": "2.0",
            "id": message.id,
            "error": {
                "code": -1,
                "message": result["error"]
            }
        });
    }

    json!({
        "jsonrpc": "2.0",
        "id": message.id,
        "result": {
            "content": [
                {
                    "type": "text",
                    "text": serde_json::to_string(&result).unwrap_or_default()
                }
            ]
        }
    })
}

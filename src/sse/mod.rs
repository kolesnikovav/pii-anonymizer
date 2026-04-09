pub mod mcp_handler;

pub use mcp_handler::{SseMcpState, sse_mcp_endpoint, send_mcp_message};

use axum::{routing::{get, post}, Router};
use tower_http::cors::{CorsLayer, Any};

/// Создание роутера с MCP SSE поддержкой
pub fn create_mcp_router(mcp_state: SseMcpState) -> Router {
    // CORS настройка
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // MCP SSE endpoint
        .route("/sse", get(sse_mcp_endpoint))
        // MCP message endpoint
        .route("/sse/message", post(send_mcp_message))
        .layer(cors)
        .with_state(mcp_state)
}

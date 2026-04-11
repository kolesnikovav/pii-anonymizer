use axum::{extract::Request, middleware::Next, response::Response};
use tracing::info;

/// Middleware для добавления request ID
pub async fn request_id_middleware(request: Request, next: Next) -> Response {
    let request_id = uuid::Uuid::new_v4().to_string();

    info!("🆔 Request ID: {}", request_id);

    next.run(request).await
}

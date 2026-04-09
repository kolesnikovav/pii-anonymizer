use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::time::Instant;
use tracing::{info, warn};

/// Middleware для логирования запросов
pub async fn request_logger(request: Request, next: Next) -> Response {
    let start = Instant::now();
    
    let method = request.method().clone();
    let uri = request.uri().clone();
    
    info!("➡️ {} {}", method, uri);
    
    let response = next.run(request).await;
    
    let elapsed = start.elapsed();
    let status = response.status();
    
    if status.is_server_error() {
        warn!("⚠️ {} {} {} - {:?}", method, uri, status, elapsed);
    } else {
        info!("✅ {} {} {} - {:?}", method, uri, status, elapsed);
    }
    
    response
}

/// Middleware для добавления CORS заголовков
pub async fn cors_middleware(request: Request, next: Next) -> Response {
    let response = next.run(request).await;
    
    response
}

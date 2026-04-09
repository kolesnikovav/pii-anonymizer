use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;

use crate::config::Settings;
use crate::anonymizer::AnonymizerEngine;
use crate::api::handlers;
use crate::middleware::{request_logger, request_id_middleware};

pub fn create_router(settings: Settings, anonymizer: AnonymizerEngine) -> Router {
    // CORS настройка
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // Health check
        .route("/api/v1/health", get(handlers::health_check))
        // Основные эндпоинты
        .route("/api/v1/anonymize", post(handlers::anonymize))
        .route("/api/v1/detect", post(handlers::detect_pii))
        .route("/api/v1/batch", post(handlers::batch_anonymize))
        .route("/api/v1/stats", post(handlers::get_stats))
        // SSE
        .route("/api/v1/sse/stream", get(handlers::sse_stream))
        // Middleware
        .layer(axum::middleware::from_fn(request_logger))
        .layer(axum::middleware::from_fn(request_id_middleware))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state((settings, anonymizer))
}

use axum::{
    routing::{get, post},
    Router,
};
use crate::config::Settings;
use crate::anonymizer::AnonymizerEngine;
use crate::api::handlers;

pub fn create_router(settings: Settings, anonymizer: AnonymizerEngine) -> Router {
    Router::new()
        .route("/api/v1/health", get(handlers::health_check))
        .route("/api/v1/anonymize", post(handlers::anonymize))
        .route("/api/v1/detect", post(handlers::detect_pii))
        .route("/api/v1/sse/stream", get(handlers::sse_stream))
        .with_state((settings, anonymizer))
}

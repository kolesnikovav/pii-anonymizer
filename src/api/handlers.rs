use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use axum::response::Sse;
use std::convert::Infallible;
use std::time::Duration;
use tokio::time::interval;
use tokio_stream::wrappers::IntervalStream;
use tokio_stream::StreamExt;
use serde_json::json;

use crate::config::Settings;
use crate::anonymizer::AnonymizerEngine;
use crate::models::{AnonymizeRequest, AnonymizeResponse};

type AppState = (Settings, AnonymizerEngine);

/// Health check
pub async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "pii-anonymizer",
        "version": "0.1.0"
    }))
}

/// Анонимизация текста
pub async fn anonymize(
    State(state): State<AppState>,
    Json(request): Json<AnonymizeRequest>,
) -> Result<Json<AnonymizeResponse>, StatusCode> {
    let (_, anonymizer) = state;
    
    if request.text.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    let response = anonymizer.anonymize(&request);
    Ok(Json(response))
}

/// Обнаружение PII
pub async fn detect_pii(
    State(state): State<AppState>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let (_, anonymizer) = state;
    
    let text = request
        .get("text")
        .and_then(|t| t.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let detected = anonymizer.detect_pii(text);
    Ok(Json(json!({ "detected_pii": detected })))
}

/// SSE стрим
pub async fn sse_stream() -> Sse<impl StreamExt<Item = Result<axum::response::sse::Event, Infallible>>> {
    let interval = interval(Duration::from_secs(1));
    let stream = IntervalStream::new(interval);
    
    let stream = stream.map(|_| {
        Ok(axum::response::sse::Event::default()
            .json_data(json!({
                "status": "running",
                "timestamp": chrono::Utc::now().to_rfc3339()
            }))
            .unwrap())
    });
    
    Sse::new(stream)
}

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
use tracing::{info, error};
use validator::Validate;

use crate::config::Settings;
use crate::anonymizer::AnonymizerEngine;
use crate::models::{
    AnonymizeRequest, AnonymizeResponse, 
    BatchAnonymizeRequest, BatchAnonymizeResponse,
    PIIStats
};
use crate::api::error::AppError;

type AppState = (Settings, AnonymizerEngine);

/// Health check
pub async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "pii-anonymizer",
        "version": "0.1.0",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Анонимизация текста
pub async fn anonymize(
    State(state): State<AppState>,
    Json(request): Json<AnonymizeRequest>,
) -> Result<Json<AnonymizeResponse>, AppError> {
    // Валидация
    request.validate().map_err(AppError::from)?;
    
    let (settings, anonymizer) = state;
    
    info!("📨 Запрос на анонимизацию: {} символов", request.text.len());
    
    let response = anonymizer.anonymize(&request);
    
    info!("✅ Анонимизация завершена: {} PII найдено", response.detected_pii.len());
    
    Ok(Json(response))
}

/// Обнаружение PII
pub async fn detect_pii(
    State(state): State<AppState>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let text = request
        .get("text")
        .and_then(|t| t.as_str())
        .ok_or_else(|| AppError::BadRequest("Поле 'text' обязательно".to_string()))?;
    
    if text.is_empty() || text.len() > 100000 {
        return Err(AppError::BadRequest("Текст должен быть от 1 до 100000 символов".to_string()));
    }
    
    let (_, anonymizer) = state;
    let detected = anonymizer.detect_pii(text);
    
    Ok(Json(json!({ 
        "detected_pii": detected,
        "total_found": detected.len()
    })))
}

/// Пакетная обработка
pub async fn batch_anonymize(
    State(state): State<AppState>,
    Json(request): Json<BatchAnonymizeRequest>,
) -> Result<Json<BatchAnonymizeResponse>, AppError> {
    // Валидация
    request.validate().map_err(AppError::from)?;
    
    let (_, anonymizer) = state;
    
    info!("📨 Пакетный запрос: {} элементов", request.requests.len());
    
    let results = anonymizer.anonymize_batch(&request.requests);
    let total_processed = results.len();
    
    Ok(Json(BatchAnonymizeResponse {
        results,
        total_processed,
    }))
}

/// SSE стрим
pub async fn sse_stream() -> Sse<impl StreamExt<Item = Result<axum::response::sse::Event, Infallible>>> {
    let interval = interval(Duration::from_secs(1));
    let stream = IntervalStream::new(interval);
    
    let stream = stream.map(|tick| {
        let timestamp = chrono::Utc::now().to_rfc3339();
        Ok(axum::response::sse::Event::default()
            .json_data(json!({
                "status": "running",
                "timestamp": timestamp,
                "uptime_seconds": tick.elapsed().as_secs()
            }))
            .unwrap())
    });
    
    info!("📡 SSE стрим запущен");
    Sse::new(stream)
}

/// Статистика PII
pub async fn get_stats(
    State(state): State<AppState>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<PIIStats>, AppError> {
    let text = request
        .get("text")
        .and_then(|t| t.as_str())
        .ok_or_else(|| AppError::BadRequest("Поле 'text' обязательно".to_string()))?;
    
    let (_, anonymizer) = state;
    let detected = anonymizer.detect_pii(text);
    
    let mut by_type = std::collections::HashMap::new();
    for pii in &detected {
        let type_name = format!("{:?}", pii.pii_type);
        *by_type.entry(type_name).or_insert(0) += 1;
    }
    
    Ok(Json(PIIStats {
        total_detected: detected.len(),
        by_type,
    }))
}

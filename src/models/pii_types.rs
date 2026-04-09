use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, ToSchema, Validate)]
pub struct AnonymizeRequest {
    #[validate(length(min = 1, max = 100000))]
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AnonymizeResponse {
    pub original_text: String,
    pub anonymized_text: String,
    pub detected_pii: Vec<DetectedPII>,
    pub strategy: String,
}

#[derive(Debug, Serialize, Clone, ToSchema)]
pub struct DetectedPII {
    pub pii_type: PIIType,
    pub value: String,
    pub start: usize,
    pub end: usize,
    pub confidence: f64,
}

#[derive(Debug, Serialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PIIType {
    Email,
    Phone,
    Passport,
    CreditCard,
    IpAddress,
    Snils,
    Inn,
    Address,
    FullName,
    Unknown,
}

impl std::fmt::Display for PIIType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PIIType::Email => write!(f, "EMAIL"),
            PIIType::Phone => write!(f, "PHONE"),
            PIIType::Passport => write!(f, "PASSPORT"),
            PIIType::CreditCard => write!(f, "CREDIT_CARD"),
            PIIType::IpAddress => write!(f, "IP_ADDRESS"),
            PIIType::Snils => write!(f, "SNILS"),
            PIIType::Inn => write!(f, "INN"),
            PIIType::Address => write!(f, "ADDRESS"),
            PIIType::FullName => write!(f, "FULL_NAME"),
            PIIType::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

/// Запрос для пакетной обработки
#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct BatchAnonymizeRequest {
    #[validate(length(min = 1, max = 1000))]
    pub requests: Vec<AnonymizeRequest>,
}

/// Ответ для пакетной обработки
#[derive(Debug, Serialize, ToSchema)]
pub struct BatchAnonymizeResponse {
    pub results: Vec<AnonymizeResponse>,
    pub total_processed: usize,
}

/// Статистика PII
#[derive(Debug, Serialize, ToSchema)]
pub struct PIIStats {
    pub total_detected: usize,
    pub by_type: std::collections::HashMap<String, usize>,
}

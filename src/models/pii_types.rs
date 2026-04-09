use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct AnonymizeRequest {
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
            PIIType::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

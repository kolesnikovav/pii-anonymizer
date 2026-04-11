use serde::Deserialize;

/// Кастомный PII паттерн из конфига
#[derive(Debug, Deserialize, Clone)]
pub struct CustomPatternConfig {
    pub name: String,
    pub pii_type: String,
    pub pattern: String,
    #[serde(default = "default_confidence")]
    pub confidence: f64,
}

fn default_confidence() -> f64 {
    0.85
}

use crate::config::AnonymizerSettings;
use crate::models::{AnonymizeRequest, AnonymizeResponse, DetectedPII, PIIType};

#[derive(Debug, Clone)]
pub struct AnonymizerEngine {
    settings: AnonymizerSettings,
    patterns: Vec<regex::Regex>,
}

impl AnonymizerEngine {
    pub fn new(settings: &AnonymizerSettings) -> Self {
        let patterns = Self::compile_patterns(&settings.patterns);
        
        Self {
            settings: settings.clone(),
            patterns,
        }
    }

    /// Анонимизация текста
    pub fn anonymize(&self, request: &AnonymizeRequest) -> AnonymizeResponse {
        let text = &request.text;
        let mut detected_pii = Vec::new();
        let mut anonymized_text = text.clone();

        // Обнаружение PII
        for pattern in &self.patterns {
            for mat in pattern.find_iter(text) {
                let pii_type = self.classify_pattern(mat.as_str());
                detected_pii.push(DetectedPII {
                    pii_type: pii_type.clone(),
                    value: mat.as_str().to_string(),
                    start: mat.start(),
                    end: mat.end(),
                    confidence: 0.95,
                });

                // Замена PII
                let replacement = self.create_replacement(&pii_type, mat.as_str());
                anonymized_text.replace_range(mat.start()..mat.end(), &replacement);
            }
        }

        AnonymizeResponse {
            original_text: text.clone(),
            anonymized_text,
            detected_pii,
            strategy: request.strategy.clone().unwrap_or(self.settings.default_strategy.clone()),
        }
    }

    /// Компиляция regex паттернов
    fn compile_patterns(pattern_names: &[String]) -> Vec<regex::Regex> {
        let mut patterns = Vec::new();

        for name in pattern_names {
            if let Some(pattern) = Self::get_pattern_by_name(name) {
                if let Ok(regex) = regex::Regex::new(pattern) {
                    patterns.push(regex);
                }
            }
        }

        patterns
    }

    /// Получение паттерна по имени
    fn get_pattern_by_name(name: &str) -> Option<&'static str> {
        match name {
            "email" => Some(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}"),
            "phone" => Some(r"\+7\s?\(?\d{3}\)?\s?\d{3}[-.]?\d{2}[-.]?\d{2}"),
            "passport" => Some(r"\b\d{4}\s?\d{6}\b"),
            "credit_card" => Some(r"\b\d{4}[-\s]?\d{4}[-\s]?\d{4}[-\s]?\d{4}\b"),
            "ip_address" => Some(r"\b(?:\d{1,3}\.){3}\d{1,3}\b"),
            _ => None,
        }
    }

    /// Классификация найденного совпадения
    fn classify_pattern(&self, value: &str) -> PIIType {
        if value.contains('@') {
            PIIType::Email
        } else if value.starts_with("+7") || value.starts_with("8(") {
            PIIType::Phone
        } else if value.len() == 10 || value.len() == 11 {
            PIIType::Passport
        } else if value.len() == 16 || value.len() == 19 {
            PIIType::CreditCard
        } else {
            PIIType::Unknown
        }
    }

    /// Создание замены для PII
    fn create_replacement(&self, pii_type: &PIIType, original: &str) -> String {
        match self.settings.default_strategy.as_str() {
            "mask" => self.settings.mask_char.to_string().repeat(self.settings.mask_length),
            "hash" => format!("[HASH:{}]", self.simple_hash(original)),
            "redact" => format!("[{} REMOVED]", pii_type),
            _ => "[REDACTED]".to_string(),
        }
    }

    /// Простой хеш для анонимизации
    fn simple_hash(&self, input: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Обнаружение PII без замены
    pub fn detect_pii(&self, text: &str) -> Vec<DetectedPII> {
        let mut detected = Vec::new();

        for pattern in &self.patterns {
            for mat in pattern.find_iter(text) {
                let pii_type = self.classify_pattern(mat.as_str());
                detected.push(DetectedPII {
                    pii_type,
                    value: mat.as_str().to_string(),
                    start: mat.start(),
                    end: mat.end(),
                    confidence: 0.95,
                });
            }
        }

        detected
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AnonymizerSettings;

    fn create_test_engine() -> AnonymizerEngine {
        let settings = AnonymizerSettings {
            default_strategy: "mask".to_string(),
            patterns: vec!["email".to_string(), "phone".to_string()],
            mask_char: '*',
            mask_length: 10,
        };
        AnonymizerEngine::new(&settings)
    }

    #[test]
    fn test_detect_email() {
        let engine = create_test_engine();
        let text = "Contact: test@example.com";
        let detected = engine.detect_pii(text);
        
        assert_eq!(detected.len(), 1);
        assert_eq!(detected[0].pii_type, PIIType::Email);
    }

    #[test]
    fn test_anonymize_text() {
        let engine = create_test_engine();
        let request = AnonymizeRequest {
            text: "Email: test@example.com".to_string(),
            strategy: None,
        };
        let response = engine.anonymize(&request);
        
        assert!(!response.anonymized_text.contains("test@example.com"));
        assert!(!response.detected_pii.is_empty());
    }
}

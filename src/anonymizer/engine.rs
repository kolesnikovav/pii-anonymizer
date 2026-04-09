use crate::config::AnonymizerSettings;
use crate::models::{AnonymizeRequest, AnonymizeResponse, DetectedPII, PIIType};
use crate::anonymizer::patterns::{get_all_patterns, PIIPattern};
use crate::anonymizer::strategies::AnonymizationStrategy;
use tracing::{debug, info};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct AnonymizerEngine {
    settings: AnonymizerSettings,
    patterns: Vec<PIIPattern>,
}

impl AnonymizerEngine {
    pub fn new(settings: &AnonymizerSettings) -> Self {
        let patterns = Self::build_patterns(settings);
        info!("🔍 Анонимизатор инициализирован с {} паттернами", patterns.len());
        
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
        
        // Определение стратегии
        let strategy = request
            .strategy
            .as_ref()
            .map(|s| AnonymizationStrategy::from_str(s))
            .unwrap_or_else(|| AnonymizationStrategy::from_str(&self.settings.default_strategy));

        // Обнаружение и замена с счётчиком для Replace стратегии
        let mut matches: Vec<(usize, usize, String, String)> = Vec::new();
        let mut pii_counters: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

        for pattern in &self.patterns {
            for mat in pattern.pattern.find_iter(text) {
                // Проверка на дубликаты (пересечения)
                let is_duplicate = matches.iter().any(|(start, end, _, _)| {
                    mat.start() >= *start && mat.end() <= *end
                });
                
                if !is_duplicate {
                    let pii_type_str = pattern.pii_type.to_string();
                    let counter = pii_counters.entry(pii_type_str.clone()).or_insert(0);
                    *counter += 1;
                    let current_counter = *counter;

                    detected_pii.push(DetectedPII {
                        pii_type: self.map_pii_type(&pattern.pii_type),
                        value: mat.as_str().to_string(),
                        start: mat.start(),
                        end: mat.end(),
                        confidence: pattern.confidence,
                    });

                    let replacement = strategy.apply(
                        mat.as_str(),
                        &pii_type_str,
                        current_counter,
                    );
                    
                    matches.push((mat.start(), mat.end(), pii_type_str, replacement));
                }
            }
        }

        // Сортировка по позициям с конца для корректной замены
        matches.sort_by(|a, b| b.0.cmp(&a.0));
        
        // Применение замен
        for (start, end, _, replacement) in &matches {
            anonymized_text.replace_range(*start..*end, replacement);
        }

        info!(
            "✅ Анонимизация завершена: найдено {} PII, стратегия: {:?}",
            detected_pii.len(),
            strategy
        );

        AnonymizeResponse {
            original_text: text.clone(),
            anonymized_text,
            detected_pii,
            strategy: format!("{:?}", strategy).to_lowercase(),
        }
    }

    /// Построение списка паттернов на основе настроек
    fn build_patterns(settings: &AnonymizerSettings) -> Vec<PIIPattern> {
        let all_patterns = get_all_patterns();
        let enabled_patterns: HashSet<String> = settings.patterns.iter().cloned().collect();
        
        all_patterns
            .into_iter()
            .filter(|p| enabled_patterns.contains(&p.name) || enabled_patterns.contains(&p.pii_type.to_string().to_lowercase()))
            .collect()
    }

    /// Маппинг внутреннего PIIType на модель
    fn map_pii_type(&self, pii_type: &crate::anonymizer::patterns::PIIType) -> PIIType {
        match pii_type {
            crate::anonymizer::patterns::PIIType::Email => PIIType::Email,
            crate::anonymizer::patterns::PIIType::Phone => PIIType::Phone,
            crate::anonymizer::patterns::PIIType::Passport => PIIType::Passport,
            crate::anonymizer::patterns::PIIType::CreditCard => PIIType::CreditCard,
            crate::anonymizer::patterns::PIIType::IpAddress => PIIType::IpAddress,
            crate::anonymizer::patterns::PIIType::Snils => PIIType::Snils,
            crate::anonymizer::patterns::PIIType::Inn => PIIType::Inn,
            crate::anonymizer::patterns::PIIType::Address => PIIType::Address,
            crate::anonymizer::patterns::PIIType::FullName => PIIType::FullName,
            crate::anonymizer::patterns::PIIType::Unknown => PIIType::Unknown,
        }
    }

    /// Обнаружение PII без замены
    pub fn detect_pii(&self, text: &str) -> Vec<DetectedPII> {
        let mut detected = Vec::new();

        for pattern in &self.patterns {
            for mat in pattern.pattern.find_iter(text) {
                detected.push(DetectedPII {
                    pii_type: self.map_pii_type(&pattern.pii_type),
                    value: mat.as_str().to_string(),
                    start: mat.start(),
                    end: mat.end(),
                    confidence: pattern.confidence,
                });
            }
        }

        debug!("🔍 Обнаружено {} PII в тексте", detected.len());
        detected
    }

    /// Пакетная обработка
    pub fn anonymize_batch(&self, requests: &[AnonymizeRequest]) -> Vec<AnonymizeResponse> {
        info!("📦 Пакетная обработка: {} запросов", requests.len());
        requests.iter().map(|req| self.anonymize(req)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AnonymizerSettings;

    fn create_test_engine() -> AnonymizerEngine {
        let settings = AnonymizerSettings {
            default_strategy: "mask".to_string(),
            patterns: vec![
                "email".to_string(),
                "phone_ru".to_string(),
                "passport_ru".to_string(),
                "credit_card".to_string(),
                "ip_address".to_string(),
                "snils".to_string(),
            ],
        };
        AnonymizerEngine::new(&settings)
    }

    #[test]
    fn test_detect_email() {
        let engine = create_test_engine();
        let text = "Contact: test@example.com for info";
        let detected = engine.detect_pii(text);
        
        assert_eq!(detected.len(), 1);
        assert_eq!(detected[0].pii_type, PIIType::Email);
        assert_eq!(detected[0].value, "test@example.com");
    }

    #[test]
    fn test_detect_phone() {
        let engine = create_test_engine();
        let text = "Call me at +7 (999) 123-45-67";
        let detected = engine.detect_pii(text);
        
        assert_eq!(detected.len(), 1);
        assert_eq!(detected[0].pii_type, PIIType::Phone);
    }

    #[test]
    fn test_detect_multiple_pii() {
        let engine = create_test_engine();
        let text = "Email: user@test.com, Phone: +7 (999) 123-45-67, IP: 192.168.1.1";
        let detected = engine.detect_pii(text);
        
        assert!(detected.len() >= 3);
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
        assert_eq!(response.strategy, "mask");
    }

    #[test]
    fn test_anonymize_with_strategy() {
        let engine = create_test_engine();
        let request = AnonymizeRequest {
            text: "Email: test@example.com".to_string(),
            strategy: Some("replace".to_string()),
        };
        let response = engine.anonymize(&request);
        
        assert!(!response.anonymized_text.contains("test@example.com"));
        assert!(response.anonymized_text.contains("[EMAIL_"));
    }

    #[test]
    fn test_anonymize_preserves_non_pii() {
        let engine = create_test_engine();
        let request = AnonymizeRequest {
            text: "Hello world, this is a test".to_string(),
            strategy: None,
        };
        let response = engine.anonymize(&request);
        
        assert_eq!(response.anonymized_text, "Hello world, this is a test");
        assert!(response.detected_pii.is_empty());
    }

    #[test]
    fn test_batch_processing() {
        let engine = create_test_engine();
        let requests = vec![
            AnonymizeRequest {
                text: "Email: test1@example.com".to_string(),
                strategy: None,
            },
            AnonymizeRequest {
                text: "Phone: +7 (999) 123-45-67".to_string(),
                strategy: None,
            },
        ];
        
        let responses = engine.anonymize_batch(&requests);
        assert_eq!(responses.len(), 2);
        assert!(!responses[0].anonymized_text.contains("test1@example.com"));
        assert!(!responses[1].anonymized_text.contains("+7 (999) 123-45-67"));
    }

    #[test]
    fn test_empty_text() {
        let engine = create_test_engine();
        let request = AnonymizeRequest {
            text: "".to_string(),
            strategy: None,
        };
        let response = engine.anonymize(&request);
        
        assert_eq!(response.anonymized_text, "");
        assert!(response.detected_pii.is_empty());
    }

    #[test]
    fn test_confidence_scores() {
        let engine = create_test_engine();
        let text = "Contact test@example.com";
        let detected = engine.detect_pii(text);
        
        assert!(!detected.is_empty());
        assert!(detected[0].confidence > 0.0 && detected[0].confidence <= 1.0);
    }
}

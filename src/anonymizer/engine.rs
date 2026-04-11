use crate::anonymizer::patterns::{get_all_patterns, is_known_domain, PIIPattern};
use crate::anonymizer::strategies::AnonymizationStrategy;
use crate::config::AnonymizerSettings;
use crate::models::{AnonymizeRequest, AnonymizeResponse, DetectedPII, PIIType};
use std::collections::HashSet;
use tracing::{debug, info, warn};

#[derive(Debug, Clone)]
pub struct AnonymizerEngine {
    settings: AnonymizerSettings,
    patterns: Vec<PIIPattern>,
    custom_known_domains: Vec<String>,
}

impl AnonymizerEngine {
    pub fn new(settings: &AnonymizerSettings) -> Self {
        // Load custom patterns from config
        let custom_patterns: Vec<PIIPattern> = settings
            .custom_patterns
            .iter()
            .filter_map(|cp| {
                match PIIPattern::from_config(&cp.name, &cp.pii_type, &cp.pattern, cp.confidence) {
                    Ok(p) => {
                        info!("Custom pattern: {}", cp.name);
                        Some(p)
                    }
                    Err(e) => {
                        warn!("Skipping custom pattern '{}': {}", cp.name, e);
                        None
                    }
                }
            })
            .collect();

        let patterns = Self::build_patterns(settings, custom_patterns);

        info!(
            "Anonymizer: {} patterns, {} custom domains",
            patterns.len(),
            settings.custom_known_domains.len()
        );

        Self {
            settings: settings.clone(),
            patterns,
            custom_known_domains: settings.custom_known_domains.clone(),
        }
    }

    /// Анонимизация текста
    pub fn anonymize(&self, request: &AnonymizeRequest) -> AnonymizeResponse {
        let text = &request.text;
        let mut detected_pii = Vec::new();
        let mut anonymized_text = text.clone();

        let strategy = request
            .strategy
            .as_ref()
            .map(|s| AnonymizationStrategy::parse_strategy(s))
            .unwrap_or_else(|| AnonymizationStrategy::parse_strategy(&self.settings.default_strategy));

        let mut matches: Vec<(usize, usize, String, String)> = Vec::new();
        let mut pii_counters: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        for pattern in &self.patterns {
            for mat in pattern.pattern.find_iter(text) {
                if pattern.pii_type == crate::anonymizer::patterns::PIIType::Domain {
                    let domain_value = mat.as_str();
                    let clean_domain = self.extract_domain_from_match(domain_value);
                    if is_known_domain(&clean_domain, &self.custom_known_domains) {
                        debug!("Skipping known domain: {}", clean_domain);
                        continue;
                    }
                }

                let is_duplicate = matches
                    .iter()
                    .any(|(start, end, _, _)| mat.start() >= *start && mat.end() <= *end);

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

                    let replacement = strategy.apply(mat.as_str(), &pii_type_str, current_counter);

                    matches.push((mat.start(), mat.end(), pii_type_str, replacement));
                }
            }
        }

        matches.sort_by(|a, b| b.0.cmp(&a.0));

        for (start, end, _, replacement) in &matches {
            anonymized_text.replace_range(*start..*end, replacement);
        }

        info!(
            "Anonymization complete: {} PII found, strategy: {:?}",
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

    /// Build pattern list: built-in (enabled) + custom
    fn build_patterns(settings: &AnonymizerSettings, custom: Vec<PIIPattern>) -> Vec<PIIPattern> {
        let all_patterns = get_all_patterns();
        let enabled: HashSet<String> = settings.patterns.iter().cloned().collect();

        let builtin: Vec<PIIPattern> = all_patterns
            .into_iter()
            .filter(|p| {
                enabled.contains(&p.name)
                    || enabled.contains(&p.pii_type.to_string().to_lowercase())
            })
            .collect();

        let mut result = builtin;
        result.extend(custom);
        result
    }

    fn extract_domain_from_match(&self, matched_text: &str) -> String {
        let without_protocol = matched_text
            .trim_start_matches("http://")
            .trim_start_matches("https://");
        let without_www = without_protocol.trim_start_matches("www.");
        without_www
            .split('/')
            .next()
            .unwrap_or(without_www)
            .to_string()
    }

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
            crate::anonymizer::patterns::PIIType::ApiKey => PIIType::ApiKey,
            crate::anonymizer::patterns::PIIType::AccessToken => PIIType::AccessToken,
            crate::anonymizer::patterns::PIIType::SshKey => PIIType::SshKey,
            crate::anonymizer::patterns::PIIType::Domain => PIIType::Domain,
            crate::anonymizer::patterns::PIIType::Unknown => PIIType::Unknown,
        }
    }

    /// Detect PII without replacement
    pub fn detect_pii(&self, text: &str) -> Vec<DetectedPII> {
        let mut detected = Vec::new();

        for pattern in &self.patterns {
            for mat in pattern.pattern.find_iter(text) {
                if pattern.pii_type == crate::anonymizer::patterns::PIIType::Domain {
                    let domain_value = mat.as_str();
                    let clean_domain = self.extract_domain_from_match(domain_value);
                    if is_known_domain(&clean_domain, &self.custom_known_domains) {
                        continue;
                    }
                }

                detected.push(DetectedPII {
                    pii_type: self.map_pii_type(&pattern.pii_type),
                    value: mat.as_str().to_string(),
                    start: mat.start(),
                    end: mat.end(),
                    confidence: pattern.confidence,
                });
            }
        }

        debug!("Detected {} PII in text", detected.len());
        detected
    }

    /// Batch processing
    pub fn anonymize_batch(&self, requests: &[AnonymizeRequest]) -> Vec<AnonymizeResponse> {
        info!("Batch request: {} items", requests.len());
        requests.iter().map(|req| self.anonymize(req)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AnonymizerSettings, CustomPatternConfig};

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
                "api_key_aws".to_string(),
                "api_key_github".to_string(),
                "access_token_jwt".to_string(),
                "ssh_key_rsa".to_string(),
                "ssh_key_ed25519".to_string(),
                "domain_unknown".to_string(),
            ],
            custom_patterns: vec![],
            custom_known_domains: vec![],
        };
        AnonymizerEngine::new(&settings)
    }

    fn create_engine_with_custom(
        custom_patterns: Vec<CustomPatternConfig>,
        custom_domains: Vec<String>,
    ) -> AnonymizerEngine {
        let settings = AnonymizerSettings {
            default_strategy: "mask".to_string(),
            patterns: vec!["email".to_string(), "domain_unknown".to_string()],
            custom_patterns,
            custom_known_domains: custom_domains,
        };
        AnonymizerEngine::new(&settings)
    }

    #[test]
    fn test_detect_email() {
        let engine = create_test_engine();
        let text = "Contact: test@example.com for info";
        let detected = engine.detect_pii(text);

        assert!(detected.len() >= 1);
        assert!(detected.iter().any(|p| p.pii_type == PIIType::Email));
        assert_eq!(
            detected
                .iter()
                .find(|p| p.pii_type == PIIType::Email)
                .unwrap()
                .value,
            "test@example.com"
        );
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

    #[test]
    fn test_detect_aws_api_key() {
        let engine = create_test_engine();
        let text = "My AWS key is AKIAIOSFODNN7EXAMPLE for access";
        let detected = engine.detect_pii(text);

        assert!(detected.iter().any(|p| p.pii_type == PIIType::ApiKey));
    }

    #[test]
    fn test_detect_jwt_token() {
        let engine = create_test_engine();
        let text = "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U";
        let detected = engine.detect_pii(text);

        assert!(detected.iter().any(|p| p.pii_type == PIIType::AccessToken));
    }

    #[test]
    fn test_detect_ssh_key() {
        let engine = create_test_engine();
        let text = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQDVvvHkGphJbBX8rPnJqL3VzRmK";
        let detected = engine.detect_pii(text);

        assert!(detected.iter().any(|p| p.pii_type == PIIType::SshKey));
    }

    #[test]
    fn test_domain_masking_skips_known() {
        let engine = create_test_engine();
        let text =
            "Visit https://google.com for search and http://my-secret-server.ru for internal";
        let detected = engine.detect_pii(text);

        assert!(!detected
            .iter()
            .any(|p| p.pii_type == PIIType::Domain && p.value.contains("google.com")));

        assert!(detected
            .iter()
            .any(|p| p.pii_type == PIIType::Domain && p.value.contains("my-secret-server")));
    }

    #[test]
    fn test_anonymize_skips_known_domains() {
        let engine = create_test_engine();
        let request = AnonymizeRequest {
            text: "Search on google.com or visit my-private-company.ru".to_string(),
            strategy: None,
        };
        let response = engine.anonymize(&request);

        assert!(response.anonymized_text.contains("google.com"));
        assert!(
            !response.anonymized_text.contains("my-private-company.ru")
                || response.anonymized_text.contains("***")
        );
    }

    // Тесты кастомных паттернов
    #[test]
    fn test_custom_pattern() {
        let custom = vec![CustomPatternConfig {
            name: "custom_sku".to_string(),
            pii_type: "unknown".to_string(),
            pattern: r"\bSKU-\d{4,}\b".to_string(),
            confidence: 0.9,
        }];
        let engine = create_engine_with_custom(custom, vec![]);
        let detected = engine.detect_pii("Order SKU-12345 confirmed");

        assert_eq!(detected.len(), 1);
        assert_eq!(detected[0].value, "SKU-12345");
    }

    #[test]
    fn test_custom_known_domains_skip() {
        let engine = create_engine_with_custom(vec![], vec!["internal.corp".to_string()]);
        let detected =
            engine.detect_pii("Visit https://internal.corp/dashboard and https://unknown.site");

        // internal.corp пропущен
        assert!(!detected
            .iter()
            .any(|p| p.pii_type == PIIType::Domain && p.value.contains("internal.corp")));
        // unknown.site обнаружен
        assert!(detected
            .iter()
            .any(|p| p.pii_type == PIIType::Domain && p.value.contains("unknown.site")));
    }

    #[test]
    fn test_invalid_custom_pattern_skipped() {
        let custom = vec![CustomPatternConfig {
            name: "bad_regex".to_string(),
            pii_type: "unknown".to_string(),
            pattern: r"[invalid(".to_string(),
            confidence: 0.9,
        }];
        let engine = create_engine_with_custom(custom, vec![]);
        // bad_regex пропущен, email + domain из example.com
        let detected = engine.detect_pii("test@example.com");
        assert!(detected.len() >= 1);
        assert!(detected.iter().any(|p| p.pii_type == PIIType::Email));
    }
}

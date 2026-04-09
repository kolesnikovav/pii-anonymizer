use regex::Regex;
use serde::{Deserialize, Serialize};

/// Типы PII данных
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

/// Паттерн PII данных
#[derive(Debug, Clone)]
pub struct PIIPattern {
    pub pii_type: PIIType,
    pub name: String,
    pub pattern: Regex,
    pub confidence: f64,
}

impl PIIPattern {
    pub fn new(name: &str, pii_type: PIIType, pattern: &str) -> Option<Self> {
        Regex::new(pattern).ok().map(|pattern| Self {
            name: name.to_string(),
            pii_type,
            pattern,
            confidence: 0.9,
        })
    }

    pub fn with_confidence(name: &str, pii_type: PIIType, pattern: &str, confidence: f64) -> Option<Self> {
        Regex::new(pattern).ok().map(|pattern| Self {
            name: name.to_string(),
            pii_type,
            pattern,
            confidence,
        })
    }
}

/// Все доступные PII паттерны
pub fn get_all_patterns() -> Vec<PIIPattern> {
    vec![
        // Email
        PIIPattern::with_confidence(
            "email",
            PIIType::Email,
            r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}",
            0.98,
        ).unwrap(),
        
        // Телефон РФ
        PIIPattern::with_confidence(
            "phone_ru",
            PIIType::Phone,
            r"(?:\+7|8)[\s\-]?\(?\d{3}\)?[\s\-]?\d{3}[\s\-]?\d{2}[\s\-]?\d{2}",
            0.95,
        ).unwrap(),
        
        // Телефон международный
        PIIPattern::with_confidence(
            "phone_intl",
            PIIType::Phone,
            r"\+\d{1,3}[\s\-]?\(?\d{1,4}\)?[\s\-]?\d{1,4}[\s\-]?\d{1,4}[\s\-]?\d{1,4}",
            0.90,
        ).unwrap(),
        
        // Паспорт РФ
        PIIPattern::with_confidence(
            "passport_ru",
            PIIType::Passport,
            r"\b\d{4}[\s\-]?\d{6}\b",
            0.92,
        ).unwrap(),
        
        // Кредитная карта
        PIIPattern::with_confidence(
            "credit_card",
            PIIType::CreditCard,
            r"\b(?:\d{4}[\s\-]?){3}\d{4}\b",
            0.95,
        ).unwrap(),
        
        // IP адрес
        PIIPattern::with_confidence(
            "ip_address",
            PIIType::IpAddress,
            r"\b(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\b",
            0.99,
        ).unwrap(),
        
        // СНИЛС
        PIIPattern::with_confidence(
            "snils",
            PIIType::Snils,
            r"\b\d{3}[\s\-]?\d{3}[\s\-]?\d{3}[\s\-]?\d{2}\b",
            0.93,
        ).unwrap(),
        
        // ИНН физического лица
        PIIPattern::with_confidence(
            "inn_individual",
            PIIType::Inn,
            r"\b\d{12}\b",
            0.85,
        ).unwrap(),
        
        // ИНН юридического лица
        PIIPattern::with_confidence(
            "inn_legal",
            PIIType::Inn,
            r"\b\d{10}\b",
            0.85,
        ).unwrap(),
    ]
}

/// Получение паттерна по имени
pub fn get_pattern_by_name(name: &str) -> Option<PIIPattern> {
    get_all_patterns().into_iter().find(|p| p.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_pattern() {
        let patterns = get_all_patterns();
        let email_pattern = patterns.iter().find(|p| p.pii_type == PIIType::Email).unwrap();
        
        assert!(email_pattern.pattern.is_match("test@example.com"));
        assert!(email_pattern.pattern.is_match("user.name@domain.org"));
        assert!(!email_pattern.pattern.is_match("not-an-email"));
    }

    #[test]
    fn test_phone_pattern() {
        let patterns = get_all_patterns();
        let phone_pattern = patterns.iter().find(|p| p.name == "phone_ru").unwrap();
        
        assert!(phone_pattern.pattern.is_match("+7 (999) 123-45-67"));
        assert!(phone_pattern.pattern.is_match("8-999-123-45-67"));
        assert!(!phone_pattern.pattern.is_match("12345"));
    }

    #[test]
    fn test_passport_pattern() {
        let patterns = get_all_patterns();
        let passport_pattern = patterns.iter().find(|p| p.name == "passport_ru").unwrap();
        
        assert!(passport_pattern.pattern.is_match("1234 567890"));
        assert!(passport_pattern.pattern.is_match("1234567890"));
        assert!(!passport_pattern.pattern.is_match("12345"));
    }

    #[test]
    fn test_credit_card_pattern() {
        let patterns = get_all_patterns();
        let cc_pattern = patterns.iter().find(|p| p.name == "credit_card").unwrap();
        
        assert!(cc_pattern.pattern.is_match("1234-5678-9012-3456"));
        assert!(cc_pattern.pattern.is_match("1234567890123456"));
        assert!(!cc_pattern.pattern.is_match("12345"));
    }

    #[test]
    fn test_ip_address_pattern() {
        let patterns = get_all_patterns();
        let ip_pattern = patterns.iter().find(|p| p.name == "ip_address").unwrap();
        
        assert!(ip_pattern.pattern.is_match("192.168.1.1"));
        assert!(ip_pattern.pattern.is_match("10.0.0.1"));
        assert!(!ip_pattern.pattern.is_match("999.999.999.999"));
    }

    #[test]
    fn test_snils_pattern() {
        let patterns = get_all_patterns();
        let snils_pattern = patterns.iter().find(|p| p.name == "snils").unwrap();
        
        assert!(snils_pattern.pattern.is_match("123-456-789 01"));
        assert!(snils_pattern.pattern.is_match("12345678901"));
        assert!(!snils_pattern.pattern.is_match("12345"));
    }

    #[test]
    fn test_pii_type_display() {
        assert_eq!(PIIType::Email.to_string(), "EMAIL");
        assert_eq!(PIIType::Phone.to_string(), "PHONE");
        assert_eq!(PIIType::Snils.to_string(), "SNILS");
    }

    #[test]
    fn test_get_all_patterns_count() {
        let patterns = get_all_patterns();
        assert!(patterns.len() >= 7); // Минимум 7 паттернов
    }
}

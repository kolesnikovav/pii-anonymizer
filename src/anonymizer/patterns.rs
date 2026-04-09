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
    ApiKey,
    AccessToken,
    SshKey,
    Domain,
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
            PIIType::ApiKey => write!(f, "API_KEY"),
            PIIType::AccessToken => write!(f, "ACCESS_TOKEN"),
            PIIType::SshKey => write!(f, "SSH_KEY"),
            PIIType::Domain => write!(f, "DOMAIN"),
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
        
        // API ключи (различные форматы)
        PIIPattern::with_confidence(
            "api_key_generic",
            PIIType::ApiKey,
            r#"(?:api[_-]?key|apikey)[\s:="'"]+\s*([a-zA-Z0-9_\-]{20,})"#,
            0.92,
        ).unwrap(),
        
        PIIPattern::with_confidence(
            "api_key_bearer",
            PIIType::ApiKey,
            r#"(?:bearer|token)[\s]+([a-zA-Z0-9_\-\.]{20,})"#,
            0.90,
        ).unwrap(),
        
        PIIPattern::with_confidence(
            "api_key_aws",
            PIIType::ApiKey,
            r"AKIA[0-9A-Z]{16}",
            0.98,
        ).unwrap(),
        
        PIIPattern::with_confidence(
            "api_key_github",
            PIIType::ApiKey,
            r"gh[pousr]_[A-Za-z0-9_]{36,}",
            0.98,
        ).unwrap(),
        
        PIIPattern::with_confidence(
            "api_key_google",
            PIIType::ApiKey,
            r"AIza[0-9A-Za-z_\-]{35}",
            0.97,
        ).unwrap(),
        
        // Токены доступа
        PIIPattern::with_confidence(
            "access_token_jwt",
            PIIType::AccessToken,
            r"eyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}",
            0.96,
        ).unwrap(),
        
        PIIPattern::with_confidence(
            "access_token_generic",
            PIIType::AccessToken,
            r#"(?:access[_-]?token|auth[_-]?token)[\s:="'"]+\s*([a-zA-Z0-9_\-\.]{20,})"#,
            0.88,
        ).unwrap(),
        
        // SSH ключи
        PIIPattern::with_confidence(
            "ssh_key_rsa",
            PIIType::SshKey,
            r"ssh-rsa\s+[A-Za-z0-9+/]{20,}",
            0.98,
        ).unwrap(),
        
        PIIPattern::with_confidence(
            "ssh_key_ed25519",
            PIIType::SshKey,
            r"ssh-ed25519\s+[A-Za-z0-9+/]{20,}",
            0.98,
        ).unwrap(),
        
        PIIPattern::with_confidence(
            "ssh_key_ecdsa",
            PIIType::SshKey,
            r"ecdsa-sha2-nistp\d+\s+[A-Za-z0-9+/]{20,}",
            0.97,
        ).unwrap(),
        
        // Домены (кроме общеизвестных)
        PIIPattern::with_confidence(
            "domain_unknown",
            PIIType::Domain,
            r"(?:https?://)?(?:www\.)?([a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?\.(?:[a-zA-Z]{2,}))\b",
            0.75,
        ).unwrap(),
    ]
}

/// Получение паттерна по имени
pub fn get_pattern_by_name(name: &str) -> Option<PIIPattern> {
    get_all_patterns().into_iter().find(|p| p.name == name)
}

/// Список общеизвестных доменов, которые не нужно маскировать
const KNOWN_DOMAINS: &[&str] = &[
    "google.com", "google.ru", "google.org", "google.net",
    "yandex.ru", "yandex.com", "yandex.net", "ya.ru",
    "mail.ru", "mail.com",
    "gmail.com", "yahoo.com", "yahoo.ru",
    "outlook.com", "hotmail.com",
    "microsoft.com", "microsoftonline.com",
    "apple.com", "icloud.com",
    "amazon.com", "amazonaws.com",
    "github.com", "gitlab.com", "bitbucket.org",
    "stackoverflow.com",
    "wikipedia.org",
    "facebook.com", "fb.com", "instagram.com",
    "twitter.com", "x.com",
    "linkedin.com",
    "youtube.com", "youtu.be",
    "telegram.org", "t.me",
    "whatsapp.com",
    "docker.com", "hub.docker.com",
    "npmjs.com", "npmjs.org",
    "crates.io", "rust-lang.org",
];

/// Проверка, является ли домен общеизвестным
pub fn is_known_domain(domain: &str) -> bool {
    let domain_lower = domain.to_lowercase();
    KNOWN_DOMAINS.iter().any(|known| domain_lower == *known || domain_lower.ends_with(&format!(".{}", known)))
}

/// Фильтрация доменов - исключение общеизвестных
pub fn filter_known_domains(domains: Vec<String>) -> Vec<String> {
    domains.into_iter().filter(|d| !is_known_domain(d)).collect()
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
        assert!(patterns.len() >= 16); // Минимум 16 паттернов (включая новые)
    }

    #[test]
    fn test_api_key_aws_pattern() {
        let patterns = get_all_patterns();
        let aws_pattern = patterns.iter().find(|p| p.name == "api_key_aws").unwrap();
        
        assert!(aws_pattern.pattern.is_match("AKIAIOSFODNN7EXAMPLE"));
        assert!(!aws_pattern.pattern.is_match("AKIA123"));
    }

    #[test]
    fn test_api_key_github_pattern() {
        let patterns = get_all_patterns();
        let github_pattern = patterns.iter().find(|p| p.name == "api_key_github").unwrap();
        
        assert!(github_pattern.pattern.is_match("ghp_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefgh12"));
        assert!(!github_pattern.pattern.is_match("ghp_short"));
    }

    #[test]
    fn test_jwt_token_pattern() {
        let patterns = get_all_patterns();
        let jwt_pattern = patterns.iter().find(|p| p.name == "access_token_jwt").unwrap();
        
        assert!(jwt_pattern.pattern.is_match("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U"));
        assert!(!jwt_pattern.pattern.is_match("not.a.jwt"));
    }

    #[test]
    fn test_ssh_key_rsa_pattern() {
        let patterns = get_all_patterns();
        let ssh_pattern = patterns.iter().find(|p| p.name == "ssh_key_rsa").unwrap();
        
        assert!(ssh_pattern.pattern.is_match("ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQDVvvHkGphJbBX8"));
        assert!(!ssh_pattern.pattern.is_match("ssh-rsa short"));
    }

    #[test]
    fn test_ssh_key_ed25519_pattern() {
        let patterns = get_all_patterns();
        let ssh_pattern = patterns.iter().find(|p| p.name == "ssh_key_ed25519").unwrap();
        
        assert!(ssh_pattern.pattern.is_match("ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIG"));
        assert!(!ssh_pattern.pattern.is_match("ssh-ed25519 short"));
    }

    #[test]
    fn test_is_known_domain() {
        assert!(is_known_domain("google.com"));
        assert!(is_known_domain("yandex.ru"));
        assert!(is_known_domain("github.com"));
        assert!(is_known_domain("mail.google.com"));
        assert!(!is_known_domain("my-private-company.com"));
        assert!(!is_known_domain("secret-server.ru"));
    }

    #[test]
    fn test_filter_known_domains() {
        let domains = vec![
            "google.com".to_string(),
            "my-secret-server.com".to_string(),
            "yandex.ru".to_string(),
            "internal-api.company.ru".to_string(),
        ];
        
        let filtered = filter_known_domains(domains);
        assert_eq!(filtered.len(), 2);
        assert!(filtered.contains(&"my-secret-server.com".to_string()));
        assert!(filtered.contains(&"internal-api.company.ru".to_string()));
    }
}

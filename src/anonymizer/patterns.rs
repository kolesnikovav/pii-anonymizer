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
    pub fn with_confidence(name: &str, pii_type: PIIType, pattern: &str, confidence: f64) -> Option<Self> {
        Regex::new(pattern).ok().map(|pattern| Self {
            name: name.to_string(),
            pii_type,
            pattern,
            confidence,
        })
    }
    
    /// Создание паттерна из строкового типа PII
    pub fn from_config(name: &str, pii_type: &str, pattern: &str, confidence: f64) -> Result<Self, String> {
        let regex = Regex::new(pattern).map_err(|e| format!("Невалидный regex: {}", e))?;
        
        let pii_type_enum = match pii_type.to_lowercase().as_str() {
            "email" => PIIType::Email,
            "phone" => PIIType::Phone,
            "passport" => PIIType::Passport,
            "credit_card" => PIIType::CreditCard,
            "ip_address" => PIIType::IpAddress,
            "snils" => PIIType::Snils,
            "inn" => PIIType::Inn,
            "address" => PIIType::Address,
            "full_name" => PIIType::FullName,
            "api_key" => PIIType::ApiKey,
            "access_token" => PIIType::AccessToken,
            "ssh_key" => PIIType::SshKey,
            "domain" => PIIType::Domain,
            _ => PIIType::Unknown,
        };
        
        Ok(Self {
            name: name.to_string(),
            pii_type: pii_type_enum,
            pattern: regex,
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

/// Список общеизвестных доменов, которые не нужно маскировать
const BUILTIN_KNOWN_DOMAINS: &[&str] = &[
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

/// Получить встроенные известные домены
pub fn get_builtin_domains() -> Vec<String> {
    BUILTIN_KNOWN_DOMAINS.iter().map(|s| s.to_string()).collect()
}

/// Проверка, является ли домен общеизвестным
pub fn is_known_domain(domain: &str, custom_domains: &[String]) -> bool {
    let domain_lower = domain.to_lowercase();
    
    // Проверка встроенных доменов
    if BUILTIN_KNOWN_DOMAINS.iter().any(|known| domain_lower == *known || domain_lower.ends_with(&format!(".{}", known))) {
        return true;
    }
    
    // Проверка кастомных доменов
    if custom_domains.iter().any(|known| {
        let known_lower = known.to_lowercase();
        domain_lower == known_lower || domain_lower.ends_with(&format!(".{}", known_lower))
    }) {
        return true;
    }
    
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_pattern() {
        let patterns = get_all_patterns();
        let email_pattern = patterns.iter().find(|p| p.pii_type == PIIType::Email).unwrap();
        assert!(email_pattern.pattern.is_match("test@example.com"));
    }

    #[test]
    fn test_phone_pattern() {
        let patterns = get_all_patterns();
        let phone_pattern = patterns.iter().find(|p| p.name == "phone_ru").unwrap();
        assert!(phone_pattern.pattern.is_match("+7 (999) 123-45-67"));
    }

    #[test]
    fn test_api_key_aws_pattern() {
        let patterns = get_all_patterns();
        let aws_pattern = patterns.iter().find(|p| p.name == "api_key_aws").unwrap();
        assert!(aws_pattern.pattern.is_match("AKIAIOSFODNN7EXAMPLE"));
    }

    #[test]
    fn test_jwt_token_pattern() {
        let patterns = get_all_patterns();
        let jwt_pattern = patterns.iter().find(|p| p.name == "access_token_jwt").unwrap();
        assert!(jwt_pattern.pattern.is_match("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U"));
    }

    #[test]
    fn test_ssh_key_rsa_pattern() {
        let patterns = get_all_patterns();
        let ssh_pattern = patterns.iter().find(|p| p.name == "ssh_key_rsa").unwrap();
        assert!(ssh_pattern.pattern.is_match("ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQDVvvHkGphJbBX8"));
    }

    #[test]
    fn test_is_known_domain() {
        assert!(is_known_domain("google.com", &[]));
        assert!(is_known_domain("my-private-company.com", &[]) == false);
        // Custom domain
        assert!(is_known_domain("internal.corp", &["internal.corp".to_string()]));
        assert!(is_known_domain("unknown.site", &["internal.corp".to_string()]) == false);
    }
}

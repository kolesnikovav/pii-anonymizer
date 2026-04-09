use serde::{Deserialize, Serialize};

/// Стратегии анонимизации
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnonymizationStrategy {
    /// Замена символов на mask_char
    Mask,
    /// Хэширование значения
    Hash,
    /// Полная замена на placeholder
    Replace,
    /// Полное удаление (redaction)
    Redact,
}

impl AnonymizationStrategy {
    pub fn from_str(s: &str) -> Self {
        match s {
            "mask" => Self::Mask,
            "hash" => Self::Hash,
            "replace" => Self::Replace,
            "redact" => Self::Redact,
            _ => Self::Mask,
        }
    }

    /// Применение стратегии к значению
    pub fn apply(&self, value: &str, pii_type: &str, mask_char: char, mask_length: usize) -> String {
        match self {
            AnonymizationStrategy::Mask => mask_char.to_string().repeat(mask_length),
            AnonymizationStrategy::Hash => format!("[HASH:{}]", self.simple_hash(value)),
            AnonymizationStrategy::Replace => format!("[{}]", pii_type.to_uppercase()),
            AnonymizationStrategy::Redact => String::new(),
        }
    }

    fn simple_hash(&self, input: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_strategy() {
        let strategy = AnonymizationStrategy::Mask;
        let result = strategy.apply("test@example.com", "email", '*', 10);
        assert_eq!(result, "**********");
    }

    #[test]
    fn test_hash_strategy() {
        let strategy = AnonymizationStrategy::Hash;
        let result = strategy.apply("test", "email", '*', 10);
        assert!(result.starts_with("[HASH:"));
        assert!(result.ends_with("]"));
    }

    #[test]
    fn test_replace_strategy() {
        let strategy = AnonymizationStrategy::Replace;
        let result = strategy.apply("test@example.com", "email", '*', 10);
        assert_eq!(result, "[EMAIL]");
    }

    #[test]
    fn test_redact_strategy() {
        let strategy = AnonymizationStrategy::Redact;
        let result = strategy.apply("secret", "email", '*', 10);
        assert_eq!(result, "");
    }

    #[test]
    fn test_from_str() {
        assert_eq!(AnonymizationStrategy::from_str("mask"), AnonymizationStrategy::Mask);
        assert_eq!(AnonymizationStrategy::from_str("hash"), AnonymizationStrategy::Hash);
        assert_eq!(AnonymizationStrategy::from_str("replace"), AnonymizationStrategy::Replace);
        assert_eq!(AnonymizationStrategy::from_str("redact"), AnonymizationStrategy::Redact);
        assert_eq!(AnonymizationStrategy::from_str("unknown"), AnonymizationStrategy::Mask);
    }
}

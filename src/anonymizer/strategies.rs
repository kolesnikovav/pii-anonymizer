use serde::{Deserialize, Serialize};

/// Стратегии анонимизации
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnonymizationStrategy {
    /// Полная замена на placeholder [EMAIL_1], [PHONE_2] и т.д.
    Replace,
    /// Частичная маска: a***2.com (первые 1-2 символа + *** + последние 1-2 символа)
    Mask,
    /// Частичный хеш с сохранением контекста: a_45454545.com
    Hash,
}

impl AnonymizationStrategy {
    pub fn from_str(s: &str) -> Self {
        match s {
            "replace" => Self::Replace,
            "mask" => Self::Mask,
            "hash" => Self::Hash,
            _ => Self::Replace,
        }
    }

    /// Применение стратегии к значению
    pub fn apply(&self, value: &str, pii_type: &str, counter: usize) -> String {
        match self {
            // Replace: [EMAIL_1], [PHONE_2]
            AnonymizationStrategy::Replace => format!("[{}_{}]", pii_type.to_uppercase(), counter),
            
            // Mask: a***2.com - сохраняем первые и последние символы
            AnonymizationStrategy::Mask => self.partial_mask(value),
            
            // Hash: a_45454545.com - хешируем среднюю часть, сохраняем контекст
            AnonymizationStrategy::Hash => self.partial_hash(value),
        }
    }

    /// Частичная маска: сохраняем первые 1-2 и последние 1-2 символа
    fn partial_mask(&self, value: &str) -> String {
        let len = value.chars().count();
        
        if len <= 4 {
            // Для коротких значений - просто маска
            return "*".repeat(len);
        }

        // Для email - особая обработка: сохраняем часть local + @ + часть домена
        if let Some(at_pos) = value.find('@') {
            let local_part = &value[..at_pos];
            let domain_with_at = &value[at_pos..]; // включает @
            
            let local_prefix = if local_part.len() >= 2 { &local_part[..2] } else { local_part };
            // Сохраняем @ и часть домена: "@company.org" -> "@" + последние 2 символа
            let domain_suffix = if domain_with_at.len() >= 3 { 
                &domain_with_at[domain_with_at.len()-2..] 
            } else { 
                domain_with_at 
            };
            
            return format!("{}***@***{}", local_prefix, domain_suffix);
        }

        // Определяем границы для разных типов PII
        let (prefix_len, suffix_len) = self.get_mask_boundaries(value, len);
        
        let prefix: String = value.chars().take(prefix_len).collect();
        let suffix: String = value.chars().skip(len - suffix_len).collect();
        
        format!("{}***{}", prefix, suffix)
    }

    /// Частичный хеш: сохраняем первые символы + хеш + последние символы
    fn partial_hash(&self, value: &str) -> String {
        let len = value.chars().count();
        
        if len <= 4 {
            return format!("_{}", self.simple_hash(value).chars().take(6).collect::<String>());
        }

        // Для email - особая обработка: сохраняем часть local + хеш + @ + часть домена
        if let Some(at_pos) = value.find('@') {
            let local_part = &value[..at_pos];
            let domain_with_at = &value[at_pos..]; // включает @
            
            let local_prefix = if local_part.len() >= 2 { &local_part[..2] } else { local_part };
            let domain_suffix = if domain_with_at.len() >= 3 { 
                &domain_with_at[domain_with_at.len()-2..] 
            } else { 
                domain_with_at 
            };
            let hash = self.simple_hash(value).chars().take(8).collect::<String>();
            
            return format!("{}_{}@{}", local_prefix, hash, domain_suffix);
        }

        // Для не-email используем обычные границы
        let (prefix_len, suffix_len) = self.get_mask_boundaries(value, len);
        
        let prefix: String = value.chars().take(prefix_len).collect();
        let suffix: String = value.chars().skip(len - suffix_len).collect();
        let hash = self.simple_hash(value).chars().take(8).collect::<String>();
        
        format!("{}_{}{}", prefix, hash, suffix)
    }

    /// Определение границ маскирования для разных типов PII
    fn get_mask_boundaries(&self, value: &str, len: usize) -> (usize, usize) {
        // Для email сохраняем локальную часть и домен
        if value.contains('@') {
            if let Some(at_pos) = value.find('@') {
                let prefix_len = if at_pos >= 2 { 2 } else { at_pos };
                let domain_part = &value[at_pos..];
                let suffix_len = if domain_part.len() >= 2 { 2 } else { domain_part.len() };
                return (prefix_len, suffix_len);
            }
        }
        
        // Для телефонов сохраняем код страны и последние 2 цифры
        if value.starts_with('+') || value.starts_with('8') {
            return (3, 2);
        }
        
        // Для API ключей и токенов - первые 4 и последние 4
        if len >= 32 {
            return (4, 4);
        }
        
        // Для SSH ключей - первые 8 и последние 8
        if len >= 100 {
            return (8, 8);
        }
        
        // Для доменов - первые 3 и последние 3
        if value.contains('.') && !value.contains('@') {
            return (3.min(len), 3.min(len));
        }
        
        // Для паспортов - первые 2 и последние 2
        if len == 10 || len == 11 {
            return (2, 2);
        }
        
        // Для кредитных карт - первые 4 и последние 4
        if len == 16 || len == 19 {
            return (4, 4);
        }
        
        // По умолчанию: первые 2 и последние 2
        (2.min(len), 2.min(len))
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
    fn test_replace_strategy() {
        let strategy = AnonymizationStrategy::Replace;
        
        assert_eq!(strategy.apply("test@example.com", "email", 1), "[EMAIL_1]");
        assert_eq!(strategy.apply("+79991234567", "phone", 2), "[PHONE_2]");
        assert_eq!(strategy.apply("192.168.1.1", "ip_address", 3), "[IP_ADDRESS_3]");
    }

    #[test]
    fn test_mask_strategy_email() {
        let strategy = AnonymizationStrategy::Mask;
        let result = strategy.apply("test@example.com", "email", 0);
        
        assert!(result.starts_with("te"));
        assert!(result.ends_with("om"));
        assert!(result.contains("***"));
        // Результат: te***om
    }

    #[test]
    fn test_mask_strategy_phone() {
        let strategy = AnonymizationStrategy::Mask;
        let result = strategy.apply("+79991234567", "phone", 0);
        
        assert!(result.starts_with("+79"));
        assert!(result.ends_with("67"));
        assert!(result.contains("***"));
    }

    #[test]
    fn test_mask_strategy_short() {
        let strategy = AnonymizationStrategy::Mask;
        let result = strategy.apply("ab", "unknown", 0);
        
        assert_eq!(result, "**");
    }

    #[test]
    fn test_hash_strategy_email() {
        let strategy = AnonymizationStrategy::Hash;
        let result = strategy.apply("test@example.com", "email", 0);
        
        // Hash сохраняет начало и конец с хешем в середине
        assert!(result.contains("_"));
        assert!(result.contains("@"));
    }

    #[test]
    fn test_hash_strategy_phone() {
        let strategy = AnonymizationStrategy::Hash;
        let result = strategy.apply("+79991234567", "phone", 0);
        
        assert!(result.starts_with("+79"));
        assert!(result.ends_with("67"));
        // Содержит хеш в середине
        assert!(result.contains("_"));
    }

    #[test]
    fn test_hash_strategy_short() {
        let strategy = AnonymizationStrategy::Hash;
        let result = strategy.apply("ab", "unknown", 0);
        
        assert!(result.starts_with("_"));
        assert_eq!(result.len(), 7); // _ + 6 символов хеша
    }

    #[test]
    fn test_mask_preserves_email_structure() {
        let strategy = AnonymizationStrategy::Mask;
        let result = strategy.apply("john.doe@company.org", "email", 0);
        
        // Маска сохраняет структуру email
        assert!(result.contains("@"));
        assert!(result.contains("***"));
    }

    #[test]
    fn test_hash_preserves_email_structure() {
        let strategy = AnonymizationStrategy::Hash;
        let result = strategy.apply("john.doe@company.org", "email", 0);
        
        // Hash должен сохранить часть email и добавить хеш
        assert!(!result.is_empty());
        assert!(result.contains("_"));
    }

    #[test]
    fn test_from_str() {
        assert_eq!(AnonymizationStrategy::from_str("replace"), AnonymizationStrategy::Replace);
        assert_eq!(AnonymizationStrategy::from_str("mask"), AnonymizationStrategy::Mask);
        assert_eq!(AnonymizationStrategy::from_str("hash"), AnonymizationStrategy::Hash);
        assert_eq!(AnonymizationStrategy::from_str("unknown"), AnonymizationStrategy::Replace);
    }

    #[test]
    fn test_counter_in_replace() {
        let strategy = AnonymizationStrategy::Replace;
        
        let r1 = strategy.apply("email1@test.com", "email", 1);
        let r2 = strategy.apply("email2@test.com", "email", 2);
        let r3 = strategy.apply("email3@test.com", "email", 3);
        
        assert_eq!(r1, "[EMAIL_1]");
        assert_eq!(r2, "[EMAIL_2]");
        assert_eq!(r3, "[EMAIL_3]");
    }
}

#[derive(Debug, Clone)]
pub enum AnonymizationStrategy {
    Mask,
    Hash,
    Replace,
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
}

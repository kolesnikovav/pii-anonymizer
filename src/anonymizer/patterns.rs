use regex::Regex;

#[derive(Debug, Clone)]
pub struct PIIPattern {
    pub name: String,
    pub pattern: Regex,
}

impl PIIPattern {
    pub fn new(name: &str, pattern: &str) -> Option<Self> {
        Regex::new(pattern).ok().map(|pattern| Self {
            name: name.to_string(),
            pattern,
        })
    }
}

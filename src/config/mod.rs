use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub server: ServerSettings,
    #[serde(flatten)]
    pub anonymizer: AnonymizerSettings,
    pub mcp: McpSettings,
    pub proxy: ProxySettings,
    pub logging: LoggingSettings,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerSettings {
    pub host: String,
    pub port: u16,
    pub workers: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AnonymizerSettings {
    pub default_strategy: String,
    pub patterns: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct McpSettings {
    pub enabled: bool,
    pub transport: String,
    pub server_name: String,
    pub server_version: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UpstreamServer {
    pub name: String,
    pub url: String,
    pub timeout: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ProxySettings {
    pub enabled: bool,
    pub upstream_servers: Vec<UpstreamServer>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingSettings {
    pub level: String,
    pub format: String,
}

impl Settings {
    /// Загрузка конфигурации из файла
    pub fn from_file(path: &str) -> Result<Self, ConfigError> {
        let config = Config::builder()
            .add_source(File::with_name(path).required(true))
            .add_source(
                Environment::with_prefix("ANONYMIZER")
                    .separator("__"),
            )
            .build()?;

        config.try_deserialize()
    }
}

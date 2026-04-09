use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub server: ServerSettings,
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
    pub mask_char: char,
    pub mask_length: usize,
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
    pub fn new() -> Result<Self, ConfigError> {
        let config = Config::builder()
            // Загрузка из файла конфигурации
            .add_source(File::with_name("config/settings").required(false))
            // Переопределение из переменных окружения
            .add_source(
                Environment::with_prefix("ANONYMIZER")
                    .prefix_separator("_")
                    .separator("__"),
            )
            .build()?;

        config.try_deserialize()
    }
}

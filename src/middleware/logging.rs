use tracing::info;

#[derive(Debug, Clone)]
pub struct RequestLogger;

impl RequestLogger {
    pub fn new() -> Self {
        info!("Request Logger инициализирован");
        Self
    }
}

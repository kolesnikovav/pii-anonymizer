use tracing::info;

#[derive(Debug, Clone)]
pub struct SseEmitter;

impl SseEmitter {
    pub fn new() -> Self {
        info!("SSE Emitter инициализирован");
        Self
    }
}

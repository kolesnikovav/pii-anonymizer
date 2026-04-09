use tokio::signal;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod config;
mod anonymizer;
mod api;
mod mcp;
mod sse;
mod middleware;
mod models;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Инициализация логирования
    init_logging();
    
    info!("🚀 Запуск PII Anonymizer MCP Server");
    
    // Загрузка конфигурации
    let settings = config::Settings::new()?;
    info!("📋 Конфигурация загружена: {}:{}", settings.server.host, settings.server.port);
    
    // Инициализация анонимизатора
    let anonymizer = anonymizer::AnonymizerEngine::new(&settings.anonymizer);
    info!("🔍 Анонимизатор инициализирован");
    
    // Создание и запуск HTTP сервера
    let app = api::create_router(settings.clone(), anonymizer);
    
    let addr = format!("{}:{}", settings.server.host, settings.server.port);
    info!("🌐 HTTP сервер слушает на {}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    
    Ok(())
}

fn init_logging() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .pretty()
        .init();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Не удалось установить обработчик Ctrl+C");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Не удалось установить обработчик SIGTERM")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("🛑 Получен сигнал завершения, graceful shutdown...");
}

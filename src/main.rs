use clap::Parser;
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

/// PII Anonymizer MCP Server
#[derive(Parser, Debug)]
#[command(name = "pii-anonymizer")]
#[command(about = "Сервис для анонимизации поисковых запросов с удалением PII данных", long_about = None)]
struct Args {
    /// Путь к файлу конфигурации
    #[arg(short, long, default_value = "config/settings.yaml")]
    config: String,

    /// Хост сервера (переопределяет конфиг)
    #[arg(long)]
    host: Option<String>,

    /// Порт сервера (переопределяет конфиг)
    #[arg(long)]
    port: Option<u16>,

    /// Стратегия анонимизации (replace, mask, hash)
    #[arg(short, long)]
    strategy: Option<String>,

    /// Режим MCP (stdio, http)
    #[arg(long, default_value = "http")]
    mcp_mode: String,

    /// Уровень логирования (trace, debug, info, warn, error)
    #[arg(long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Инициализация логирования
    init_logging(&args.log_level);
    
    info!("🚀 Запуск PII Anonymizer MCP Server");
    info!("📋 Конфигурация: {}", args.config);

    // Загрузка конфигурации из файла
    let mut settings = config::Settings::from_file(&args.config)?;
    
    // Переопределение из CLI аргументов
    if let Some(ref host) = args.host {
        settings.server.host = host.clone();
    }
    if let Some(port) = args.port {
        settings.server.port = port;
    }
    if let Some(ref strategy) = args.strategy {
        settings.anonymizer.default_strategy = strategy.clone();
    }

    info!("⚙️  Настройки: {}:{}, стратегия: {}", 
        settings.server.host, 
        settings.server.port,
        settings.anonymizer.default_strategy
    );
    
    // Инициализация анонимизатора
    let anonymizer = anonymizer::AnonymizerEngine::new(&settings.anonymizer);
    info!("🔍 Анонимизатор инициализирован");

    // Инициализация MCP сервера
    let mcp_server = mcp::McpServer::new(
        anonymizer.clone(),
        &settings.mcp.server_name,
        &settings.mcp.server_version,
    );
    info!("🤖 MCP Server инициализирован: {} v{}", 
        settings.mcp.server_name, 
        settings.mcp.server_version
    );

    // Режим MCP
    if args.mcp_mode == "stdio" {
        info!("🤖 Запуск в режиме MCP stdio");

        info!("MCP инструменты:");
        let tools = mcp_server.get_tools();
        if let Some(tools_arr) = tools.get("tools").and_then(|t| t.as_array()) {
            for tool in tools_arr {
                if let Some(name) = tool.get("name").and_then(|n| n.as_str()) {
                    info!("  - {}", name);
                }
            }
        }

        // В режиме stdio просто ждём
        shutdown_signal().await;
        return Ok(());
    }

    // HTTP режим - создаём основной роутер
    let app = api::create_router(settings.clone(), anonymizer);

    // Создаём MCP SSE роутер
    let mcp_state = sse::mcp_handler::SseMcpState {
        mcp_server: mcp_server.clone(),
    };
    let mcp_router = sse::create_mcp_router(mcp_state);

    // Объединяем роутеры
    let app = app.merge(mcp_router);

    let addr = format!("{}:{}", settings.server.host, settings.server.port);
    info!("🌐 HTTP сервер слушает на {}", addr);
    info!("📖 Документация API: http://{}/api/v1/health", addr);
    info!("🤖 MCP SSE endpoint: http://{}/sse", addr);
    info!("📨 MCP message endpoint: http://{}/sse/message", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    
    Ok(())
}

fn init_logging(log_level: &str) {
    let level = match log_level.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    FmtSubscriber::builder()
        .with_max_level(level)
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

use clap::Parser;
use tokio::signal;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod config;
mod anonymizer;
mod api;
mod mcp;
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

    // Режим MCP stdio
    if args.mcp_mode == "stdio" {
        info!("🤖 Запуск в режиме MCP stdio");
        run_mcp_stdio(anonymizer).await?;
        return Ok(());
    }

    // HTTP режим
    run_http_server(settings, anonymizer).await?;

    Ok(())
}

/// Запуск MCP в режиме stdio (через rmcp io transport)
async fn run_mcp_stdio(engine: anonymizer::AnonymizerEngine) -> Result<(), Box<dyn std::error::Error>> {
    use rmcp::service::RunningService;
    use rmcp::ServiceExt;

    let service = mcp::server::AnonymizerService::new(engine);
    let transport = rmcp::transport::stdio();
    let server: RunningService<_, _> = service.serve(transport).await?;
    server.waiting().await?;
    Ok(())
}

/// Запуск HTTP сервера с кастомным SSE транспортом для MCP
async fn run_http_server(
    settings: config::Settings,
    engine: anonymizer::AnonymizerEngine,
) -> Result<(), Box<dyn std::error::Error>> {
    use axum::Router;

    // Создаём MCP сервис с поддержкой прокси
    let mut mcp_service = mcp::ProxyMcpService::new(engine.clone());

    // Подключаемся к внешним MCP серверам (если есть в конфигурации)
    if !settings.proxy.upstream_servers.is_empty() {
        info!("🔌 Подключение к {} внешним MCP серверам...", settings.proxy.upstream_servers.len());

        let mut connections = Vec::new();

        for (name, mut config) in settings.proxy.upstream_servers {
            if !config.enabled {
                info!("   ⊘ {} отключён, пропускаем", name);
                continue;
            }

            // Подставляем GITHUB_TOKEN из окружения если есть
            if let Some(github_token) = std::env::var("GITHUB_TOKEN").ok().filter(|s| !s.is_empty()) {
                if config.env.contains_key("GITHUB_PERSONAL_ACCESS_TOKEN") {
                    config.env.insert("GITHUB_PERSONAL_ACCESS_TOKEN".to_string(), github_token);
                    info!("   🔑 GITHUB_TOKEN подставлен из окружения");
                }
            }

            match mcp::McpUpstreamConnection::connect(name.clone(), &config).await {
                Ok(conn) => {
                    info!("   ✅ {} подключён ({} инструментов)", name, conn.tools.len());
                    connections.push(conn);
                }
                Err(e) => {
                    info!("   ❌ {} ошибка: {}", name, e);
                }
            }
        }

        if !connections.is_empty() {
            let proxy_manager = mcp::McpProxyManager::new(connections);
            let anonymizing_proxy = mcp::AnonymizingProxy::new(proxy_manager, engine.clone());
            mcp_service.set_proxy(anonymizing_proxy);
        }
    }

    // Оборачиваем в Arc чтобы не клонировать
    let mcp_service = std::sync::Arc::new(mcp_service);

    // Создаём Axum роутеры
    let health_app = Router::new()
        .route("/api/v1/health", axum::routing::get(|| async { "OK" }));

    // SSE MCP роутер — передаём Arc
    let sse_app = mcp::sse_transport::create_sse_router_arc(mcp_service);

    // Объединяем в один роутер
    let app = Router::new()
        .merge(sse_app)
        .nest("/health", health_app);

    // Запускаем сервер
    let bind_addr = format!("{}:{}", settings.server.host, settings.server.port);
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;

    info!("🤖 MCP SSE сервер запущен на {}", bind_addr);
    info!("📡 SSE endpoint: http://{}/sse", bind_addr);
    info!("📨 Message endpoint: http://{}/message", bind_addr);
    info!("🏥 Health endpoint: http://{}:{}/api/v1/health", settings.server.host, settings.server.port);

    info!("MCP сервер готов к подключению клиентов");

    axum::serve(listener, app)
        .await?;

    shutdown_signal().await;
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

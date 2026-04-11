// Suppress warnings for modules that declare items not directly used in main
#![allow(unused_imports, dead_code)]

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

    /// Проверка конфигурации и выход (аналог nginx -t)
    #[arg(long)]
    config_test: bool,

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

    // Проверка конфигурации (аналог nginx -t)
    if args.config_test {
        return run_config_test(&settings);
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

        for (name, config) in settings.proxy.upstream_servers.iter() {
            if !config.enabled {
                info!("   ⊘ {} отключён, пропускаем", name);
                continue;
            }

            // Подставляем переменные окружения из окружения процесса
            // Ключи в config.env могут быть в любом регистре (serde нормализует)
            let mut config_clone = (*config).clone();
            for (key, value) in &mut config_clone.env {
                if value.is_empty() {
                    let key_upper = key.to_uppercase();
                    for env_key in [key.as_str(), key_upper.as_str()] {
                        if let Ok(env_val) = std::env::var(env_key) {
                            *value = env_val;
                            info!("   🔑 {} '{}' ← ${}", name, key, env_key);
                            break;
                        }
                    }
                }
            }

            match mcp::McpUpstreamConnection::connect(name.clone(), &config_clone).await {
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
            let mut anonymizing_proxy = mcp::AnonymizingProxy::new(proxy_manager, engine.clone());

            // Устанавливаем правила анонимизации для каждого сервера
            for (name, config) in &settings.proxy.upstream_servers {
                if !config.anonymize_fields.is_empty() {
                    let rules = mcp::ServerAnonymizationRules {
                        tool_fields: config.anonymize_fields.clone(),
                    };
                    anonymizing_proxy.set_rules(name.clone(), rules);
                }
            }

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

/// Проверка конфигурации (аналог nginx -t)
fn run_config_test(settings: &config::Settings) -> Result<(), Box<dyn std::error::Error>> {
    let mut errors: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    println!("🔍 Проверка конфигурации:");
    println!();

    // 1. Стратегия
    match settings.anonymizer.default_strategy.as_str() {
        "replace" | "mask" | "hash" => {
            println!("  ✅ Стратегия: {}", settings.anonymizer.default_strategy);
        }
        other => {
            errors.push(format!("Неизвестная стратегия '{}'. Допустимые: replace, mask, hash", other));
        }
    }

    // 2. Встроенные паттерны
    let builtin_patterns = anonymizer::patterns::get_all_patterns();
    let enabled_names: std::collections::HashSet<String> = settings.anonymizer.patterns.iter().cloned().collect();
    
    for name in &enabled_names {
        let found = builtin_patterns.iter().any(|p| &p.name == name);
        if found {
            println!("  ✅ Паттерн: {}", name);
        } else {
            warnings.push(format!("Встроенный паттерн '{}' не найден", name));
        }
    }

    // 3. Кастомные паттерны
    for cp in &settings.anonymizer.custom_patterns {
        match anonymizer::patterns::PIIPattern::from_config(&cp.name, &cp.pii_type, &cp.pattern, cp.confidence) {
            Ok(p) => {
                println!("  ✅ Кастомный паттерн: {} ({}), confidence={:.2}", cp.name, p.pii_type, cp.confidence);
            }
            Err(e) => {
                errors.push(format!("Кастомный паттерн '{}': {}", cp.name, e));
            }
        }
    }

    // 4. Кастомные домены
    if !settings.anonymizer.custom_known_domains.is_empty() {
        for domain in &settings.anonymizer.custom_known_domains {
            if domain.is_empty() {
                warnings.push("Пустой домен в custom_known_domains".to_string());
            } else {
                println!("  ✅ Домен (пропуск): {}", domain);
            }
        }
    }

    // 5. Proxy upstream серверы
    for (name, config) in &settings.proxy.upstream_servers {
        if config.enabled {
            let transport_str = match config.transport {
                mcp::client::McpTransport::Stdio => "stdio",
                mcp::client::McpTransport::Http => "http",
            };
            println!("  ✅ Proxy сервер: {} ({})", name, transport_str);
        }
    }

    // 6. Server
    println!("  ✅ Сервер: {}:{}", settings.server.host, settings.server.port);

    println!();

    // Итог
    if errors.is_empty() && warnings.is_empty() {
        println!("✅ Конфигурация валидна");
        Ok(())
    } else {
        if !warnings.is_empty() {
            println!("⚠️  Предупреждения ({}):", warnings.len());
            for w in &warnings {
                println!("   ⚠️  {}", w);
            }
            println!();
        }
        if !errors.is_empty() {
            println!("❌ Ошибки ({}):", errors.len());
            for e in &errors {
                println!("   ❌ {}", e);
            }
            println!();
            println!("❌ Конфигурация невалидна");
            return Err(format!("{} ошибок конфигурации", errors.len()).into());
        }
        println!("⚠️  Конфигурация валидна с предупреждениями");
        Ok(())
    }
}

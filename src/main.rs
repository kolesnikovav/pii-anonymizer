// Suppress warnings for modules that declare items not directly used in main
#![allow(unused_imports, dead_code)]

use clap::Parser;
use tokio::signal;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod anonymizer;
mod api;
mod config;
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

    info!("Starting PII Anonymizer MCP Server");
    info!("Config: {}", args.config);

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

    // Initialize anonymizer
    let anonymizer = anonymizer::AnonymizerEngine::new(&settings.anonymizer);
    info!("Anonymizer initialized");

    // Print status tree for all startup modes
    print_status_tree(&settings, &[], &[]);

    // Config validation (like nginx -t)
    if args.config_test {
        println!("\nConfiguration is valid");
        return Ok(());
    }

    // MCP stdio mode
    if args.mcp_mode == "stdio" {
        info!("Running in MCP stdio mode");
        run_mcp_stdio(anonymizer).await?;
        return Ok(());
    }

    // HTTP mode
    run_http_server(settings, anonymizer).await?;

    Ok(())
}

/// MCP stdio mode (via rmcp io transport)
async fn run_mcp_stdio(
    engine: anonymizer::AnonymizerEngine,
) -> Result<(), Box<dyn std::error::Error>> {
    use rmcp::service::RunningService;
    use rmcp::ServiceExt;

    let service = mcp::server::AnonymizerService::new(engine);
    let transport = rmcp::transport::stdio();
    let server: RunningService<_, _> = service.serve(transport).await?;
    server.waiting().await?;
    Ok(())
}

/// HTTP server with custom SSE transport for MCP
async fn run_http_server(
    settings: config::Settings,
    engine: anonymizer::AnonymizerEngine,
) -> Result<(), Box<dyn std::error::Error>> {
    use axum::Router;

    // Create MCP service with proxy support
    let mut mcp_service = mcp::ProxyMcpService::new(engine.clone());

    // Connect to external MCP servers (if configured)
    if !settings.proxy.upstream_servers.is_empty() {
        info!(
            "Connecting to {} external MCP servers...",
            settings.proxy.upstream_servers.len()
        );

        let mut connections = Vec::new();

        for (name, config) in settings.proxy.upstream_servers.iter() {
            if !config.enabled {
                info!("   ⊘ {} disabled, skipping", name);
                continue;
            }

            // Substitute env vars from process environment
            let mut config_clone = (*config).clone();
            for (key, value) in &mut config_clone.env {
                if value.is_empty() {
                    let key_upper = key.to_uppercase();
                    for env_key in [key.as_str(), key_upper.as_str()] {
                        if let Ok(env_val) = std::env::var(env_key) {
                            *value = env_val;
                            info!("   {} '{}' ← ${}", name, key, env_key);
                            break;
                        }
                    }
                }
            }

            match mcp::McpUpstreamConnection::connect(name.clone(), &config_clone).await {
                Ok(conn) => {
                    info!("   {} connected ({} tools)", name, conn.tools.len());
                    connections.push(conn);
                }
                Err(e) => {
                    info!("   {} error: {}", name, e);
                }
            }
        }

        if !connections.is_empty() {
            let proxy_manager = mcp::McpProxyManager::new(connections);
            let mut anonymizing_proxy = mcp::AnonymizingProxy::new(proxy_manager, engine.clone());

            // Set anonymization rules for each server
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

    // Wrap in Arc to avoid cloning
    let mcp_service = std::sync::Arc::new(mcp_service);

    // Create Axum routers
    let health_app = Router::new().route("/api/v1/health", axum::routing::get(|| async { "OK" }));

    let sse_app = mcp::sse_transport::create_sse_router_arc(mcp_service);

    let app = Router::new().merge(sse_app).nest("/health", health_app);

    // Start server
    let bind_addr = format!("{}:{}", settings.server.host, settings.server.port);
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;

    info!("MCP SSE server running on {}", bind_addr);
    info!("SSE endpoint: http://{}/sse", bind_addr);
    info!("Message endpoint: http://{}/message", bind_addr);
    info!(
        "Health endpoint: http://{}:{}/api/v1/health",
        settings.server.host, settings.server.port
    );

    info!("MCP server ready for client connections");

    axum::serve(listener, app).await?;

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

    tracing::info!("Shutdown signal received, graceful shutdown...");
}

/// Config validation (like nginx -t)
fn run_config_test(settings: &config::Settings) -> Result<(), Box<dyn std::error::Error>> {
    let mut errors: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    // Strategy validation
    match settings.anonymizer.default_strategy.as_str() {
        "replace" | "mask" | "hash" => {}
        other => {
            errors.push(format!(
                "Unknown strategy '{}'. Valid: replace, mask, hash",
                other
            ));
        }
    }

    // Collect patterns
    let builtin_patterns = anonymizer::patterns::get_all_patterns();
    let enabled_names: std::collections::HashSet<String> =
        settings.anonymizer.patterns.iter().cloned().collect();

    for name in &enabled_names {
        let found = builtin_patterns.iter().any(|p| &p.name == name);
        if !found {
            warnings.push(format!("Built-in pattern '{}' not found", name));
        }
    }

    // Validate custom patterns
    for cp in &settings.anonymizer.custom_patterns {
        if let Err(e) = anonymizer::patterns::PIIPattern::from_config(
            &cp.name,
            &cp.pii_type,
            &cp.pattern,
            cp.confidence,
        ) {
            errors.push(format!("Custom pattern '{}': {}", cp.name, e));
        }
    }

    // Validate domains
    for domain in &settings.anonymizer.custom_known_domains {
        if domain.is_empty() {
            warnings.push("Empty domain in custom_known_domains".to_string());
        }
    }

    // Print compact status output (tree-like format)
    print_status_tree(settings, &errors, &warnings);

    // Summary
    if errors.is_empty() && warnings.is_empty() {
        println!("\nConfiguration is valid");
        Ok(())
    } else {
        if !warnings.is_empty() {
            println!("\nWarnings ({}):", warnings.len());
            for w in &warnings {
                println!("  {}", w);
            }
            println!();
        }
        if !errors.is_empty() {
            println!("Errors ({}):", errors.len());
            for e in &errors {
                println!("  {}", e);
            }
            println!();
            println!("Configuration is invalid");
            return Err(format!("{} configuration errors", errors.len()).into());
        }
        println!("Configuration is valid with warnings");
        Ok(())
    }
}

/// Print compact tree-like status output (similar to ls/tree format)
fn print_status_tree(settings: &config::Settings, errors: &[String], warnings: &[String]) {
    let status_icon = if errors.is_empty() && warnings.is_empty() {
        "✓"
    } else if errors.is_empty() {
        "⚠"
    } else {
        "✗"
    };

    println!("pii-anonymizer {}", status_icon);
    println!("├── strategy: {}", settings.anonymizer.default_strategy);
    println!("├── server: {}:{}", settings.server.host, settings.server.port);

    // Patterns section
    if !settings.anonymizer.patterns.is_empty() {
        println!("├── patterns [{}]", settings.anonymizer.patterns.len());
        for (i, pattern) in settings.anonymizer.patterns.iter().enumerate() {
            let connector = if i == settings.anonymizer.patterns.len() - 1 {
                "└──"
            } else {
                "├──"
            };
            println!("│   {} {}", connector, pattern);
        }
    } else {
        println!("├── patterns: none");
    }

    // Custom patterns
    if !settings.anonymizer.custom_patterns.is_empty() {
        println!("├── custom [{}]", settings.anonymizer.custom_patterns.len());
        for cp in &settings.anonymizer.custom_patterns {
            println!("│   ├── {} ({:.0}%)", cp.name, cp.confidence * 100.0);
        }
    }

    // Domains
    if !settings.anonymizer.custom_known_domains.is_empty() {
        println!("├── domains [{}]", settings.anonymizer.custom_known_domains.len());
        for domain in &settings.anonymizer.custom_known_domains {
            println!("│   └── {}", domain);
        }
    }

    // Proxy servers
    if !settings.proxy.upstream_servers.is_empty() {
        let enabled_servers: Vec<_> = settings.proxy.upstream_servers.iter()
            .filter(|(_, config)| config.enabled)
            .collect();

        if !enabled_servers.is_empty() {
            println!("├── proxy [{}]", enabled_servers.len());
            for (i, (name, config)) in enabled_servers.iter().enumerate() {
                let transport = match config.transport {
                    mcp::client::McpTransport::Stdio => "stdio",
                    mcp::client::McpTransport::Http => "http",
                };
                let connector = if i == enabled_servers.len() - 1 {
                    "└──"
                } else {
                    "├──"
                };
                println!("│   {} {} ({})", connector, name, transport);
            }
        }
    }

    // Status summary
    let status_line = if errors.is_empty() && warnings.is_empty() {
        "valid".to_string()
    } else if errors.is_empty() {
        format!("valid with {} warnings", warnings.len())
    } else {
        format!("invalid ({} errors)", errors.len())
    };
    println!("└── status: {}", status_line);
}

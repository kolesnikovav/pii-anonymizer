# PII Anonymizer MCP Server

Service for text anonymization with PII (Personally Identifiable Information) removal. HTTP REST API, MCP server, proxying to external MCP servers.

[🇷🇺 Русская документация](docs/README_ru.md)

## Features

- **12+ PII patterns**: email, phones, passports, SNILS, INN, credit cards, API keys, JWT, SSH keys, IPs, domains
- **3 strategies**: replace, mask, hash
- REST API + MCP Server + SSE transport
- **MCP Proxy** — proxy to any external MCP servers with selective anonymization
- Custom regex patterns and known domains via config
- Config validation (`--config-test`)
- Docker ready

## Quick Start

```bash
cargo run                              # HTTP mode
cargo run -- --mcp-mode stdio          # MCP stdio mode
cargo run -- --config-test             # Validate config (like nginx -t)
docker compose up -d                   # Docker
```

## Documentation

| Section | Description |
|---------|-------------|
| [Configuration](docs/configuration.md) ([🇷🇺](docs/configuration_ru.md)) | Settings, CLI, environment variables |
| [REST API](docs/rest-api.md) ([🇷🇺](docs/rest-api_ru.md)) | Endpoints, request examples |
| [MCP Server](docs/mcp-server.md) ([🇷🇺](docs/mcp-server_ru.md)) | Tools, SSE transport, launch modes |
| [MCP Proxy](docs/mcp-proxy.md) ([🇷🇺](docs/mcp-proxy_ru.md)) | Upstream server proxy, selective anonymization |
| [Integrations](docs/integrations.md) ([🇷🇺](docs/integrations_ru.md)) | AnythingLLM, GitHub MCP, SearXNG, VS Code, Claude |
| [Strategies](docs/strategies.md) ([🇷🇺](docs/strategies_ru.md)) | Replace, mask, hash — examples and comparison |
| [PII Patterns](docs/patterns.md) ([🇷🇺](docs/patterns_ru.md)) | Full list of supported PII |
| [Custom Patterns](docs/custom-patterns.md) ([🇷🇺](docs/custom-patterns_ru.md)) | Custom regex patterns and domains |
| [Deployment](docs/deployment.md) ([🇷🇺](docs/deployment_ru.md)) | Docker, environment variables, docker socket |
| [Benchmarks](docs/benchmarks.md) ([🇷🇺](docs/benchmarks_ru.md)) | Performance tests vs Presidio |
| [Comparison](COMPARISON.md) | Presidio, Scrubadub and others |

## Testing

```bash
cargo test
```

**Status**: 80+ tests passing

## License

MIT

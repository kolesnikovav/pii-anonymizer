# PII Anonymizer MCP Server

Service for text anonymization with PII (Personally Identifiable Information) removal. HTTP REST API, MCP server, proxying to external MCP servers.

- [English documentation](docs/en/)
- [Русская документация](docs/ru/)

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
| [Configuration](docs/en/configuration.md) ([🇷🇺](docs/ru/configuration_ru.md)) | Settings, CLI, environment variables |
| [REST API](docs/en/rest-api.md) ([🇷🇺](docs/ru/rest-api_ru.md)) | Endpoints, request examples |
| [MCP Server](docs/en/mcp-server.md) ([🇷🇺](docs/ru/mcp-server_ru.md)) | Tools, SSE transport, launch modes |
| [MCP Proxy](docs/en/mcp-proxy.md) ([🇷🇺](docs/ru/mcp-proxy_ru.md)) | Upstream server proxy, selective anonymization |
| [Integrations](docs/en/integrations.md) ([🇷🇺](docs/ru/integrations_ru.md)) | AnythingLLM, GitHub MCP, SearXNG, VS Code, Claude |
| [Strategies](docs/en/strategies.md) ([🇷🇺](docs/ru/strategies_ru.md)) | Replace, mask, hash — examples and comparison |
| [PII Patterns](docs/en/patterns.md) ([🇷🇺](docs/ru/patterns_ru.md)) | Full list of supported PII |
| [Custom Patterns](docs/en/custom-patterns.md) ([🇷🇺](docs/ru/custom-patterns_ru.md)) | Custom regex patterns and domains |
| [Deployment](docs/en/deployment.md) ([🇷🇺](docs/ru/deployment_ru.md)) | Docker, environment variables, docker socket |
| [Benchmarks](docs/en/benchmarks.md) ([🇷🇺](docs/ru/benchmarks_ru.md)) | Performance tests vs Presidio |
| [Comparison](docs/en/comparison.md) ([🇷🇺](docs/ru/comparison_ru.md)) | Presidio, Scrubadub and others |
| [CI/CD](docs/en/ci-cd.md) ([🇷🇺](docs/ru/ci-cd_ru.md)) | Release pipeline, Docker, .deb packages |

## Testing

```bash
cargo test
```

**Status**: 80+ tests passing

## License

MIT

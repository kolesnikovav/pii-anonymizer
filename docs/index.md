# PII Anonymizer

[![Release](https://img.shields.io/github/v/release/kolesnikovav/pii-anonymizer?label=version&sort=semver&color=blue)](https://github.com/kolesnikovav/pii-anonymizer/releases)
[![CI](https://img.shields.io/github/actions/workflow/status/kolesnikovav/pii-anonymizer/ci.yml?branch=master&label=CI)](https://github.com/kolesnikovav/pii-anonymizer/actions/workflows/ci.yml)
[![License](https://img.shields.io/github/license/kolesnikovav/pii-anonymizer?color=green)](https://github.com/kolesnikovav/pii-anonymizer/blob/master/LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange)](https://www.rust-lang.org/)

---

**Service for text anonymization with PII (Personally Identifiable Information) removal.**

HTTP REST API, MCP Server, proxying to external MCP servers.

## Quick Start

=== "Cargo"

    ```bash
    cargo run                              # HTTP mode
    cargo run -- --mcp-mode stdio          # MCP stdio mode
    cargo run -- --config-test             # Validate config (like nginx -t)
    ```

=== "Docker"

    ```bash
    docker compose up -d
    ```

=== "Binary"

    ```bash
    ./pii-anonymizer --host 0.0.0.0 --port 3000
    ```

## Features

- **12+ PII patterns**: email, phones, passports, SNILS, INN, credit cards, API keys, JWT, SSH keys, IPs, domains
- **3 strategies**: replace, mask, hash
- **REST API** + **MCP Server** + **SSE transport**
- **MCP Proxy** — proxy to any external MCP servers with selective anonymization
- **Custom regex patterns** and known domains via config
- **Config validation** (`--config-test`)
- **Docker ready**

## Navigation

| Section | Description |
|---------|-------------|
| [Quick Start](quick-start.md) | Installation and first steps |
| [Configuration](configuration.md) | Settings, CLI, environment variables |
| [REST API](rest-api.md) | Endpoints, request examples |
| [MCP Server](mcp-server.md) | Tools, SSE transport, launch modes |
| [MCP Proxy](mcp-proxy.md) | Upstream server proxy, selective anonymization |
| [Integrations](integrations.md) | AnythingLLM, GitHub MCP, SearXNG, VS Code, Claude |
| [Strategies](strategies.md) | Replace, mask, hash — examples and comparison |
| [PII Patterns](patterns.md) | Full list of supported PII |
| [Custom Patterns](custom-patterns.md) | Custom regex patterns and domains |
| [Deployment](deployment.md) | Docker, environment variables, docker socket |
| [Benchmarks](benchmarks.md) | Performance tests vs Presidio |
| [Comparison](comparison.md) | Presidio, Scrubadub and others |
| [CI/CD](ci-cd.md) | Release pipeline, Docker, .deb packages |

## License

MIT

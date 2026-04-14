# PII Anonymizer MCP Server

[![Release](https://img.shields.io/github/v/release/kolesnikovav/pii-anonymizer?label=version&sort=semver&color=blue)](https://github.com/kolesnikovav/pii-anonymizer/releases)
[![CI](https://img.shields.io/github/actions/workflow/status/kolesnikovav/pii-anonymizer/ci.yml?branch=master&label=CI)](https://github.com/kolesnikovav/pii-anonymizer/actions/workflows/ci.yml)
[![License](https://img.shields.io/github/license/kolesnikovav/pii-anonymizer?color=green)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange)](https://www.rust-lang.org/)
[![Docker Pulls](https://img.shields.io/github/downloads/kolesnikovav/pii-anonymizer/total?label=downloads&color=9cf)](https://github.com/kolesnikovav/pii-anonymizer/releases)
[![Stars](https://img.shields.io/github/stars/kolesnikovav/pii-anonymizer?style=flat&label=%E2%AD%90&color=yellow)](https://github.com/kolesnikovav/pii-anonymizer/stargazers)
[![Docs](https://img.shields.io/badge/docs-mkdocs-blue)](https://kolesnikovav.github.io/pii-anonymizer/)

Service for text anonymization with PII (Personally Identifiable Information) removal. HTTP REST API, MCP server, proxying to external MCP servers.

- 📖 **[Documentation](https://kolesnikovav.github.io/pii-anonymizer/)**
- 🇷🇺 **[Документация на русском](https://kolesnikovav.github.io/pii-anonymizer/ru/)**

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

Full documentation with search, dark mode, and language switch: **https://kolesnikovav.github.io/pii-anonymizer/**

| Section | English | Русский |
|---------|---------|---------|
| Quick Start | [EN](https://kolesnikovav.github.io/pii-anonymizer/quick-start/) | [RU](https://kolesnikovav.github.io/pii-anonymizer/ru/quick-start/) |
| Configuration | [EN](https://kolesnikovav.github.io/pii-anonymizer/configuration/) | [RU](https://kolesnikovav.github.io/pii-anonymizer/ru/configuration/) |
| REST API | [EN](https://kolesnikovav.github.io/pii-anonymizer/rest-api/) | [RU](https://kolesnikovav.github.io/pii-anonymizer/ru/rest-api/) |
| MCP Server | [EN](https://kolesnikovav.github.io/pii-anonymizer/mcp-server/) | [RU](https://kolesnikovav.github.io/pii-anonymizer/ru/mcp-server/) |
| MCP Proxy | [EN](https://kolesnikovav.github.io/pii-anonymizer/mcp-proxy/) | [RU](https://kolesnikovav.github.io/pii-anonymizer/ru/mcp-proxy/) |
| Integrations | [EN](https://kolesnikovav.github.io/pii-anonymizer/integrations/) | [RU](https://kolesnikovav.github.io/pii-anonymizer/ru/integrations/) |
| Strategies | [EN](https://kolesnikovav.github.io/pii-anonymizer/strategies/) | [RU](https://kolesnikovav.github.io/pii-anonymizer/ru/strategies/) |
| PII Patterns | [EN](https://kolesnikovav.github.io/pii-anonymizer/patterns/) | [RU](https://kolesnikovav.github.io/pii-anonymizer/ru/patterns/) |
| Custom Patterns | [EN](https://kolesnikovav.github.io/pii-anonymizer/custom-patterns/) | [RU](https://kolesnikovav.github.io/pii-anonymizer/ru/custom-patterns/) |
| Deployment | [EN](https://kolesnikovav.github.io/pii-anonymizer/deployment/) | [RU](https://kolesnikovav.github.io/pii-anonymizer/ru/deployment/) |
| Benchmarks | [EN](https://kolesnikovav.github.io/pii-anonymizer/benchmarks/) | [RU](https://kolesnikovav.github.io/pii-anonymizer/ru/benchmarks/) |
| Comparison | [EN](https://kolesnikovav.github.io/pii-anonymizer/comparison/) | [RU](https://kolesnikovav.github.io/pii-anonymizer/ru/comparison/) |
| CI/CD | [EN](https://kolesnikovav.github.io/pii-anonymizer/ci-cd/) | [RU](https://kolesnikovav.github.io/pii-anonymizer/ru/ci-cd/) |

## Testing

```bash
cargo test
```

**Status**: 80+ tests passing

## License

MIT

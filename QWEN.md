# PII Anonymizer — Project Context

**Qwen Code Memory:** `memory/MEMORY.md` — содержит контекст проекта, настройки пользователя, feedback.

## Quick Commands

```bash
# Build
cargo build

# Run
cargo run --bin pii-anonymizer              # HTTP mode
cargo run --bin pii-anonymizer -- --mcp-mode stdio  # MCP stdio mode
cargo run --bin pii-anonymizer -- --config-test     # Validate config

# Test
cargo test

# Docker
docker compose up -d
```

## Project Structure

```
src/
├── main.rs          # CLI, startup, status tree
├── lib.rs           # Public API
├── anonymizer/      # PII engine (patterns, strategies, engine)
├── api/             # REST API handlers/routes
├── mcp/             # MCP server, client, proxy, SSE transport
├── models/          # Data models
├── config/          # Settings loading
└── middleware/      # Request logging, request ID
```

## Key Features

- 12+ PII patterns: email, phone, passport, SNILS, INN, credit cards, API keys, JWT, SSH keys, domains
- 3 strategies: replace, mask, hash
- REST API + MCP Server (stdio/SSE)
- MCP Proxy — upstream servers with selective anonymization
- Custom regex patterns + known domains whitelist
- Tree-like status output on every startup

## Config

`config/settings.yaml` — main configuration file.

## Documentation

- https://kolesnikovav.github.io/pii-anonymizer/
- https://kolesnikovav.github.io/pii-anonymizer/ru/

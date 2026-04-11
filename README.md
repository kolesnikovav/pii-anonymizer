# PII Anonymizer MCP Server

Сервис для анонимизации текста с удалением персональных данных (PII). HTTP REST API, MCP сервер, проксирование к внешним MCP серверам.

## Возможности

- **12 паттернов PII**: email, телефоны, паспорта, СНИЛС, ИНН, кредитные карты, API ключи, JWT, SSH ключи, IP, домены
- **3 стратегии**: replace, mask, hash
- REST API + MCP Server + SSE transport
- **MCP Proxy** — проксирование к любым внешним MCP серверам с выборочной анонимизацией
- Docker готовность

## Быстрый старт

```bash
cargo run                              # HTTP режим
cargo run -- --mcp-mode stdio          # MCP stdio режим
docker compose up -d                   # Docker
```

## Документация

| Раздел | Описание |
|--------|----------|
| [Конфигурация](docs/configuration.md) | Настройки, CLI, переменные окружения |
| [REST API](docs/rest-api.md) | Endpoints, примеры запросов |
| [MCP сервер](docs/mcp-server.md) | Инструменты, SSE transport, режимы запуска |
| [MCP Proxy](docs/mcp-proxy.md) | Проксирование upstream серверов, выборочная анонимизация |
| [Интеграции](docs/integrations.md) | AnythingLLM, GitHub MCP, SearXNG, VS Code, Claude |
| [Стратегии](docs/strategies.md) | Replace, mask, hash — примеры и сравнение |
| [PII паттерны](docs/patterns.md) | Полный список поддерживаемых PII |
| [Кастомные паттерны](docs/custom-patterns.md) | Свои regex паттерны и домены |
| [Развёртывание](docs/deployment.md) | Docker, переменные окружения, docker socket |
| [Бенчмарки](docs/benchmarks.md) | Тесты производительности vs Presidio |
| [Сравнение](COMPARISON.md) | Presidio, Scrubadub и другие |

## Тестирование

```bash
cargo test
```

**Статус**: 80+ тестов проходят успешно

## Лицензия

MIT

# PII Anonymizer MCP Server

Сервис для анонимизации текста с удалением персональных данных (PII). HTTP REST API, MCP сервер, проксирование к внешним MCP серверам.

[🇬🇧 English documentation](../README.md)

## Возможности

- **12+ паттернов PII**: email, телефоны, паспорта, СНИЛС, ИНН, кредитные карты, API ключи, JWT, SSH ключи, IP, домены
- **3 стратегии**: replace, mask, hash
- REST API + MCP Server + SSE transport
- **MCP Proxy** — проксирование к любым внешним MCP серверам с выборочной анонимизацией
- Кастомные regex паттерны и известные домены через конфиг
- Проверка конфигурации (`--config-test`)
- Docker готовность

## Быстрый старт

```bash
cargo run                              # HTTP режим
cargo run -- --mcp-mode stdio          # MCP stdio режим
cargo run -- --config-test             # Проверка конфига (как nginx -t)
docker compose up -d                   # Docker
```

## Документация

| Раздел | Описание |
|--------|----------|
| [Конфигурация](configuration_ru.md) | Настройки, CLI, переменные окружения |
| [REST API](rest-api_ru.md) | Endpoints, примеры запросов |
| [MCP сервер](mcp-server_ru.md) | Инструменты, SSE transport, режимы запуска |
| [MCP Proxy](mcp-proxy_ru.md) | Проксирование upstream серверов, выборочная анонимизация |
| [Интеграции](integrations_ru.md) | AnythingLLM, GitHub MCP, SearXNG, VS Code, Claude |
| [Стратегии](strategies_ru.md) | Replace, mask, hash — примеры и сравнение |
| [PII паттерны](patterns_ru.md) | Полный список поддерживаемых PII |
| [Кастомные паттерны](custom-patterns_ru.md) | Свои regex паттерны и домены |
| [Развёртывание](deployment_ru.md) | Docker, переменные окружения, docker socket |
| [Бенчмарки](benchmarks_ru.md) | Тесты производительности vs Presidio |
| [Сравнение](../COMPARISON.md) | Presidio, Scrubadub и другие |

## Тестирование

```bash
cargo test
```

**Статус**: 80+ тестов проходят успешно

## Лицензия

MIT

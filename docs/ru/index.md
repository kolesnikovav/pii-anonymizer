# PII Anonymizer

[![Release](https://img.shields.io/github/v/release/kolesnikovav/pii-anonymizer?label=version&sort=semver&color=blue)](https://github.com/kolesnikovav/pii-anonymizer/releases)
[![CI](https://img.shields.io/github/actions/workflow/status/kolesnikovav/pii-anonymizer/ci.yml?branch=master&label=CI)](https://github.com/kolesnikovav/pii-anonymizer/actions/workflows/ci.yml)
[![License](https://img.shields.io/github/license/kolesnikovav/pii-anonymizer?color=green)](https://github.com/kolesnikovav/pii-anonymizer/blob/master/LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange)](https://www.rust-lang.org/)

---

**Сервис для анонимизации текста с удалением персональных данных (PII).**

HTTP REST API, MCP сервер, проксирование к внешним MCP серверам.

## Быстрый старт

=== "Cargo"

    ```bash
    cargo run                              # HTTP режим
    cargo run -- --mcp-mode stdio          # MCP stdio режим
    cargo run -- --config-test             # Проверка конфигурации
    ```

=== "Docker"

    ```bash
    docker compose up -d
    ```

=== "Бинарный файл"

    ```bash
    ./pii-anonymizer --host 0.0.0.0 --port 3000
    ```

## Возможности

- **12+ PII паттернов**: email, телефоны, паспорта, СНИЛС, ИНН, банковские карты, API ключи, JWT, SSH ключи, IP, домены
- **3 стратегии**: replace, mask, hash
- **REST API** + **MCP сервер** + **SSE транспорт**
- **MCP прокси** — проксирование к любым внешним MCP серверам с выборочной анонимизацией
- **Кастомные regex паттерны** и известные домены через конфигурацию
- **Проверка конфигурации** (`--config-test`)
- **Docker готов к использованию**

## Навигация

| Раздел | Описание |
|--------|----------|
| [Быстрый старт](quick-start.md) | Установка и первые шаги |
| [Конфигурация](configuration.md) | Настройки, CLI, переменные окружения |
| [REST API](rest-api.md) | Endpoints, примеры запросов |
| [MCP сервер](mcp-server.md) | Инструменты, SSE транспорт, режимы запуска |
| [MCP прокси](mcp-proxy.md) | Проксирование upstream серверов |
| [Интеграции](integrations.md) | AnythingLLM, GitHub MCP, SearXNG, VS Code, Claude |
| [Стратегии](strategies.md) | Replace, mask, hash — примеры и сравнение |
| [PII паттерны](patterns.md) | Полный список поддерживаемых PII |
| [Кастомные паттерны](custom-patterns.md) | Свои regex паттерны и домены |
| [Развёртывание](deployment.md) | Docker, переменные окружения |
| [Бенчмарки](benchmarks.md) | Тесты производительности vs Presidio |
| [Сравнение](comparison.md) | Presidio, Scrubadub и другие |
| [CI/CD](ci-cd.md) | Пайплайн релизов, Docker, .deb пакеты |

## Лицензия

MIT

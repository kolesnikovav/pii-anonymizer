# PII Anonymizer MCP Server

Сервис для анонимизации поисковых запросов с удалением персональных данных (PII).

## Возможности

- 🔍 Обнаружение и удаление PII (email, телефоны, паспорта, и т.д.)
- 🌐 HTTP REST API
- 🤖 MCP Server с поддержкой проксирования
- 📡 SSE (Server-Sent Events) для стриминга
- 🐳 Готовность к запуску в Docker

## Быстрый старт

### Локальный запуск

```bash
# Клонирование репозитория
git clone <repo-url>
cd pii-anonymizer

# Запуск
cargo run
```

### Docker

```bash
docker-compose up -d
```

## API

### REST Endpoints

- `POST /api/v1/anonymize` - Анонимизация текста
- `POST /api/v1/batch` - Пакетная обработка
- `GET /api/v1/health` - Health check
- `GET /api/v1/sse/stream` - SSE стрим

### MCP Tools

- `anonymize` - Анонимизировать текст
- `detect_pii` - Обнаружить PII в тексте
- `batch_process` - Пакетная обработка

## Конфигурация

Все настройки в `config/settings.yaml` или через переменные окружения.

## Лицензия

MIT

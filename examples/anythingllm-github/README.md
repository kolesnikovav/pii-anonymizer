# Пример: AnythingLLM + PII Anonymizer + GitHub MCP Server

Интеграция AnythingLLM с PII Anonymizer, проксирующим MCP сервер GitHub.
Требует **GitHub Personal Access Token**.

## Архитектура

```
┌─────────────────┐         ┌──────────────────────┐         ┌─────────────────┐
│   AnythingLLM   │────────▶│  PII Anonymizer MCP  │────────▶│  GitHub MCP     │
│   (port 3001)   │  SSE    │  (port 3000)         │  stdio  │  (Docker)       │
└─────────────────┘         └──────────────────────┘         └─────────────────┘
```

## Быстрый старт

### 1. Подготовка токена

Создайте токен в **GitHub → Settings → Developer settings → Personal access tokens → Tokens (classic)**

Минимальные права: `repo`, `read:user`, `read:org`

### 2. Настройка .env

```bash
cp .env.example .env
# В .env укажите:
# GITHUB_PERSONAL_ACCESS_TOKEN=ghp_ваш_токен
```

### 3. Запуск

```bash
cd examples/anythingllm-github
docker compose up -d
```

### 4. Доступ

- **AnythingLLM**: http://localhost:3001
- **PII Anonymizer API**: http://localhost:3000/api/v1/health

### 5. Подключение MCP в AnythingLLM

1. Откройте http://localhost:3001 → **Settings** → **MCP Servers**
2. **Add new MCP server**:
   - **Name**: `PII Anonymizer`
   - **Type**: `SSE`
   - **URL**: `http://pii-anonymizer:3000/sse`
3. **Save** — сервер станет зелёным

### 6. Доступные инструменты

| Инструмент | Источник | Описание |
|------------|----------|----------|
| `anonymize` | PII Anonymizer | Анонимизация текста |
| `detect_pii` | PII Anonymizer | Обнаружение PII данных |
| `batch_anonymize` | PII Anonymizer | Пакетная обработка |
| `github_*` | GitHub MCP | ~20 инструментов GitHub (issues, PR, search, repos...) |

## Конфигурация

### Отключение GitHub MCP

В `../../config/settings.yaml` закомментируйте секцию `github`:

```yaml
proxy:
  upstream_servers:
    # github: ...  # закомментировать
```

### Подключение других upstream серверов

Добавьте сервер в `../../config/settings.yaml`:

```yaml
proxy:
  upstream_servers:
    my_server:
      transport: http
      url: "http://my-mcp-server:8080/sse"
      enabled: true
```

## Troubleshooting

```bash
# Логи PII Anonymizer (показывает подключение к GitHub MCP)
docker logs pii-anonymizer

# Проверка SSE endpoint
curl -N http://localhost:3000/sse

# Перезапуск
docker compose restart pii-anonymizer
```

## Остановка

```bash
docker compose down
```

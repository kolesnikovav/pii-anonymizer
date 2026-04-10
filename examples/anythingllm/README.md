# AnythingLLM + PII Anonymizer MCP Server

Интеграция PII Anonymizer с AnythingLLM через Docker Compose.

## Архитектура

```
┌─────────────────┐         ┌──────────────────────┐
│   AnythingLLM   │────────▶│  PII Anonymizer MCP  │
│   (port 3001)   │  SSE    │  (port 3000)         │
└─────────────────┘         └──────────────────────┘
        │
        └─ mcp_servers.json (volume mount)
```

## Быстрый старт

### 1. Запуск

```bash
cd examples/anythingllm
docker-compose up -d
```

### 2. Доступ

- **AnythingLLM**: http://localhost:3001
- **PII Anonymizer API**: http://localhost:3000/api/v1/health
- **MCP SSE Endpoint**: http://localhost:3000/sse

### 3. Проверка MCP сервера

```bash
# Проверка health endpoint
curl http://localhost:3000/api/v1/health

# Проверка SSE endpoint (должен вернуть event: endpoint)
curl -N http://localhost:3000/sse
```

### 4. Настройка MCP сервера в AnythingLLM

**⚠️ Важно:** MCP серверы в AnythingLLM настраиваются через веб-интерфейс.

1. Откройте http://localhost:3001
2. Войдите в систему (или создайте аккаунт при первом запуске)
3. Перейдите в **Settings** (шестерёнка в нижнем левом углу)
4. Найдите раздел **MCP Servers** в меню слева
5. Нажмите **Add new MCP server**
6. Заполните:
   - **Name**: `PII Anonymizer`
   - **Type**: `SSE`
   - **URL**: `http://pii-anonymizer:3000/sse`
7. Нажмите **Save** — сервер должен стать зелёным (Connected)

> **Примечание**: URL использует Docker DNS имя `pii-anonymizer`, а не `localhost`.

### 5. Подключение upstream MCP серверов (опционально)

PII Anonymizer может проксировать любые внешние MCP серверы к AnythingLLM.
Инструменты всех подключённых серверов автоматически появляются в AnythingLLM.

**Пример: GitHub MCP Server через Docker**

1. Раскомментируйте секцию `github` в `config/settings.yaml`:
```yaml
proxy:
  upstream_servers:
    github:
      transport: stdio
      command: docker
      args: ["run", "-i", "--rm", "-e", "GITHUB_PERSONAL_ACCESS_TOKEN", "ghcr.io/github/github-mcp-server"]
      env:
        GITHUB_PERSONAL_ACCESS_TOKEN: ""  # подставится из GITHUB_TOKEN
      enabled: true
```

2. Укажите `GITHUB_TOKEN` в `.env` файле:
```
GITHUB_TOKEN=ghp_your_token_here
```

**Пример: любой другой MCP Server через HTTP**

```yaml
proxy:
  upstream_servers:
    my_server:
      transport: http
      url: "http://my-mcp-server:8080/sse"
      enabled: true
```

### 6. Проверка работы

После подключения MCP сервера:

1. Перейдите в **Agents** → выберите агент
2. В разделе MCP инструментов должны появиться:
   - `anonymize` — анонимизация текста
   - `detect_pii` — обнаружение PII
   - `batch_anonymize` — пакетная обработка
   - `github_*` — инструменты GitHub (если подключён)
3. Попробуйте запрос к агенту с текстом содержащим PII

## Конфигурация

### Изменение стратегии анонимизации

В `docker-compose.yml` измените:
```yaml
environment:
  - ANONYMIZER__DEFAULT_STRATEGY=hash  # mask, replace, hash
```

### Монтирование своей конфигурации

```yaml
volumes:
  - ./my-config.yaml:/app/config/settings.yaml:ro
```

## Доступные MCP инструменты

| Инструмент | Описание | Параметры |
|------------|----------|-----------|
| `anonymize` | Анонимизировать текст | `text`, `strategy?` |
| `detect_pii` | Обнаружить PII | `text` |
| `batch_anonymize` | Пакетная обработка | `texts`, `strategy?` |

## Структура mcp_servers.json

```json
{
  "mcpServers": {
    "pii-anonymizer": {
      "type": "sse",
      "url": "http://pii-anonymizer:3000/sse"
    }
  }
}
```

Файл монтируется как volume в `/app/server/storage/mcp_servers.json` внутри контейнера AnythingLLM.

## Troubleshooting

### MCP сервер не подключается

1. Проверьте логи PII Anonymizer:
   ```bash
   docker logs pii-anonymizer
   ```

2. Убедитесь, что SSE endpoint работает:
   ```bash
   curl -N http://localhost:3000/sse
   ```
   Должен вернуться `event: endpoint` с URL.

3. Проверьте сеть Docker:
   ```bash
   docker network inspect anythingllm_mcp-network
   ```

### AnythingLLM не видит mcp_servers.json

1. Проверьте, что файл смонтирован:
   ```bash
   docker exec anythingllm cat /app/server/storage/mcp_servers.json
   ```

2. Перезапустите AnythingLLM:
   ```bash
   docker-compose restart anythingllm
   ```

## Остановка

```bash
docker-compose down
```

## Полный сброс

```bash
docker-compose down -v
docker volume rm anythingllm_anythingllm-storage
```

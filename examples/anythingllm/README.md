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

### 5. Проверка работы

После подключения MCP сервера:

1. Перейдите в **Agents** → выберите агент
2. В разделе MCP инструментов должен появиться `anonymize`, `detect_pii`, `batch_anonymize`
3. Попробуйте запрос к агенту с текстом содержащим PII (email, телефон и т.д.)

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

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

**⚠️ ВАЖНО:** MCP серверы в AnythingLLM требуют первоначальной активации через веб-интерфейс.

#### Шаг 1: Активация MCP в UI

1. Откройте http://localhost:3001
2. Войдите в систему (или создайте аккаунт при первом запуске)
3. Перейдите в **Settings** (нажмите на шестерёнку в нижнем левом углу)
4. В левом меню найдите и нажмите **MCP Servers**
5. Если увидите переключатель "Enable MCP" — включите его
6. Нажмите **Save**

#### Шаг 2: Добавление PII-Anonymizer

После активации MCP:

1. В том же разделе **MCP Servers** нажмите **Add new MCP server**
2. Заполните форму:
   - **Name**: `PII Anonymizer`
   - **Transport Type**: `SSE`
   - **URL**: `http://pii-anonymizer:3000/sse`
3. Нажмите **Save**
4. Сервер должен стать зелёным (статус: Connected)

#### Альтернатива: Через файл конфигурации

Если MCP уже активирован, можно отредактировать файл:

```bash
# Остановить AnythingLLM
docker stop anythingllm

# Отредактировать файл в volume
docker run --rm -v anythingllm_anythingllm-storage:/data -it alpine vi /data/plugins/anythingllm_mcp_servers.json

# Добавить содержимое:
{
  "mcpServers": {
    "pii-anonymizer": {
      "transport": "sse",
      "url": "http://pii-anonymizer:3000/sse",
      "enabled": true
    }
  }
}

# Перезапустить
docker start anythingllm
```

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

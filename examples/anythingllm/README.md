# AnythingLLM + PII Anonymizer MCP Server

Интеграция PII Anonymizer с AnythingLLM через Docker Compose.

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

1. Откройте http://localhost:3001
2. Пройдите первоначальную настройку
3. Перейдите в **Settings → MCP Servers**
4. Нажмите **Add new MCP server**
5. Заполните:
   - **Name**: `pii-anonymizer`
   - **Type**: `sse`
   - **URL**: `http://pii-anonymizer:3000/sse`
6. Сохраните — сервер должен стать зелёным (активным)

> **Примечание**: URL использует Docker DNS имя `pii-anonymizer`, а не `localhost`.

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

## Остановка

```bash
docker-compose down
```

# Примеры интеграции PII Anonymizer

## Доступные примеры

### 🔒 anythingllm-github
**AnythingLLM + PII Anonymizer + GitHub MCP Server**

- Проксирует ~20 инструментов GitHub к AnythingLLM
- Требует GitHub Personal Access Token
- `cd examples/anythingllm-github && docker compose up -d`

### 🔍 anythingllm-searxng
**AnythingLLM + PII Anonymizer + SearXNG Web Search**

- Приватный веб-поиск через Google, DuckDuckGo, Bing
- **Без токенов**, работает из РФ
- `cd examples/anythingllm-searxng && docker compose up -d`

## Архитектура

Обе примеры используют одинаковую схему:

```
┌─────────────────┐         ┌──────────────────────┐         ┌─────────────────┐
│   AnythingLLM   │────────▶│  PII Anonymizer MCP  │────────▶│  Upstream MCP   │
│   (port 3001)   │  SSE    │  (port 3000)         │  stdio  │  (Docker)       │
└─────────────────┘         └──────────────────────┘         └─────────────────┘
```

PII Anonymizer анонимизирует PII данные и проксирует инструменты
от любого количества upstream MCP серверов к AnythingLLM.

## Конфигурация

Upstream серверы настраиваются в `config/settings.yaml` проекта:

```yaml
proxy:
  upstream_servers:
    searxng:
      transport: stdio
      command: docker
      args: ["run", "-i", "--rm", "-e", "SEARXNG_URL", "isokoliuk/mcp-searxng:latest"]
      env:
        SEARXNG_URL: "http://searxng:8080"
      enabled: true
```

Пустые значения в `env` автоматически подставляются из окружения процесса
(удобно для токенов в `.env` файлах docker-compose).

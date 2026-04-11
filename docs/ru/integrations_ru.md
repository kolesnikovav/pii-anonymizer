# Интеграции

## AnythingLLM

### SearXNG пример (без токенов)

```bash
cd examples/anythingllm-searxng
docker compose up -d
```

1. http://localhost:3001 → Settings → MCP Servers
2. **Add new MCP server**:
   - **Name**: `PII Anonymizer`
   - **Type**: `SSE`
   - **URL**: `http://pii-anonymizer:3000/sse`

**Инструменты**: anonymize, detect_pii, batch_anonymize + searxng_web_search, web_url_read

### GitHub MCP Server (требует токен)

```bash
cd examples/anythingllm-github
cp .env.example .env
# Укажите GITHUB_PERSONAL_ACCESS_TOKEN в .env
docker compose up -d
```

## SearXNG (веб-поиск)

Приватный мета-поисковик. Без токенов. Работает из РФ.

Пример: [`examples/anythingllm-searxng/`](../examples/anythingllm-searxng/)

| Сервис | Порт | Описание |
|--------|------|----------|
| SearXNG | 8080 | Поисковый движок |
| PII Anonymizer | 3000 | MCP сервер + прокси |
| AnythingLLM | 3001 | Веб-интерфейс |

### Настройка поисковых движков

`searxng-settings.yml`:

```yaml
use_default_settings: true
search:
  default_lang: "ru"
engines:
  - name: yandex
    disabled: false
  - name: wikipedia
    disabled: false
```

## VS Code

```json
// .vscode/mcp.json
{
  "servers": {
    "pii-anonymizer": {
      "type": "stdio",
      "command": "pii-anonymizer",
      "args": ["--mcp-mode", "stdio", "--strategy", "mask"]
    }
  }
}
```

## Claude Desktop

```json
// claude_desktop_config.json
{
  "mcpServers": {
    "pii-anonymizer": {
      "command": "pii-anonymizer",
      "args": ["--mcp-mode", "stdio"]
    }
  }
}
```

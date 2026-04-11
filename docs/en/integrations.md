# Integrations

## AnythingLLM

### SearXNG Example (No Tokens Required)

```bash
cd examples/anythingllm-searxng
docker compose up -d
```

1. http://localhost:3001 → Settings → MCP Servers
2. **Add new MCP server**:
   - **Name**: `PII Anonymizer`
   - **Type**: `SSE`
   - **URL**: `http://pii-anonymizer:3000/sse`

**Tools**: anonymize, detect_pii, batch_anonymize + searxng_web_search, web_url_read

### GitHub MCP Server (Requires Token)

```bash
cd examples/anythingllm-github
cp .env.example .env
# Set GITHUB_PERSONAL_ACCESS_TOKEN in .env
docker compose up -d
```

## SearXNG (Web Search)

Private meta-search engine. No tokens required. Works worldwide.

Example: [`examples/anythingllm-searxng/`](../examples/anythingllm-searxng/)

| Service | Port | Description |
|---------|------|-------------|
| SearXNG | 8080 | Search engine |
| PII Anonymizer | 3000 | MCP server + proxy |
| AnythingLLM | 3001 | Web interface |

### Configuring Search Engines

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

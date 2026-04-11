# MCP Proxy — Upstream Server Proxying

PII Anonymizer connects to external MCP servers and proxies their tools. Arguments are anonymized before proxying.

## Connecting an External Server

```yaml
# config/settings.yaml
proxy:
  upstream_servers:
    github:
      transport: stdio
      command: docker
      args: ["run", "-i", "--rm", "-e", "GITHUB_PERSONAL_ACCESS_TOKEN", "ghcr.io/github/github-mcp-server"]
      env:
        GITHUB_PERSONAL_ACCESS_TOKEN: ""  # will be populated from environment
      enabled: true
```

Empty values in `env` are automatically populated from the process environment variables (convenient for `.env` in docker-compose).

## Selective Anonymization

By default, **all** string values are anonymized. You can configure this specifically:

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
      anonymize_fields:
        searxng_web_search: ["query"]   # anonymize only the query
        web_url_read: []                 # [] = do not anonymize
```

### anonymize_fields Rules

| Value | Behavior |
|----------|-----------|
| Not specified | Anonymize all strings (backward compatibility) |
| `[]` | Disable anonymization for the tool |
| `["query", "body"]` | Anonymize only the specified fields |

### Example: GitHub

```yaml
proxy:
  upstream_servers:
    github:
      transport: stdio
      command: docker
      args: ["run", "-i", "--rm", "-e", "GITHUB_PERSONAL_ACCESS_TOKEN", "ghcr.io/github/github-mcp-server"]
      env:
        GITHUB_PERSONAL_ACCESS_TOKEN: ""
      enabled: true
      anonymize_fields:
        create_issue: ["body", "title"]
        create_pull_request: ["body", "title"]
        search_code: []                 # do not anonymize the search query
```

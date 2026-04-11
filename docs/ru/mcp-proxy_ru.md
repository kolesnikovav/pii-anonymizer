# MCP Proxy — проксирование upstream серверов

PII Anonymizer подключается к внешним MCP серверам и проксирует их инструменты. Перед проксированием аргументы анонимизируются.

## Подключение внешнего сервера

```yaml
# config/settings.yaml
proxy:
  upstream_servers:
    github:
      transport: stdio
      command: docker
      args: ["run", "-i", "--rm", "-e", "GITHUB_PERSONAL_ACCESS_TOKEN", "ghcr.io/github/github-mcp-server"]
      env:
        GITHUB_PERSONAL_ACCESS_TOKEN: ""  # подставится из окружения
      enabled: true
```

Пустые значения в `env` автоматически подставляются из переменных окружения процесса (удобно для `.env` в docker-compose).

## Выборочная анонимизация

По умолчанию анонимизируются **все** строковые значения. Можно настроить конкретно:

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
        searxng_web_search: ["query"]   # анонимизировать только запрос
        web_url_read: []                 # [] = не анонимизировать
```

### Правила anonymize_fields

| Значение | Поведение |
|----------|-----------|
| Не указано | Анонимизировать все строки (обратная совместимость) |
| `[]` | Отключить анонимизацию для инструмента |
| `["query", "body"]` | Анонимизировать только указанные поля |

### Пример: GitHub

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
        search_code: []                 # не анонимизировать поисковый запрос
```

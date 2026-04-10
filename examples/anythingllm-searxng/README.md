# Пример: AnythingLLM + PII Anonymizer + SearXNG Web Search

Интеграция AnythingLLM с PII Anonymizer, проксирующим MCP сервер **SearXNG** —
приватный мета-поисковик **без токенов**, работающий из РФ.

## Архитектура

```
┌─────────────────┐         ┌──────────────────────┐         ┌─────────────────┐
│   AnythingLLM   │────────▶│  PII Anonymizer MCP  │────────▶│  SearXNG MCP    │
│   (port 3001)   │  SSE    │  (port 3000)         │  stdio  │  (Docker)       │
└─────────────────┘         └──────────────────────┘         └────────┬────────┘
                                                                      │
                                                           ┌──────────▼──────────┐
                                                           │    SearXNG Engine   │
                                                           │  Google, DuckDuckGo,│
                                                           │  Bing, Wikipedia    │
                                                           └─────────────────────┘
```

## Быстрый старт

### 1. Запуск

```bash
cd examples/anythingllm-searxng
docker compose up -d
```

Запускаются 3 сервиса:
- **SearXNG** — приватный мета-поисковик (порт 8080)
- **PII Anonymizer** — MCP сервер с проксированием (порт 3000)
- **AnythingLLM** — веб-интерфейс (порт 3001)

### 2. Доступ

- **AnythingLLM**: http://localhost:3001
- **SearXNG UI**: http://localhost:8080
- **PII Anonymizer API**: http://localhost:3000/api/v1/health

### 3. Подключение MCP в AnythingLLM

1. Откройте http://localhost:3001 → **Settings** → **MCP Servers**
2. **Add new MCP server**:
   - **Name**: `PII Anonymizer`
   - **Type**: `SSE`
   - **URL**: `http://pii-anonymizer:3000/sse`
3. **Save** — сервер станет зелёным

### 4. Доступные инструменты

| Инструмент | Источник | Описание |
|------------|----------|----------|
| `anonymize` | PII Anonymizer | Анонимизация текста |
| `detect_pii` | PII Anonymizer | Обнаружение PII данных |
| `batch_anonymize` | PII Anonymizer | Пакетная обработка |
| `searxng_web_search` | SearXNG MCP | Веб-поиск через Google, DuckDuckGo, Bing |

### 5. Примеры запросов

**Веб-поиск:**
> "Найди последние новости про Rust программирование"

**Анонимизация:**
> "Анонимизируй текст: мой email user@example.com, телефон +7-999-123-4567"

**Комбинированный запрос:**
> "Найди информацию про уязвимости Log4j и анонимизируй найденные email адреса"

## Конфигурация SearXNG

Файл `searxng-settings.yml` управляет поисковыми движками.
Для добавления Яндекс, Википедии и других движков:

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

Документация: https://docs.searxng.org/admin/settings/settings.html

## Добавление других upstream серверов

В `../../config/settings.yaml` добавьте сервер:

```yaml
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

## Troubleshooting

```bash
# Логи всех сервисов
docker compose logs pii-anonymizer
docker compose logs searxng

# Проверка SearXNG
curl "http://localhost:8080/search?q=test&format=json"

# Проверка SSE endpoint
curl -N http://localhost:3000/sse

# Рестарт
docker compose restart pii-anonymizer
```

## Остановка

```bash
docker compose down
```

## Полный сброс

```bash
docker compose down -v
docker volume rm anythingllm-searxng_storage
```

# Конфигурация

## Файл конфигурации

`config/settings.yaml`:

```yaml
server:
  host: "0.0.0.0"
  port: 3000
  workers: 4

default_strategy: "mask"  # replace, mask, hash

patterns:
  - email
  - phone_ru
  - passport_ru
  - credit_card
  - ip_address
  - snils
  - inn
  - api_key_aws
  - api_key_github
  - access_token_jwt
  - ssh_key_rsa
  - ssh_key_ed25519
  - domain_unknown

mcp:
  enabled: true
  transport: "sse"  # sse, stdio
  server_name: "PII Anonymizer"
  server_version: "0.1.0"

proxy:
  upstream_servers: {}

logging:
  level: "info"  # trace, debug, info, warn, error
  format: "pretty"
```

## CLI аргументы

| Аргумент | Описание | По умолчанию |
|----------|----------|--------------|
| `-c, --config` | Путь к конфигу | `config/settings.yaml` |
| `--host` | Хост сервера | из конфига |
| `--port` | Порт сервера | из конфига |
| `-s, --strategy` | Стратегия анонимизации | из конфига |
| `--mcp-mode` | Режим MCP: `http`, `stdio` | `http` |
| `--log-level` | Уровень логирования | `info` |

```bash
pii-anonymizer --config config/settings.yaml --strategy hash --port 8080
```

## Переменные окружения

Приоритет: CLI > ENV > файл конфигурации.

```bash
ANONYMIZER__DEFAULT_STRATEGY=mask
ANONYMIZER__SERVER__HOST=0.0.0.0
ANONYMIZER__SERVER__PORT=3000
ANONYMIZER__LOGGING__LEVEL=debug
```

Разделитель `__` для вложенных ключей.

# Развёртывание

## Docker

```bash
docker compose up -d
```

## Ручная сборка

```bash
docker build -t pii-anonymizer .
docker run -p 3000:3000 -v $(pwd)/config:/app/config:ro pii-anonymizer
```

## Переменные окружения

```bash
docker run -p 3000:3000 \
  -e ANONYMIZER__DEFAULT_STRATEGY=hash \
  -e ANONYMIZER__LOGGING__LEVEL=debug \
  pii-anonymizer
```

## MCP Proxy — доступ к Docker

Для проксирования upstream MCP серверов через Docker нужен доступ к docker socket:

```yaml
# docker-compose.yml
services:
  pii-anonymizer:
    volumes:
      - /run/user/1000/docker.sock:/var/run/docker.sock  # rootless
      # или
      - /var/run/docker.sock:/var/run/docker.sock        # стандартный
    user: root
```

## Production

### Health check

```bash
curl http://localhost:3000/api/v1/health
```

### Рекомендуемые настройки

```yaml
server:
  workers: 4
logging:
  level: "warn"
  format: "json"
```

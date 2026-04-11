# Развёртывание

## Быстрый старт

### Docker (рекомендуется)

```bash
docker pull ghcr.io/<owner>/pii-anonymizer:latest
docker run -p 3000:3000 ghcr.io/<owner>/pii-anonymizer:latest
```

Мультиархитектурность — работает на x86_64, ARM64 и ARMv7:

| Платформа | Архитектура |
|-----------|-------------|
| x86_64 desktop/сервер | `amd64` |
| Raspberry Pi 4/5, Apple Silicon | `arm64` |
| Raspberry Pi 2/3 | `arm/v7` |

### Debian пакет

```bash
sudo dpkg -i pii-anonymizer_*.deb
sudo systemctl start pii-anonymizer
sudo systemctl status pii-anonymizer
```

### Бинарник

```bash
chmod +x pii-anonymizer-*-x86_64-linux
./pii-anonymizer-*-x86_64-linux
```

---

## Docker

### Pull из реестра

```bash
docker pull ghcr.io/<owner>/pii-anonymizer:latest
```

### Запуск с конфигом

```bash
docker run -p 3000:3000 \
  -v $(pwd)/config:/etc/pii-anonymizer \
  ghcr.io/<owner>/pii-anonymizer:latest
```

### Docker Compose

```bash
docker compose up -d
```

### Ручная сборка

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

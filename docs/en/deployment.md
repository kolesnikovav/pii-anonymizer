# Deployment

## Quick Start

### Docker (recommended)

```bash
docker pull ghcr.io/<owner>/pii-anonymizer:latest
docker run -p 3000:3000 ghcr.io/<owner>/pii-anonymizer:latest
```

Multi-architecture support — works on x86_64, ARM64, and ARMv7:

| Platform | Architecture |
|----------|-------------|
| x86_64 desktop/server | `amd64` |
| Raspberry Pi 4/5, Apple Silicon | `arm64` |
| Raspberry Pi 2/3 | `arm/v7` |

### Debian Package

```bash
sudo dpkg -i pii-anonymizer_*.deb
sudo systemctl start pii-anonymizer
sudo systemctl status pii-anonymizer
```

### Binary

```bash
chmod +x pii-anonymizer-*-x86_64-linux
./pii-anonymizer-*-x86_64-linux
```

---

## Docker

### Pull from Registry

```bash
docker pull ghcr.io/<owner>/pii-anonymizer:latest
```

### Run with Config

```bash
docker run -p 3000:3000 \
  -v $(pwd)/config:/etc/pii-anonymizer \
  ghcr.io/<owner>/pii-anonymizer:latest
```

### Docker Compose

```bash
docker compose up -d
```

### Manual Build

```bash
docker build -t pii-anonymizer .
docker run -p 3000:3000 -v $(pwd)/config:/app/config:ro pii-anonymizer
```

## Environment Variables

```bash
docker run -p 3000:3000 \
  -e ANONYMIZER__DEFAULT_STRATEGY=hash \
  -e ANONYMIZER__LOGGING__LEVEL=debug \
  pii-anonymizer
```

## MCP Proxy — Docker Access

To proxy upstream MCP servers through Docker, access to the Docker socket is required:

```yaml
# docker-compose.yml
services:
  pii-anonymizer:
    volumes:
      - /run/user/1000/docker.sock:/var/run/docker.sock  # rootless
      # or
      - /var/run/docker.sock:/var/run/docker.sock        # standard
    user: root
```

## Production

### Health Check

```bash
curl http://localhost:3000/api/v1/health
```

### Recommended Settings

```yaml
server:
  workers: 4
logging:
  level: "warn"
  format: "json"
```

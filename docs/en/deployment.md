# Deployment

## Docker

```bash
docker compose up -d
```

## Manual Build

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

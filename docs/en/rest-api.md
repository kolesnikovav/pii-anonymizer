# REST API

## Endpoints

### POST /api/v1/anonymize

Text anonymization.

```bash
curl -X POST http://localhost:3000/api/v1/anonymize \
  -H "Content-Type: application/json" \
  -d '{"text": "Email: john@test.com, phone: +7-999-123-45-67"}'
```

**Response:**
```json
{
  "anonymized_text": "Email: jo***@***om, phone: +79***67",
  "detected_pii": [
    {"pii_type": "email", "value": "john@test.com", "start": 7, "end": 20},
    {"pii_type": "phone", "value": "+7-999-123-45-67", "start": 29, "end": 45}
  ]
}
```

### POST /api/v1/detect

PII detection without replacement.

```bash
curl -X POST http://localhost:3000/api/v1/detect \
  -H "Content-Type: application/json" \
  -d '{"text": "Email: test@example.com"}'
```

### POST /api/v1/batch

Batch processing.

```bash
curl -X POST http://localhost:3000/api/v1/batch \
  -H "Content-Type: application/json" \
  -d '{"requests": [
    {"text": "Email: user1@test.com", "strategy": "mask"},
    {"text": "Phone: +7-999-123-45-67", "strategy": "hash"}
  ]}'
```

### GET /api/v1/health

Health check.

```bash
curl http://localhost:3000/api/v1/health
```

### SSE Streaming

- `GET /api/v1/sse/stream` -- SSE streaming for anonymization
- `GET /sse` -- MCP SSE endpoint for client connections

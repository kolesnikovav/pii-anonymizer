# MCP сервер

## Инструменты

| Инструмент | Описание | Параметры |
|------------|----------|-----------|
| `anonymize` | Анонимизация текста | `text` (req), `strategy` (opt) |
| `detect_pii` | Обнаружение PII | `text` (req) |
| `batch_anonymize` | Пакетная обработка | `texts` (req), `strategy` (opt) |

## Режимы запуска

| Режим | Транспорт | Для чего |
|-------|-----------|----------|
| HTTP SSE | `GET /sse` + `POST /message` | AnythingLLM, веб-клиенты |
| STDIO | stdin/stdout | Claude Desktop, VS Code |

## SSE Transport

### 1. Подключение

```bash
curl -N http://localhost:3000/sse
# Возвращает:
# event: endpoint
# data: /message?sessionId=abc123
```

### 2. Инициализация

```bash
curl -X POST "http://localhost:3000/message?sessionId=abc123" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"client","version":"0.1.0"}}}'
```

### 3. Список инструментов

```bash
curl -X POST "http://localhost:3000/message?sessionId=abc123" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}'
```

### 4. Вызов инструмента

```bash
curl -X POST "http://localhost:3000/message?sessionId=abc123" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"anonymize","arguments":{"text":"Email: test@example.com"}}}'
```

### Запуск в режиме stdio

```bash
cargo run -- --mcp-mode stdio
```

Для Claude Desktop, VS Code — подключается через stdin/stdout автоматически.

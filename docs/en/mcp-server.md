# MCP Server

## Tools

| Tool | Description | Parameters |
|------------|----------|-----------|
| `anonymize` | Text anonymization | `text` (req), `strategy` (opt) |
| `detect_pii` | PII detection | `text` (req) |
| `batch_anonymize` | Batch processing | `texts` (req), `strategy` (opt) |

## Launch Modes

| Mode | Transport | Use Case |
|-------|-----------|----------|
| HTTP SSE | `GET /sse` + `POST /message` | AnythingLLM, web clients |
| STDIO | stdin/stdout | Claude Desktop, VS Code |

## SSE Transport

### 1. Connection

```bash
curl -N http://localhost:3000/sse
# Returns:
# event: endpoint
# data: /message?sessionId=abc123
```

### 2. Initialization

```bash
curl -X POST "http://localhost:3000/message?sessionId=abc123" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"client","version":"0.1.0"}}}'
```

### 3. List Tools

```bash
curl -X POST "http://localhost:3000/message?sessionId=abc123" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}'
```

### 4. Tool Call

```bash
curl -X POST "http://localhost:3000/message?sessionId=abc123" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"anonymize","arguments":{"text":"Email: test@example.com"}}}'
```

### Running in stdio Mode

```bash
cargo run -- --mcp-mode stdio
```

For Claude Desktop, VS Code -- connects automatically via stdin/stdout.

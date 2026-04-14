# Quick Start

Get up and running with PII Anonymizer in minutes.

## Prerequisites

- **Rust 1.75+** (for building from source)
- **Docker** (optional, for containerized deployment)

## Installation

### Option 1: Build from Source

```bash
# Clone the repository
git clone https://github.com/kolesnikovav/pii-anonymizer.git
cd pii-anonymizer

# Build
cargo build --release

# Run
./target/release/pii-anonymizer --config-test
```

### Option 2: Docker

```bash
docker compose up -d
```

### Option 3: Download Binary

Download the latest release from [GitHub Releases](https://github.com/kolesnikovav/pii-anonymizer/releases).

## Running Modes

PII Anonymizer supports three operating modes:

=== "HTTP REST API"

    Default mode. Runs HTTP server with REST API endpoints.

    ```bash
    cargo run
    ```

    Server starts on `0.0.0.0:3000` (configured in `config/settings.yaml`).

=== "MCP Stdio"

    MCP server mode for integration with AI assistants (Claude Desktop, Cursor, etc).

    ```bash
    cargo run -- --mcp-mode stdio
    ```

    Communicates via standard input/output using JSON-RPC protocol.

=== "Config Test"

    Validate configuration file without starting the server.

    ```bash
    cargo run -- --config-test
    ```

    Output:
    ```
    pii-anonymizer ✓
    ├── strategy: mask
    ├── server: 0.0.0.0:3000
    ├── patterns [13]
    │   ├── email
    │   ├── phone_ru
    │   └── ...
    └── status: valid

    Configuration is valid
    ```

## First API Call

```bash
# Start server in background
cargo run &

# Anonymize text
curl -X POST http://localhost:3000/api/v1/anonymize \
  -H "Content-Type: application/json" \
  -d '{
    "text": "Contact: user@example.com, Phone: +7 (916) 123-45-67",
    "strategy": "replace"
  }'
```

Response:
```json
{
  "anonymized_text": "Contact: [EMAIL], Phone: [PHONE]",
  "detected_pii": [
    {"pii_type": "email", "original_value": "user@example.com"},
    {"pii_type": "phone", "original_value": "+7 (916) 123-45-67"}
  ]
}
```

## Configuration

Main config file: `config/settings.yaml`

```yaml
server:
  host: "0.0.0.0"
  port: 3000

default_strategy: "mask"  # replace, mask, hash

patterns:
  - email
  - phone_ru
  - passport_ru
  # ... more patterns
```

CLI arguments override config:

```bash
cargo run -- --host 127.0.0.1 --port 8080 --strategy hash
```

## Next Steps

- [Configuration](configuration.md) — detailed settings reference
- [REST API](rest-api.md) — all endpoints and examples
- [MCP Server](mcp-server.md) — integration with AI assistants

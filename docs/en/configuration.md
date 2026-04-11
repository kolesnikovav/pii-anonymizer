# Configuration

## Configuration File

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

## CLI Arguments

| Argument | Description | Default |
|----------|-------------|---------|
| `-c, --config` | Path to configuration file | `config/settings.yaml` |
| `--host` | Server host | from config |
| `--port` | Server port | from config |
| `-s, --strategy` | Anonymization strategy | from config |
| `--mcp-mode` | MCP mode: `http`, `stdio` | `http` |
| `--log-level` | Logging level | `info` |

```bash
pii-anonymizer --config config/settings.yaml --strategy hash --port 8080
```

## Environment Variables

Priority: CLI > ENV > configuration file.

```bash
ANONYMIZER__DEFAULT_STRATEGY=mask
ANONYMIZER__SERVER__HOST=0.0.0.0
ANONYMIZER__SERVER__PORT=3000
ANONYMIZER__LOGGING__LEVEL=debug
```

Use `__` as the separator for nested keys.

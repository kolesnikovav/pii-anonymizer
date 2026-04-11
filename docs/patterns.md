# PII Patterns

## Personal Data

| Pattern | Description | Example |
|---------|----------|--------|
| `email` | Email addresses | `user@example.com` |
| `phone_ru` | Russian phone numbers | `+7-999-123-45-67` |
| `passport_ru` | Russian passports | `4510 123456` |
| `snils` | SNILS (Russian pension insurance number) | `123-456-789 00` |
| `inn` | INN (tax ID for individuals/entities) | `7707083893` |

## Financial

| Pattern | Description | Example |
|---------|----------|--------|
| `credit_card` | Credit cards | `4111 1111 1111 1111` |

## Technical Secrets

| Pattern | Description | Example |
|---------|----------|--------|
| `api_key_aws` | AWS keys | `AKIAIOSFODNN7EXAMPLE` |
| `api_key_github` | GitHub tokens | `ghp_...`, `gho_...` |
| `access_token_jwt` | JWT tokens | `eyJhbG...` |
| `ssh_key_rsa` | SSH RSA keys | `ssh-rsa AAAA...` |
| `ssh_key_ed25519` | SSH ED25519 keys | `ssh-ed25519 AAAA...` |

## Network Data

| Pattern | Description | Example |
|---------|----------|--------|
| `ip_address` | IPv4 addresses | `192.168.1.1` |
| `domain_unknown` | Unknown domains | `secret-server.ru` |

**Smart domain filtering**: skips 30+ known domains (google.com, yandex.ru, github.com, etc.)

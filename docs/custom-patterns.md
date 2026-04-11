# Custom Patterns and Domains

PII Anonymizer allows you to add custom regex patterns for PII detection and configure a list of known domains that should not be masked.

## How it works

All settings are read **once at startup** from `config/settings.yaml`. To apply changes, **restart the service**.

## Custom PII Patterns

Add your own regex patterns to the `custom_patterns` section:

```yaml
custom_patterns:
  - name: "order_number"
    pii_type: "unknown"
    pattern: "\\bORD-\\d{6,}\\b"
    confidence: 0.9

  - name: "employee_id"
    pii_type: "unknown"
    pattern: "\\bEMP-[A-Z]{2}-\\d{4}\\b"
    confidence: 0.85
```

### Pattern Fields

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Unique pattern name |
| `pii_type` | string | PII type: `email`, `phone`, `passport`, `credit_card`, `ip_address`, `snils`, `inn`, `address`, `full_name`, `api_key`, `access_token`, `ssh_key`, `domain`, `unknown` |
| `pattern` | string | Regex pattern (escape `\` as `\\`) |
| `confidence` | float | Confidence level 0.0–1.0 (default: 0.85) |

### Custom Pattern Examples

**Order number:**
```yaml
- name: "order_number"
  pii_type: "unknown"
  pattern: "\\bORD-\\d{6,}\\b"
  confidence: 0.9
```
Text: `Order ORD-123456 confirmed` → `Order [UNKNOWN_1] confirmed`

**Employee ID:**
```yaml
- name: "employee_id"
  pii_type: "unknown"
  pattern: "\\bEMP-[A-Z]{2}-\\d{4}\\b"
  confidence: 0.85
```
Text: `Employee EMP-AB-1234 access granted` → `Employee [UNKNOWN_1] access granted`

**License key:**
```yaml
- name: "license_key"
  pii_type: "api_key"
  pattern: "\\b[A-Z]{5}-[A-Z]{5}-[A-Z]{5}-[A-Z]{5}\\b"
  confidence: 0.9
```
Text: `Key: ABCDE-FGHIJ-KLMNO-PQRST` → `Key: ABC***ST`

## Custom Known Domains

Domains in this list **will not be masked**. Built-in known domains include: google.com, yandex.ru, github.com, and others (see `src/anonymizer/patterns.rs`).

Add your own domains:

```yaml
custom_known_domains:
  - "my-company.com"
  - "internal.corp"
  - "partner-site.ru"
```

### Example

**Without custom domain:**
```
Text: Visit https://internal.corp/dashboard
Result: Visit https://in***rp/dashboard
```

**With custom domain:**
```yaml
custom_known_domains:
  - "internal.corp"
```
```
Text: Visit https://internal.corp/dashboard
Result: Visit https://internal.corp/dashboard  (unchanged)
```

## Built-in Patterns

List of built-in patterns from `src/anonymizer/patterns.rs`:

| Name | Type | Description |
|------|------|-------------|
| `email` | email | Email addresses |
| `phone_ru` | phone | Russian phone numbers |
| `phone_intl` | phone | International phone numbers |
| `passport_ru` | passport | Russian passports |
| `credit_card` | credit_card | Credit card numbers |
| `ip_address` | ip_address | IP addresses |
| `snils` | snils | Russian SNILS |
| `api_key_aws` | api_key | AWS access keys |
| `api_key_github` | api_key | GitHub tokens |
| `api_key_google` | api_key | Google API keys |
| `access_token_jwt` | access_token | JWT tokens |
| `ssh_key_rsa` | ssh_key | RSA SSH keys |
| `ssh_key_ed25519` | ssh_key | ED25519 SSH keys |
| `domain_unknown` | domain | Unknown domains |

## Full Configuration Example

```yaml
default_strategy: "mask"
patterns:
  - email
  - phone_ru
  - ip_address
  - domain_unknown

custom_patterns:
  - name: "order_number"
    pii_type: "unknown"
    pattern: "\\bORD-\\d{6,}\\b"
    confidence: 0.9
  - name: "license_key"
    pii_type: "api_key"
    pattern: "\\b[A-Z]{5}-[A-Z]{5}-[A-Z]{5}-[A-Z]{5}\\b"
    confidence: 0.9

custom_known_domains:
  - "my-company.com"
  - "internal.corp"
```

## Validation Errors

If a regex pattern is invalid, the service **will skip it** with a warning in the logs and continue running:

```
⚠️ Skipping custom pattern 'bad_regex': regex parse error
```

---

*Documentation: April 11, 2026*

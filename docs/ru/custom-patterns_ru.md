# Кастомные паттерны и домены

PII Anonymizer позволяет добавлять собственные regex-паттерны для обнаружения PII и настраивать список известных доменов, которые не нужно маскировать.

## Как это работает

Все настройки читаются **один раз при запуске** сервиса из файла `config/settings.yaml`. Для применения изменений **перезапустите сервис**.

## Кастомные PII паттерны

Добавьте свои regex-паттерны в секцию `custom_patterns`:

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

### Поля паттерна

| Поле | Тип | Описание |
|------|-----|----------|
| `name` | string | Уникальное имя паттерна |
| `pii_type` | string | Тип PII: `email`, `phone`, `passport`, `credit_card`, `ip_address`, `snils`, `inn`, `address`, `full_name`, `api_key`, `access_token`, `ssh_key`, `domain`, `unknown` |
| `pattern` | string | Regex паттерн (экранируйте `\` как `\\`) |
| `confidence` | float | Уровень уверенности 0.0–1.0 (по умолчанию 0.85) |

### Примеры кастомных паттернов

**Номер заказа:**
```yaml
- name: "order_number"
  pii_type: "unknown"
  pattern: "\\bORD-\\d{6,}\\b"
  confidence: 0.9
```
Текст: `Order ORD-123456 confirmed` → `Order [UNKNOWN_1] confirmed`

**Внутренний ID сотрудника:**
```yaml
- name: "employee_id"
  pii_type: "unknown"
  pattern: "\\bEMP-[A-Z]{2}-\\d{4}\\b"
  confidence: 0.85
```
Текст: `Employee EMP-AB-1234 access granted` → `Employee [UNKNOWN_1] access granted`

**Лицензионный ключ:**
```yaml
- name: "license_key"
  pii_type: "api_key"
  pattern: "\\b[A-Z]{5}-[A-Z]{5}-[A-Z]{5}-[A-Z]{5}\\b"
  confidence: 0.9
```
Текст: `Key: ABCDE-FGHIJ-KLMNO-PQRST` → `Key: ABC***ST`

## Кастомные известные домены

Домены из этого списка **не будут маскироваться**. Встроенные известные домены включают: google.com, yandex.ru, github.com и другие (см. `src/anonymizer/patterns.rs`).

Добавьте свои домены:

```yaml
custom_known_domains:
  - "my-company.com"
  - "internal.corp"
  - "partner-site.ru"
```

### Пример

**Без кастомного домена:**
```
Текст: Visit https://internal.corp/dashboard
Результат: Visit https://in***rp/dashboard
```

**С кастомным доменом:**
```yaml
custom_known_domains:
  - "internal.corp"
```
```
Текст: Visit https://internal.corp/dashboard
Результат: Visit https://internal.corp/dashboard  (не изменён)
```

## Встроенные паттерны

Список встроенных паттернов можно посмотреть в `src/anonymizer/patterns.rs`:

| Имя | Тип | Описание |
|-----|-----|----------|
| `email` | email | Email адреса |
| `phone_ru` | phone | Российские телефоны |
| `phone_intl` | phone | Международные телефоны |
| `passport_ru` | passport | Паспорта РФ |
| `credit_card` | credit_card | Кредитные карты |
| `ip_address` | ip_address | IP адреса |
| `snils` | snils | СНИЛС |
| `api_key_aws` | api_key | AWS ключи |
| `api_key_github` | api_key | GitHub токены |
| `api_key_google` | api_key | Google API ключи |
| `access_token_jwt` | access_token | JWT токены |
| `ssh_key_rsa` | ssh_key | RSA SSH ключи |
| `ssh_key_ed25519` | ssh_key | ED25519 SSH ключи |
| `domain_unknown` | domain | Неизвестные домены |

## Полный пример конфигурации

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

## Ошибки валидации

Если regex паттерн невалидный, сервис **пропустит его** с предупреждением в логах и продолжит работу:

```
⚠️ Пропущен кастомный паттерн 'bad_regex': Невалидный regex: regex parse error
```

---

*Документация: 11 апреля 2026*

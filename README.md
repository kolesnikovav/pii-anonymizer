# PII Anonymizer MCP Server

Сервис для анонимизации поисковых запросов с удалением персональных данных (PII). Работает как HTTP REST API и MCP сервер с поддержкой проксирования.

## 🚀 Возможности

- 🔍 **Обнаружение PII**:
  - Персональные: email, телефоны, паспорта РФ, СНИЛС, ИНН
  - Финансовые: кредитные карты
  - Технические: **API ключи** (AWS, GitHub, Google), **токены доступа** (JWT), **SSH ключи** (RSA, ED25519, ECDSA)
  - Сетевые: IP-адреса, **домены** (с фильтрацией общеизвестных)
- 🎭 **3 стратегии маскирования**:
  - **Replace**: Полная замена → `[EMAIL_1]`, `[API_KEY_2]`, `[SSH_KEY_3]`
  - **Mask**: Частичная маска → `te***om`, `+79***67`, `AKIA***MPLE`
  - **Hash**: Частичный хеш → `te_4f2a8b1c@om`, `+79_8e3f2a1d67`, `AKIA_4f2a8bMPLE`
- 🌐 **HTTP REST API** с CORS и middleware
- 🤖 **MCP Server** с инструментами для LLM
- 🔄 **MCP Proxy** для проксирования к другим MCP серверам
- 📡 **SSE** (Server-Sent Events) для стриминга
- 🐳 **Docker** готовность
- ⚙️ **CLI** с чтением конфигурации из файла
- 🛡 **Умное маскирование доменов**: пропускает google.com, yandex.ru и другие известные домены

## 📋 Быстрый старт

### Локальный запуск

```bash
# Клонирование
git clone <repo-url> && cd pii-anonymizer

# Запуск с конфигурацией по умолчанию
cargo run

# Запуск с указанием файла конфигурации
cargo run -- --config config/settings.yaml

# Переопределение настроек через CLI
cargo run -- --config config/settings.yaml --strategy hash --port 8080

# MCP режим (stdio)
cargo run -- --mcp-mode stdio --config config/settings.yaml
```

### Docker

```bash
# Сборка и запуск
docker-compose up -d

# Или вручную
docker build -t pii-anonymizer .
docker run -p 3000:3000 -v $(pwd)/config:/app/config pii-anonymizer
```

## ⚙️ Конфигурация

### Файл конфигурации (config/settings.yaml)

```yaml
server:
  host: "0.0.0.0"
  port: 3000

anonymizer:
  default_strategy: "mask"  # replace, mask, hash
  patterns:
    - email
    - phone_ru
    - passport_ru
    - credit_card
    - ip_address
    - snils
    - inn

mcp:
  enabled: true
  server_name: "PII Anonymizer"
  server_version: "0.1.0"

proxy:
  enabled: false
  upstream_servers:
    - name: "external-ai"
      url: "http://ai-service:8080/mcp"
      timeout: 30

logging:
  level: "info"  # trace, debug, info, warn, error
  format: "pretty"
```

### CLI аргументы

```bash
pii-anonymizer [OPTIONS]

Options:
  -c, --config <FILE>      Путь к файлу конфигурации [default: config/settings.yaml]
      --host <HOST>        Хост сервера (переопределяет конфиг)
      --port <PORT>        Порт сервера (переопределяет конфиг)
  -s, --strategy <STRAT>   Стратегия: replace, mask, hash
      --mcp-mode <MODE>    Режим MCP: stdio, http [default: http]
      --log-level <LEVEL>  Уровень логирования [default: info]
  -h, --help               Показать справку
```

### Переменные окружения

```bash
ANONYMIZER__SERVER__HOST=0.0.0.0
ANONYMIZER__SERVER__PORT=3000
ANONYMIZER__ANONYMIZER__DEFAULT_STRATEGY=mask
ANONYMIZER__LOGGING__LEVEL=info
```

## 🌐 REST API

### Endpoints

#### POST /api/v1/anonymize
Анонимизация текста

```bash
curl -X POST http://localhost:3000/api/v1/anonymize \
  -H "Content-Type: application/json" \
  -d '{
    "text": "Contact: john.doe@company.org, phone: +7 (999) 123-45-67",
    "strategy": "mask"
  }'
```

**Ответ (strategy: "replace")**:
```json
{
  "original_text": "Contact: john.doe@company.org, phone: +7 (999) 123-45-67",
  "anonymized_text": "Contact: [EMAIL_1], phone: [PHONE_2]",
  "detected_pii": [
    {"pii_type": "email", "value": "john.doe@company.org", "start": 10, "end": 30, "confidence": 0.98},
    {"pii_type": "phone", "value": "+7 (999) 123-45-67", "start": 40, "end": 57, "confidence": 0.95}
  ],
  "strategy": "replace"
}
```

**Ответ (strategy: "mask")**:
```json
{
  "anonymized_text": "Contact: jo***@***rg, phone: +79***67"
}
```

**Ответ (strategy: "hash")**:
```json
{
  "anonymized_text": "Contact: te_4f2a8b1c@om, phone: +79_8e3f2a1d67"
}
```

#### Примеры обнаружения новых PII

**API ключи**:
```bash
# AWS
Input:  "My AWS key is AKIAIOSFODNN7EXAMPLE"
Output: "My AWS key is AKIA***MPLE"  (mask)
        "My AWS key is [API_KEY_1]"  (replace)

# GitHub
Input:  "Token: ghp_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefgh12"
Output: "Token: ghp_***h12"  (mask)
```

**Токены доступа (JWT)**:
```bash
Input:  "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U"
Output: "Bearer eyJ_***R8U"  (mask)
        "Bearer [ACCESS_TOKEN_1]"  (replace)
```

**SSH ключи**:
```bash
Input:  "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQDVvvHkGphJbBX8"
Output: "ssh-rsa AAAA***BX8"  (mask)
        "ssh-rsa [SSH_KEY_1]"  (replace)
```

**Домены (с фильтрацией известных)**:
```bash
# Известные домены пропускаются
Input:  "Search on google.com or visit secret-server.ru"
Output: "Search on google.com or visit secr***.ru"  (google.com пропущен!)

# Приватные домены маскируются
Input:  "Internal API: https://api.my-company.internal/v1"
Output: "Internal API: htt***v1"  (mask)
        "Internal API: [DOMAIN_1]"  (replace)
```

#### POST /api/v1/detect
Обнаружение PII без замены

```bash
curl -X POST http://localhost:3000/api/v1/detect \
  -H "Content-Type: application/json" \
  -d '{"text": "Email: test@example.com"}'
```

#### POST /api/v1/batch
Пакетная обработка

```bash
curl -X POST http://localhost:3000/api/v1/batch \
  -H "Content-Type: application/json" \
  -d '{
    "requests": [
      {"text": "Email: user1@test.com", "strategy": "mask"},
      {"text": "Phone: +7 (999) 123-45-67", "strategy": "hash"}
    ]
  }'
```

#### POST /api/v1/stats
Статистика PII

```bash
curl -X POST http://localhost:3000/api/v1/stats \
  -H "Content-Type: application/json" \
  -d '{"text": "Email: a@b.com, Phone: +79991234567, Email: c@d.com"}'
```

#### GET /api/v1/health
Health check

```bash
curl http://localhost:3000/api/v1/health
```

#### GET /api/v1/sse/stream
SSE стриминг

```bash
curl -N http://localhost:3000/api/v1/sse/stream
```

#### GET /sse
MCP SSE endpoint для подключения клиентов к серверу как к MCP серверу.

```bash
curl -N http://localhost:3000/sse
curl -N "http://localhost:3000/sse?session_id=my-session"
```

При подключении клиент получает событие инициализации с информацией о сервере.

#### POST /sse/message
Отправка MCP JSON-RPC сообщений на сервер.

```bash
# Initialize
curl -X POST http://localhost:3000/sse/message \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {}
  }'

# Список инструментов
curl -X POST http://localhost:3000/sse/message \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/list",
    "params": {}
  }'

# Вызов инструмента anonymize
curl -X POST http://localhost:3000/sse/message \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 3,
    "method": "tools/call",
    "params": {
      "name": "anonymize",
      "arguments": {
        "text": "Contact: john@example.com",
        "strategy": "mask"
      }
    }
  }'

# Вызов инструмента detect_pii
curl -X POST http://localhost:3000/sse/message \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 4,
    "method": "tools/call",
    "params": {
      "name": "detect_pii",
      "arguments": {
        "text": "Email: user@test.com, Phone: +7 (999) 123-45-67"
      }
    }
  }'
```

**Ответ tools/list**:
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "tools": [
      {
        "name": "anonymize",
        "description": "Анонимизировать текст, удаляя PII данные",
        "input_schema": { ... }
      },
      {
        "name": "detect_pii",
        "description": "Обнаружить PII данные в тексте",
        "input_schema": { ... }
      },
      {
        "name": "batch_anonymize",
        "description": "Пакетная анонимизация нескольких текстов",
        "input_schema": { ... }
      }
    ]
  }
}
```

**Ответ tools/call (anonymize)**:
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\"anonymized_text\":\"Contact: jo***@***e.com\",\"detected_pii_count\":1,...}"
      }
    ]
  }
}
```

## 🤖 MCP Интеграция

### Поддержка SSE Transport

Сервер поддерживает MCP через SSE transport, что позволяет подключаться клиентам, использующим SSE для коммуникации.

**Архитектура**:
1. Клиент подключается к `GET /sse` для получения SSE stream
2. Клиент отправляет JSON-RPC сообщения на `POST /sse/message`
3. Сервер обрабатывает запросы и возвращает результаты через оба канала

**Поддерживаемые JSON-RPC методы**:
- `initialize` - инициализация подключения
- `notifications/initialized` - уведомление о завершении инициализации
- `tools/list` - получение списка доступных инструментов
- `tools/call` - вызов конкретного инструмента

### Доступные инструменты

| Инструмент | Описание | Параметры |
|------------|----------|-----------|
| `anonymize` | Анонимизировать текст | `text` (string), `strategy` (string, optional) |
| `detect_pii` | Обнаружить PII | `text` (string) |
| `batch_anonymize` | Пакетная обработка | `texts` (array), `strategy` (string, optional) |

### Интеграция с AnythingLLM

1. **Настройка MCP сервера**:

```json
// В настройках AnythingLLM → Settings → MCP Servers
{
  "mcpServers": {
    "pii-anonymizer": {
      "command": "pii-anonymizer",
      "args": ["--mcp-mode", "stdio", "--config", "/path/to/config.yaml"],
      "transport": "stdio"
    }
  }
}
```

2. **Использование в чате**:

```
User: Анонимизируй этот текст: "Contact john@test.com for info"
Assistant: [Использует инструмент anonymize]
Result: "Contact [EMAIL_1] for info"
```

### Интеграция с VS Code (GitHub Copilot / Claude)

1. **Установка MCP через расширение**:

```json
// .vscode/mcp.json
{
  "servers": {
    "pii-anonymizer": {
      "type": "stdio",
      "command": "cargo",
      "args": ["run", "--", "--mcp-mode", "stdio", "--config", "config/settings.yaml"]
    }
  }
}
```

2. **Использование в Copilot Chat**:

```
@workspace Анонимизируй email и телефоны в этом файле
```

### Интеграция с Claude Desktop

```json
// %APPDATA%/Claude/claude_desktop_config.json
{
  "mcpServers": {
    "pii-anonymizer": {
      "command": "pii-anonymizer.exe",
      "args": ["--mcp-mode", "stdio", "--strategy", "mask"],
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

## 🔄 Пример: Проксирование поисковых запросов

### Сценарий
Поисковые запросы проходят через анонимизатор перед отправкой во внешние AI сервисы.

### Конфигурация proxy

```yaml
# config/proxy.yaml
proxy:
  enabled: true
  upstream_servers:
    - name: "openai-service"
      url: "http://openai-mcp:8080"
      timeout: 30
    - name: "claude-service"
      url: "http://claude-mcp:8081"
      timeout: 30
    - name: "local-llm"
      url: "http://localhost:8082"
      timeout: 60
```

### Запуск с проксированием

```bash
# Запуск с конфигурацией прокси
cargo run -- --config config/proxy.yaml

# Анонимизация + проксирование к upstream
curl -X POST http://localhost:3000/api/v1/anonymize \
  -H "Content-Type: application/json" \
  -d '{
    "text": "My email is john.doe@company.com, call me at +7 (999) 123-45-67. How to fix bug #1234?",
    "strategy": "hash"
  }'
```

### Результат

**Оригинал**:
```
"My email is john.doe@company.com, call me at +7 (999) 123-45-67. How to fix bug #1234?"
```

**Анонимизировано (hash)**:
```
"My email is jo_4f2a8b1c@om, call me at +79_8e3f2a1d67. How to fix bug #1234?"
```

Теперь можно безопасно отправить в внешний AI сервис!

## 🎭 Стратегии маскирования

### 1. Replace - Полная замена

Заменяет PII на плейсхолдеры с счётчиком.

```
Input:  "Email: john@test.com, AWS Key: AKIAIOSFODNN7EXAMPLE"
Output: "Email: [EMAIL_1], AWS Key: [API_KEY_2]"
```

**Плюсы**: Полная анонимность, легко подсчитать PII  
**Минусы**: Теряется контекст данных

### 2. Mask - Частичная маска

Сохраняет первые и последние символы, остальное `***`.

```
Email:    "john.doe@company.org"  →  "jo***@***rg"
Phone:    "+7 (999) 123-45-67"    →  "+79***67"
API Key:  "AKIAIOSFODNN7EXAMPLE"  →  "AKIA***MPLE"
SSH Key:  "ssh-rsa AAAAB3Nza..."  →  "ssh-rsa AAAA***..."
Domain:   "secret-server.ru"      →  "sec***.ru"
```

**Плюсы**: Сохраняется формат данных  
**Минусы**: Частичное раскрытие информации

### 3. Hash - Частичный хеш

Хеширует среднюю часть, сохраняет контекст.

```
Email:    "john.doe@company.org"  →  "jo_4f2a8b1c@om"
Phone:    "+7 (999) 123-45-67"    →  "+79_8e3f2a1d67"
API Key:  "AKIAIOSFODNN7EXAMPLE"  →  "AKIA_4f2a8bMPLE"
SSH Key:  "ssh-rsa AAAAB3Nza..."  →  "ssh-rsa AAAA_4f2a...Nza"
```

**Плюсы**: Обратимость отсутствует, сохраняется структура  
**Минусы**: Хеш может быть расшифрован перебором

## 🔐 Поддерживаемые PII

### Персональные данные
- ✅ Email адреса
- ✅ Телефоны (РФ + международные)
- ✅ Паспорта РФ
- ✅ СНИЛС
- ✅ ИНН (физ + юр лица)

### Финансовые данные
- ✅ Кредитные карты

### Технические секреты
- ✅ **API ключи**:
  - AWS (AKIA...)
  - GitHub (ghp_, gho_, ghs_, ghp_, ghr_)
  - Google (AIza...)
  - Generic (api_key=..., bearer token)
- ✅ **Токены доступа**:
  - JWT (eyJ...)
  - Generic (access_token=..., auth_token=...)
- ✅ **SSH ключи**:
  - RSA (ssh-rsa ...)
  - ED25519 (ssh-ed25519 ...)
  - ECDSA (ecdsa-sha2-nistp...)

### Сетевые данные
- ✅ IP-адреса (IPv4)
- ✅ **Домены** с умной фильтрацией:
  - Пропускает: google.com, yandex.ru, github.com, etc. (30+ доменов)
  - Маскирует: приватные и неизвестные домены

## 🧪 Тестирование

```bash
# Запуск всех тестов
cargo test

# Запуск с выводом
cargo test -- --nocapture

# Интеграционные тесты
cargo test --test api_integration_test

# Покрытие (требует cargo-tarpaulin)
cargo tarpaulin --out Html
```

**Текущий статус**: ✅ 80 тестов проходят успешно

## 📁 Структура проекта

```
pii-anonymizer/
├── src/
│   ├── main.rs                 # Точка входа + CLI
│   ├── lib.rs                  # Публичное API
│   ├── config/                 # Конфигурация
│   ├── anonymizer/             # Ядро анонимизатора
│   │   ├── engine.rs           # Движок обработки
│   │   ├── patterns.rs         # PII паттерны
│   │   └── strategies.rs       # Стратегии маскирования
│   ├── api/                    # REST API
│   │   ├── routes.rs           # Маршруты
│   │   ├── handlers.rs         # Обработчики
│   │   └── error.rs            # Обработка ошибок
│   ├── mcp/                    # MCP сервер/клиент
│   │   ├── server.rs           # MCP сервер
│   │   ├── client.rs           # MCP клиент
│   │   └── proxy.rs            # Проксирование
│   ├── sse/                    # SSE поддержка
│   ├── middleware/             # Middleware
│   └── models/                 # Модели данных
├── tests/                      # Интеграционные тесты
├── config/
│   └── settings.yaml           # Конфигурация по умолчанию
├── Dockerfile
├── docker-compose.yml
└── Cargo.toml
```

## 🔧 Разработка

### Добавление нового PII паттерна

```rust
// src/anonymizer/patterns.rs
PIIPattern::with_confidence(
    "new_pattern_name",
    PIIType::NewType,
    r"\bregex_pattern\b",
    0.95, // confidence
).unwrap(),
```

### Добавление новой стратегии

```rust
// src/anonymizer/strategies.rs
impl AnonymizationStrategy {
    fn new_strategy(&self, value: &str) -> String {
        // Логика маскирования
    }
}
```

## 🐳 Docker

### Multi-stage build

```dockerfile
FROM rust:1.75-slim-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/pii-anonymizer /usr/local/bin/
EXPOSE 3000
CMD ["pii-anonymizer"]
```

### docker-compose

```yaml
version: '3.8'
services:
  pii-anonymizer:
    build: .
    ports:
      - "3000:3000"
    volumes:
      - ./config:/app/config:ro
    environment:
      - ANONYMIZER__ANONYMIZER__DEFAULT_STRATEGY=hash
```

## 📊 Производительность

- **Обработка текста**: < 10ms для текстов до 10KB
- **Пакетная обработка**: 1000 запросов/сек
- **Память**: < 50MB в idle

## 🛡 Безопасность

- ✅ PII не сохраняется в логах
- ✅ Хэширование с односторонней функцией
- ✅ Нет внешних зависимостей с доступом к данным
- ✅ Валидация входных данных

## 📊 Сравнение с аналогами

Подробное сравнение с Presidio, Scrubadub и другими решениями: **[COMPARISON.md](COMPARISON.md)**

| Критерий | PII Anonymizer | Presidio | Scrubadub |
|----------|:--------------:|:--------:|:---------:|
| Производительность | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ |
| Технические PII | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ |
| MCP интеграция | ⭐⭐⭐⭐⭐ | ❌ | ❌ |
| NLP/ML | ❌ | ⭐⭐⭐⭐⭐ | ❌ |


## 📝 Лицензия

MIT

## 🤝 Contributing

1. Fork репозиторий
2. Создайте ветку (`git checkout -b feature/amazing-feature`)
3. Commit изменения (`git commit -m 'feat: add amazing feature'`)
4. Push в ветку (`git push origin feature/amazing-feature`)
5. Откройте Pull Request

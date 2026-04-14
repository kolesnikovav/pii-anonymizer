# Быстрый старт

Запустите PII Anonymizer за несколько минут.

## Требования

- **Rust 1.75+** (для сборки из исходников)
- **Docker** (опционально, для контейнеризированного развёртывания)

## Установка

### Вариант 1: Сборка из исходников

```bash
# Клонировать репозиторий
git clone https://github.com/kolesnikovav/pii-anonymizer.git
cd pii-anonymizer

# Собрать
cargo build --release

# Запустить
./target/release/pii-anonymizer --config-test
```

### Вариант 2: Docker

```bash
docker compose up -d
```

### Вариант 3: Скачать бинарный файл

Скачайте последний релиз со страницы [GitHub Releases](https://github.com/kolesnikovav/pii-anonymizer/releases).

## Режимы работы

PII Anonymizer поддерживает три режима работы:

=== "HTTP REST API"

    Режим по умолчанию. Запускает HTTP сервер с REST API endpoints.

    ```bash
    cargo run
    ```

    Сервер запускается на `0.0.0.0:3000` (настроено в `config/settings.yaml`).

=== "MCP Stdio"

    Режим MCP сервера для интеграции с AI ассистентами (Claude Desktop, Cursor и др).

    ```bash
    cargo run -- --mcp-mode stdio
    ```

    Общение происходит через стандартный ввод/вывод с использованием JSON-RPC протокола.

=== "Проверка конфигурации"

    Проверка файла конфигурации без запуска сервера.

    ```bash
    cargo run -- --config-test
    ```

    Вывод:
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

## Первый запрос к API

```bash
# Запустить сервер в фоне
cargo run &

# Анонимизировать текст
curl -X POST http://localhost:3000/api/v1/anonymize \
  -H "Content-Type: application/json" \
  -d '{
    "text": "Контакт: user@example.com, Тел: +7 (916) 123-45-67",
    "strategy": "replace"
  }'
```

Ответ:
```json
{
  "anonymized_text": "Контакт: [EMAIL], Тел: [PHONE]",
  "detected_pii": [
    {"pii_type": "email", "original_value": "user@example.com"},
    {"pii_type": "phone", "original_value": "+7 (916) 123-45-67"}
  ]
}
```

## Конфигурация

Основной файл конфигурации: `config/settings.yaml`

```yaml
server:
  host: "0.0.0.0"
  port: 3000

default_strategy: "mask"  # replace, mask, hash

patterns:
  - email
  - phone_ru
  - passport_ru
  # ... другие паттерны
```

Аргументы командной строки переопределяют конфигурацию:

```bash
cargo run -- --host 127.0.0.1 --port 8080 --strategy hash
```

## Следующие шаги

- [Конфигурация](configuration.md) — подробное описание настрое
- [REST API](rest-api.md) — все endpoints и примеры
- [MCP сервер](mcp-server.md) — интеграция с AI ассистентами

# CI/CD Пайплайн

## Обзор

Проект использует GitHub Actions для автоматического тестирования, сборки и публикации релизов.

## Воркфлоу

### CI (`ci.yml`)

Запускается при каждом push в `master` и при pull request'ах:

- **cargo fmt** — проверка форматирования кода
- **cargo clippy** — линт с ошибками за предупреждения
- **cargo test** — запуск всех тестов
- **cargo build --release** — сборка релизного бинарника

### Release (`release.yml`)

Запускается по тегу версии (`v*`) или вручную через `workflow_dispatch`.

**Задачи:**

| Задача | Описание |
|--------|----------|
| `test` | Запуск тестов и линт |
| `build-binary` | Сборка stripped релизного бинарника |
| `build-deb` | Сборка Debian пакета (.deb) с systemd сервисом |
| `build-docker` | Сборка и публикация Docker-образа в GHCR |
| `create-release` | Создание GitHub Release со всеми артефактами |

## Создание релиза

### Способ 1: Git тег (рекомендуется)

```bash
# Обновите версию в Cargo.toml
# Затем:
git tag v0.2.0
git push origin v0.2.0
```

### Способ 2: Через GitHub UI

1. Перейдите в **Actions** → **Release**
2. Нажмите **Run workflow**
3. Введите версию (например `0.2.0`)
4. Нажмите **Run workflow**

## Артефакты

Каждый релиз создаёт:

| Артефакт | Где хранится | Пример |
|----------|-------------|--------|
| Docker-образ | GHCR | `ghcr.io/<owner>/pii-anonymizer:0.2.0` |
| .deb пакет | GitHub Release | `pii-anonymizer_0.2.0_amd64.deb` |
| Бинарник | GitHub Release | `pii-anonymizer-v0.2.0-x86_64-linux` |

## Установка

### Docker

```bash
docker pull ghcr.io/<owner>/pii-anonymizer:0.2.0
docker run -p 3000:3000 \
  -v $(pwd)/config:/etc/pii-anonymizer \
  ghcr.io/<owner>/pii-anonymizer:0.2.0
```

### Debian

```bash
sudo dpkg -i pii-anonymizer_0.2.0_amd64.deb

# Конфиг: /etc/pii-anonymizer/settings.yaml
sudo systemctl start pii-anonymizer
sudo systemctl status pii-anonymizer
sudo systemctl enable pii-anonymizer
```

### Бинарник

```bash
chmod +x pii-anonymizer-v0.2.0-x86_64-linux
./pii-anonymizer-v0.2.0-x86_64-linux --config-test
./pii-anonymizer-v0.2.0-x86_64-linux
```

## Содержимое .deb пакета

| Путь | Описание |
|------|----------|
| `/usr/local/bin/pii-anonymizer` | Бинарник |
| `/etc/pii-anonymizer/settings.yaml` | Конфиг (conffile) |
| `/lib/systemd/system/pii-anonymizer.service` | systemd сервис |
| `/usr/share/doc/pii-anonymizer/README.md` | Документация |

.deb пакет собирается через [cargo-deb](https://github.com/kornelski/cargo-deb).

## Docker теги

| Тег | Описание |
|-----|----------|
| `latest` | Последний релиз |
| `0.2.0` | Конкретная версия |

# CI/CD Pipeline

## Overview

The project uses GitHub Actions for automated testing, building, and publishing releases.

## Workflows

### CI (`ci.yml`)

Runs on every push to `master` and on pull requests:

- **cargo fmt** — check code formatting
- **cargo clippy** — lint with warnings as errors
- **cargo test** — run all tests
- **cargo build --release** — build release binary

### Release (`release.yml`)

Triggered by pushing a version tag (`v*`) or manually via `workflow_dispatch`.

**Jobs:**

| Job | Description |
|-----|-------------|
| `test` | Run tests and linting |
| `build-binary` | Build stripped release binary |
| `build-deb` | Build Debian package (.deb) with systemd service |
| `build-docker` | Build and push Docker image to GHCR |
| `create-release` | Create GitHub Release with all artifacts |

## Creating a Release

### Method 1: Git Tag (recommended)

```bash
# Update version in Cargo.toml
# Then:
git tag v0.2.0
git push origin v0.2.0
```

### Method 2: GitHub UI

1. Go to **Actions** → **Release**
2. Click **Run workflow**
3. Enter version (e.g. `0.2.0`)
4. Click **Run workflow**

## Artifacts

Each release produces:

| Artifact | Location | Example |
|----------|----------|---------|
| Docker image | GHCR | `ghcr.io/<owner>/pii-anonymizer:0.2.0` |
| .deb package | GitHub Release | `pii-anonymizer_0.2.0_amd64.deb` |
| Binary | GitHub Release | `pii-anonymizer-v0.2.0-x86_64-linux` |

## Installing

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

# Config is at /etc/pii-anonymizer/settings.yaml
sudo systemctl start pii-anonymizer
sudo systemctl status pii-anonymizer
sudo systemctl enable pii-anonymizer
```

### Binary

```bash
chmod +x pii-anonymizer-v0.2.0-x86_64-linux
./pii-anonymizer-v0.2.0-x86_64-linux --config-test
./pii-anonymizer-v0.2.0-x86_64-linux
```

## .deb Package Contents

| Path | Description |
|------|-------------|
| `/usr/local/bin/pii-anonymizer` | Binary |
| `/etc/pii-anonymizer/settings.yaml` | Config file (marked as conffile) |
| `/lib/systemd/system/pii-anonymizer.service` | systemd service |
| `/usr/share/doc/pii-anonymizer/README.md` | Documentation |

The `.deb` package is built using [cargo-deb](https://github.com/kornelski/cargo-deb).

## Docker Tags

| Tag | Description |
|-----|-------------|
| `latest` | Latest release |
| `0.2.0` | Specific version |

## Supported Architectures

| Architecture | Platform | Example |
|-------------|----------|---------|
| `amd64` | x86_64 | Desktop, server |
| `arm64` | AArch64 | Raspberry Pi 4/5, Apple Silicon |
| `arm/v7` | ARMv7 | Raspberry Pi 2/3 |

Docker автоматически выберет нужную архитектуру при `docker pull`:

```bash
# На Raspberry Pi:
docker pull ghcr.io/<owner>/pii-anonymizer:0.2.0
# автоматически скачается arm64 или arm/v7 образ

# Проверить:
docker inspect ghcr.io/<owner>/pii-anonymizer:0.2.0 | grep Architecture
```

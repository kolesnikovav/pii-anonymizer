# Multi-stage build
FROM rust:1.86-slim-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Копирование файлов манифеста для кеширования зависимостей
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && echo "pub fn dummy() {}" > src/lib.rs
RUN cargo build --release --locked 2>/dev/null || true
RUN rm -f target/release/deps/pii_anonymizer*

# Копирование исходного кода (только src и config)
COPY src/ ./src/
COPY config/ ./config/

# Финальная сборка
RUN cargo build --release --locked
RUN strip /app/target/release/pii-anonymizer

# Финальный образ (минимальный)
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && rm -rf /var/cache/apt/*

WORKDIR /app

# Копирование только бинарника и конфига
COPY --from=builder /app/target/release/pii-anonymizer /usr/local/bin/
COPY --from=builder /app/config/settings.yaml /app/config/

# Настройка окружения
ENV RUST_LOG=info
EXPOSE 3000
EXPOSE 3001

# Health check (на порту 3001)
HEALTHCHECK --interval=30s --timeout=10s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:3001/api/v1/health || exit 1

# Запуск приложения
CMD ["pii-anonymizer"]

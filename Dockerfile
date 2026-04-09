# Multi-stage build
FROM rust:1.75-slim-bookworm AS builder

WORKDIR /app

# Копирование файлов манифеста для кеширования
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --locked || true

# Копирование исходного кода
COPY . .
RUN cargo build --release --locked

# Финальный образ
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Копирование бинарника
COPY --from=builder /app/target/release/pii-anonymizer /usr/local/bin/
COPY --from=builder /app/config/settings.yaml /app/config/

# Настройка окружения
ENV RUST_LOG=info
EXPOSE 3000

# Запуск приложения
CMD ["pii-anonymizer"]

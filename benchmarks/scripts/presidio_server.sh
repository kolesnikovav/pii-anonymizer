#!/usr/bin/env bash
set -e

# Скрипт для запуска и остановки Presidio сервера
# Использование: ./presidio_server.sh start|stop|restart

PRESIDIO_PORT=5002
PRESIDIO_PIDFILE="/tmp/presidio_server.pid"

start_presidio() {
    echo "Запуск Presidio сервера на порту $PRESIDIO_PORT..."
    
    # Проверка, запущен ли уже
    if [ -f "$PRESIDIO_PIDFILE" ]; then
        PID=$(cat "$PRESIDIO_PIDFILE")
        if kill -0 $PID 2>/dev/null; then
            echo "Presidio уже запущен (PID: $PID)"
            return 0
        fi
    fi
    
    # Запуск Presidio через Docker
    docker run -d \
        --name presidio-anonymizer \
        -p $PRESIDIO_PORT:3000 \
        mcr.microsoft.com/presidio-anonymizer:latest &
    
    # Сохраняем PID контейнера
    docker inspect -f '{{.State.Pid}}' presidio-anonymizer > "$PRESIDIO_PIDFILE" 2>/dev/null || true
    
    echo "Ожидание запуска Presidio..."
    for i in {1..30}; do
        if curl -s "http://localhost:$PRESIDIO_PORT/health" > /dev/null 2>&1; then
            echo "✓ Presidio запущен"
            return 0
        fi
        sleep 1
    done
    
    echo "❌ Presidio не запустился за 30 секунд"
    return 1
}

stop_presidio() {
    echo "Остановка Presidio..."
    
    if [ -f "$PRESIDIO_PIDFILE" ]; then
        PID=$(cat "$PRESIDIO_PIDFILE")
        if kill -0 $PID 2>/dev/null; then
            docker stop presidio-anonymizer 2>/dev/null || true
            docker rm presidio-anonymizer 2>/dev/null || true
            echo "✓ Presidio остановлен"
        fi
        rm -f "$PRESIDIO_PIDFILE"
    else
        # Попытка остановить контейнер по имени
        docker stop presidio-anonymizer 2>/dev/null || true
        docker rm presidio-anonymizer 2>/dev/null || true
    fi
    
    echo "Presidio остановлен"
}

status_presidio() {
    if [ -f "$PRESIDIO_PIDFILE" ]; then
        PID=$(cat "$PRESIDIO_PIDFILE")
        if kill -0 $PID 2>/dev/null; then
            echo "Presidio запущен (PID: $PID)"
            return 0
        fi
    fi
    
    if docker ps --format '{{.Names}}' | grep -q presidio-anonymizer; then
        echo "Presidio запущен в Docker"
        return 0
    fi
    
    echo "Presidio не запущен"
    return 1
}

case "${1:-status}" in
    start)
        start_presidio
        ;;
    stop)
        stop_presidio
        ;;
    restart)
        stop_presidio
        sleep 2
        start_presidio
        ;;
    status)
        status_presidio
        ;;
    *)
        echo "Использование: $0 {start|stop|restart|status}"
        exit 1
        ;;
esac

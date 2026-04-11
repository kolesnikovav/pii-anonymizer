#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RESULTS_DIR="$SCRIPT_DIR/results"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

mkdir -p "$RESULTS_DIR"

echo "========================================"
echo "  PII Anonymizer vs Presidio"
echo "  Бенчмарк производительности"
echo "========================================"
echo ""

# Цвета для вывода
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Переменные для результатов
PII_ANON_TIME=""
PII_ANON_THROUGHPUT=""
PRESIDIO_TIME=""
PRESIDIO_THROUGHPUT=""

# ========== БЕНЧМАРК PII ANONYMIZER ==========
echo -e "${GREEN}[1/2]${NC} Запуск бенчмарка PII Anonymizer..."
echo ""

# Собираем и запускаем бенчмарк
cd "$PROJECT_DIR"
cargo build --release 2>&1 | tail -5

echo ""
echo "Запуск бенчмарка..."
PII_BENCH_RESULT=$(cargo run --release --bin benchmark_pii_anonymizer 2>&1 | tee "$RESULTS_DIR/pii_anonymizer.log")

echo "$PII_BENCH_RESULT"
echo ""

# Парсим результаты
PII_ANON_TIME=$(echo "$PII_BENCH_RESULT" | grep "Время:" | awk '{print $2}')
PII_ANON_THROUGHPUT=$(echo "$PII_BENCH_RESULT" | grep "Производительность:" | awk '{print $2}')

echo ""

# ========== БЕНЧМАРК PRESIDIO ==========
echo -e "${GREEN}[2/2]${NC} Запуск бенчмарка Presidio..."
echo ""

# Проверяем доступность Docker
if ! command -v docker &> /dev/null; then
    echo -e "${YELLOW}⚠ Docker не найден. Пропускаем бенчмарк Presidio.${NC}"
    echo ""
else
    # Запускаем Presidio
    echo "Запуск Presidio сервера..."
    bash "$SCRIPT_DIR/scripts/presidio_server.sh" start
    
    echo ""
    echo "Ожидание полной готовности Presidio..."
    sleep 5
    
    # Запускаем бенчмарк Presidio
    echo "Запуск бенчмарка Presidio..."
    PRESIDIO_BENCH_RESULT=$(python3 "$SCRIPT_DIR/benchmark_presidio.py" http://localhost:5002 2>&1 | tee "$RESULTS_DIR/presidio.log")
    
    echo "$PRESIDIO_BENCH_RESULT"
    echo ""
    
    # Парсим результаты
    PRESIDIO_TIME=$(echo "$PRESIDIO_BENCH_RESULT" | grep "Время:" | awk '{print $2}')
    PRESIDIO_THROUGHPUT=$(echo "$PRESIDIO_BENCH_RESULT" | grep "Производительность:" | awk '{print $2}')
    
    # Останавливаем Presidio
    echo ""
    echo "Остановка Presidio..."
    bash "$SCRIPT_DIR/scripts/presidio_server.sh" stop
fi

# ========== ИТОГОВЫЕ РЕЗУЛЬТАТЫ ==========
echo ""
echo "========================================"
echo "  Итоговые результаты"
echo "========================================"
echo ""

echo "PII Anonymizer:"
echo "  Время: ${PII_ANON_TIME:-N/A} сек"
echo "  Производительность: ${PII_ANON_THROUGHPUT:-N/A} текстов/сек"
echo ""

if [ -n "$PRESIDIO_TIME" ]; then
    echo "Presidio:"
    echo "  Время: ${PRESIDIO_TIME:-N/A} сек"
    echo "  Производительность: ${PRESIDIO_THROUGHPUT:-N/A} текстов/сек"
    echo ""
    
    # Вычисляем соотношение
    if [ -n "$PII_ANON_THROUGHPUT" ] && [ -n "$PRESIDIO_THROUGHPUT" ]; then
        PII_NUM=$(echo "$PII_ANON_THROUGHPUT" | sed 's/[^0-9.]//g')
        PRESIDIO_NUM=$(echo "$PRESIDIO_THROUGHPUT" | sed 's/[^0-9.]//g')
        
        if [ -n "$PII_NUM" ] && [ -n "$PRESIDIO_NUM" ] && [ "$PRESIDIO_NUM" != "0" ]; then
            RATIO=$(echo "scale=1; $PII_NUM / $PRESIDIO_NUM" | bc 2>/dev/null || echo "N/A")
            echo "PII Anonymizer быстрее в $RATIO раз(а)"
        fi
    fi
else
    echo "Presidio: пропущено"
    echo ""
fi

echo ""

# Сохраняем результаты в файл
cat > "$RESULTS_DIR/benchmark_results.txt" << EOF
Дата: $(date '+%Y-%m-%d %H:%M:%S')
PII Anonymizer:
  Время: ${PII_ANON_TIME:-N/A} сек
  Производительность: ${PII_ANON_THROUGHPUT:-N/A} текстов/сек

Presidio:
  Время: ${PRESIDIO_TIME:-N/A} сек
  Производительность: ${PRESIDIO_THROUGHPUT:-N/A} текстов/сек
EOF

echo "Результаты сохранены в: $RESULTS_DIR/benchmark_results.txt"
echo ""
echo "✓ Бенчмарк завершен"

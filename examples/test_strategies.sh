#!/bin/bash
# Быстрое тестирование всех стратегий PII Anonymizer
# Использование: ./examples/test_strategies.sh

set -e

API_URL="${API_URL:-http://localhost:8080/api/v1/anonymize}"
HEALTH_URL="${HEALTH_URL:-http://localhost:8080/api/v1/health}"

# Цвета для вывода
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Примеры текстов
declare -A SAMPLES
SAMPLES[email]="Контакт: ivan.petrov@gmail.com или support@example.org"
SAMPLES[phone]="Звоните: +7 (916) 123-45-67 или 8-800-555-35-35"
SAMPLES[snils]="Номер СНИЛС: 123-456-789 00"
SAMPLES[inn]="ИНН физического лица: 7707083893"
SAMPLES[passport]="Паспорт: серия 4508 номер 123456, выдан 01.02.2020"
SAMPLES[card]="Номер карты: 4276 5500 1234 5678"
SAMPLES[complex]="Пользователь ivanov.ivan@company.ru, тел: +7 (495) 987-65-43, СНИЛС: 111-222-333 44"

STRATEGIES=("replace" "mask" "hash")

# Проверка сервера
echo -e "${BLUE}Проверка сервера...${NC}"
if curl -s "$HEALTH_URL" > /dev/null; then
    echo -e "${GREEN}✅ Сервер доступен${NC}\n"
else
    echo -e "${YELLOW}❌ Сервер не доступен. Запустите: cargo run --bin pii-anonymizer${NC}"
    exit 1
fi

# Тестирование
echo "═══════════════════════════════════════════════════════════"
echo "PII ANONYMIZER - ТЕСТ СТРАТЕГИЙ"
echo "═══════════════════════════════════════════════════════════"

for strategy in "${STRATEGIES[@]}"; do
    echo -e "\n${BLUE}⚙️  СТРАТЕГИЯ: ${strategy^^}${NC}"
    echo "───────────────────────────────────────────────────────"

    for name in "${!SAMPLES[@]}"; do
        text="${SAMPLES[$name]}"
        
        echo -e "\n📝 $name"
        echo "   Оригинал: $text"
        
        # Отправляем запрос
        response=$(curl -s -X POST "$API_URL" \
            -H "Content-Type: application/json" \
            -d "{\"text\": \"$text\", \"strategy\": \"$strategy\"}")
        
        # Извлекаем результат
        anonymized=$(echo "$response" | jq -r '.anonymized_text' 2>/dev/null || echo "Ошибка")
        pii_count=$(echo "$response" | jq -r '.detected_pii | length' 2>/dev/null || echo "0")
        
        echo -e "   Результат: ${GREEN}$anonymized${NC}"
        echo "   Найдено PII: $pii_count"
    done
    
    echo ""
    echo "───────────────────────────────────────────────────────"
done

echo -e "\n${GREEN}✅ Тестирование завершено${NC}"

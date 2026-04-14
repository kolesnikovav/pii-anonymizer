#!/usr/bin/env python3
"""
Демонстрация работы PII Anonymizer со всеми стратегиями.
Запускает сервер и показывает результаты анонимизации для каждого примера.

Использование:
    python examples/demo_strategies.py          # Все примеры и стратегии
    python examples/demo_strategies --complex   # Только комплексный пример
    python examples/demo_strategies --email     # Только email примеры
"""

import subprocess
import sys
import time
import requests
import json
from pathlib import Path

# Примеры текстов
SAMPLES = {
    "email": "Контакт: ivan.petrov@gmail.com или support@example.org",
    
    "phone": "Звоните: +7 (916) 123-45-67 или 8-800-555-35-35",
    
    "snils": "Номер СНИЛС: 123-456-789 00",
    
    "inn": "ИНН физического лица: 7707083893",
    
    "passport": "Паспорт: серия 4508 номер 123456, выдан 01.02.2020",
    
    "card": "Номер карты: 4276 5500 1234 5678, срок 12/25, CVV 123",
    
    "api_key": "GitHub Token: ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
    
    "jwt": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c",
    
    "ip": "Сервер доступен по адресу 192.168.1.100 или 10.0.0.1",
    
    "domain": "Сайт: https://example.com/api/v1/users",
    
    "complex": """Пользователь Иванов Иван Иванович зарегистрировался на сайте example.com.
Email: ivanov.ivan@company.ru
Телефон: +7 (495) 987-65-43
СНИЛС: 111-222-333 44
ИНН: 7707123456
Паспорт: 4510 654321
Карта: 5213 4567 8901 2345
IP: 172.16.254.1"""
}

STRATEGIES = ["replace", "mask", "hash"]

API_URL = "http://localhost:8080/api/v1/anonymize"


def check_server_running():
    """Проверка, запущен ли сервер."""
    try:
        resp = requests.get("http://localhost:8080/api/v1/health", timeout=2)
        return resp.status_code == 200
    except:
        return False


def start_server():
    """Запуск сервера в фоне."""
    print("🚀 Запуск PII Anonymizer сервера...")
    project_root = Path(__file__).parent.parent
    proc = subprocess.Popen(
        [sys.executable, "-m", "cargo", "run", "--bin", "pii-anonymizer"],
        cwd=project_root,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE
    )
    # Ждем запуска
    for i in range(10):
        time.sleep(1)
        if check_server_running():
            print("✅ Сервер запущен\n")
            return proc
    print("❌ Не удалось запустить сервер")
    return None


def anonymize_text(text: str, strategy: str) -> dict:
    """Отправить запрос на анонимизацию."""
    payload = {"text": text, "strategy": strategy}
    resp = requests.post(API_URL, json=payload, timeout=5)
    resp.raise_for_status()
    return resp.json()


def print_separator():
    print("─" * 80)


def print_example(name: str, original: str, strategy: str, result: dict):
    """Вывод результата для одного примера."""
    print(f"\n📝 {name}")
    print(f"   Оригинал: {original}")
    print(f"   Стратегия: {strategy}")
    print(f"   Результат: {result['anonymized_text']}")
    print(f"   Найдено PII: {len(result['detected_pii'])}")
    if result['detected_pii']:
        for pii in result['detected_pii'][:3]:  # Показываем первые 3
            print(f"      - {pii['pii_type']}: {pii['original_value'][:30]}...")
    print()


def run_demo(samples_filter=None):
    """Запуск демонстрации."""
    server_proc = None
    try:
        # Проверяем, запущен ли сервер
        if not check_server_running():
            server_proc = start_server()
            if not server_proc:
                sys.exit(1)
        else:
            print("✅ Сервер уже запущен\n")

        # Фильтрация примеров
        if samples_filter:
            samples = {k: v for k, v in SAMPLES.items() if k in samples_filter}
        else:
            samples = SAMPLES

        # Вывод
        print("=" * 80)
        print("PII ANONYMIZER - ДЕМО ВСЕХ СТРАТЕГИЙ")
        print("=" * 80)

        for strategy in STRATEGIES:
            print_separator()
            print(f"⚙️  СТРАТЕГИЯ: {strategy.upper()}")
            print_separator()

            for name, text in samples.items():
                try:
                    result = anonymize_text(text, strategy)
                    print_example(name, text, strategy, result)
                except Exception as e:
                    print(f"❌ Ошибка для '{name}': {e}\n")

    finally:
        if server_proc:
            print("\n🛑 Остановка сервера...")
            server_proc.terminate()


if __name__ == "__main__":
    import argparse
    
    parser = argparse.ArgumentParser(description="PII Anonymizer Demo")
    parser.add_argument("--complex", action="store_true", help="Только комплексный пример")
    parser.add_argument("--email", action="store_true", help="Только email примеры")
    args = parser.parse_args()

    if args.complex:
        run_demo(["complex"])
    elif args.email:
        run_demo(["email"])
    else:
        run_demo()

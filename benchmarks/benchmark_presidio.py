#!/usr/bin/env python3
"""Бенчмарк для Presidio - оценка скорости анонимизации"""

import time
import requests
import sys

def load_test_data():
    """Генерация тестовых данных"""
    emails = [
        "user1@example.com", "john.doe@company.org", "admin@test.ru",
        "support@service.net", "info@business.com", "contact@site.org",
        "hello@world.com", "team@startup.io", "dev@tech.co", "mail@domain.ru"
    ]
    
    phones = [
        "+7 (999) 123-45-67", "+7 (916) 234-56-78", "+7 (903) 345-67-89",
        "+7 (926) 456-78-90", "+7 (999) 567-89-01", "+7 (916) 678-90-12",
        "+7 (903) 789-01-23", "+7 (926) 890-12-34", "+7 (999) 901-23-45", "+7 (916) 012-34-56"
    ]
    
    ips = [
        "192.168.1.1", "10.0.0.1", "172.16.0.1", "8.8.8.8", "1.1.1.1",
        "192.168.0.100", "10.10.10.10", "172.20.0.1", "255.255.255.0", "127.0.0.1"
    ]
    
    credit_cards = [
        "4111111111111111", "5500000000000004", "340000000000009",
        "4012888888881881", "5105105105105100", "378282246310005",
        "4222222222222", "5555555555554444", "371449635398431", "4000000000000002"
    ]
    
    texts = []
    
    # Генерация различных текстов
    for i in range(100):
        idx = i % 10
        
        # Email
        texts.append(f"Contact me at {emails[idx]}")
        texts.append(f"Send email to {emails[(idx + 1) % 10]} for support")
        
        # Phone
        texts.append(f"Call {phones[idx]}")
        texts.append(f"Phone number: {phones[(idx + 1) % 10]}")
        
        # IP
        texts.append(f"Server IP: {ips[idx]}")
        texts.append(f"Connected from {ips[(idx + 1) % 10]}")
        
        # Credit Card
        texts.append(f"Card: {credit_cards[idx]}")
        texts.append(f"Payment with {credit_cards[(idx + 1) % 10]}")
        
        # Комбинированные тексты
        texts.append(f"User {emails[idx]} with phone {phones[idx]} and IP {ips[idx]}")
        texts.append(f"Contact: {emails[(idx + 3) % 10]}, Phone: {phones[(idx + 5) % 10]}")
        texts.append(f"Server {ips[idx]} at IP {ips[(idx + 1) % 10]} with card {credit_cards[idx]}")
    
    return texts


def benchmark_presidio(analyzer_url, anonymizer_url, texts):
    """Бенчмарк Presidio через HTTP API"""
    print("=== Presidio Benchmark ===")
    print(f"Текстов: {len(texts)}")
    print(f"Analyzer URL: {analyzer_url}")
    print(f"Anonymizer URL: {anonymizer_url}")
    print()

    # Проверка доступности
    try:
        response = requests.get(f"{analyzer_url}/health", timeout=5)
        if response.status_code != 200:
            print(f"❌ Presidio Analyzer не доступен (status: {response.status_code})")
            sys.exit(1)
    except requests.exceptions.RequestException as e:
        print(f"❌ Presidio Analyzer не доступен: {e}")
        sys.exit(1)

    try:
        response = requests.get(f"{anonymizer_url}/health", timeout=5)
        if response.status_code != 200:
            print(f"❌ Presidio Anonymizer не доступен (status: {response.status_code})")
            sys.exit(1)
    except requests.exceptions.RequestException as e:
        print(f"❌ Presidio Anonymizer не доступен: {e}")
        sys.exit(1)

    print("✓ Presidio доступен")
    print("Запуск бенчмарка...")
    print()

    # Бенчмарк
    start = time.time()
    success_count = 0
    error_count = 0

    for i, text in enumerate(texts):
        try:
            # Анализ
            analyze_response = requests.post(
                f"{analyzer_url}/analyze",
                json={"text": text, "language": "en"},
                timeout=10
            )

            if analyze_response.status_code == 200:
                results = analyze_response.json()

                # Анонимизация
                if results:
                    anonymize_payload = {
                        "text": text,
                        "anonymizers": {
                            "DEFAULT": {"type": "replace", "new_value": "<ANONYMIZED>"}
                        },
                        "analyzer_results": results
                    }
                    
                    anonymize_response = requests.post(
                        f"{anonymizer_url}/anonymize",
                        json=anonymize_payload,
                        timeout=10
                    )

                    if anonymize_response.status_code == 200:
                        success_count += 1
                    else:
                        error_count += 1
                else:
                    success_count += 1
            else:
                error_count += 1

        except Exception as e:
            error_count += 1
            print(f"Ошибка на тексте {i}: {e}")

        if (i + 1) % 100 == 0:
            print(f"Обработано: {i + 1} текстов")

    duration = time.time() - start
    total = len(texts)
    throughput = total / duration if duration > 0 else 0

    print()
    print("=== Результаты ===")
    print(f"Всего текстов: {total}")
    print(f"Успешно: {success_count}")
    print(f"Ошибок: {error_count}")
    print(f"Время: {duration:.3f} сек")
    print(f"Производительность: {throughput:.0f} текстов/сек")


if __name__ == "__main__":
    analyzer_url = sys.argv[1] if len(sys.argv) > 1 else "http://localhost:5001"
    anonymizer_url = sys.argv[2] if len(sys.argv) > 2 else "http://localhost:5002"
    texts = load_test_data()
    benchmark_presidio(analyzer_url, anonymizer_url, texts)

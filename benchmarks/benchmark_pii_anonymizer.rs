use std::time::Instant;
use pii_anonymizer::anonymizer::AnonymizerEngine;
use pii_anonymizer::config::AnonymizerSettings;
use pii_anonymizer::models::AnonymizeRequest;

fn main() {
    // Настройки с всеми паттернами
    let settings = AnonymizerSettings {
        default_strategy: "mask".to_string(),
        patterns: vec![
            "email".to_string(),
            "phone_ru".to_string(),
            "passport_ru".to_string(),
            "credit_card".to_string(),
            "ip_address".to_string(),
            "snils".to_string(),
            "inn".to_string(),
            "api_key_aws".to_string(),
            "api_key_github".to_string(),
            "api_key_google".to_string(),
            "access_token_jwt".to_string(),
            "ssh_key_rsa".to_string(),
            "ssh_key_ed25519".to_string(),
            "domain_unknown".to_string(),
        ],
        custom_patterns: vec![],
        custom_known_domains: vec![],
    };

    let engine = AnonymizerEngine::new(&settings);

    // Тестовые данные
    let test_texts = load_test_data();
    
    println!("=== PII Anonymizer Benchmark ===");
    println!("Текстов: {}", test_texts.len());
    println!();

    // Бенчмарк
    let start = Instant::now();
    
    for (i, text) in test_texts.iter().enumerate() {
        let request = AnonymizeRequest {
            text: text.clone(),
            strategy: None,
        };
        let _result = engine.anonymize(&request);
        
        if (i + 1) % 1000 == 0 {
            println!("Обработано: {} текстов", i + 1);
        }
    }
    
    let duration = start.elapsed();
    let total = test_texts.len();
    let throughput = total as f64 / duration.as_secs_f64();
    
    println!();
    println!("=== Результаты ===");
    println!("Всего текстов: {}", total);
    println!("Время: {:.3} сек", duration.as_secs_f64());
    println!("Производительность: {:.0} текстов/сек", throughput);
}

fn load_test_data() -> Vec<String> {
    let mut texts = Vec::new();
    
    // Базовые шаблоры для генерации
    let emails = vec![
        "user1@example.com", "john.doe@company.org", "admin@test.ru",
        "support@service.net", "info@business.com", "contact@site.org",
        "hello@world.com", "team@startup.io", "dev@tech.co", "mail@domain.ru"
    ];
    
    let phones = vec![
        "+7 (999) 123-45-67", "+7 (916) 234-56-78", "+7 (903) 345-67-89",
        "+7 (926) 456-78-90", "+7 (999) 567-89-01", "+7 (916) 678-90-12",
        "+7 (903) 789-01-23", "+7 (926) 890-12-34", "+7 (999) 901-23-45", "+7 (916) 012-34-56"
    ];
    
    let ips = vec![
        "192.168.1.1", "10.0.0.1", "172.16.0.1", "8.8.8.8", "1.1.1.1",
        "192.168.0.100", "10.10.10.10", "172.20.0.1", "255.255.255.0", "127.0.0.1"
    ];
    
    let passports = vec![
        "4510 123456", "4515 234567", "4520 345678", "4525 456789", "4530 567890",
        "4535 678901", "4540 789012", "4545 890123", "4550 901234", "4555 012345"
    ];
    
    let snils = vec![
        "123-456-789 00", "234-567-890 11", "345-678-901 22", "456-789-012 33", "567-890-123 44",
        "678-901-234 55", "789-012-345 66", "890-123-456 77", "901-234-567 88", "012-345-678 99"
    ];
    
    let aws_keys = vec![
        "AKIAIOSFODNN7EXAMPLE", "AKIAI44QH8DHBEXAMPLE", "AKIAIOSFODNN8EXAMPLE",
        "AKIAI44QH8DHCAMPLE", "AKIAIOSFODNN9EXAMPLE", "AKIAI44QH8DHDAAMPLE",
        "AKIAIOSFODNNAEXAMPLE", "AKIAI44QH8DHDAAMMPLE", "AKIAIOSFODNNBEXAMPLE", "AKIAI44QH8DHDAAAMPLE"
    ];
    
    let ssh_keys = vec![
        "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQDVvvHkGphJbBX8",
        "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIGtIvqxKPmN",
        "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAACAQDGpBvHkGphJbBX9",
        "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIHtJwryLQnO",
        "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQDWrCvIlHqiKcCY0",
        "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIIuKxszMRoP",
        "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQDXsDwJmIrlLdDZ1",
        "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIJvLyatNSpQ",
        "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQDYtExKnJsmMeEa2",
        "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIKwMzbuOTqR"
    ];
    
    // Генерация различных текстов
    for i in 0..100 {
        let idx = i % 10;
        
        // Email
        texts.push(format!("Contact me at {}", emails[idx]));
        texts.push(format!("Send email to {} for support", emails[(idx + 1) % 10]));
        
        // Phone
        texts.push(format!("Call {}", phones[idx]));
        texts.push(format!("Phone number: {}", phones[(idx + 1) % 10]));
        
        // IP
        texts.push(format!("Server IP: {}", ips[idx]));
        texts.push(format!("Connected from {}", ips[(idx + 1) % 10]));
        
        // Passport
        texts.push(format!("Passport: {}", passports[idx]));
        texts.push(format!("Document number {}", passports[(idx + 1) % 10]));
        
        // SNILS
        texts.push(format!("SNILS: {}", snils[idx]));
        texts.push(format!("Insurance number {}", snils[(idx + 1) % 10]));
        
        // AWS Key
        texts.push(format!("AWS key: {}", aws_keys[idx]));
        texts.push(format!("Access key ID: {}", aws_keys[(idx + 1) % 10]));
        
        // SSH Key
        texts.push(format!("{}", ssh_keys[idx]));
        texts.push(format!("Public key: {}", ssh_keys[(idx + 1) % 10]));
        
        // Комбинированные тексты
        texts.push(format!("User {} with phone {} and IP {}", emails[idx], phones[idx], ips[idx]));
        texts.push(format!("Contact: {}, Phone: {}", emails[(idx + 3) % 10], phones[(idx + 5) % 10]));
        texts.push(format!("Server {} at IP {} with key {}", ips[idx], ips[(idx + 1) % 10], aws_keys[idx]));
    }
    
    texts
}

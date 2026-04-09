use pii_anonymizer::config::{AnonymizerSettings, Settings, McpSettings, ProxySettings, LoggingSettings};
use pii_anonymizer::anonymizer::AnonymizerEngine;
use pii_anonymizer::api::create_router;
use pii_anonymizer::models::AnonymizeRequest;

fn create_test_app() -> axum::Router {
    let settings = Settings {
        server: pii_anonymizer::config::ServerSettings {
            host: "127.0.0.1".to_string(),
            port: 3000,
            workers: 1,
        },
        anonymizer: AnonymizerSettings {
            default_strategy: "mask".to_string(),
            patterns: vec![
                "email".to_string(),
                "phone_ru".to_string(),
                "ip_address".to_string(),
            ],
        },
        mcp: McpSettings {
            enabled: false,
            transport: "sse".to_string(),
            server_name: "test".to_string(),
            server_version: "0.1.0".to_string(),
        },
        proxy: ProxySettings {
            enabled: false,
            upstream_servers: vec![],
        },
        logging: LoggingSettings {
            level: "info".to_string(),
            format: "pretty".to_string(),
        },
    };
    
    let anonymizer = AnonymizerEngine::new(&settings.anonymizer);
    create_router(settings, anonymizer)
}

#[tokio::test]
async fn test_health_check() {
    let app = create_test_app();
    
    let response = axum_test::TestServer::new(app)
        .unwrap()
        .get("/api/v1/health")
        .await;
    
    assert_eq!(response.status_code(), 200);
    let json: serde_json::Value = response.json();
    assert_eq!(json["status"], "healthy");
    assert_eq!(json["service"], "pii-anonymizer");
}

#[tokio::test]
async fn test_anonymize_email() {
    let app = create_test_app();
    
    let request = AnonymizeRequest {
        text: "Contact me at test@example.com".to_string(),
        strategy: None,
    };
    
    let response = axum_test::TestServer::new(app)
        .unwrap()
        .post("/api/v1/anonymize")
        .json(&request)
        .await;
    
    assert_eq!(response.status_code(), 200);
    let json: serde_json::Value = response.json();
    assert!(!json["anonymized_text"].as_str().unwrap().contains("test@example.com"));
    assert!(json["detected_pii"].as_array().unwrap().len() > 0);
}

#[tokio::test]
async fn test_anonymize_with_strategy() {
    let app = create_test_app();
    
    let request = AnonymizeRequest {
        text: "Email: user@test.com".to_string(),
        strategy: Some("replace".to_string()),
    };
    
    let response = axum_test::TestServer::new(app)
        .unwrap()
        .post("/api/v1/anonymize")
        .json(&request)
        .await;
    
    assert_eq!(response.status_code(), 200);
    let json: serde_json::Value = response.json();
    assert!(json["anonymized_text"].as_str().unwrap().contains("[EMAIL_"));
}

#[tokio::test]
async fn test_detect_pii() {
    let app = create_test_app();
    
    let request = serde_json::json!({
        "text": "Phone: +7 (999) 123-45-67"
    });
    
    let response = axum_test::TestServer::new(app)
        .unwrap()
        .post("/api/v1/detect")
        .json(&request)
        .await;
    
    assert_eq!(response.status_code(), 200);
    let json: serde_json::Value = response.json();
    assert!(json["total_found"].as_u64().unwrap() > 0);
}

#[tokio::test]
async fn test_anonymize_empty_text() {
    let app = create_test_app();
    
    let request = AnonymizeRequest {
        text: "".to_string(),
        strategy: None,
    };
    
    let response = axum_test::TestServer::new(app)
        .unwrap()
        .post("/api/v1/anonymize")
        .json(&request)
        .await;
    
    // Должна вернуться ошибка валидации
    assert_eq!(response.status_code(), 422);
}

#[tokio::test]
async fn test_no_pii_found() {
    let app = create_test_app();
    
    let request = AnonymizeRequest {
        text: "Hello world, this is a test".to_string(),
        strategy: None,
    };
    
    let response = axum_test::TestServer::new(app)
        .unwrap()
        .post("/api/v1/anonymize")
        .json(&request)
        .await;
    
    assert_eq!(response.status_code(), 200);
    let json: serde_json::Value = response.json();
    assert_eq!(json["anonymized_text"], "Hello world, this is a test");
    assert_eq!(json["detected_pii"].as_array().unwrap().len(), 0);
}

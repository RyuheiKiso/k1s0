// schemaregistry の外部結合テスト。
// 設定デフォルト値、スキーマ型、エラーバリアントを検証する。

use k1s0_schemaregistry::{
    CompatibilityMode, RegisteredSchema, SchemaRegistryConfig, SchemaRegistryError, SchemaType,
};

// --- 設定デフォルト値テスト ---

// SchemaRegistryConfig::new のデフォルト値が正しいことを確認する。
#[test]
fn test_config_defaults() {
    let cfg = SchemaRegistryConfig::new("http://schema-registry:8081");
    assert_eq!(cfg.url, "http://schema-registry:8081");
    assert_eq!(cfg.compatibility, CompatibilityMode::Backward);
    assert_eq!(cfg.timeout_secs, 30);
}

// URL のみの JSON からデシリアライズした場合にデフォルト値が適用されることを確認する。
#[test]
fn test_config_deserialize_defaults() {
    let json = r#"{"url": "http://localhost:8081"}"#;
    let cfg: SchemaRegistryConfig = serde_json::from_str(json).unwrap();
    assert_eq!(cfg.compatibility, CompatibilityMode::Backward);
    assert_eq!(cfg.timeout_secs, 30);
}

// 全フィールド指定の JSON から正しくデシリアライズされることを確認する。
#[test]
fn test_config_deserialize_custom() {
    let json = r#"{
        "url": "http://sr:8081",
        "compatibility": "FULL_TRANSITIVE",
        "timeout_secs": 120
    }"#;
    let cfg: SchemaRegistryConfig = serde_json::from_str(json).unwrap();
    assert_eq!(cfg.url, "http://sr:8081");
    assert_eq!(cfg.compatibility, CompatibilityMode::FullTransitive);
    assert_eq!(cfg.timeout_secs, 120);
}

// Kafka トピック名からサブジェクト名が正しく生成されることを確認する。
#[test]
fn test_subject_name() {
    let subject = SchemaRegistryConfig::subject_name("k1s0.system.auth.user-created.v1");
    assert_eq!(subject, "k1s0.system.auth.user-created.v1-value");
}

// シンプルなトピック名からサブジェクト名が正しく生成されることを確認する。
#[test]
fn test_subject_name_simple() {
    assert_eq!(SchemaRegistryConfig::subject_name("orders"), "orders-value");
}

// SchemaRegistryConfig の Clone が全フィールドを保持することを確認する。
#[test]
fn test_config_clone() {
    let cfg = SchemaRegistryConfig::new("http://sr:8081");
    let cloned = cfg.clone();
    assert_eq!(cloned.url, cfg.url);
    assert_eq!(cloned.compatibility, cfg.compatibility);
    assert_eq!(cloned.timeout_secs, cfg.timeout_secs);
}

// --- スキーマ型テスト ---

// SchemaType::as_str が各バリアントで正しい文字列を返すことを確認する。
#[test]
fn test_schema_type_as_str() {
    assert_eq!(SchemaType::Avro.as_str(), "AVRO");
    assert_eq!(SchemaType::Json.as_str(), "JSON");
    assert_eq!(SchemaType::Protobuf.as_str(), "PROTOBUF");
}

// SchemaType の Display 実装が as_str と同じ値を返すことを確認する。
#[test]
fn test_schema_type_display() {
    assert_eq!(SchemaType::Avro.to_string(), "AVRO");
    assert_eq!(SchemaType::Json.to_string(), "JSON");
    assert_eq!(SchemaType::Protobuf.to_string(), "PROTOBUF");
}

// SchemaType が Copy トレイトを実装していることを確認する。
#[test]
fn test_schema_type_copy() {
    let t = SchemaType::Protobuf;
    let copied = t;
    assert_eq!(t, copied);
}

// SchemaType の PartialEq が正しく動作することを確認する。
#[test]
fn test_schema_type_equality() {
    assert_eq!(SchemaType::Avro, SchemaType::Avro);
    assert_ne!(SchemaType::Avro, SchemaType::Json);
    assert_ne!(SchemaType::Json, SchemaType::Protobuf);
}

// RegisteredSchema が全フィールドを正しく保持することを確認する。
#[test]
fn test_registered_schema_fields() {
    let schema = RegisteredSchema {
        id: 42,
        subject: "k1s0.system.auth.user-created.v1-value".to_string(),
        version: 3,
        schema: r#"syntax = "proto3"; message User {}"#.to_string(),
        schema_type: SchemaType::Protobuf,
    };
    assert_eq!(schema.id, 42);
    assert_eq!(schema.subject, "k1s0.system.auth.user-created.v1-value");
    assert_eq!(schema.version, 3);
    assert_eq!(schema.schema_type, SchemaType::Protobuf);
}

// RegisteredSchema の Clone が正しく動作することを確認する。
#[test]
fn test_registered_schema_clone() {
    let schema = RegisteredSchema {
        id: 1,
        subject: "test-value".to_string(),
        version: 1,
        schema: "schema-def".to_string(),
        schema_type: SchemaType::Avro,
    };
    let cloned = schema.clone();
    assert_eq!(cloned.id, schema.id);
    assert_eq!(cloned.subject, schema.subject);
    assert_eq!(cloned.version, schema.version);
    assert_eq!(cloned.schema_type, schema.schema_type);
}

// RegisteredSchema の JSON ラウンドトリップが正しく動作することを確認する。
#[test]
fn test_registered_schema_serde_roundtrip() {
    let schema = RegisteredSchema {
        id: 10,
        subject: "events-value".to_string(),
        version: 2,
        schema: r#"syntax = "proto3";"#.to_string(),
        schema_type: SchemaType::Protobuf,
    };
    let json = serde_json::to_string(&schema).unwrap();
    let deserialized: RegisteredSchema = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.id, 10);
    assert_eq!(deserialized.version, 2);
    assert_eq!(deserialized.schema_type, SchemaType::Protobuf);
}

// --- CompatibilityMode テスト ---

// CompatibilityMode の全バリアントが SCREAMING_SNAKE_CASE にシリアライズされることを確認する。
#[test]
fn test_compatibility_mode_serialization() {
    let cases = vec![
        (CompatibilityMode::Backward, r#""BACKWARD""#),
        (
            CompatibilityMode::BackwardTransitive,
            r#""BACKWARD_TRANSITIVE""#,
        ),
        (CompatibilityMode::Forward, r#""FORWARD""#),
        (
            CompatibilityMode::ForwardTransitive,
            r#""FORWARD_TRANSITIVE""#,
        ),
        (CompatibilityMode::Full, r#""FULL""#),
        (CompatibilityMode::FullTransitive, r#""FULL_TRANSITIVE""#),
        (CompatibilityMode::None, r#""NONE""#),
    ];
    for (mode, expected) in cases {
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, expected, "failed for {:?}", mode);
    }
}

// CompatibilityMode のシリアライズ・デシリアライズラウンドトリップが正しいことを確認する。
#[test]
fn test_compatibility_mode_roundtrip() {
    let modes = vec![
        CompatibilityMode::Backward,
        CompatibilityMode::BackwardTransitive,
        CompatibilityMode::Forward,
        CompatibilityMode::ForwardTransitive,
        CompatibilityMode::Full,
        CompatibilityMode::FullTransitive,
        CompatibilityMode::None,
    ];
    for mode in modes {
        let json = serde_json::to_string(&mode).unwrap();
        let deserialized: CompatibilityMode = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, mode);
    }
}

// CompatibilityMode が Copy を実装していることを確認する。
#[test]
fn test_compatibility_mode_copy() {
    let mode = CompatibilityMode::Full;
    let copied = mode;
    assert_eq!(mode, copied);
}

// --- エラーバリアントテスト ---

// SchemaNotFound エラーがサブジェクトとバージョン情報を含むことを確認する。
#[test]
fn test_error_schema_not_found() {
    let err = SchemaRegistryError::SchemaNotFound {
        subject: "test-value".to_string(),
        version: Some(5),
    };
    let msg = err.to_string();
    assert!(msg.contains("test-value"));
    assert!(msg.contains("5"));
}

// CompatibilityViolation エラーがサブジェクトと理由を含むことを確認する。
#[test]
fn test_error_compatibility_violation() {
    let err = SchemaRegistryError::CompatibilityViolation {
        subject: "orders-value".to_string(),
        reason: "removed field".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("orders-value"));
    assert!(msg.contains("removed field"));
}

// InvalidSchema エラーが詳細メッセージを含むことを確認する。
#[test]
fn test_error_invalid_schema() {
    let err = SchemaRegistryError::InvalidSchema("missing syntax".to_string());
    assert!(err.to_string().contains("missing syntax"));
}

// Unavailable エラーが詳細メッセージを含むことを確認する。
#[test]
fn test_error_unavailable() {
    let err = SchemaRegistryError::Unavailable("connection refused".to_string());
    assert!(err.to_string().contains("connection refused"));
}

// すべてのエラーバリアントが std::error::Error を実装していることを確認する。
#[test]
fn test_error_implements_error_trait() {
    fn assert_error<E: std::error::Error>(_: &E) {}

    assert_error(&SchemaRegistryError::SchemaNotFound {
        subject: "t".to_string(),
        version: None,
    });
    assert_error(&SchemaRegistryError::CompatibilityViolation {
        subject: "t".to_string(),
        reason: "r".to_string(),
    });
    assert_error(&SchemaRegistryError::InvalidSchema("x".to_string()));
    assert_error(&SchemaRegistryError::Unavailable("x".to_string()));
}

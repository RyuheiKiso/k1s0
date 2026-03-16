// building-blocks ファサードクレートの外部結合テスト。
// 各サブクレートの型が building-blocks 経由でアクセスできることを確認する。

// Component トレイトを name() 等のメソッド呼び出しのためにインポートする
use k1s0_building_blocks::Component;

// --- コア型の再エクスポート アクセス可能性テスト ---

// ComponentStatus が building-blocks の component モジュールからアクセスできることを確認する。
#[test]
fn test_component_status_accessible() {
    let status = k1s0_building_blocks::ComponentStatus::Ready;
    assert_eq!(status, k1s0_building_blocks::component::ComponentStatus::Ready);
}

// ComponentConfig が building-blocks の config モジュールからアクセスできることを確認する。
#[test]
fn test_component_config_accessible() {
    let json = r#"{"name": "test", "type": "statestore"}"#;
    let cfg: k1s0_building_blocks::config::ComponentConfig =
        serde_json::from_str(json).unwrap();
    assert_eq!(cfg.name, "test");
    assert_eq!(cfg.component_type, "statestore");
}

// ComponentsConfig が building-blocks の config モジュールからアクセスできることを確認する。
#[test]
fn test_components_config_accessible() {
    let yaml = r#"
components:
  - name: test-comp
    type: pubsub
"#;
    let config = k1s0_building_blocks::ComponentsConfig::from_yaml(yaml).unwrap();
    assert_eq!(config.components.len(), 1);
    assert_eq!(config.components[0].name, "test-comp");
}

// ComponentRegistry が building-blocks の registry モジュールからアクセスできることを確認する。
#[tokio::test]
async fn test_component_registry_accessible() {
    let registry = k1s0_building_blocks::ComponentRegistry::new();
    assert!(registry.get("missing").await.is_none());
}

// ComponentError が building-blocks の error モジュールからアクセスできることを確認する。
#[test]
fn test_component_error_accessible() {
    let err = k1s0_building_blocks::ComponentError::Init("test".to_string());
    assert!(err.to_string().contains("test"));
}

// --- バインディング型のアクセス可能性テスト ---

// InMemoryInputBinding が building-blocks::binding からアクセスできることを確認する。
#[tokio::test]
async fn test_binding_input_accessible() {
    let binding = k1s0_building_blocks::binding::InMemoryInputBinding::new("test-input");
    assert_eq!(binding.name(), "test-input");
}

// InMemoryOutputBinding が building-blocks::binding からアクセスできることを確認する。
#[tokio::test]
async fn test_binding_output_accessible() {
    let binding = k1s0_building_blocks::binding::InMemoryOutputBinding::new("test-output");
    assert_eq!(binding.name(), "test-output");
}

// BindingError が building-blocks::binding からアクセスできることを確認する。
#[test]
fn test_binding_error_accessible() {
    let err = k1s0_building_blocks::binding::BindingError::Invoke("test".to_string());
    assert!(err.to_string().contains("test"));
}

// --- PubSub 型のアクセス可能性テスト ---

// InMemoryPubSub が building-blocks::pubsub からアクセスできることを確認する。
#[tokio::test]
async fn test_pubsub_accessible() {
    let pubsub = k1s0_building_blocks::pubsub::InMemoryPubSub::new("test-pubsub");
    assert_eq!(pubsub.name(), "test-pubsub");
}

// PubSubError が building-blocks::pubsub からアクセスできることを確認する。
#[test]
fn test_pubsub_error_accessible() {
    let err = k1s0_building_blocks::pubsub::PubSubError::Publish("test".to_string());
    assert!(err.to_string().contains("test"));
}

// --- SecretStore 型のアクセス可能性テスト ---

// InMemorySecretStore が building-blocks::secretstore からアクセスできることを確認する。
#[tokio::test]
async fn test_secretstore_accessible() {
    let store = k1s0_building_blocks::secretstore::InMemorySecretStore::new("test-secrets");
    assert_eq!(store.name(), "test-secrets");
}

// SecretStoreError が building-blocks::secretstore からアクセスできることを確認する。
#[test]
fn test_secretstore_error_accessible() {
    let err =
        k1s0_building_blocks::secretstore::SecretStoreError::NotFound("test".to_string());
    assert!(err.to_string().contains("test"));
}

// --- StateStore 型のアクセス可能性テスト ---

// InMemoryStateStore が building-blocks::statestore からアクセスできることを確認する。
#[tokio::test]
async fn test_statestore_accessible() {
    let store = k1s0_building_blocks::statestore::InMemoryStateStore::new("test-store");
    assert_eq!(store.name(), "test-store");
}

// StateStoreError が building-blocks::statestore からアクセスできることを確認する。
#[test]
fn test_statestore_error_accessible() {
    let err = k1s0_building_blocks::statestore::StateStoreError::NotFound("test".to_string());
    assert!(err.to_string().contains("test"));
}

// --- トップレベル再エクスポートテスト ---

// Component トレイトがトップレベルからアクセスできることを確認する。
// ※ トレイトの存在を確認するため、コンパイルが通ることで証明する。
#[test]
fn test_top_level_reexports_compile() {
    // 以下の型が k1s0_building_blocks のトップレベルからアクセスできることをコンパイルで確認する
    let _status = k1s0_building_blocks::ComponentStatus::Uninitialized;
    let _err = k1s0_building_blocks::ComponentError::NotFound("x".to_string());

    // ComponentsConfig は from_yaml メソッドでアクセス確認
    let yaml = "components: []";
    let _ = k1s0_building_blocks::ComponentsConfig::from_yaml(yaml).unwrap();
}

// 全モジュールが独立してインポートできることを確認する。
#[test]
fn test_all_modules_importable() {
    // コンパイルが通ることで全モジュールがインポート可能であることを確認する
    use k1s0_building_blocks::binding;
    use k1s0_building_blocks::component;
    use k1s0_building_blocks::config;
    use k1s0_building_blocks::error;
    use k1s0_building_blocks::pubsub;
    use k1s0_building_blocks::registry;
    use k1s0_building_blocks::secretstore;
    use k1s0_building_blocks::statestore;

    // 各モジュールの型が存在することを確認する
    let _ = component::ComponentStatus::Ready;
    let _ = error::ComponentError::Init("test".to_string());
    let _ = config::ComponentConfig {
        name: "t".to_string(),
        component_type: "t".to_string(),
        version: None,
        metadata: std::collections::HashMap::new(),
    };
    let _ = binding::BindingError::Read("test".to_string());
    let _ = pubsub::PubSubError::Publish("test".to_string());
    let _ = secretstore::SecretStoreError::NotFound("test".to_string());
    let _ = statestore::StateStoreError::NotFound("test".to_string());
    let _ = registry::ComponentRegistry::new();
}

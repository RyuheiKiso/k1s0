// bb-core の外部結合テスト。
// ComponentRegistry と ComponentConfig の動作を検証する。

use std::collections::HashMap;
use std::sync::Arc;

use k1s0_bb_core::{
    Component, ComponentConfig, ComponentError, ComponentRegistry, ComponentStatus,
    ComponentsConfig,
};

// テスト用のダミーコンポーネント実装。
struct DummyComponent {
    name: String,
    comp_type: String,
}

impl DummyComponent {
    fn new(name: &str, comp_type: &str) -> Self {
        Self {
            name: name.to_string(),
            comp_type: comp_type.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl Component for DummyComponent {
    fn name(&self) -> &str {
        &self.name
    }

    fn component_type(&self) -> &str {
        &self.comp_type
    }

    async fn init(&self) -> Result<(), ComponentError> {
        Ok(())
    }

    async fn close(&self) -> Result<(), ComponentError> {
        Ok(())
    }

    async fn status(&self) -> ComponentStatus {
        ComponentStatus::Ready
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("backend".to_string(), "test".to_string());
        meta
    }
}

// --- ComponentRegistry テスト ---

// コンポーネントを登録して名前で取得できることを確認する。
#[tokio::test]
async fn test_registry_register_and_get() {
    let registry = ComponentRegistry::new();
    let comp = Arc::new(DummyComponent::new("redis-store", "statestore"));
    registry.register(comp).await.unwrap();

    let retrieved = registry.get("redis-store").await;
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name(), "redis-store");
}

// 存在しないコンポーネント名で get すると None が返ることを確認する。
#[tokio::test]
async fn test_registry_get_not_found() {
    let registry = ComponentRegistry::new();
    assert!(registry.get("missing").await.is_none());
}

// 同名のコンポーネントを重複登録するとエラーが返ることを確認する。
#[tokio::test]
async fn test_registry_register_duplicate() {
    let registry = ComponentRegistry::new();
    let c1 = Arc::new(DummyComponent::new("dup", "statestore"));
    let c2 = Arc::new(DummyComponent::new("dup", "pubsub"));
    registry.register(c1).await.unwrap();
    let result = registry.register(c2).await;
    assert!(result.is_err());
}

// 複数のコンポーネントを登録して init_all で全て初期化できることを確認する。
#[tokio::test]
async fn test_registry_init_all() {
    let registry = ComponentRegistry::new();
    registry
        .register(Arc::new(DummyComponent::new("comp-a", "statestore")))
        .await
        .unwrap();
    registry
        .register(Arc::new(DummyComponent::new("comp-b", "pubsub")))
        .await
        .unwrap();
    let result = registry.init_all().await;
    assert!(result.is_ok());
}

// close_all が全コンポーネントを正常にクローズすることを確認する。
#[tokio::test]
async fn test_registry_close_all() {
    let registry = ComponentRegistry::new();
    registry
        .register(Arc::new(DummyComponent::new("comp-a", "statestore")))
        .await
        .unwrap();
    let result = registry.close_all().await;
    assert!(result.is_ok());
}

// status_all が全コンポーネントのステータスを返すことを確認する。
#[tokio::test]
async fn test_registry_status_all() {
    let registry = ComponentRegistry::new();
    registry
        .register(Arc::new(DummyComponent::new("a", "type-a")))
        .await
        .unwrap();
    registry
        .register(Arc::new(DummyComponent::new("b", "type-b")))
        .await
        .unwrap();

    let statuses = registry.status_all().await;
    assert_eq!(statuses.len(), 2);
    assert_eq!(statuses.get("a").unwrap(), &ComponentStatus::Ready);
    assert_eq!(statuses.get("b").unwrap(), &ComponentStatus::Ready);
}

// Default トレイト実装が new() と同等であることを確認する。
#[tokio::test]
async fn test_registry_default() {
    let registry = ComponentRegistry::default();
    assert!(registry.get("anything").await.is_none());
}

// --- ComponentConfig テスト ---

// 有効な YAML から ComponentsConfig を正しくパースできることを確認する。
#[test]
fn test_components_config_from_yaml() {
    let yaml = r#"
components:
  - name: redis-store
    type: statestore
    version: "1.0"
    metadata:
      host: localhost
      port: "6379"
  - name: kafka-pubsub
    type: pubsub
"#;
    let config = ComponentsConfig::from_yaml(yaml).unwrap();
    assert_eq!(config.components.len(), 2);
    assert_eq!(config.components[0].name, "redis-store");
    assert_eq!(config.components[0].component_type, "statestore");
    assert_eq!(config.components[0].version.as_deref(), Some("1.0"));
    assert_eq!(
        config.components[0].metadata.get("host").unwrap(),
        "localhost"
    );
    assert_eq!(config.components[1].name, "kafka-pubsub");
    assert!(config.components[1].version.is_none());
    assert!(config.components[1].metadata.is_empty());
}

// 不正な YAML からパースするとエラーが返ることを確認する。
#[test]
fn test_components_config_from_yaml_invalid() {
    let result = ComponentsConfig::from_yaml("not: valid: yaml: [");
    assert!(result.is_err());
}

// ComponentConfig が JSON デシリアライズで正しく動作することを確認する。
#[test]
fn test_component_config_serde() {
    let json = r#"{
        "name": "my-component",
        "type": "statestore",
        "version": "2.0",
        "metadata": {"key": "value"}
    }"#;
    let cfg: ComponentConfig = serde_json::from_str(json).unwrap();
    assert_eq!(cfg.name, "my-component");
    assert_eq!(cfg.component_type, "statestore");
    assert_eq!(cfg.version.as_deref(), Some("2.0"));
    assert_eq!(cfg.metadata.get("key").unwrap(), "value");
}

// version が未指定の場合に None になることを確認する。
#[test]
fn test_component_config_optional_version() {
    let json = r#"{"name": "comp", "type": "pubsub"}"#;
    let cfg: ComponentConfig = serde_json::from_str(json).unwrap();
    assert!(cfg.version.is_none());
    assert!(cfg.metadata.is_empty());
}

// --- ComponentStatus テスト ---

// ComponentStatus の各バリアントが正しくシリアライズ・デシリアライズされることを確認する。
#[test]
fn test_component_status_serde() {
    let statuses = vec![
        ComponentStatus::Uninitialized,
        ComponentStatus::Ready,
        ComponentStatus::Degraded,
        ComponentStatus::Closed,
        ComponentStatus::Error("some error".to_string()),
    ];
    for status in statuses {
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: ComponentStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, status);
    }
}

// ComponentStatus の PartialEq が正しく動作することを確認する。
#[test]
fn test_component_status_equality() {
    assert_eq!(ComponentStatus::Ready, ComponentStatus::Ready);
    assert_ne!(ComponentStatus::Ready, ComponentStatus::Closed);
    assert_ne!(
        ComponentStatus::Error("a".to_string()),
        ComponentStatus::Error("b".to_string())
    );
}

// --- ComponentError テスト ---

// ComponentError の各バリアントが適切な Display 出力を生成することを確認する。
#[test]
fn test_component_error_display() {
    let err = ComponentError::Init("init failed".to_string());
    assert!(err.to_string().contains("init failed"));

    let err = ComponentError::Config("bad config".to_string());
    assert!(err.to_string().contains("bad config"));

    let err = ComponentError::Runtime("runtime error".to_string());
    assert!(err.to_string().contains("runtime error"));

    let err = ComponentError::Shutdown("shutdown failed".to_string());
    assert!(err.to_string().contains("shutdown failed"));

    let err = ComponentError::NotFound("missing".to_string());
    assert!(err.to_string().contains("missing"));
}

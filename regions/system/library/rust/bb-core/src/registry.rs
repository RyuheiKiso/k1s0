use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::{error, info};

use crate::component::{Component, ComponentStatus};
use crate::ComponentError;

/// ComponentRegistry はビルディングブロックの登録・管理を行うレジストリ。
pub struct ComponentRegistry {
    components: RwLock<HashMap<String, Arc<dyn Component>>>,
}

impl ComponentRegistry {
    pub fn new() -> Self {
        Self {
            components: RwLock::new(HashMap::new()),
        }
    }

    /// コンポーネントを登録する。同名のコンポーネントが既に存在する場合はエラーを返す。
    pub async fn register(&self, component: Arc<dyn Component>) -> Result<(), ComponentError> {
        let name = component.name().to_string();
        let mut components = self.components.write().await;
        if components.contains_key(&name) {
            return Err(ComponentError::Init(format!(
                "コンポーネント '{name}' は既に登録されています"
            )));
        }
        info!(component = %name, "コンポーネントを登録しました");
        components.insert(name, component);
        Ok(())
    }

    /// 名前でコンポーネントを取得する。
    pub async fn get(&self, name: &str) -> Option<Arc<dyn Component>> {
        let components = self.components.read().await;
        components.get(name).cloned()
    }

    /// 全コンポーネントを初期化する。
    pub async fn init_all(&self) -> Result<(), ComponentError> {
        let components = self.components.read().await;
        for (name, component) in components.iter() {
            info!(component = %name, "コンポーネントを初期化中");
            if let Err(e) = component.init().await {
                error!(component = %name, error = %e, "コンポーネントの初期化に失敗しました");
                return Err(e);
            }
        }
        Ok(())
    }

    /// 全コンポーネントをクローズする。
    pub async fn close_all(&self) -> Result<(), ComponentError> {
        let components = self.components.read().await;
        for (name, component) in components.iter() {
            info!(component = %name, "コンポーネントをクローズ中");
            if let Err(e) = component.close().await {
                error!(component = %name, error = %e, "コンポーネントのクローズに失敗しました");
                return Err(e);
            }
        }
        Ok(())
    }

    /// 全コンポーネントのステータスを返す。
    pub async fn status_all(&self) -> HashMap<String, ComponentStatus> {
        let components = self.components.read().await;
        let mut statuses = HashMap::new();
        for (name, component) in components.iter() {
            statuses.insert(name.clone(), component.status().await);
        }
        statuses
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    struct TestComponent {
        name: String,
    }

    impl TestComponent {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
            }
        }
    }

    #[async_trait::async_trait]
    impl Component for TestComponent {
        fn name(&self) -> &str {
            &self.name
        }

        fn component_type(&self) -> &str {
            "test"
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
            HashMap::new()
        }
    }

    // コンポーネントを登録して名前で取得できることを確認する。
    #[tokio::test]
    async fn test_register_and_get() {
        let registry = ComponentRegistry::new();
        let component = Arc::new(TestComponent::new("test-1"));
        registry.register(component).await.unwrap();

        let retrieved = registry.get("test-1").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name(), "test-1");
    }

    // 同名のコンポーネントを重複登録するとエラーが返ることを確認する。
    #[tokio::test]
    async fn test_register_duplicate() {
        let registry = ComponentRegistry::new();
        let c1 = Arc::new(TestComponent::new("dup"));
        let c2 = Arc::new(TestComponent::new("dup"));
        registry.register(c1).await.unwrap();
        let result = registry.register(c2).await;
        assert!(result.is_err());
    }

    // 存在しない名前でコンポーネントを取得すると None が返ることを確認する。
    #[tokio::test]
    async fn test_get_not_found() {
        let registry = ComponentRegistry::new();
        assert!(registry.get("missing").await.is_none());
    }

    // init_all が登録済みの全コンポーネントを初期化することを確認する。
    #[tokio::test]
    async fn test_init_all() {
        let registry = ComponentRegistry::new();
        registry
            .register(Arc::new(TestComponent::new("a")))
            .await
            .unwrap();
        registry
            .register(Arc::new(TestComponent::new("b")))
            .await
            .unwrap();
        registry.init_all().await.unwrap();
    }

    // close_all が登録済みの全コンポーネントをクローズすることを確認する。
    #[tokio::test]
    async fn test_close_all() {
        let registry = ComponentRegistry::new();
        registry
            .register(Arc::new(TestComponent::new("a")))
            .await
            .unwrap();
        registry.close_all().await.unwrap();
    }

    // status_all が全コンポーネントのステータスを返すことを確認する。
    #[tokio::test]
    async fn test_status_all() {
        let registry = ComponentRegistry::new();
        registry
            .register(Arc::new(TestComponent::new("a")))
            .await
            .unwrap();
        registry
            .register(Arc::new(TestComponent::new("b")))
            .await
            .unwrap();
        let statuses = registry.status_all().await;
        assert_eq!(statuses.len(), 2);
        assert_eq!(statuses.get("a").unwrap(), &ComponentStatus::Ready);
        assert_eq!(statuses.get("b").unwrap(), &ComponentStatus::Ready);
    }
}

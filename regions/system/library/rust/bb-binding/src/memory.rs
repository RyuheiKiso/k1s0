use std::collections::HashMap;

use async_trait::async_trait;
use tokio::sync::RwLock;
use tracing::info;

use k1s0_bb_core::{Component, ComponentError, ComponentStatus};

use crate::BindingError;
use crate::traits::{BindingData, BindingResponse, InputBinding, OutputBinding};

/// InMemoryInputBinding はテスト・開発用のインメモリ InputBinding 実装。
pub struct InMemoryInputBinding {
    name: String,
    status: RwLock<ComponentStatus>,
    queue: RwLock<Vec<BindingData>>,
}

impl InMemoryInputBinding {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: RwLock::new(ComponentStatus::Uninitialized),
            queue: RwLock::new(Vec::new()),
        }
    }

    /// テスト用にデータをキューに追加する。
    pub async fn push(&self, data: BindingData) {
        let mut queue = self.queue.write().await;
        queue.push(data);
    }
}

#[async_trait]
impl Component for InMemoryInputBinding {
    fn name(&self) -> &str {
        &self.name
    }

    fn component_type(&self) -> &str {
        "binding.input"
    }

    async fn init(&self) -> Result<(), ComponentError> {
        let mut status = self.status.write().await;
        *status = ComponentStatus::Ready;
        info!(component = %self.name, "InMemoryInputBinding を初期化しました");
        Ok(())
    }

    async fn close(&self) -> Result<(), ComponentError> {
        let mut status = self.status.write().await;
        *status = ComponentStatus::Closed;
        let mut queue = self.queue.write().await;
        queue.clear();
        info!(component = %self.name, "InMemoryInputBinding をクローズしました");
        Ok(())
    }

    async fn status(&self) -> ComponentStatus {
        self.status.read().await.clone()
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("backend".to_string(), "memory".to_string());
        meta.insert("direction".to_string(), "input".to_string());
        meta
    }
}

#[async_trait]
impl InputBinding for InMemoryInputBinding {
    async fn read(&self) -> Result<BindingData, BindingError> {
        let mut queue = self.queue.write().await;
        if queue.is_empty() {
            return Err(BindingError::Read("キューが空です".to_string()));
        }
        Ok(queue.remove(0))
    }
}

/// InMemoryOutputBinding はテスト・開発用のインメモリ OutputBinding 実装。
pub struct InMemoryOutputBinding {
    name: String,
    status: RwLock<ComponentStatus>,
    // 呼び出し履歴を保持するフィールド（操作名、データ、メタデータのタプル）。
    #[allow(clippy::type_complexity)]
    invocations: RwLock<Vec<(String, Vec<u8>, HashMap<String, String>)>>,
}

impl InMemoryOutputBinding {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: RwLock::new(ComponentStatus::Uninitialized),
            invocations: RwLock::new(Vec::new()),
        }
    }

    /// テスト用に記録された呼び出しを取得する。
    pub async fn invocations(&self) -> Vec<(String, Vec<u8>, HashMap<String, String>)> {
        self.invocations.read().await.clone()
    }
}

#[async_trait]
impl Component for InMemoryOutputBinding {
    fn name(&self) -> &str {
        &self.name
    }

    fn component_type(&self) -> &str {
        "binding.output"
    }

    async fn init(&self) -> Result<(), ComponentError> {
        let mut status = self.status.write().await;
        *status = ComponentStatus::Ready;
        info!(component = %self.name, "InMemoryOutputBinding を初期化しました");
        Ok(())
    }

    async fn close(&self) -> Result<(), ComponentError> {
        let mut status = self.status.write().await;
        *status = ComponentStatus::Closed;
        let mut invocations = self.invocations.write().await;
        invocations.clear();
        info!(component = %self.name, "InMemoryOutputBinding をクローズしました");
        Ok(())
    }

    async fn status(&self) -> ComponentStatus {
        self.status.read().await.clone()
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("backend".to_string(), "memory".to_string());
        meta.insert("direction".to_string(), "output".to_string());
        meta
    }
}

#[async_trait]
impl OutputBinding for InMemoryOutputBinding {
    async fn invoke(
        &self,
        operation: &str,
        data: &[u8],
        metadata: Option<HashMap<String, String>>,
    ) -> Result<BindingResponse, BindingError> {
        let meta = metadata.unwrap_or_default();
        let mut invocations = self.invocations.write().await;
        invocations.push((operation.to_string(), data.to_vec(), meta.clone()));

        Ok(BindingResponse {
            data: data.to_vec(),
            metadata: meta,
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // InMemoryInputBinding の初期化でステータスが Ready に遷移することを確認する。
    #[tokio::test]
    async fn test_input_binding_init() {
        let binding = InMemoryInputBinding::new("test-input");
        assert_eq!(binding.status().await, ComponentStatus::Uninitialized);
        binding.init().await.unwrap();
        assert_eq!(binding.status().await, ComponentStatus::Ready);
    }

    // キューにデータを追加して read で取得できることを確認する。
    #[tokio::test]
    async fn test_input_binding_read() {
        let binding = InMemoryInputBinding::new("test-input");
        binding
            .push(BindingData {
                data: b"hello".to_vec(),
                metadata: HashMap::new(),
            })
            .await;

        let data = binding.read().await.unwrap();
        assert_eq!(data.data, b"hello");
    }

    // 空のキューから read するとエラーが返ることを確認する。
    #[tokio::test]
    async fn test_input_binding_read_empty() {
        let binding = InMemoryInputBinding::new("test-input");
        let result = binding.read().await;
        assert!(result.is_err());
    }

    // キューが FIFO 順序でデータを返すことを確認する。
    #[tokio::test]
    async fn test_input_binding_fifo() {
        let binding = InMemoryInputBinding::new("test-input");
        binding
            .push(BindingData {
                data: b"first".to_vec(),
                metadata: HashMap::new(),
            })
            .await;
        binding
            .push(BindingData {
                data: b"second".to_vec(),
                metadata: HashMap::new(),
            })
            .await;

        let d1 = binding.read().await.unwrap();
        assert_eq!(d1.data, b"first");
        let d2 = binding.read().await.unwrap();
        assert_eq!(d2.data, b"second");
    }

    // InMemoryOutputBinding の初期化でステータスが Ready に遷移することを確認する。
    #[tokio::test]
    async fn test_output_binding_init() {
        let binding = InMemoryOutputBinding::new("test-output");
        assert_eq!(binding.status().await, ComponentStatus::Uninitialized);
        binding.init().await.unwrap();
        assert_eq!(binding.status().await, ComponentStatus::Ready);
    }

    // invoke が呼び出し結果をレスポンスとして返し呼び出し履歴を記録することを確認する。
    #[tokio::test]
    async fn test_output_binding_invoke() {
        let binding = InMemoryOutputBinding::new("test-output");
        let response = binding.invoke("create", b"data", None).await.unwrap();
        assert_eq!(response.data, b"data");

        let invocations = binding.invocations().await;
        assert_eq!(invocations.len(), 1);
        assert_eq!(invocations[0].0, "create");
        assert_eq!(invocations[0].1, b"data");
    }

    // メタデータ付きで invoke するとレスポンスにメタデータが含まれることを確認する。
    #[tokio::test]
    async fn test_output_binding_invoke_with_metadata() {
        let binding = InMemoryOutputBinding::new("test-output");
        let mut meta = HashMap::new();
        meta.insert("key".to_string(), "value".to_string());

        let response = binding
            .invoke("update", b"payload", Some(meta))
            .await
            .unwrap();
        assert_eq!(response.metadata.get("key").unwrap(), "value");
    }

    // InMemoryInputBinding のクローズでステータスが Closed に遷移することを確認する。
    #[tokio::test]
    async fn test_input_binding_close() {
        let binding = InMemoryInputBinding::new("test-input");
        binding.init().await.unwrap();
        binding.close().await.unwrap();
        assert_eq!(binding.status().await, ComponentStatus::Closed);
    }

    // InMemoryOutputBinding のクローズでステータスが Closed に遷移することを確認する。
    #[tokio::test]
    async fn test_output_binding_close() {
        let binding = InMemoryOutputBinding::new("test-output");
        binding.init().await.unwrap();
        binding.close().await.unwrap();
        assert_eq!(binding.status().await, ComponentStatus::Closed);
    }

    // InMemoryInputBinding のメタデータに backend と direction が含まれることを確認する。
    #[tokio::test]
    async fn test_input_binding_metadata() {
        let binding = InMemoryInputBinding::new("test-input");
        let meta = binding.metadata();
        assert_eq!(meta.get("backend").unwrap(), "memory");
        assert_eq!(meta.get("direction").unwrap(), "input");
    }

    // InMemoryOutputBinding のメタデータに backend と direction が含まれることを確認する。
    #[tokio::test]
    async fn test_output_binding_metadata() {
        let binding = InMemoryOutputBinding::new("test-output");
        let meta = binding.metadata();
        assert_eq!(meta.get("backend").unwrap(), "memory");
        assert_eq!(meta.get("direction").unwrap(), "output");
    }

    // InMemoryInputBinding の component_type が "binding.input" を返すことを確認する。
    #[tokio::test]
    async fn test_input_binding_component_type() {
        let binding = InMemoryInputBinding::new("test-input");
        assert_eq!(binding.component_type(), "binding.input");
    }

    // InMemoryOutputBinding の component_type が "binding.output" を返すことを確認する。
    #[tokio::test]
    async fn test_output_binding_component_type() {
        let binding = InMemoryOutputBinding::new("test-output");
        assert_eq!(binding.component_type(), "binding.output");
    }
}

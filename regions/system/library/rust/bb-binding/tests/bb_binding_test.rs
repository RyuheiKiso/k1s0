// bb-binding の外部結合テスト。
// InMemoryInputBinding / InMemoryOutputBinding の動作を検証する。

use std::collections::HashMap;

use k1s0_bb_binding::{
    BindingData, InMemoryInputBinding, InMemoryOutputBinding, InputBinding, OutputBinding,
};
use k1s0_bb_core::{Component, ComponentStatus};

// --- InMemoryInputBinding テスト ---

// InMemoryInputBinding の初期化でステータスが Ready に遷移することを確認する。
#[tokio::test]
async fn test_input_binding_lifecycle() {
    let binding = InMemoryInputBinding::new("test-input");

    // 初期状態は Uninitialized
    assert_eq!(binding.status().await, ComponentStatus::Uninitialized);

    // 初期化後は Ready
    binding.init().await.unwrap();
    assert_eq!(binding.status().await, ComponentStatus::Ready);

    // クローズ後は Closed
    binding.close().await.unwrap();
    assert_eq!(binding.status().await, ComponentStatus::Closed);
}

// キューにデータを追加して read で取得できることを確認する。
#[tokio::test]
async fn test_input_binding_push_and_read() {
    let binding = InMemoryInputBinding::new("test-input");
    binding
        .push(BindingData {
            data: b"hello world".to_vec(),
            metadata: HashMap::new(),
        })
        .await;

    let data = binding.read().await.unwrap();
    assert_eq!(data.data, b"hello world");
}

// 空のキューから read するとエラーが返ることを確認する。
#[tokio::test]
async fn test_input_binding_read_empty_queue() {
    let binding = InMemoryInputBinding::new("test-input");
    let result = binding.read().await;
    assert!(result.is_err());
}

// キューが FIFO 順序でデータを返すことを確認する。
#[tokio::test]
async fn test_input_binding_fifo_order() {
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
    binding
        .push(BindingData {
            data: b"third".to_vec(),
            metadata: HashMap::new(),
        })
        .await;

    assert_eq!(binding.read().await.unwrap().data, b"first");
    assert_eq!(binding.read().await.unwrap().data, b"second");
    assert_eq!(binding.read().await.unwrap().data, b"third");
    assert!(binding.read().await.is_err());
}

// メタデータ付きの BindingData が正しく読み取れることを確認する。
#[tokio::test]
async fn test_input_binding_with_metadata() {
    let binding = InMemoryInputBinding::new("test-input");
    let mut meta = HashMap::new();
    meta.insert("source".to_string(), "external-api".to_string());

    binding
        .push(BindingData {
            data: b"data".to_vec(),
            metadata: meta,
        })
        .await;

    let data = binding.read().await.unwrap();
    assert_eq!(data.metadata.get("source").unwrap(), "external-api");
}

// InMemoryInputBinding のコンポーネントメタデータが正しいことを確認する。
#[test]
fn test_input_binding_component_metadata() {
    let binding = InMemoryInputBinding::new("test-input");
    assert_eq!(binding.name(), "test-input");
    assert_eq!(binding.component_type(), "binding.input");
    let meta = binding.metadata();
    assert_eq!(meta.get("backend").unwrap(), "memory");
    assert_eq!(meta.get("direction").unwrap(), "input");
}

// --- InMemoryOutputBinding テスト ---

// InMemoryOutputBinding の初期化でステータスが Ready に遷移することを確認する。
#[tokio::test]
async fn test_output_binding_lifecycle() {
    let binding = InMemoryOutputBinding::new("test-output");

    assert_eq!(binding.status().await, ComponentStatus::Uninitialized);

    binding.init().await.unwrap();
    assert_eq!(binding.status().await, ComponentStatus::Ready);

    binding.close().await.unwrap();
    assert_eq!(binding.status().await, ComponentStatus::Closed);
}

// invoke が呼び出し結果をレスポンスとして返すことを確認する。
#[tokio::test]
async fn test_output_binding_invoke() {
    let binding = InMemoryOutputBinding::new("test-output");
    let response = binding.invoke("create", b"data-payload", None).await.unwrap();
    assert_eq!(response.data, b"data-payload");
}

// 複数の invoke 呼び出しが履歴に記録されることを確認する。
#[tokio::test]
async fn test_output_binding_invocation_history() {
    let binding = InMemoryOutputBinding::new("test-output");

    binding.invoke("create", b"data-1", None).await.unwrap();
    binding.invoke("update", b"data-2", None).await.unwrap();
    binding.invoke("delete", b"data-3", None).await.unwrap();

    let invocations = binding.invocations().await;
    assert_eq!(invocations.len(), 3);
    assert_eq!(invocations[0].0, "create");
    assert_eq!(invocations[1].0, "update");
    assert_eq!(invocations[2].0, "delete");
}

// メタデータ付きで invoke するとレスポンスにメタデータが含まれることを確認する。
#[tokio::test]
async fn test_output_binding_invoke_with_metadata() {
    let binding = InMemoryOutputBinding::new("test-output");
    let mut meta = HashMap::new();
    meta.insert("x-request-id".to_string(), "req-123".to_string());

    let response = binding
        .invoke("send", b"payload", Some(meta))
        .await
        .unwrap();
    assert_eq!(response.metadata.get("x-request-id").unwrap(), "req-123");
}

// InMemoryOutputBinding のコンポーネントメタデータが正しいことを確認する。
#[test]
fn test_output_binding_component_metadata() {
    let binding = InMemoryOutputBinding::new("test-output");
    assert_eq!(binding.name(), "test-output");
    assert_eq!(binding.component_type(), "binding.output");
    let meta = binding.metadata();
    assert_eq!(meta.get("backend").unwrap(), "memory");
    assert_eq!(meta.get("direction").unwrap(), "output");
}

// close 後に invocations がクリアされることを確認する。
#[tokio::test]
async fn test_output_binding_close_clears_invocations() {
    let binding = InMemoryOutputBinding::new("test-output");
    binding.invoke("op", b"data", None).await.unwrap();
    assert_eq!(binding.invocations().await.len(), 1);

    binding.close().await.unwrap();
    assert_eq!(binding.invocations().await.len(), 0);
}

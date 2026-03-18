# bb-core ライブラリ設計

## 概要

Building Block（BB）システムの基盤クレート。全 BB 実装（PubSub・StateStore・SecretStore・Binding）が共通して依存するコアトレイト・型・レジストリを提供する。各 BB クレート（`bb-pubsub`・`bb-statestore` 等）はこのクレートの `Component` トレイトを実装し、`ComponentRegistry` で一括管理される。

**配置先**: `regions/system/library/rust/bb-core/`

**パッケージ名**: `k1s0-bb-core`

## 責務

- BB コンポーネントの共通ライフサイクルインターフェース（`Component` トレイト）の定義
- コンポーネントの宣言的設定（`ComponentConfig` / `ComponentsConfig`）の定義とパース
- 複数コンポーネントの一括管理（`ComponentRegistry`）の提供
- 共通エラー型（`ComponentError`）の定義

## 公開型・トレイト

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `Component` | トレイト | BB の基本インターフェース。名前・種別・初期化・クローズ・ステータス・メタデータを定義 |
| `ComponentStatus` | enum | コンポーネントの状態（`Uninitialized`・`Ready`・`Degraded`・`Closed`・`Error(String)`） |
| `ComponentConfig` | 構造体 | 個々の BB コンポーネント設定（name・type・version・metadata） |
| `ComponentsConfig` | 構造体 | 複数コンポーネントの設定コレクション。YAML からのパースをサポート |
| `ComponentRegistry` | 構造体 | 複数コンポーネントの登録・取得・一括ライフサイクル管理 |
| `ComponentError` | enum | 共通エラー型（`Init`・`Config`・`Runtime`・`Shutdown`・`NotFound`） |

## Component トレイト

全 BB 実装が満たすべき基本インターフェース。`Send + Sync` を要求し、マルチスレッド環境での利用を前提とする。

```rust
#[async_trait]
pub trait Component: Send + Sync {
    /// コンポーネント名を返す。
    fn name(&self) -> &str;

    /// コンポーネント種別を返す（例: "pubsub", "statestore"）。
    fn component_type(&self) -> &str;

    /// コンポーネントを初期化する。
    async fn init(&self) -> Result<(), ComponentError>;

    /// コンポーネントをクローズする。
    async fn close(&self) -> Result<(), ComponentError>;

    /// コンポーネントの現在のステータスを返す。
    async fn status(&self) -> ComponentStatus;

    /// コンポーネントのメタデータを返す。
    fn metadata(&self) -> HashMap<String, String>;
}
```

## ComponentRegistry

複数の `Component` を名前で登録・管理し、一括ライフサイクル制御を提供するレジストリ。スレッドセーフ設計（`tokio::sync::RwLock`）。

| メソッド | 説明 |
|---------|------|
| `new()` | 空のレジストリを作成 |
| `register(component)` | コンポーネントを登録。同名が既に存在する場合は `ComponentError::Init` を返す |
| `get(name)` | 名前でコンポーネントを取得。見つからない場合は `None` |
| `init_all()` | 登録済み全コンポーネントを順次初期化 |
| `close_all()` | 登録済み全コンポーネントを順次クローズ |
| `status_all()` | 全コンポーネントのステータスを `HashMap<String, ComponentStatus>` で返す |

## ComponentsConfig

YAML ファイルからコンポーネント設定を宣言的にロードする。

```yaml
components:
  - name: redis-store
    type: statestore
    version: "1.0"
    metadata:
      host: localhost
      port: "6379"
  - name: kafka-pubsub
    type: pubsub
```

パースメソッド:

| メソッド | 説明 |
|---------|------|
| `from_yaml(yaml: &str)` | YAML 文字列からパース |
| `from_file(path: &Path)` | YAML ファイルから読み込み |

## 使用例

```rust
use std::sync::Arc;
use k1s0_bb_core::{ComponentRegistry, Component, ComponentError, ComponentStatus};

// サービス起動時にコンポーネントを登録・初期化する
let registry = ComponentRegistry::new();
registry.register(Arc::new(KafkaPubSub::new(config))).await?;
registry.register(Arc::new(RedisStateStore::new(config))).await?;

// 全コンポーネントを一括初期化
registry.init_all().await?;

// ヘルスチェック時に全コンポーネントのステータスを確認
let statuses = registry.status_all().await;
for (name, status) in &statuses {
    println!("{name}: {status:?}");
}

// サービス終了時に一括クローズ
registry.close_all().await?;
```

## Feature フラグ

| Feature | 説明 |
|---------|------|
| `mock` | `mockall` による `MockComponent` を有効化（テスト用） |

## 依存クレート

| クレート | 用途 |
|---------|------|
| `async-trait` | 非同期トレイト定義 |
| `serde` / `serde_yaml` | 設定の (デ)シリアライズ |
| `thiserror` | エラー型の派生 |
| `tokio` | 非同期ランタイム・`RwLock` |
| `tracing` | 構造化ログ出力 |
| `mockall` | モック生成（`mock` feature 有効時のみ） |

**依存追加**: `k1s0-bb-core = { path = "../../system/library/rust/bb-core" }`（[追加方法参照](../_common/共通実装パターン.md#cargo依存追加)）

## 関連ドキュメント

- [Building Blocks 抽象化](../_common/building-blocks.md) -- BB アーキテクチャ全体設計
- [PubSub 設計](../messaging/pubsub.md) -- PubSub Building Block 詳細
- [StateStore 設計](../data/statestore.md) -- StateStore Building Block 詳細
- [SecretStore 設計](../auth-security/secret-store.md) -- SecretStore Building Block 詳細
- [Binding 設計](../config/binding.md) -- Binding Building Block 詳細

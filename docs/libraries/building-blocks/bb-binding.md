# bb-binding ライブラリ設計

## 概要

外部サービスとの入出力接続を抽象化する Binding Building Block クレート。入力バインディング（外部ソースからのデータ受信）と出力バインディング（外部サービスへのデータ送信）の2方向を統一インターフェースで提供する。本番実装として HTTP 出力バインディング（`reqwest` 使用）、開発・テスト用としてインメモリ実装を備える。

**配置先**: `regions/system/library/rust/bb-binding/`

**パッケージ名**: `k1s0-bb-binding`

## 設計思想

マイクロサービスは外部 API・Webhook・メッセージエンドポイントなど多様な接続先を持つ。Binding 抽象化により、接続先の変更（HTTP → キュー等）をアプリケーションコードの変更なく差し替え可能にする。`bb-core` の `Component` トレイトを拡張することで、他の BB コンポーネント（PubSub・StateStore 等）と同じライフサイクル管理（init / close / status）の枠組みに統合される。入力と出力を別トレイトに分離することで、責務の明確化とテスト時の差し替え容易性を両立する。

## トレイト定義

### データ型

| 型 | 説明 |
|----|------|
| `BindingData` | 入力バインディングから受信したデータ（`data: Vec<u8>` + `metadata: HashMap<String, String>`） |
| `BindingResponse` | 出力バインディングの呼び出し結果（`data: Vec<u8>` + `metadata: HashMap<String, String>`） |

### InputBinding トレイト

外部ソースからデータを受信する入力バインディングの抽象インターフェース。`k1s0_bb_core::Component` を拡張する。

```rust
#[async_trait]
pub trait InputBinding: k1s0_bb_core::Component {
    /// 外部ソースからデータを読み取る。
    async fn read(&self) -> Result<BindingData, BindingError>;
}
```

### OutputBinding トレイト

外部サービスにデータを送信する出力バインディングの抽象インターフェース。`k1s0_bb_core::Component` を拡張する。

```rust
#[async_trait]
pub trait OutputBinding: k1s0_bb_core::Component {
    /// 指定した操作でデータを外部サービスに送信する。
    async fn invoke(
        &self,
        operation: &str,
        data: &[u8],
        metadata: Option<HashMap<String, String>>,
    ) -> Result<BindingResponse, BindingError>;
}
```

## 実装バリエーション

### HTTP 出力バインディング（`HttpOutputBinding`、`http` feature 必須）

`reqwest::Client` を使用して HTTP リクエストを送信する本番向け出力バインディング実装。

| 項目 | 内容 |
|------|------|
| `component_type` | `"binding.output"` |
| metadata `backend` | `"http"` |
| metadata `direction` | `"output"` |
| サポート operation | HTTP メソッド文字列（`GET`、`POST`、`PUT`、`DELETE`、`PATCH` 等） |
| 必須 metadata | `"url"`: リクエスト先 URL |
| 省略可能 metadata | `"content-type"`: リクエストボディの Content-Type（省略時 `"application/octet-stream"`） |
| レスポンス metadata | `"status-code"` とレスポンスヘッダー一式 |
| エラー判定 | 4xx / 5xx レスポンスを `BindingError::Invoke` に変換 |

コンストラクタ:
- `HttpOutputBinding::new(name)` — デフォルト `reqwest::Client` を使用
- `HttpOutputBinding::with_client(name, client)` — テスト用カスタムクライアント注入

### インメモリ入力バインディング（`InMemoryInputBinding`）

FIFO キューをバックエンドとするテスト・開発用入力バインディング。

| 項目 | 内容 |
|------|------|
| `component_type` | `"binding.input"` |
| metadata `backend` | `"memory"` |
| metadata `direction` | `"input"` |
| `push(data)` | テスト用にキューへデータを追加する |
| `read()` の挙動 | FIFO 順でキューの先頭データを取得・除去。空の場合は `BindingError::Read` |

### インメモリ出力バインディング（`InMemoryOutputBinding`）

呼び出し履歴をメモリに記録するテスト・開発用出力バインディング。

| 項目 | 内容 |
|------|------|
| `component_type` | `"binding.output"` |
| metadata `backend` | `"memory"` |
| metadata `direction` | `"output"` |
| `invoke()` の挙動 | 呼び出し（操作名・データ・メタデータ）を履歴に記録し、入力データをそのままレスポンスとして返す |
| `invocations()` | 記録された呼び出し履歴 `Vec<(String, Vec<u8>, HashMap<String, String>)>` を返す |

## 使用例

```rust
use std::sync::Arc;
use std::collections::HashMap;
use k1s0_bb_binding::{HttpOutputBinding, OutputBinding, InMemoryInputBinding, InputBinding};
use k1s0_bb_binding::BindingData;

// HTTP 出力バインディングを初期化する
let http_binding = HttpOutputBinding::new("external-api");
http_binding.init().await?;

// 外部 REST API に POST リクエストを送信する
let mut meta = HashMap::new();
meta.insert("url".to_string(), "https://api.example.com/events".to_string());
meta.insert("content-type".to_string(), "application/json".to_string());

let response = http_binding
    .invoke("POST", br#"{"event": "order_created"}"#, Some(meta))
    .await?;

println!(
    "ステータス: {}",
    response.metadata.get("status-code").unwrap_or(&"".to_string())
);

// テスト時はインメモリ実装に差し替える
let input_binding = InMemoryInputBinding::new("mock-source");
input_binding.init().await?;

// テストデータをキューに追加する
input_binding
    .push(BindingData {
        data: b"test event".to_vec(),
        metadata: HashMap::new(),
    })
    .await;

let data = input_binding.read().await?;
assert_eq!(data.data, b"test event");
```

## エラーハンドリング

`BindingError` は `thiserror` で定義された enum。

| バリアント | 説明 |
|-----------|------|
| `Invoke(String)` | 出力バインディングの呼び出しエラー（HTTP 4xx/5xx、必須 metadata 欠落等） |
| `Read(String)` | 入力バインディングの読み取りエラー（キュー空等） |
| `UnsupportedOperation(String)` | サポートされていない操作名（不正な HTTP メソッド等） |
| `Connection(String)` | ネットワーク接続エラー |
| `Component(ComponentError)` | `bb-core` のコンポーネントエラー（`#[from]` 自動変換） |

## 依存関係

| クレート | 用途 |
|---------|------|
| `k1s0-bb-core` | `Component` トレイト・`ComponentError`・`ComponentStatus` |
| `async-trait` | 非同期トレイト定義 |
| `serde` | シリアライズ（将来拡張用） |
| `thiserror` | エラー型の派生 |
| `tokio` | 非同期ランタイム・`RwLock` |
| `tracing` | 構造化ログ出力 |
| `reqwest` | HTTP クライアント（`http` feature 有効時のみ） |
| `mockall` | モック生成（`mock` feature 有効時のみ） |

### Feature フラグ

| Feature | 説明 |
|---------|------|
| `http` | `reqwest` による `HttpOutputBinding` を有効化 |
| `mock` | `mockall` によるモック実装を有効化（テスト用） |

**依存追加**: `k1s0-bb-binding = { path = "../../system/library/rust/bb-binding" }`

HTTP 実装を使用する場合: `k1s0-bb-binding = { path = "../../system/library/rust/bb-binding", features = ["http"] }`

## 関連ドキュメント

- [Building Blocks 抽象化](../_common/building-blocks.md) — BB アーキテクチャ全体設計
- [bb-core 設計](bb-core.md) — BB 基盤クレート
- [Binding 設計](../../architecture/config/binding.md) — Binding Building Block 詳細

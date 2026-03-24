# bb-pubsub ライブラリ設計

## 概要

パブリッシュ・サブスクライブ（PubSub）機能を抽象化する Building Block クレート。トピックへのメッセージ送信（publish）とトピックの購読（subscribe）を統一インターフェースで提供する。本番実装として Kafka バックエンド（`k1s0-messaging` の `EventProducer` をラップ）、開発・テスト用としてインメモリ実装を備える。

**配置先**: `regions/system/library/rust/bb-pubsub/`

**パッケージ名**: `k1s0-bb-pubsub`

## 設計思想

サービス間のイベント駆動通信において、メッセージブローカーの種類（Kafka・インメモリ等）をアプリケーションコードから隠蔽する。`bb-core` の `Component` トレイトを拡張することで、PubSub コンポーネントもレジストリに統合してライフサイクルを一元管理できる。メッセージハンドラーを `MessageHandler` トレイトで抽象化することで、受信ロジックのテスト時差し替えも容易にする。Kafka の subscribe は `EventConsumer` を直接使用するアーキテクチャとし、`KafkaPubSub` は publish 専用として責務を限定している。

## トレイト定義

### データ型

| 型 | 説明 |
|----|------|
| `Message` | PubSub メッセージ（`topic: String`・`data: Vec<u8>`・`metadata: HashMap<String, String>`・`id: String`） |

### MessageHandler トレイト

サブスクライブ時のメッセージ受信ハンドラーの抽象インターフェース。

```rust
#[async_trait]
pub trait MessageHandler: Send + Sync {
    /// 受信メッセージを処理する。
    async fn handle(&self, message: Message) -> Result<(), PubSubError>;
}
```

### PubSub トレイト

パブリッシュ・サブスクライブ機能の抽象インターフェース。`k1s0_bb_core::Component` を拡張する。

```rust
#[async_trait]
pub trait PubSub: k1s0_bb_core::Component {
    /// トピックにメッセージをパブリッシュする。
    async fn publish(
        &self,
        topic: &str,
        data: &[u8],
        metadata: Option<HashMap<String, String>>,
    ) -> Result<(), PubSubError>;

    /// トピックをサブスクライブし、サブスクリプション ID を返す。
    async fn subscribe(
        &self,
        topic: &str,
        handler: Box<dyn MessageHandler>,
    ) -> Result<String, PubSubError>;

    /// サブスクリプション ID を指定してサブスクリプションを解除する。
    async fn unsubscribe(&self, subscription_id: &str) -> Result<(), PubSubError>;
}
```

## 実装バリエーション

### Kafka PubSub（`KafkaPubSub`、`kafka` feature 必須）

`k1s0-messaging` の `EventProducer` トレイトをラップした Kafka バックエンドの本番向け PubSub 実装。

| 項目 | 内容 |
|------|------|
| `component_type` | `"pubsub"` |
| metadata `backend` | `"kafka"` |
| `publish()` の挙動 | `EventEnvelope` に変換して `EventProducer::publish()` を呼び出す |
| `subscribe()` の挙動 | 未サポート（`EventConsumer` を直接使用するよう `PubSubError::Subscribe` を返す） |
| `unsubscribe()` の挙動 | 未サポート（同上） |

コンストラクタ:
- `KafkaPubSub::new(name, producer)` — `Arc<dyn EventProducer>` を注入

> Kafka のメッセージ受信は `k1s0-messaging` の `EventConsumer` を直接使用する設計。`subscribe` は将来の拡張用プレースホルダーとして定義されている。

### インメモリ PubSub（`InMemoryPubSub`）

サブスクリプションをメモリ内に保持し、publish 時に同期的にハンドラーを呼び出すテスト・開発用実装。

| 項目 | 内容 |
|------|------|
| `component_type` | `"pubsub"` |
| metadata `backend` | `"memory"` |
| `publish()` の挙動 | UUID でメッセージ ID を生成し、同トピックのサブスクリプションのハンドラーを順次呼び出す |
| `subscribe()` の挙動 | UUID でサブスクリプション ID を生成して登録し、ID を返す |
| `unsubscribe()` の挙動 | ID が存在しない場合は `PubSubError::SubscriptionNotFound` を返す |

## 使用例

```rust
use std::sync::Arc;
use std::collections::HashMap;
use async_trait::async_trait;
use k1s0_bb_pubsub::{InMemoryPubSub, PubSub, Message, MessageHandler, PubSubError};

// ハンドラーを実装する
struct LogHandler;

#[async_trait]
impl MessageHandler for LogHandler {
    async fn handle(&self, message: Message) -> Result<(), PubSubError> {
        println!("受信: topic={}, id={}", message.topic, message.id);
        Ok(())
    }
}

// インメモリ PubSub を初期化する
let pubsub = InMemoryPubSub::new("event-bus");
pubsub.init().await?;

// トピックをサブスクライブする
let sub_id = pubsub
    .subscribe("order.created", Box::new(LogHandler))
    .await?;

// メッセージをパブリッシュする
let mut meta = HashMap::new();
meta.insert("source".to_string(), "order-service".to_string());

pubsub
    .publish("order.created", br#"{"order_id": "123"}"#, Some(meta))
    .await?;

// サブスクリプションを解除する
pubsub.unsubscribe(&sub_id).await?;

// 本番環境では KafkaPubSub に差し替える（kafka feature 使用時）
// use k1s0_bb_pubsub::KafkaPubSub;
// let kafka = KafkaPubSub::new("kafka-bus", Arc::new(producer));
// kafka.init().await?;
// kafka.publish("order.created", data, None).await?;
```

## エラーハンドリング

`PubSubError` は `thiserror` で定義された enum。

| バリアント | 説明 |
|-----------|------|
| `Publish(String)` | メッセージのパブリッシュ失敗 |
| `Subscribe(String)` | サブスクリプション登録失敗（Kafka では常にこのエラーを返す） |
| `SubscriptionNotFound(String)` | 指定したサブスクリプション ID が存在しない |
| `Connection(String)` | ブローカーへの接続エラー |
| `Serialization(String)` | メッセージのシリアライズ・デシリアライズエラー |
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
| `uuid` | メッセージ ID・サブスクリプション ID の生成 |
| `k1s0-messaging` | Kafka `EventProducer` トレイト（`kafka` feature 有効時のみ） |
| `k1s0-kafka` | Kafka 実装（`kafka` feature 有効時のみ） |
| `mockall` | モック生成（`mock` feature 有効時のみ） |

### Feature フラグ

| Feature | 説明 |
|---------|------|
| `kafka` | `k1s0-messaging` / `k1s0-kafka` による `KafkaPubSub` を有効化 |
| `mock` | `mockall` によるモック実装を有効化（テスト用） |

**依存追加**: `k1s0-bb-pubsub = { path = "../../system/library/rust/bb-pubsub" }`

Kafka 実装を使用する場合: `k1s0-bb-pubsub = { path = "../../system/library/rust/bb-pubsub", features = ["kafka"] }`

## 関連ドキュメント

- [Building Blocks 抽象化](../_common/building-blocks.md) — BB アーキテクチャ全体設計
- [bb-core 設計](bb-core.md) — BB 基盤クレート
- [PubSub 設計](../../architecture/messaging/pubsub.md) — PubSub Building Block 詳細

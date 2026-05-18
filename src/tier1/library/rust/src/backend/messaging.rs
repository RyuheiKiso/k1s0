// messaging.rs — k1s0 tier1 Library backend: Messaging/EventBus L1+ trait
// backend 専用カテゴリ（frontend には提供しない）。
// L1+: OSS の全機能を表現 + tier1 横断要素（auth context 伝播 / retry / tracing）を強制。
// 公開 API に OSS 型（rskafka::client::Client / rdkafka::producer::FutureProducer 等）を露出しない。
// Kafka をバックエンドとする Outbox パターンも本 trait 経由で扱う。

// async_trait: async fn in trait を stable で使用するためのマクロ
use async_trait::async_trait;
// anyhow: エラーハンドリング（Result 型の統一）
use anyhow::Result;
// serde: メッセージのシリアライズに使用する
use serde::{Deserialize, Serialize};
// AuthContext: メッセージ送受信に認証コンテキストを伝播する
use crate::core::auth::AuthContext;
// ServicePolicy: retry / timeout / circuit breaker を強制する
use crate::core::policy::ServicePolicy;
// SpanContext: メッセージに tracing コンテキストを伝播する
use crate::core::observability::SpanContext;

// MessageHeaders はメッセージのヘッダーを表す Library 独自型。
// Kafka Record Headers / CloudEvents attributes を Library 独自語彙で表現する。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageHeaders {
    // trace_parent: W3C traceparent（tracing 伝播用）
    pub trace_parent: Option<String>,
    // auth_class: 送信元の AuthContext.auth_class 文字列（audit ログ用）
    pub auth_class: String,
    // tenant_id: メッセージが属するテナントの識別子（マルチテナント分離）
    pub tenant_id: String,
    // schema_id: メッセージスキーマの識別子（Schema Registry と連携する）
    pub schema_id: Option<String>,
    // idempotency_key: べき等性キー（exactly-once 相当の保証に使用する）
    pub idempotency_key: Option<String>,
    // custom: ユーザー定義ヘッダー（key-value ペア）
    pub custom: std::collections::HashMap<String, String>,
}

// Message はメッセージングシステムの 1 メッセージを表す Library 独自型。
// Kafka Record / CloudEvent を Library 独自型に変換して露出しない。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    // topic: メッセージが属するトピック名
    pub topic: String,
    // partition: パーティション番号（0 始まり; None は自動割り当て）
    pub partition: Option<i32>,
    // offset: パーティション内のオフセット（consume 時に設定される）
    pub offset: Option<i64>,
    // key_bytes: メッセージキーのバイト列（ルーティング / compaction に使用する）
    pub key_bytes: Option<Vec<u8>>,
    // value_bytes: メッセージ値のバイト列（ペイロード）
    pub value_bytes: Vec<u8>,
    // headers: メッセージヘッダー
    pub headers: MessageHeaders,
}

// ProduceResult はメッセージ送信の結果を表す struct。
#[derive(Debug, Clone)]
pub struct ProduceResult {
    // topic: 送信先トピック名
    pub topic: String,
    // partition: 割り当てられたパーティション番号
    pub partition: i32,
    // offset: 割り当てられたオフセット
    pub offset: i64,
}

// ConsumeOptions はメッセージ受信時のオプションを表す struct。
#[derive(Debug, Clone)]
pub struct ConsumeOptions {
    // max_messages: 1 回の poll で受信する最大メッセージ数
    pub max_messages: usize,
    // timeout_ms: poll のタイムアウト（ミリ秒; 0 はブロッキング）
    pub timeout_ms: u64,
    // auto_commit: offset の自動コミットを有効にするかどうか
    pub auto_commit: bool,
}

// ConsumeOptions のデフォルト値
impl Default for ConsumeOptions {
    fn default() -> Self {
        // L1+ 向け実用的なデフォルト設定
        Self {
            // max_messages: 100 件
            max_messages: 100,
            // timeout_ms: 1000ms（1 秒ポーリング）
            timeout_ms: 1000,
            // auto_commit: false（明示的 commit を要求する）
            auto_commit: false,
        }
    }
}

// MessageProducer は Messaging/EventBus の送信側 L1+ 抽象 trait。
// auth context 伝播 / retry / tracing は強制する。
#[async_trait]
pub trait MessageProducer: Send + Sync {
    // produce はメッセージを送信する。
    // auth_ctx は MessageHeaders の auth_class と tenant_id に自動注入する。
    // span_ctx は MessageHeaders の trace_parent に W3C traceparent 形式で注入する。
    // policy は retry / timeout / circuit breaker を適用する。
    async fn produce(
        &self,
        topic: &str,
        key_bytes: Option<Vec<u8>>,
        value_bytes: Vec<u8>,
        auth_ctx: &AuthContext,
        span_ctx: Option<&SpanContext>,
        policy: &ServicePolicy,
    ) -> Result<ProduceResult>;

    // produce_batch は複数メッセージを 1 バッチで送信する（スループット最適化）。
    // auth_ctx / span_ctx / policy は全メッセージに共通適用する。
    async fn produce_batch(
        &self,
        messages: Vec<(String, Option<Vec<u8>>, Vec<u8>)>,
        auth_ctx: &AuthContext,
        span_ctx: Option<&SpanContext>,
        policy: &ServicePolicy,
    ) -> Result<Vec<ProduceResult>>;

    // flush は内部バッファを強制送信する（プロセス終了前に呼び出す）。
    async fn flush(&self) -> Result<()>;
}

// MessageConsumer は Messaging/EventBus の受信側 L1+ 抽象 trait。
// auth context 検証 / tracing 伝播 / explicit commit を強制する。
#[async_trait]
pub trait MessageConsumer: Send + Sync {
    // subscribe はトピックとコンシューマーグループを指定して購読を開始する。
    // auth_ctx は購読権限の確認と audit ログに使用する。
    async fn subscribe(
        &self,
        topics: &[&str],
        group_id: &str,
        auth_ctx: &AuthContext,
    ) -> Result<()>;

    // poll は利用可能なメッセージを取得する（options.timeout_ms まで待機する）。
    // 返されたメッセージは commit を呼ぶまで「処理済み」にならない。
    async fn poll(&self, options: ConsumeOptions) -> Result<Vec<Message>>;

    // commit は指定されたメッセージの offset を commit する。
    // auto_commit=false の場合は明示的に呼び出す必要がある。
    async fn commit(&self, messages: &[Message]) -> Result<()>;

    // unsubscribe は購読を解除する。
    async fn unsubscribe(&self) -> Result<()>;
}

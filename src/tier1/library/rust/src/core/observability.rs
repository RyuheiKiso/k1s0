// observability.rs — k1s0 tier1 Library core: Observability L3 trait 群
// Observability/Logging・Tracing・Metrics の 3 サブカテゴリを L3 として定義する。
// OSS 型（tracing::Span / opentelemetry::Meter 等）を公開シグネチャに露出しない。
// 全 trait は Send + Sync を要求する。

// async_trait: async fn in trait を stable で使用するためのマクロ
use async_trait::async_trait;
// anyhow: エラーハンドリング（Result 型の統一）
use anyhow::Result;
// serde: ログ構造体のシリアライズに使用する
use serde::{Deserialize, Serialize};
// std::collections: メトリクスラベルに使用する
use std::collections::HashMap;

// ----- Logging -----

// LogLevel は Library 独自の重要度分類（OSS の log::Level / tracing::Level に依存しない）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    // Trace: 最も詳細なレベル（開発デバッグ用）
    Trace,
    // Debug: デバッグ情報（開発・テスト環境向け）
    Debug,
    // Info: 通常操作の情報（production でも出力する）
    Info,
    // Warn: 注意が必要な状態（degraded だが動作継続）
    Warn,
    // Error: エラーが発生した状態（対応が必要）
    Error,
}

// LogRecord は構造化ログの 1 エントリを表す Library 独自型。
// OSS の Record / Event 型には依存しない。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRecord {
    // level: ログの重要度
    pub level: LogLevel,
    // message: ログのメッセージ文字列
    pub message: String,
    // fields: 構造化フィールド（key-value ペア）
    pub fields: HashMap<String, String>,
    // trace_id: 現在の trace ID（tracing context から伝播する; 未設定は空文字列）
    pub trace_id: String,
    // span_id: 現在の span ID（未設定は空文字列）
    pub span_id: String,
}

// Logger は Library 独自のロギング抽象 trait（L3）。
// OSS の log::Log / tracing::Subscriber を公開 API に露出しない。
#[async_trait]
pub trait Logger: Send + Sync {
    // log は LogRecord を受け取り、バックエンドに書き出す。
    // 実装は OTLP exporter / stdout 等を選択できるが呼び出し元は意識しない。
    async fn log(&self, record: LogRecord) -> Result<()>;

    // flush は内部バッファを強制的にフラッシュする（プロセス終了前に呼び出す）。
    async fn flush(&self) -> Result<()>;
}

// ----- Tracing -----

// SpanContext は Library 独自の trace コンテキスト（OSS の SpanContext に依存しない）。
// W3C TraceContext (traceparent) 形式で伝播する値を保持する。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanContext {
    // trace_id: W3C TraceContext の trace_id（128-bit hex 文字列）
    pub trace_id: String,
    // span_id: W3C TraceContext の span_id（64-bit hex 文字列）
    pub span_id: String,
    // parent_span_id: 親 span の ID（root span は空文字列）
    pub parent_span_id: String,
    // sampled: サンプリング有無（false の場合は記録しない）
    pub sampled: bool,
}

// SpanStatus は span の完了ステータスを表す enum。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpanStatus {
    // Ok: 正常完了
    Ok,
    // Error: エラー終了（エラーメッセージ付き）
    Error(String),
    // Unset: ステータス未設定（デフォルト）
    Unset,
}

// Tracer は Library 独自の tracing 抽象 trait（L3）。
// OSS の opentelemetry::Tracer / tracing::Instrument を公開 API に露出しない。
#[async_trait]
pub trait Tracer: Send + Sync {
    // start_span は名前を受け取り、新しい SpanContext を返す。
    // 呼び出し元は SpanContext を伝播させて子 span を生成する。
    async fn start_span(&self, name: &str, parent: Option<&SpanContext>) -> Result<SpanContext>;

    // end_span は span を完了させ、status と属性を記録する。
    async fn end_span(
        &self,
        ctx: &SpanContext,
        status: SpanStatus,
        attributes: HashMap<String, String>,
    ) -> Result<()>;

    // inject は SpanContext を HTTP ヘッダーマップに W3C traceparent 形式で注入する。
    // gRPC / Kafka header への伝播にも使用する。
    fn inject(&self, ctx: &SpanContext, headers: &mut HashMap<String, String>);

    // extract は HTTP ヘッダーマップから SpanContext を抽出する。
    // None の場合は root span を新規生成する。
    fn extract(&self, headers: &HashMap<String, String>) -> Option<SpanContext>;
}

// ----- Metrics -----

// MetricKind は Library 独自のメトリクス種別（OSS の InstrumentKind に依存しない）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricKind {
    // Counter: 単調増加カウンター（リクエスト数 / エラー数等）
    Counter,
    // Gauge: 任意方向の瞬間値（メモリ使用量 / キューサイズ等）
    Gauge,
    // Histogram: 値の分布（レイテンシ / ペイロードサイズ等）
    Histogram,
}

// MetricPoint は 1 計測ポイントを表す Library 独自型。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricPoint {
    // name: メトリクス名（snake_case）
    pub name: String,
    // kind: メトリクス種別
    pub kind: MetricKind,
    // value: 計測値（f64 で統一する）
    pub value: f64,
    // labels: ラベルセット（key-value ペア）
    pub labels: HashMap<String, String>,
}

// Meter は Library 独自のメトリクス抽象 trait（L3）。
// OSS の opentelemetry::Meter / prometheus::Registry を公開 API に露出しない。
#[async_trait]
pub trait Meter: Send + Sync {
    // record は MetricPoint を受け取り、バックエンドに記録する。
    // OTLP exporter / Prometheus push gateway 等を実装で選択できる。
    async fn record(&self, point: MetricPoint) -> Result<()>;

    // flush は内部バッファを強制的にフラッシュする（プロセス終了前に呼び出す）。
    async fn flush(&self) -> Result<()>;
}

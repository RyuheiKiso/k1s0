//! メトリクス
//!
//! OpenTelemetry メトリクスの初期化と基本操作を提供する。

use crate::config::ObservabilityConfig;

/// メトリクス設定
#[derive(Debug, Clone)]
pub struct MetricsConfig {
    /// サービス名
    pub service_name: String,
    /// 環境名
    pub env: String,
    /// OTel エンドポイント
    pub endpoint: Option<String>,
}

impl MetricsConfig {
    /// ObservabilityConfig から作成
    pub fn from_config(config: &ObservabilityConfig) -> Self {
        Self {
            service_name: config.service_name().to_string(),
            env: config.env().to_string(),
            endpoint: config.otel_endpoint().map(|s| s.to_string()),
        }
    }
}

/// 標準メトリクス名
///
/// k1s0 で使用する標準メトリクス名を定義。
pub struct MetricNames;

impl MetricNames {
    // === HTTP メトリクス ===

    /// HTTP リクエスト数
    pub const HTTP_REQUESTS_TOTAL: &'static str = "http_requests_total";
    /// HTTP リクエスト時間（ヒストグラム）
    pub const HTTP_REQUEST_DURATION_SECONDS: &'static str = "http_request_duration_seconds";
    /// HTTP リクエストサイズ
    pub const HTTP_REQUEST_SIZE_BYTES: &'static str = "http_request_size_bytes";
    /// HTTP レスポンスサイズ
    pub const HTTP_RESPONSE_SIZE_BYTES: &'static str = "http_response_size_bytes";
    /// HTTP アクティブリクエスト数
    pub const HTTP_ACTIVE_REQUESTS: &'static str = "http_active_requests";

    // === gRPC メトリクス ===

    /// gRPC リクエスト数
    pub const GRPC_REQUESTS_TOTAL: &'static str = "grpc_requests_total";
    /// gRPC リクエスト時間（ヒストグラム）
    pub const GRPC_REQUEST_DURATION_SECONDS: &'static str = "grpc_request_duration_seconds";
    /// gRPC メッセージ受信数
    pub const GRPC_MESSAGES_RECEIVED: &'static str = "grpc_messages_received_total";
    /// gRPC メッセージ送信数
    pub const GRPC_MESSAGES_SENT: &'static str = "grpc_messages_sent_total";

    // === DB メトリクス ===

    /// DB クエリ数
    pub const DB_QUERIES_TOTAL: &'static str = "db_queries_total";
    /// DB クエリ時間（ヒストグラム）
    pub const DB_QUERY_DURATION_SECONDS: &'static str = "db_query_duration_seconds";
    /// DB コネクションプールサイズ
    pub const DB_CONNECTIONS_POOL_SIZE: &'static str = "db_connections_pool_size";
    /// DB アクティブコネクション数
    pub const DB_CONNECTIONS_ACTIVE: &'static str = "db_connections_active";

    // === エラーメトリクス ===

    /// エラー数
    pub const ERRORS_TOTAL: &'static str = "errors_total";
    /// 依存障害数
    pub const DEPENDENCY_FAILURES_TOTAL: &'static str = "dependency_failures_total";

    // === ビジネスメトリクス（例） ===

    /// 処理件数
    pub const PROCESSED_ITEMS_TOTAL: &'static str = "processed_items_total";
}

/// 標準ラベル名
pub struct LabelNames;

impl LabelNames {
    /// サービス名
    pub const SERVICE: &'static str = "service";
    /// 環境名
    pub const ENV: &'static str = "env";
    /// HTTP メソッド
    pub const METHOD: &'static str = "method";
    /// HTTP パス
    pub const PATH: &'static str = "path";
    /// HTTP ステータスコード
    pub const STATUS_CODE: &'static str = "status_code";
    /// gRPC サービス
    pub const GRPC_SERVICE: &'static str = "grpc_service";
    /// gRPC メソッド
    pub const GRPC_METHOD: &'static str = "grpc_method";
    /// gRPC ステータス
    pub const GRPC_STATUS: &'static str = "grpc_status";
    /// エラーの種類
    pub const ERROR_KIND: &'static str = "error_kind";
    /// エラーコード
    pub const ERROR_CODE: &'static str = "error_code";
    /// 依存先
    pub const DEPENDENCY: &'static str = "dependency";
}

/// メトリクスラベル
#[derive(Debug, Clone, Default)]
pub struct MetricLabels {
    labels: Vec<(String, String)>,
}

impl MetricLabels {
    /// 新しいラベルセットを作成
    pub fn new() -> Self {
        Self::default()
    }

    /// ラベルを追加
    pub fn add(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.push((key.into(), value.into()));
        self
    }

    /// サービスラベルを追加
    pub fn service(self, name: &str) -> Self {
        self.add(LabelNames::SERVICE, name)
    }

    /// 環境ラベルを追加
    pub fn env(self, env: &str) -> Self {
        self.add(LabelNames::ENV, env)
    }

    /// HTTP ラベルを追加
    pub fn http(self, method: &str, path: &str, status_code: u16) -> Self {
        self.add(LabelNames::METHOD, method)
            .add(LabelNames::PATH, path)
            .add(LabelNames::STATUS_CODE, status_code.to_string())
    }

    /// gRPC ラベルを追加
    pub fn grpc(self, service: &str, method: &str, status: &str) -> Self {
        self.add(LabelNames::GRPC_SERVICE, service)
            .add(LabelNames::GRPC_METHOD, method)
            .add(LabelNames::GRPC_STATUS, status)
    }

    /// エラーラベルを追加
    pub fn error(self, kind: &str, code: &str) -> Self {
        self.add(LabelNames::ERROR_KIND, kind)
            .add(LabelNames::ERROR_CODE, code)
    }

    /// ラベルのリストを取得
    pub fn into_vec(self) -> Vec<(String, String)> {
        self.labels
    }

    /// ラベル数を取得
    pub fn len(&self) -> usize {
        self.labels.len()
    }

    /// ラベルが空かどうか
    pub fn is_empty(&self) -> bool {
        self.labels.is_empty()
    }
}

/// ヒストグラムバケット
///
/// レイテンシ計測用の標準バケット。
pub struct Buckets;

impl Buckets {
    /// HTTP レイテンシ用バケット（秒）
    pub const HTTP_LATENCY: &'static [f64] = &[
        0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
    ];

    /// gRPC レイテンシ用バケット（秒）
    pub const GRPC_LATENCY: &'static [f64] = &[
        0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5,
    ];

    /// DB クエリレイテンシ用バケット（秒）
    pub const DB_LATENCY: &'static [f64] = &[
        0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0,
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_config_from_config() {
        let config = ObservabilityConfig::builder()
            .service_name("test-service")
            .env("dev")
            .otel_endpoint("http://localhost:4317")
            .build()
            .unwrap();

        let metrics_config = MetricsConfig::from_config(&config);

        assert_eq!(metrics_config.service_name, "test-service");
        assert_eq!(metrics_config.env, "dev");
        assert_eq!(
            metrics_config.endpoint,
            Some("http://localhost:4317".to_string())
        );
    }

    #[test]
    fn test_metric_names() {
        assert_eq!(MetricNames::HTTP_REQUESTS_TOTAL, "http_requests_total");
        assert_eq!(MetricNames::GRPC_REQUESTS_TOTAL, "grpc_requests_total");
        assert_eq!(MetricNames::ERRORS_TOTAL, "errors_total");
    }

    #[test]
    fn test_label_names() {
        assert_eq!(LabelNames::SERVICE, "service");
        assert_eq!(LabelNames::METHOD, "method");
        assert_eq!(LabelNames::ERROR_CODE, "error_code");
    }

    #[test]
    fn test_metric_labels() {
        let labels = MetricLabels::new()
            .service("test-service")
            .env("dev")
            .http("GET", "/api/users", 200);

        let vec = labels.into_vec();
        assert_eq!(vec.len(), 5);
    }

    #[test]
    fn test_metric_labels_grpc() {
        let labels = MetricLabels::new()
            .service("test-service")
            .grpc("UserService", "GetUser", "OK");

        let vec = labels.into_vec();
        assert!(vec.iter().any(|(k, v)| k == "grpc_service" && v == "UserService"));
    }

    #[test]
    fn test_buckets() {
        assert!(!Buckets::HTTP_LATENCY.is_empty());
        assert!(!Buckets::GRPC_LATENCY.is_empty());
        assert!(!Buckets::DB_LATENCY.is_empty());
    }
}

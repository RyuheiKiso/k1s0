//! メトリクス
//!
//! Prometheus 形式のメトリクス収集と公開を提供する。
//!
//! # 機能
//!
//! - prometheus-client によるメトリクスレジストリ
//! - 標準メトリクス名とラベルの定義
//! - /metrics エンドポイント用のハンドラ
//!
//! # 使用例
//!
//! ```ignore
//! use k1s0_observability::metrics::{MetricsRegistry, MetricNames};
//!
//! // レジストリの作成
//! let registry = MetricsRegistry::new("my-service", "dev");
//!
//! // カウンタの作成とインクリメント
//! let counter = registry.register_counter(
//!     MetricNames::HTTP_REQUESTS_TOTAL,
//!     "Total HTTP requests",
//! );
//! counter.inc();
//!
//! // メトリクスのエンコード
//! let output = registry.encode();
//! ```

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

// ============================================================================
// Prometheus メトリクスレジストリ
// ============================================================================

#[cfg(feature = "prometheus")]
pub use prometheus_impl::*;

#[cfg(feature = "prometheus")]
mod prometheus_impl {
    use super::*;
    use prometheus_client::{
        encoding::{text::encode, EncodeLabelSet, EncodeLabelValue},
        metrics::{
            counter::Counter,
            family::Family,
            gauge::Gauge,
            histogram::{exponential_buckets, Histogram},
        },
        registry::Registry,
    };
    use std::sync::Arc;

    /// メトリクスレジストリ
    ///
    /// prometheus-client を使用したメトリクス管理。
    #[derive(Clone)]
    pub struct MetricsRegistry {
        inner: Arc<MetricsRegistryInner>,
    }

    struct MetricsRegistryInner {
        registry: std::sync::RwLock<Registry>,
        service_name: String,
        service_env: String,
    }

    impl MetricsRegistry {
        /// 新しいレジストリを作成
        pub fn new(service_name: impl Into<String>, service_env: impl Into<String>) -> Self {
            Self {
                inner: Arc::new(MetricsRegistryInner {
                    registry: std::sync::RwLock::new(Registry::default()),
                    service_name: service_name.into(),
                    service_env: service_env.into(),
                }),
            }
        }

        /// ObservabilityConfig から作成
        pub fn from_config(config: &ObservabilityConfig) -> Self {
            Self::new(config.service_name(), config.env())
        }

        /// サービス名を取得
        pub fn service_name(&self) -> &str {
            &self.inner.service_name
        }

        /// 環境名を取得
        pub fn service_env(&self) -> &str {
            &self.inner.service_env
        }

        /// カウンタを登録
        pub fn register_counter(&self, name: &str, help: &str) -> Counter {
            let counter = Counter::default();
            if let Ok(mut registry) = self.inner.registry.write() {
                registry.register(name, help, counter.clone());
            }
            counter
        }

        /// ラベル付きカウンタファミリーを登録
        pub fn register_counter_family<L: EncodeLabelSet + Clone + std::hash::Hash + Eq + Send + Sync + 'static>(
            &self,
            name: &str,
            help: &str,
        ) -> Family<L, Counter> {
            let family = Family::<L, Counter>::default();
            if let Ok(mut registry) = self.inner.registry.write() {
                registry.register(name, help, family.clone());
            }
            family
        }

        /// ゲージを登録
        pub fn register_gauge(&self, name: &str, help: &str) -> Gauge {
            let gauge = Gauge::default();
            if let Ok(mut registry) = self.inner.registry.write() {
                registry.register(name, help, gauge.clone());
            }
            gauge
        }

        /// ラベル付きゲージファミリーを登録
        pub fn register_gauge_family<L: EncodeLabelSet + Clone + std::hash::Hash + Eq + Send + Sync + 'static>(
            &self,
            name: &str,
            help: &str,
        ) -> Family<L, Gauge> {
            let family = Family::<L, Gauge>::default();
            if let Ok(mut registry) = self.inner.registry.write() {
                registry.register(name, help, family.clone());
            }
            family
        }

        /// ヒストグラムを登録
        pub fn register_histogram(&self, name: &str, help: &str, buckets: &[f64]) -> Histogram {
            let histogram = Histogram::new(buckets.iter().copied());
            if let Ok(mut registry) = self.inner.registry.write() {
                registry.register(name, help, histogram.clone());
            }
            histogram
        }

        /// ラベル付きヒストグラムファミリーを登録
        pub fn register_histogram_family<L: EncodeLabelSet + Clone + std::hash::Hash + Eq + Send + Sync + 'static>(
            &self,
            name: &str,
            help: &str,
            buckets: &[f64],
        ) -> HistogramFamily<L> {
            let buckets_vec: Vec<f64> = buckets.to_vec();
            let family = HistogramFamily::new(buckets_vec);
            // Note: prometheus-client doesn't support histogram families directly,
            // so we use a custom wrapper
            family
        }

        /// Prometheus 形式でエンコード
        pub fn encode(&self) -> String {
            let mut output = String::new();
            if let Ok(registry) = self.inner.registry.read() {
                let _ = encode(&mut output, &registry);
            }
            output
        }
    }

    /// ヒストグラムファミリー
    ///
    /// prometheus-client はヒストグラムファミリーを直接サポートしないため、
    /// カスタム実装を提供する。
    #[derive(Clone)]
    pub struct HistogramFamily<L>
    where
        L: EncodeLabelSet + Clone + std::hash::Hash + Eq,
    {
        histograms: Arc<std::sync::RwLock<std::collections::HashMap<L, Histogram>>>,
        buckets: Vec<f64>,
    }

    impl<L> HistogramFamily<L>
    where
        L: EncodeLabelSet + Clone + std::hash::Hash + Eq,
    {
        /// 新しいヒストグラムファミリーを作成
        pub fn new(buckets: Vec<f64>) -> Self {
            Self {
                histograms: Arc::new(std::sync::RwLock::new(std::collections::HashMap::new())),
                buckets,
            }
        }

        /// ラベルに対応するヒストグラムを取得または作成
        pub fn get_or_create(&self, labels: &L) -> Histogram {
            if let Ok(mut map) = self.histograms.write() {
                map.entry(labels.clone())
                    .or_insert_with(|| Histogram::new(self.buckets.iter().copied()))
                    .clone()
            } else {
                Histogram::new(self.buckets.iter().copied())
            }
        }
    }

    /// HTTP メトリクスラベル
    #[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
    pub struct HttpLabels {
        pub method: String,
        pub path: String,
        pub status_code: u16,
    }

    /// gRPC メトリクスラベル
    #[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
    pub struct GrpcLabels {
        pub service: String,
        pub method: String,
        pub status: String,
    }

    /// 標準メトリクスセット
    ///
    /// HTTP/gRPC/DB の標準メトリクスをまとめて登録する。
    pub struct StandardMetrics {
        pub registry: MetricsRegistry,
        pub http_requests_total: Family<HttpLabels, Counter>,
        pub http_request_duration: HistogramFamily<HttpLabels>,
        pub http_active_requests: Gauge,
        pub grpc_requests_total: Family<GrpcLabels, Counter>,
        pub grpc_request_duration: HistogramFamily<GrpcLabels>,
        pub errors_total: Family<ErrorLabels, Counter>,
    }

    /// エラーラベル
    #[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
    pub struct ErrorLabels {
        pub kind: String,
        pub code: String,
    }

    impl StandardMetrics {
        /// 標準メトリクスを作成
        pub fn new(config: &ObservabilityConfig) -> Self {
            let registry = MetricsRegistry::from_config(config);

            let http_requests_total = registry.register_counter_family(
                MetricNames::HTTP_REQUESTS_TOTAL,
                "Total number of HTTP requests",
            );

            let http_request_duration = registry.register_histogram_family(
                MetricNames::HTTP_REQUEST_DURATION_SECONDS,
                "HTTP request duration in seconds",
                Buckets::HTTP_LATENCY,
            );

            let http_active_requests = registry.register_gauge(
                MetricNames::HTTP_ACTIVE_REQUESTS,
                "Number of active HTTP requests",
            );

            let grpc_requests_total = registry.register_counter_family(
                MetricNames::GRPC_REQUESTS_TOTAL,
                "Total number of gRPC requests",
            );

            let grpc_request_duration = registry.register_histogram_family(
                MetricNames::GRPC_REQUEST_DURATION_SECONDS,
                "gRPC request duration in seconds",
                Buckets::GRPC_LATENCY,
            );

            let errors_total = registry.register_counter_family(
                MetricNames::ERRORS_TOTAL,
                "Total number of errors",
            );

            Self {
                registry,
                http_requests_total,
                http_request_duration,
                http_active_requests,
                grpc_requests_total,
                grpc_request_duration,
                errors_total,
            }
        }

        /// HTTP リクエスト完了を記録
        pub fn record_http_request(
            &self,
            method: &str,
            path: &str,
            status_code: u16,
            duration_seconds: f64,
        ) {
            let labels = HttpLabels {
                method: method.to_string(),
                path: path.to_string(),
                status_code,
            };
            self.http_requests_total.get_or_create(&labels).inc();
            self.http_request_duration
                .get_or_create(&labels)
                .observe(duration_seconds);
        }

        /// gRPC リクエスト完了を記録
        pub fn record_grpc_request(
            &self,
            service: &str,
            method: &str,
            status: &str,
            duration_seconds: f64,
        ) {
            let labels = GrpcLabels {
                service: service.to_string(),
                method: method.to_string(),
                status: status.to_string(),
            };
            self.grpc_requests_total.get_or_create(&labels).inc();
            self.grpc_request_duration
                .get_or_create(&labels)
                .observe(duration_seconds);
        }

        /// エラーを記録
        pub fn record_error(&self, kind: &str, code: &str) {
            let labels = ErrorLabels {
                kind: kind.to_string(),
                code: code.to_string(),
            };
            self.errors_total.get_or_create(&labels).inc();
        }

        /// Prometheus 形式でエンコード
        pub fn encode(&self) -> String {
            self.registry.encode()
        }
    }

    /// /metrics エンドポイント用のハンドラ（axum 向け）
    #[cfg(feature = "axum-layer")]
    pub async fn metrics_handler(
        axum::extract::State(metrics): axum::extract::State<Arc<StandardMetrics>>,
    ) -> impl axum::response::IntoResponse {
        use axum::http::{header, StatusCode};

        let body = metrics.encode();
        (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
            body,
        )
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_metrics_registry() {
            let registry = MetricsRegistry::new("test-service", "dev");
            assert_eq!(registry.service_name(), "test-service");
            assert_eq!(registry.service_env(), "dev");
        }

        #[test]
        fn test_counter() {
            let registry = MetricsRegistry::new("test", "dev");
            let counter = registry.register_counter("test_counter", "A test counter");
            counter.inc();

            let output = registry.encode();
            assert!(output.contains("test_counter"));
        }

        #[test]
        fn test_gauge() {
            let registry = MetricsRegistry::new("test", "dev");
            let gauge = registry.register_gauge("test_gauge", "A test gauge");
            gauge.set(42);

            let output = registry.encode();
            assert!(output.contains("test_gauge"));
        }

        #[test]
        fn test_histogram() {
            let registry = MetricsRegistry::new("test", "dev");
            let histogram = registry.register_histogram(
                "test_histogram",
                "A test histogram",
                Buckets::HTTP_LATENCY,
            );
            histogram.observe(0.1);
            histogram.observe(0.5);

            let output = registry.encode();
            assert!(output.contains("test_histogram"));
        }

        #[test]
        fn test_standard_metrics() {
            let config = ObservabilityConfig::builder()
                .service_name("test-service")
                .env("dev")
                .build()
                .unwrap();

            let metrics = StandardMetrics::new(&config);

            // HTTP リクエストを記録
            metrics.record_http_request("GET", "/api/users", 200, 0.1);

            // gRPC リクエストを記録
            metrics.record_grpc_request("UserService", "GetUser", "OK", 0.05);

            // エラーを記録
            metrics.record_error("INTERNAL", "ERR001");

            let output = metrics.encode();
            assert!(output.contains("http_requests_total"));
        }
    }
}

use prometheus::{
    CounterVec, Encoder, HistogramOpts, HistogramVec, IntCounterVec, Opts, Registry, TextEncoder,
};

/// Metrics は Prometheus メトリクスのヘルパー構造体である。
/// RED メソッド（Rate, Errors, Duration）のメトリクスに加え、
/// DB クエリ、Kafka、キャッシュのメトリクスも提供する。
pub struct Metrics {
    pub http_requests_total: Option<CounterVec>,
    pub http_request_duration: Option<HistogramVec>,
    pub grpc_handled_total: Option<CounterVec>,
    pub grpc_handling_duration: Option<HistogramVec>,
    pub db_query_duration: Option<HistogramVec>,
    pub kafka_messages_produced_total: Option<IntCounterVec>,
    pub kafka_messages_consumed_total: Option<IntCounterVec>,
    pub cache_hits_total: Option<IntCounterVec>,
    pub cache_misses_total: Option<IntCounterVec>,
    registry: Registry,
}

/// デフォルトのヒストグラムバケット（Go 実装と同一）。
const DEFAULT_BUCKETS: &[f64] = &[
    0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
];

impl Metrics {
    /// new は Prometheus メトリクスを初期化して返す。
    /// service_name はメトリクスの service ラベルに使用される。
    pub fn new(service_name: &str) -> Self {
        let registry = Registry::new();

        let http_requests_total = CounterVec::new(
            Opts::new("http_requests_total", "Total number of HTTP requests")
                .const_label("service", service_name),
            &["method", "path", "status"],
        )
        .expect("failed to create http_requests_total counter");

        let http_request_duration = HistogramVec::new(
            HistogramOpts::new(
                "http_request_duration_seconds",
                "Histogram of HTTP request latency",
            )
            .const_label("service", service_name)
            .buckets(DEFAULT_BUCKETS.to_vec()),
            &["method", "path"],
        )
        .expect("failed to create http_request_duration histogram");

        let grpc_handled_total = CounterVec::new(
            Opts::new(
                "grpc_server_handled_total",
                "Total number of RPCs completed on the server",
            )
            .const_label("service", service_name),
            &["grpc_service", "grpc_method", "grpc_code"],
        )
        .expect("failed to create grpc_server_handled_total counter");

        let grpc_handling_duration = HistogramVec::new(
            HistogramOpts::new(
                "grpc_server_handling_seconds",
                "Histogram of response latency of gRPC",
            )
            .const_label("service", service_name)
            .buckets(DEFAULT_BUCKETS.to_vec()),
            &["grpc_service", "grpc_method"],
        )
        .expect("failed to create grpc_server_handling_seconds histogram");

        let db_query_duration = HistogramVec::new(
            HistogramOpts::new(
                "db_query_duration_seconds",
                "Histogram of database query latency",
            )
            .const_label("service", service_name)
            .buckets(DEFAULT_BUCKETS.to_vec()),
            &["query_name", "table"],
        )
        .expect("failed to create db_query_duration histogram");

        let kafka_messages_produced_total = IntCounterVec::new(
            Opts::new(
                "kafka_messages_produced_total",
                "Total number of Kafka messages produced",
            )
            .const_label("service", service_name),
            &["topic"],
        )
        .expect("failed to create kafka_messages_produced_total counter");

        let kafka_messages_consumed_total = IntCounterVec::new(
            Opts::new(
                "kafka_messages_consumed_total",
                "Total number of Kafka messages consumed",
            )
            .const_label("service", service_name),
            &["topic", "consumer_group"],
        )
        .expect("failed to create kafka_messages_consumed_total counter");

        let cache_hits_total = IntCounterVec::new(
            Opts::new("cache_hits_total", "Total number of cache hits")
                .const_label("service", service_name),
            &["cache_name"],
        )
        .expect("failed to create cache_hits_total counter");

        let cache_misses_total = IntCounterVec::new(
            Opts::new("cache_misses_total", "Total number of cache misses")
                .const_label("service", service_name),
            &["cache_name"],
        )
        .expect("failed to create cache_misses_total counter");

        registry
            .register(Box::new(http_requests_total.clone()))
            .expect("failed to register http_requests_total");
        registry
            .register(Box::new(http_request_duration.clone()))
            .expect("failed to register http_request_duration");
        registry
            .register(Box::new(grpc_handled_total.clone()))
            .expect("failed to register grpc_handled_total");
        registry
            .register(Box::new(grpc_handling_duration.clone()))
            .expect("failed to register grpc_handling_duration");
        registry
            .register(Box::new(db_query_duration.clone()))
            .expect("failed to register db_query_duration");
        registry
            .register(Box::new(kafka_messages_produced_total.clone()))
            .expect("failed to register kafka_messages_produced_total");
        registry
            .register(Box::new(kafka_messages_consumed_total.clone()))
            .expect("failed to register kafka_messages_consumed_total");
        registry
            .register(Box::new(cache_hits_total.clone()))
            .expect("failed to register cache_hits_total");
        registry
            .register(Box::new(cache_misses_total.clone()))
            .expect("failed to register cache_misses_total");

        Self {
            http_requests_total: Some(http_requests_total),
            http_request_duration: Some(http_request_duration),
            grpc_handled_total: Some(grpc_handled_total),
            grpc_handling_duration: Some(grpc_handling_duration),
            db_query_duration: Some(db_query_duration),
            kafka_messages_produced_total: Some(kafka_messages_produced_total),
            kafka_messages_consumed_total: Some(kafka_messages_consumed_total),
            cache_hits_total: Some(cache_hits_total),
            cache_misses_total: Some(cache_misses_total),
            registry,
        }
    }

    /// record_http_request は HTTP リクエストカウンタをインクリメントする。
    pub fn record_http_request(&self, method: &str, path: &str, status: &str) {
        if let Some(ref counter) = self.http_requests_total {
            counter.with_label_values(&[method, path, status]).inc();
        }
    }

    /// record_http_duration は HTTP リクエストのレイテンシをヒストグラムに記録する。
    pub fn record_http_duration(&self, method: &str, path: &str, duration_secs: f64) {
        if let Some(ref histogram) = self.http_request_duration {
            histogram
                .with_label_values(&[method, path])
                .observe(duration_secs);
        }
    }

    /// record_grpc_request は gRPC リクエストカウンタをインクリメントする。
    pub fn record_grpc_request(&self, service: &str, method: &str, code: &str) {
        if let Some(ref counter) = self.grpc_handled_total {
            counter.with_label_values(&[service, method, code]).inc();
        }
    }

    /// record_grpc_duration は gRPC リクエストのレイテンシをヒストグラムに記録する。
    pub fn record_grpc_duration(&self, service: &str, method: &str, duration_secs: f64) {
        if let Some(ref histogram) = self.grpc_handling_duration {
            histogram
                .with_label_values(&[service, method])
                .observe(duration_secs);
        }
    }

    /// record_db_query_duration は DB クエリのレイテンシをヒストグラムに記録する。
    pub fn record_db_query_duration(
        &self,
        query_name: &str,
        table: &str,
        duration_secs: f64,
    ) {
        if let Some(ref histogram) = self.db_query_duration {
            histogram
                .with_label_values(&[query_name, table])
                .observe(duration_secs);
        }
    }

    /// record_kafka_message_produced は Kafka メッセージ送信カウンタをインクリメントする。
    pub fn record_kafka_message_produced(&self, topic: &str) {
        if let Some(ref counter) = self.kafka_messages_produced_total {
            counter.with_label_values(&[topic]).inc();
        }
    }

    /// record_kafka_message_consumed は Kafka メッセージ受信カウンタをインクリメントする。
    pub fn record_kafka_message_consumed(&self, topic: &str, consumer_group: &str) {
        if let Some(ref counter) = self.kafka_messages_consumed_total {
            counter
                .with_label_values(&[topic, consumer_group])
                .inc();
        }
    }

    /// record_cache_hit はキャッシュヒットカウンタをインクリメントする。
    pub fn record_cache_hit(&self, cache_name: &str) {
        if let Some(ref counter) = self.cache_hits_total {
            counter.with_label_values(&[cache_name]).inc();
        }
    }

    /// record_cache_miss はキャッシュミスカウンタをインクリメントする。
    pub fn record_cache_miss(&self, cache_name: &str) {
        if let Some(ref counter) = self.cache_misses_total {
            counter.with_label_values(&[cache_name]).inc();
        }
    }

    /// gather_metrics は Prometheus テキストフォーマットでメトリクスを返す。
    /// /metrics エンドポイントのハンドラで使用する。
    pub fn gather_metrics(&self) -> String {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();
        String::from_utf8(buffer).unwrap()
    }
}

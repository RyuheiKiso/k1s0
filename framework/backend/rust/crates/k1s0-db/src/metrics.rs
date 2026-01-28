//! クエリ計測
//!
//! クエリの遅延、件数、失敗を計測し、trace span と連携する。

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// クエリ種別
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryType {
    /// SELECT
    Select,
    /// INSERT
    Insert,
    /// UPDATE
    Update,
    /// DELETE
    Delete,
    /// その他
    Other,
}

impl QueryType {
    /// 文字列表現を取得
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Select => "select",
            Self::Insert => "insert",
            Self::Update => "update",
            Self::Delete => "delete",
            Self::Other => "other",
        }
    }
}

/// クエリ結果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryResult {
    /// 成功
    Success,
    /// エラー
    Error,
    /// タイムアウト
    Timeout,
}

impl QueryResult {
    /// 文字列表現を取得
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Error => "error",
            Self::Timeout => "timeout",
        }
    }
}

/// クエリメトリクス
///
/// 単一クエリの計測結果を保持する。
#[derive(Debug, Clone)]
pub struct QueryMetrics {
    /// クエリ種別
    pub query_type: QueryType,
    /// テーブル名（任意）
    pub table: Option<String>,
    /// 実行時間
    pub duration: Duration,
    /// 結果
    pub result: QueryResult,
    /// 影響行数（INSERT/UPDATE/DELETE の場合）
    pub rows_affected: Option<u64>,
    /// 取得行数（SELECT の場合）
    pub rows_fetched: Option<u64>,
}

impl QueryMetrics {
    /// 新しいメトリクスを作成
    pub fn new(query_type: QueryType, duration: Duration, result: QueryResult) -> Self {
        Self {
            query_type,
            table: None,
            duration,
            result,
            rows_affected: None,
            rows_fetched: None,
        }
    }

    /// テーブル名を設定
    pub fn with_table(mut self, table: impl Into<String>) -> Self {
        self.table = Some(table.into());
        self
    }

    /// 影響行数を設定
    pub fn with_rows_affected(mut self, rows: u64) -> Self {
        self.rows_affected = Some(rows);
        self
    }

    /// 取得行数を設定
    pub fn with_rows_fetched(mut self, rows: u64) -> Self {
        self.rows_fetched = Some(rows);
        self
    }

    /// 成功かどうか
    pub fn is_success(&self) -> bool {
        self.result == QueryResult::Success
    }

    /// 実行時間（ミリ秒）
    pub fn duration_ms(&self) -> u64 {
        self.duration.as_millis() as u64
    }
}

/// クエリタイマー
///
/// クエリの実行時間を計測する。
pub struct QueryTimer {
    start: Instant,
    query_type: QueryType,
    table: Option<String>,
}

impl QueryTimer {
    /// 計測を開始
    pub fn start(query_type: QueryType) -> Self {
        Self {
            start: Instant::now(),
            query_type,
            table: None,
        }
    }

    /// テーブル名を設定
    pub fn with_table(mut self, table: impl Into<String>) -> Self {
        self.table = Some(table.into());
        self
    }

    /// 計測を終了（成功）
    pub fn finish_success(self) -> QueryMetrics {
        self.finish(QueryResult::Success)
    }

    /// 計測を終了（エラー）
    pub fn finish_error(self) -> QueryMetrics {
        self.finish(QueryResult::Error)
    }

    /// 計測を終了（タイムアウト）
    pub fn finish_timeout(self) -> QueryMetrics {
        self.finish(QueryResult::Timeout)
    }

    /// 計測を終了
    fn finish(self, result: QueryResult) -> QueryMetrics {
        let mut metrics = QueryMetrics::new(self.query_type, self.start.elapsed(), result);
        if let Some(table) = self.table {
            metrics = metrics.with_table(table);
        }
        metrics
    }

    /// 経過時間を取得
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

/// 集約されたDBメトリクス
///
/// 接続プール全体の統計情報を保持する。
#[derive(Debug)]
pub struct DbMetrics {
    /// 総クエリ数
    total_queries: AtomicU64,
    /// 成功クエリ数
    successful_queries: AtomicU64,
    /// 失敗クエリ数
    failed_queries: AtomicU64,
    /// タイムアウトクエリ数
    timeout_queries: AtomicU64,
    /// 総実行時間（マイクロ秒）
    total_duration_us: AtomicU64,
    /// 最大実行時間（マイクロ秒）
    max_duration_us: AtomicU64,
    /// 接続取得数
    connections_acquired: AtomicU64,
    /// 接続解放数
    connections_released: AtomicU64,
    /// 接続取得失敗数
    connection_failures: AtomicU64,
}

impl DbMetrics {
    /// 新しいメトリクスを作成
    pub fn new() -> Self {
        Self {
            total_queries: AtomicU64::new(0),
            successful_queries: AtomicU64::new(0),
            failed_queries: AtomicU64::new(0),
            timeout_queries: AtomicU64::new(0),
            total_duration_us: AtomicU64::new(0),
            max_duration_us: AtomicU64::new(0),
            connections_acquired: AtomicU64::new(0),
            connections_released: AtomicU64::new(0),
            connection_failures: AtomicU64::new(0),
        }
    }

    /// クエリメトリクスを記録
    pub fn record_query(&self, metrics: &QueryMetrics) {
        self.total_queries.fetch_add(1, Ordering::SeqCst);

        match metrics.result {
            QueryResult::Success => {
                self.successful_queries.fetch_add(1, Ordering::SeqCst);
            }
            QueryResult::Error => {
                self.failed_queries.fetch_add(1, Ordering::SeqCst);
            }
            QueryResult::Timeout => {
                self.timeout_queries.fetch_add(1, Ordering::SeqCst);
            }
        }

        let duration_us = metrics.duration.as_micros() as u64;
        self.total_duration_us
            .fetch_add(duration_us, Ordering::SeqCst);

        // 最大実行時間の更新（CASで競合を避ける）
        let mut current_max = self.max_duration_us.load(Ordering::SeqCst);
        while duration_us > current_max {
            match self.max_duration_us.compare_exchange_weak(
                current_max,
                duration_us,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => break,
                Err(actual) => current_max = actual,
            }
        }
    }

    /// 接続取得を記録
    pub fn record_connection_acquired(&self) {
        self.connections_acquired.fetch_add(1, Ordering::SeqCst);
    }

    /// 接続解放を記録
    pub fn record_connection_released(&self) {
        self.connections_released.fetch_add(1, Ordering::SeqCst);
    }

    /// 接続失敗を記録
    pub fn record_connection_failure(&self) {
        self.connection_failures.fetch_add(1, Ordering::SeqCst);
    }

    /// 総クエリ数を取得
    pub fn total_queries(&self) -> u64 {
        self.total_queries.load(Ordering::SeqCst)
    }

    /// 成功クエリ数を取得
    pub fn successful_queries(&self) -> u64 {
        self.successful_queries.load(Ordering::SeqCst)
    }

    /// 失敗クエリ数を取得
    pub fn failed_queries(&self) -> u64 {
        self.failed_queries.load(Ordering::SeqCst)
    }

    /// タイムアウトクエリ数を取得
    pub fn timeout_queries(&self) -> u64 {
        self.timeout_queries.load(Ordering::SeqCst)
    }

    /// 平均実行時間（マイクロ秒）を取得
    pub fn average_duration_us(&self) -> u64 {
        let total = self.total_queries.load(Ordering::SeqCst);
        if total == 0 {
            0
        } else {
            self.total_duration_us.load(Ordering::SeqCst) / total
        }
    }

    /// 最大実行時間（マイクロ秒）を取得
    pub fn max_duration_us(&self) -> u64 {
        self.max_duration_us.load(Ordering::SeqCst)
    }

    /// 接続取得数を取得
    pub fn connections_acquired(&self) -> u64 {
        self.connections_acquired.load(Ordering::SeqCst)
    }

    /// 接続失敗数を取得
    pub fn connection_failures(&self) -> u64 {
        self.connection_failures.load(Ordering::SeqCst)
    }

    /// 成功率を取得（0.0 - 1.0）
    pub fn success_rate(&self) -> f64 {
        let total = self.total_queries.load(Ordering::SeqCst);
        if total == 0 {
            1.0
        } else {
            self.successful_queries.load(Ordering::SeqCst) as f64 / total as f64
        }
    }
}

impl Default for DbMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// trace span 用のラベル
#[derive(Debug, Clone)]
pub struct DbSpanLabels {
    /// データベース名
    pub db_name: String,
    /// テーブル名（任意）
    pub table: Option<String>,
    /// 操作種別
    pub operation: String,
}

impl DbSpanLabels {
    /// 新しいラベルを作成
    pub fn new(db_name: impl Into<String>, operation: impl Into<String>) -> Self {
        Self {
            db_name: db_name.into(),
            table: None,
            operation: operation.into(),
        }
    }

    /// テーブル名を設定
    pub fn with_table(mut self, table: impl Into<String>) -> Self {
        self.table = Some(table.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_type() {
        assert_eq!(QueryType::Select.as_str(), "select");
        assert_eq!(QueryType::Insert.as_str(), "insert");
        assert_eq!(QueryType::Update.as_str(), "update");
        assert_eq!(QueryType::Delete.as_str(), "delete");
    }

    #[test]
    fn test_query_result() {
        assert_eq!(QueryResult::Success.as_str(), "success");
        assert_eq!(QueryResult::Error.as_str(), "error");
        assert_eq!(QueryResult::Timeout.as_str(), "timeout");
    }

    #[test]
    fn test_query_metrics() {
        let metrics = QueryMetrics::new(
            QueryType::Select,
            Duration::from_millis(100),
            QueryResult::Success,
        )
        .with_table("users")
        .with_rows_fetched(10);

        assert!(metrics.is_success());
        assert_eq!(metrics.duration_ms(), 100);
        assert_eq!(metrics.table, Some("users".to_string()));
        assert_eq!(metrics.rows_fetched, Some(10));
    }

    #[test]
    fn test_query_timer() {
        let timer = QueryTimer::start(QueryType::Select).with_table("users");
        std::thread::sleep(Duration::from_millis(10));
        let metrics = timer.finish_success();

        assert!(metrics.duration.as_millis() >= 10);
        assert!(metrics.is_success());
    }

    #[test]
    fn test_db_metrics() {
        let metrics = DbMetrics::new();

        // クエリを記録
        let query1 = QueryMetrics::new(
            QueryType::Select,
            Duration::from_micros(1000),
            QueryResult::Success,
        );
        metrics.record_query(&query1);

        let query2 = QueryMetrics::new(
            QueryType::Select,
            Duration::from_micros(2000),
            QueryResult::Error,
        );
        metrics.record_query(&query2);

        assert_eq!(metrics.total_queries(), 2);
        assert_eq!(metrics.successful_queries(), 1);
        assert_eq!(metrics.failed_queries(), 1);
        assert_eq!(metrics.average_duration_us(), 1500);
        assert_eq!(metrics.max_duration_us(), 2000);
        assert_eq!(metrics.success_rate(), 0.5);
    }

    #[test]
    fn test_db_metrics_connection() {
        let metrics = DbMetrics::new();

        metrics.record_connection_acquired();
        metrics.record_connection_acquired();
        metrics.record_connection_released();
        metrics.record_connection_failure();

        assert_eq!(metrics.connections_acquired(), 2);
        assert_eq!(metrics.connection_failures(), 1);
    }

    #[test]
    fn test_db_span_labels() {
        let labels = DbSpanLabels::new("mydb", "select").with_table("users");
        assert_eq!(labels.db_name, "mydb");
        assert_eq!(labels.operation, "select");
        assert_eq!(labels.table, Some("users".to_string()));
    }
}

//! エラーメトリクス
//!
//! エラーのメトリクス出力を統一する。
//! `error_kind` / `error_code` / status code をラベルとして使用。

use crate::application::AppError;
use crate::domain::ErrorKind;
use crate::presentation::GrpcStatusCode;

/// メトリクス用のエラーラベル
///
/// Prometheus 等のメトリクスシステムで使用するラベルを提供。
#[derive(Debug, Clone)]
pub struct ErrorMetricLabels {
    /// エラーの種類（error_kind）
    pub error_kind: String,
    /// エラーコード（error_code）
    pub error_code: String,
    /// HTTP ステータスコード（REST の場合）
    pub http_status_code: Option<String>,
    /// gRPC ステータスコード（gRPC の場合）
    pub grpc_status_code: Option<String>,
    /// リトライ可能かどうか
    pub retryable: String,
}

impl ErrorMetricLabels {
    /// AppError からラベルを作成
    pub fn from_app_error(app_error: &AppError) -> Self {
        Self {
            error_kind: app_error.kind().as_str().to_string(),
            error_code: app_error.error_code().to_string(),
            http_status_code: None,
            grpc_status_code: None,
            retryable: app_error.is_retryable().to_string(),
        }
    }

    /// HTTP ステータスコードを設定
    pub fn with_http_status(mut self, status_code: u16) -> Self {
        self.http_status_code = Some(status_code.to_string());
        self
    }

    /// gRPC ステータスコードを設定
    pub fn with_grpc_status(mut self, status_code: GrpcStatusCode) -> Self {
        self.grpc_status_code = Some(status_code.as_str().to_string());
        self
    }

    /// ラベルをキーバリューペアとして取得
    pub fn as_pairs(&self) -> Vec<(&'static str, &str)> {
        let mut pairs = vec![
            ("error_kind", self.error_kind.as_str()),
            ("error_code", self.error_code.as_str()),
            ("retryable", self.retryable.as_str()),
        ];

        if let Some(ref status) = self.http_status_code {
            pairs.push(("http_status_code", status.as_str()));
        }

        if let Some(ref status) = self.grpc_status_code {
            pairs.push(("grpc_status_code", status.as_str()));
        }

        pairs
    }
}

/// メトリクス出力のためのトレイト
///
/// エラーをメトリクスとして出力可能にする。
pub trait Measurable {
    /// メトリクスラベルに変換
    fn to_metric_labels(&self) -> ErrorMetricLabels;
}

impl Measurable for AppError {
    fn to_metric_labels(&self) -> ErrorMetricLabels {
        ErrorMetricLabels::from_app_error(self)
    }
}

/// エラーカウンターの名前定義
pub struct ErrorMetricNames;

impl ErrorMetricNames {
    /// HTTP エラーカウンター
    pub const HTTP_ERRORS_TOTAL: &'static str = "http_errors_total";

    /// gRPC エラーカウンター
    pub const GRPC_ERRORS_TOTAL: &'static str = "grpc_errors_total";

    /// アプリケーションエラーカウンター
    pub const APP_ERRORS_TOTAL: &'static str = "app_errors_total";

    /// 依存障害カウンター
    pub const DEPENDENCY_FAILURES_TOTAL: &'static str = "dependency_failures_total";
}

/// エラーの種類ごとのカウント用ヘルパー
#[derive(Debug, Default)]
pub struct ErrorCounter {
    /// 入力不備
    pub invalid_input: u64,
    /// リソースが見つからない
    pub not_found: u64,
    /// 競合
    pub conflict: u64,
    /// 認証エラー
    pub unauthorized: u64,
    /// 認可エラー
    pub forbidden: u64,
    /// 依存障害
    pub dependency_failure: u64,
    /// 一時障害
    pub transient: u64,
    /// 不変条件違反
    pub invariant_violation: u64,
    /// 内部エラー
    pub internal: u64,
}

impl ErrorCounter {
    /// 新しいカウンターを作成
    pub fn new() -> Self {
        Self::default()
    }

    /// エラーをカウント
    pub fn count(&mut self, kind: ErrorKind) {
        match kind {
            ErrorKind::InvalidInput => self.invalid_input += 1,
            ErrorKind::NotFound => self.not_found += 1,
            ErrorKind::Conflict => self.conflict += 1,
            ErrorKind::Unauthorized => self.unauthorized += 1,
            ErrorKind::Forbidden => self.forbidden += 1,
            ErrorKind::DependencyFailure => self.dependency_failure += 1,
            ErrorKind::Transient => self.transient += 1,
            ErrorKind::InvariantViolation => self.invariant_violation += 1,
            ErrorKind::Internal => self.internal += 1,
        }
    }

    /// 合計を取得
    pub fn total(&self) -> u64 {
        self.invalid_input
            + self.not_found
            + self.conflict
            + self.unauthorized
            + self.forbidden
            + self.dependency_failure
            + self.transient
            + self.invariant_violation
            + self.internal
    }

    /// クライアントエラーの合計
    pub fn client_errors(&self) -> u64 {
        self.invalid_input
            + self.not_found
            + self.conflict
            + self.unauthorized
            + self.forbidden
            + self.invariant_violation
    }

    /// サーバーエラーの合計
    pub fn server_errors(&self) -> u64 {
        self.dependency_failure + self.transient + self.internal
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DomainError, ErrorCode};

    #[test]
    fn test_error_metric_labels_from_app_error() {
        let domain_err = DomainError::not_found("User", "123");
        let app_err = AppError::from_domain(domain_err, ErrorCode::new("USER_NOT_FOUND"));

        let labels = ErrorMetricLabels::from_app_error(&app_err);

        assert_eq!(labels.error_kind, "NOT_FOUND");
        assert_eq!(labels.error_code, "USER_NOT_FOUND");
        assert_eq!(labels.retryable, "false");
    }

    #[test]
    fn test_error_metric_labels_with_http_status() {
        let domain_err = DomainError::not_found("User", "123");
        let app_err = AppError::from_domain(domain_err, ErrorCode::not_found());

        let labels = ErrorMetricLabels::from_app_error(&app_err).with_http_status(404);

        assert_eq!(labels.http_status_code, Some("404".to_string()));
    }

    #[test]
    fn test_error_metric_labels_with_grpc_status() {
        let domain_err = DomainError::not_found("User", "123");
        let app_err = AppError::from_domain(domain_err, ErrorCode::not_found());

        let labels =
            ErrorMetricLabels::from_app_error(&app_err).with_grpc_status(GrpcStatusCode::NotFound);

        assert_eq!(labels.grpc_status_code, Some("NOT_FOUND".to_string()));
    }

    #[test]
    fn test_error_metric_labels_as_pairs() {
        let domain_err = DomainError::not_found("User", "123");
        let app_err = AppError::from_domain(domain_err, ErrorCode::new("USER_NOT_FOUND"));

        let labels = ErrorMetricLabels::from_app_error(&app_err).with_http_status(404);
        let pairs = labels.as_pairs();

        assert!(pairs.contains(&("error_kind", "NOT_FOUND")));
        assert!(pairs.contains(&("error_code", "USER_NOT_FOUND")));
        assert!(pairs.contains(&("http_status_code", "404")));
    }

    #[test]
    fn test_measurable_trait() {
        let app_err = AppError::from_domain(
            DomainError::internal("test"),
            ErrorCode::internal(),
        );

        let labels = app_err.to_metric_labels();
        assert_eq!(labels.error_kind, "INTERNAL");
    }

    #[test]
    fn test_error_counter() {
        let mut counter = ErrorCounter::new();

        counter.count(ErrorKind::NotFound);
        counter.count(ErrorKind::NotFound);
        counter.count(ErrorKind::Internal);
        counter.count(ErrorKind::Unauthorized);

        assert_eq!(counter.not_found, 2);
        assert_eq!(counter.internal, 1);
        assert_eq!(counter.unauthorized, 1);
        assert_eq!(counter.total(), 4);
        assert_eq!(counter.client_errors(), 3);
        assert_eq!(counter.server_errors(), 1);
    }

    #[test]
    fn test_metric_names() {
        assert_eq!(ErrorMetricNames::HTTP_ERRORS_TOTAL, "http_errors_total");
        assert_eq!(ErrorMetricNames::GRPC_ERRORS_TOTAL, "grpc_errors_total");
    }
}

//! k1s0-server-common: Shared server infrastructure for k1s0 system tier.
//!
//! Provides structured error codes following the `SYS_{SERVICE}_{ERROR}` pattern,
//! unified error response types, and axum integration for HTTP error responses.

pub mod auth;
/// 可観測性（Observability）設定の共通構造体モジュール。
/// 全サーバーで重複していた ObservabilityConfig 等を一箇所に集約する。
pub mod config;
pub mod error;
pub mod infra_guard;
#[cfg(any(feature = "middleware", test))]
pub mod middleware;
pub mod pagination;
pub mod response;
/// グレースフルシャットダウン用のシグナル待機モジュール
#[cfg(feature = "shutdown")]
pub mod shutdown;
/// サーバー起動ボイラープレート削減のための ServerBuilder モジュール。
/// テレメトリ初期化・DB プール・JWKS 検証器・Metrics 生成の共通処理を提供する。
#[cfg(feature = "startup")]
pub mod startup;

#[cfg(feature = "startup")]
/// `parse_pool_duration` を再エクスポートする。
/// 各サーバーの DB プール設定で `conn_max_lifetime` 文字列を Duration に変換するために使用する。
pub use startup::parse_pool_duration;

/// デフォルトの OpenTelemetry コレクターエンドポイント。
/// 全サーバーの設定デフォルト値として使用する。エンドポイント変更時はここだけ修正すればよい。
pub const DEFAULT_OTEL_ENDPOINT: &str = "http://otel-collector.observability:4317";

/// デフォルトの gRPC ポート番号。
/// 全サーバーの設定デフォルト値として使用する。ポート変更時はここだけ修正すればよい。
pub const DEFAULT_GRPC_PORT: u16 = 50051;

/// デフォルトの HTTP/REST ポート番号。
/// 全サーバーの設定デフォルト値として使用する。
pub const DEFAULT_HTTP_PORT: u16 = 8080;

pub use auth::{allow_insecure_no_auth, require_auth_state};
#[cfg(feature = "grpc-auth")]
pub use error::{map_anyhow_to_grpc_status, IntoGrpcStatus};
pub use error::{ErrorBody, ErrorCode, ErrorDetail, ErrorResponse, ServiceError};
pub use infra_guard::{allow_in_memory_infra, require_infra, InfraKind};
pub use pagination::{PaginatedResponse, PaginationResponse};
pub use response::ApiResponse;

/// 可観測性設定の共通型を再エクスポートする。
/// サーバー側で `use k1s0_server_common::{ObservabilityConfig, LogConfig, ...}` と書ける。
pub use config::{LogConfig, MetricsConfig, ObservabilityConfig, TraceConfig};

/// LIKE/ILIKE 検索パターンの特殊文字（\, %, _）をエスケープする。
/// SQL クエリで `ESCAPE '\'` と組み合わせて使用すること。
/// M-02 監査対応: 意図しない全件マッチや検索精度劣化を防ぐ。
#[must_use]
pub fn escape_like_pattern(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

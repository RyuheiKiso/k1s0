//! k1s0-health: ヘルスチェックエンドポイント
//!
//! このクレートは、k1s0 フレームワークにおけるヘルスチェック機能の
//! 標準化されたインターフェースを提供する。
//!
//! ## 機能
//!
//! - **ヘルスステータス**: Healthy / Degraded / Unhealthy の3段階
//! - **コンポーネントヘルス**: 各コンポーネント（DB、キャッシュなど）の状態
//! - **Kubernetes プローブ**: readiness / liveness / startup プローブ対応
//! - **JSON レスポンス**: 標準化された JSON 形式
//!
//! ## Kubernetes 対応
//!
//! - `/readyz` - Readiness プローブ（トラフィック受け入れ可能か）
//! - `/livez` - Liveness プローブ（プロセスが生きているか）
//! - `/startupz` - Startup プローブ（起動完了か）
//!
//! ## 使用例
//!
//! ```rust
//! use k1s0_health::{HealthResponse, HealthStatus, ComponentHealth};
//! use k1s0_health::probe::{ProbeHandler, ReadinessState};
//! use std::sync::Arc;
//!
//! // シンプルなヘルスレスポンス
//! let response = HealthResponse::new("my-service")
//!     .with_version("1.0.0")
//!     .with_component(ComponentHealth::healthy("database"))
//!     .with_component(ComponentHealth::healthy("cache"));
//!
//! assert_eq!(response.status, HealthStatus::Healthy);
//!
//! // プローブハンドラーの使用
//! let readiness = Arc::new(ReadinessState::ready());
//! let handler = ProbeHandler::new("my-service")
//!     .with_version("1.0.0")
//!     .with_readiness(readiness.clone());
//!
//! // Liveness は常に成功
//! let liveness = handler.liveness();
//! assert!(liveness.status.is_healthy());
//!
//! // Graceful shutdown 時に readiness を false に
//! readiness.set_not_ready();
//! ```

pub mod check;
pub mod probe;

// 主要な型の再エクスポート
pub use check::{ComponentHealth, HealthResponse, HealthStatus, SimpleHealthResponse};
pub use probe::{
    CheckFn, HealthChecker, ProbeConfig, ProbeHandler, ProbeType, ReadinessState,
};

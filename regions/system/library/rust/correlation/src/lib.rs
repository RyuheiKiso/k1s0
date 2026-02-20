//! k1s0-correlation: 相関 ID・トレース ID の生成・伝播ライブラリ。
//!
//! 分散システムにおけるリクエスト追跡のための
//! 相関 ID と OpenTelemetry トレース ID の管理を提供する。

pub mod context;
pub mod id;

pub use context::{CorrelationContext, CorrelationHeaders};
pub use id::{CorrelationId, TraceId};

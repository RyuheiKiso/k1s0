//! 認証ミドルウェアの再エクスポート。
//!
//! 実際の認証ロジックは `k1s0-server-common` クレートに実装されている。
//! このモジュールはサービス固有のカスタマイズポイントとして存在する。

pub use k1s0_server_common::middleware::auth_middleware::{auth_middleware, AuthState};

/// Claims にシステムティアのアクセス権があるかどうかを判定する。
pub(crate) fn claims_have_system_tier(claims: &k1s0_auth::Claims) -> bool {
    k1s0_auth::has_tier_access(claims, "system")
}

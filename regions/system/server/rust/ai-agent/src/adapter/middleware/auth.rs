//! 認証ミドルウェアの再エクスポート。
//!
//! 実際の認証ロジックは `k1s0-server-common` クレートに実装されている。
//! このモジュールはサービス固有のカスタマイズポイントとして存在する。

pub use k1s0_server_common::middleware::auth_middleware::{auth_middleware, AuthState};


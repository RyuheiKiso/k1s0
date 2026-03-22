// 認証状態の型定義。
use std::sync::Arc;

/// 認証状態（JWT 検証器を保持）
#[derive(Clone)]
pub struct AuthState {
    pub verifier: Arc<k1s0_auth::JwksVerifier>,
}

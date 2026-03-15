use anyhow::{bail, Result};
use tracing::warn;

/// 認証バイパス判定（デバッグビルドまたは dev-auth-bypass フィーチャー有効時のみ）。
/// dev/test 環境かつ環境変数 ALLOW_INSECURE_NO_AUTH=true の場合のみ認証をスキップする。
/// リリースビルドかつフィーチャー無効時はこの関数は常に false を返し、バイパスは不可能になる。
///
/// 有効化条件:
/// - `cargo run`（デバッグビルド）: 自動的に有効
/// - `cargo build --release --features k1s0-server-common/dev-auth-bypass`: 明示的に有効化
/// - `cargo build --release`（本番 Dockerfile）: 完全に除去
#[cfg(any(debug_assertions, feature = "dev-auth-bypass"))]
pub fn allow_insecure_no_auth(environment: &str) -> bool {
    matches!(environment, "dev" | "test")
        && std::env::var("ALLOW_INSECURE_NO_AUTH")
            .map(|value| value.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
}

/// リリースビルド用（dev-auth-bypass フィーチャー無効時）: 認証バイパスは常に不許可。
/// プロダクションバイナリからバイパスロジックを完全に除去する。
#[cfg(not(any(debug_assertions, feature = "dev-auth-bypass")))]
pub fn allow_insecure_no_auth(_environment: &str) -> bool {
    false
}

/// 認証状態の検証。auth_state が None の場合、バイパスが有効でなければエラーを返す。
pub fn require_auth_state<T>(
    service_name: &str,
    environment: &str,
    auth_state: Option<T>,
) -> Result<Option<T>> {
    if auth_state.is_some() {
        return Ok(auth_state);
    }

    if allow_insecure_no_auth(environment) {
        warn!(
            environment = %environment,
            service = %service_name,
            "service is running without authentication because ALLOW_INSECURE_NO_AUTH=true"
        );
        return Ok(None);
    }

    bail!(
        "auth configuration is required for {} (environment: {}). \
Set auth.* in the config, or use ALLOW_INSECURE_NO_AUTH=true only for dev/test.",
        service_name,
        environment
    )
}

#[cfg(test)]
mod tests {
    use super::{allow_insecure_no_auth, require_auth_state};
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    // ALLOW_INSECURE_NO_AUTH が true のとき dev と test 環境のみ認証なしが許可されることを確認する。
    #[test]
    fn allows_insecure_auth_override_only_for_dev_and_test() {
        let _guard = env_lock().lock().unwrap();
        std::env::set_var("ALLOW_INSECURE_NO_AUTH", "true");

        assert!(allow_insecure_no_auth("dev"));
        assert!(allow_insecure_no_auth("test"));
        assert!(!allow_insecure_no_auth("staging"));

        std::env::remove_var("ALLOW_INSECURE_NO_AUTH");
    }

    // オーバーライドなしで認証設定が未指定の場合にエラーが返されることを確認する。
    #[test]
    fn rejects_missing_auth_without_override() {
        let _guard = env_lock().lock().unwrap();
        std::env::remove_var("ALLOW_INSECURE_NO_AUTH");

        let err = require_auth_state::<()>("example-service", "dev", None).unwrap_err();

        assert!(err
            .to_string()
            .contains("auth configuration is required for example-service"));
    }

    // ALLOW_INSECURE_NO_AUTH 有効時に認証設定なしでも None が返されることを確認する。
    #[test]
    fn accepts_missing_auth_when_override_is_enabled() {
        let _guard = env_lock().lock().unwrap();
        std::env::set_var("ALLOW_INSECURE_NO_AUTH", "true");

        let auth_state = require_auth_state::<()>("example-service", "dev", None).unwrap();

        assert!(auth_state.is_none());

        std::env::remove_var("ALLOW_INSECURE_NO_AUTH");
    }

    // リリースビルドでは認証バイパスが常に拒否されることを確認する。
    // NOTE: テストは debug_assertions が有効な状態で実行されるため、
    // リリースビルドの挙動は `cargo test --release` で別途検証する。
    #[test]
    fn allow_insecure_no_auth_rejects_production() {
        let _guard = env_lock().lock().unwrap();
        std::env::set_var("ALLOW_INSECURE_NO_AUTH", "true");

        // production/staging 環境は debug ビルドでも拒否される
        assert!(!allow_insecure_no_auth("production"));
        assert!(!allow_insecure_no_auth("staging"));
        assert!(!allow_insecure_no_auth("prod"));

        std::env::remove_var("ALLOW_INSECURE_NO_AUTH");
    }
}

// インフラ設定ガードモジュール
// stable サービスが設定不足時に in-memory/noop で静かに起動するのを防止する

use std::fmt;

/// インフラ設定のバイパスが許可されているか判定する。
/// dev/test 環境かつ `ALLOW_IN_MEMORY_INFRA=true` の場合のみ有効。
#[cfg(any(debug_assertions, feature = "dev-infra-bypass"))]
#[must_use]
pub fn allow_in_memory_infra(environment: &str) -> bool {
    // dev/test 環境でのみバイパスを許可
    let is_dev = matches!(environment, "development" | "dev" | "test" | "local");
    let env_flag = std::env::var("ALLOW_IN_MEMORY_INFRA")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false);
    is_dev && env_flag
}

/// リリースビルドかつ dev-infra-bypass フィーチャー無効の場合は常に false
#[cfg(not(any(debug_assertions, feature = "dev-infra-bypass")))]
pub fn allow_in_memory_infra(_environment: &str) -> bool {
    false
}

/// インフラ種別を表す列挙型
#[derive(Debug, Clone, Copy)]
pub enum InfraKind {
    Database,
    Kafka,
    Redis,
    Storage,
}

impl fmt::Display for InfraKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InfraKind::Database => write!(f, "Database"),
            InfraKind::Kafka => write!(f, "Kafka"),
            InfraKind::Redis => write!(f, "Redis"),
            InfraKind::Storage => write!(f, "Storage"),
        }
    }
}

/// インフラ設定の存在を検証するガード関数。
///
/// - value が Some → そのまま返す
/// - None + バイパス有効 → warn ログ + Ok(None) を返す
/// - None + バイパス無効 → エラーを返す
pub fn require_infra<T>(
    name: &str,
    kind: InfraKind,
    environment: &str,
    value: Option<T>,
) -> anyhow::Result<Option<T>> {
    match value {
        Some(v) => Ok(Some(v)),
        None => {
            if allow_in_memory_infra(environment) {
                tracing::warn!(
                    service = name,
                    infra = %kind,
                    environment,
                    "{}の{}設定が未指定です。in-memory/noop フォールバックで起動します（開発環境のみ許可）",
                    name,
                    kind,
                );
                Ok(None)
            } else {
                anyhow::bail!(
                    "{name}の{kind}設定が必要です。環境: {environment} — 開発環境では ALLOW_IN_MEMORY_INFRA=true で回避できます",
                )
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    /// テスト間で環境変数操作を排他制御するためのロック
    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    /// dev 環境 + 環境変数なし → バイパス不可
    #[test]
    fn test_allow_in_memory_infra_dev_without_env() {
        let _guard = env_lock().lock().unwrap();
        // 環境変数をクリア
        std::env::remove_var("ALLOW_IN_MEMORY_INFRA");
        assert!(!allow_in_memory_infra("development"));
    }

    /// dev 環境 + ALLOW_IN_MEMORY_INFRA=true → バイパス可
    #[test]
    fn test_allow_in_memory_infra_dev_with_env() {
        let _guard = env_lock().lock().unwrap();
        std::env::set_var("ALLOW_IN_MEMORY_INFRA", "true");
        let result = allow_in_memory_infra("development");
        std::env::remove_var("ALLOW_IN_MEMORY_INFRA");
        assert!(result);
    }

    /// production 環境 → 常にバイパス不可
    #[test]
    fn test_allow_in_memory_infra_production() {
        let _guard = env_lock().lock().unwrap();
        std::env::set_var("ALLOW_IN_MEMORY_INFRA", "true");
        let result = allow_in_memory_infra("production");
        std::env::remove_var("ALLOW_IN_MEMORY_INFRA");
        assert!(!result);
    }

    /// require_infra: 値がある場合は即座に返す
    #[test]
    fn test_require_infra_some() {
        let _guard = env_lock().lock().unwrap();
        let result = require_infra(
            "test-server",
            InfraKind::Database,
            "production",
            Some("url"),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("url"));
    }

    /// require_infra: None + production → エラー
    #[test]
    fn test_require_infra_none_production() {
        let _guard = env_lock().lock().unwrap();
        std::env::remove_var("ALLOW_IN_MEMORY_INFRA");
        let result: anyhow::Result<Option<String>> =
            require_infra("test-server", InfraKind::Database, "production", None);
        assert!(result.is_err());
    }

    /// require_infra: None + dev + bypass → Ok(None)
    #[test]
    fn test_require_infra_none_dev_with_bypass() {
        let _guard = env_lock().lock().unwrap();
        std::env::set_var("ALLOW_IN_MEMORY_INFRA", "true");
        let result: anyhow::Result<Option<String>> =
            require_infra("test-server", InfraKind::Database, "development", None);
        std::env::remove_var("ALLOW_IN_MEMORY_INFRA");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }
}

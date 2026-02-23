use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration;

// ============================================================================
// リトライ設定
// ============================================================================

/// リトライ設定。
pub struct RetryConfig {
    /// 最大リトライ回数
    pub max_retries: u32,
    /// 初回遅延（ミリ秒）
    pub initial_delay_ms: u64,
    /// バックオフ倍率
    pub backoff_multiplier: u64,
    /// 最大遅延（ミリ秒）
    pub max_delay_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 1000,
            backoff_multiplier: 2,
            max_delay_ms: 10000,
        }
    }
}

// ============================================================================
// リトライ判定
// ============================================================================

/// コマンド名がリトライ対象かどうかを判定する。
///
/// リトライ対象: go, cargo, npm, flutter, buf
/// 非対象: oapi-codegen, sqlx, gqlgen など
pub fn is_retryable(cmd: &str) -> bool {
    matches!(cmd, "go" | "cargo" | "npm" | "flutter" | "buf")
}

/// コマンド名 + 引数の組み合わせでリトライ対象かどうかを判定する。
///
/// ネットワーク依存のコマンドのみリトライ対象とする:
/// - go mod tidy -> リトライ対象
/// - go run github.com/99designs/gqlgen generate -> 非対象
/// - cargo check -> リトライ対象
/// - cargo xtask codegen -> 非対象
/// - npm install -> リトライ対象
/// - flutter pub get -> リトライ対象
/// - buf generate -> リトライ対象
pub fn is_retryable_command(cmd: &str, args: &[&str]) -> bool {
    if !is_retryable(cmd) {
        return false;
    }

    match cmd {
        "cargo" => {
            // cargo xtask は非対象
            args.first().is_none_or(|&a| a != "xtask")
        }
        "go" => {
            // go run ... gqlgen ... は非対象
            if args.first().is_some_and(|&a| a == "run") {
                !args.iter().any(|a| a.contains("gqlgen"))
            } else {
                true
            }
        }
        _ => true,
    }
}

// ============================================================================
// 遅延計算
// ============================================================================

/// 指数バックオフの遅延時間を計算する（ミリ秒）。
///
/// delay = `initial_delay_ms` * (`backoff_multiplier` ^ attempt)
/// ただし `max_delay_ms` を超えない。
pub fn calculate_delay(config: &RetryConfig, attempt: u32) -> u64 {
    let multiplier = config.backoff_multiplier.saturating_pow(attempt);
    let delay = config.initial_delay_ms.saturating_mul(multiplier);
    delay.min(config.max_delay_ms)
}

// ============================================================================
// リトライ付きコマンド実行
// ============================================================================

/// コマンドをリトライ付きで実行する。
///
/// リトライ対象のコマンドが失敗した場合、指数バックオフで最大 `max_retries` 回リトライする。
/// リトライ非対象のコマンドは1回だけ実行する。
///
/// # Errors
///
/// 指定されたコマンドが全リトライ回数を超えても失敗した場合、またはコマンドが見つからない場合にエラー文字列を返す。
pub fn run_with_retry(
    cmd: &str,
    args: &[&str],
    working_dir: &Path,
    config: &RetryConfig,
) -> Result<(), String> {
    let retryable = is_retryable_command(cmd, args);
    let max_attempts = if retryable { config.max_retries } else { 1 };

    for attempt in 0..max_attempts {
        match Command::new(cmd)
            .args(args)
            .current_dir(working_dir)
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    return Ok(());
                }
                let stderr = String::from_utf8_lossy(&output.stderr);
                if attempt + 1 < max_attempts {
                    let delay = calculate_delay(config, attempt);
                    eprintln!(
                        "コマンド '{} {}' が失敗しました（{}/{} 回目）: {}",
                        cmd,
                        args.join(" "),
                        attempt + 1,
                        max_attempts,
                        stderr.trim()
                    );
                    eprintln!("{delay}ms 後にリトライします...");
                    thread::sleep(Duration::from_millis(delay));
                } else {
                    return Err(format!(
                        "コマンド '{} {}' が {} 回のリトライ後も失敗しました: {}",
                        cmd,
                        args.join(" "),
                        max_attempts,
                        stderr.trim()
                    ));
                }
            }
            Err(e) => {
                if attempt + 1 < max_attempts {
                    let delay = calculate_delay(config, attempt);
                    eprintln!(
                        "コマンド '{} {}' の実行に失敗しました（{}/{} 回目）: {}",
                        cmd,
                        args.join(" "),
                        attempt + 1,
                        max_attempts,
                        e
                    );
                    eprintln!("{delay}ms 後にリトライします...");
                    thread::sleep(Duration::from_millis(delay));
                } else {
                    return Err(format!(
                        "コマンド '{} {}' が {} 回のリトライ後も実行に失敗しました: {}",
                        cmd,
                        args.join(" "),
                        max_attempts,
                        e
                    ));
                }
            }
        }
    }

    Err(format!(
        "コマンド '{} {}' の実行に失敗しました",
        cmd,
        args.join(" ")
    ))
}

// ============================================================================
// テスト
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- is_retryable ---

    #[test]
    fn test_is_retryable_go() {
        assert!(is_retryable("go"));
    }

    #[test]
    fn test_is_retryable_cargo() {
        assert!(is_retryable("cargo"));
    }

    #[test]
    fn test_is_retryable_npm() {
        assert!(is_retryable("npm"));
    }

    #[test]
    fn test_is_retryable_flutter() {
        assert!(is_retryable("flutter"));
    }

    #[test]
    fn test_is_retryable_buf() {
        assert!(is_retryable("buf"));
    }

    #[test]
    fn test_is_not_retryable_oapi_codegen() {
        assert!(!is_retryable("oapi-codegen"));
    }

    #[test]
    fn test_is_not_retryable_sqlx() {
        assert!(!is_retryable("sqlx"));
    }

    #[test]
    fn test_is_not_retryable_gqlgen() {
        assert!(!is_retryable("gqlgen"));
    }

    // --- RetryConfig::default ---

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_delay_ms, 1000);
        assert_eq!(config.backoff_multiplier, 2);
        assert_eq!(config.max_delay_ms, 10000);
    }

    // --- calculate_delay ---

    #[test]
    fn test_calculate_delay_attempt_0() {
        let config = RetryConfig::default();
        // 1000 * 2^0 = 1000
        assert_eq!(calculate_delay(&config, 0), 1000);
    }

    #[test]
    fn test_calculate_delay_attempt_1() {
        let config = RetryConfig::default();
        // 1000 * 2^1 = 2000
        assert_eq!(calculate_delay(&config, 1), 2000);
    }

    #[test]
    fn test_calculate_delay_attempt_2() {
        let config = RetryConfig::default();
        // 1000 * 2^2 = 4000
        assert_eq!(calculate_delay(&config, 2), 4000);
    }

    #[test]
    fn test_calculate_delay_caps_at_max() {
        let config = RetryConfig {
            max_retries: 10,
            initial_delay_ms: 5000,
            backoff_multiplier: 3,
            max_delay_ms: 10000,
        };
        // 5000 * 3^2 = 5000 * 9 = 45000 -> capped at 10000
        assert_eq!(calculate_delay(&config, 2), 10000);
    }

    #[test]
    fn test_calculate_delay_exact_max() {
        let config = RetryConfig {
            max_retries: 5,
            initial_delay_ms: 1000,
            backoff_multiplier: 2,
            max_delay_ms: 8000,
        };
        // 1000 * 2^3 = 8000 (exactly max)
        assert_eq!(calculate_delay(&config, 3), 8000);
    }

    // --- is_retryable_command ---

    #[test]
    fn test_is_retryable_command_cargo_check() {
        assert!(is_retryable_command("cargo", &["check"]));
    }

    #[test]
    fn test_is_not_retryable_command_cargo_xtask() {
        assert!(!is_retryable_command("cargo", &["xtask", "codegen"]));
    }

    #[test]
    fn test_is_retryable_command_go_mod_tidy() {
        assert!(is_retryable_command("go", &["mod", "tidy"]));
    }

    #[test]
    fn test_is_not_retryable_command_go_run_gqlgen() {
        assert!(!is_retryable_command(
            "go",
            &["run", "github.com/99designs/gqlgen", "generate"]
        ));
    }

    #[test]
    fn test_is_retryable_command_npm_install() {
        assert!(is_retryable_command("npm", &["install"]));
    }

    #[test]
    fn test_is_retryable_command_flutter_pub_get() {
        assert!(is_retryable_command("flutter", &["pub", "get"]));
    }

    #[test]
    fn test_is_retryable_command_buf_generate() {
        assert!(is_retryable_command("buf", &["generate"]));
    }

    #[test]
    fn test_is_not_retryable_command_oapi_codegen() {
        assert!(!is_retryable_command(
            "oapi-codegen",
            &["-generate", "types,server"]
        ));
    }

    #[test]
    fn test_is_not_retryable_command_sqlx() {
        assert!(!is_retryable_command("sqlx", &["database", "create"]));
    }

    // --- run_with_retry (non-retryable command, single attempt) ---

    #[test]
    fn test_run_with_retry_non_retryable_fails_once() {
        let config = RetryConfig::default();
        let tmp = std::env::temp_dir();
        // "oapi-codegen" は存在しないためエラーになるが、リトライしない
        let result = run_with_retry("oapi-codegen", &["--version"], &tmp, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_run_with_retry_retryable_command_not_found() {
        let config = RetryConfig {
            max_retries: 1, // 1回でリトライ打ち切り（テスト高速化）
            initial_delay_ms: 1,
            backoff_multiplier: 2,
            max_delay_ms: 10,
        };
        let tmp = std::env::temp_dir();
        // 存在しないリトライ対象コマンド
        let result = run_with_retry("buf", &["generate"], &tmp, &config);
        assert!(result.is_err());
    }
}

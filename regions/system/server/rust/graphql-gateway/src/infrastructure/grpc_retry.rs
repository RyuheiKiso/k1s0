/// gRPC リトライポリシーモジュール（P2-25）
///
/// 一時的なエラー（UNAVAILABLE, DEADLINE_EXCEEDED）に対してリトライを行う。
/// 恒久的なエラー（NOT_FOUND, PERMISSION_DENIED 等）はリトライしない。
///
/// tower::retry::Policy を実装し、ServiceBuilder でチャンネルに適用できる。
use std::future::Future;

use tokio::time::Duration;
use tonic::Status;
use tracing::warn;

/// リトライ対象となる一時的な tonic エラーコードを判定する。
/// UNAVAILABLE（サーバー未起動・過負荷）と DEADLINE_EXCEEDED（タイムアウト）のみリトライ。
fn is_transient(status: &Status) -> bool {
    matches!(
        status.code(),
        tonic::Code::Unavailable | tonic::Code::DeadlineExceeded
    )
}

/// gRPC 呼び出しを指数バックオフ付きでリトライする汎用ヘルパー。
///
/// # 引数
/// * `max_attempts` - 最大試行回数（初回 + リトライ回数）
/// * `op` - リトライ対象の非同期クロージャ。`tonic::Status` を返す。
///
/// # リトライ条件
/// `tonic::Code::Unavailable` または `tonic::Code::DeadlineExceeded` のみリトライ。
/// それ以外のエラーは即座に返却する。
///
/// # バックオフ
/// 初回失敗後 100ms → 200ms → 400ms（最大 3 回リトライ時）の指数バックオフ。
///
/// # 上限
/// `max_attempts` が `MAX_RETRY_LIMIT`（10）を超える場合はクランプされる（H-09 監査対応）。
/// リトライ回数の上限値は `MAX_RETRY_LIMIT`（10）に固定する（無制限リトライによるリソース枯渇防止）。
// H-02 監査対応: doc comment の空行による empty_line_after_doc_comments 警告を修正
const MAX_RETRY_LIMIT: u32 = 10;

pub async fn with_retry<F, Fut, T>(
    operation_name: &str,
    max_attempts: u32,
    op: F,
) -> Result<T, Status>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, Status>>,
{
    // max_attempts が 0 の場合はパニックを回避するため事前にエラーを返す（CRITICAL-CODE-01 監査対応）
    if max_attempts == 0 {
        return Err(Status::internal(format!(
            "{}: max_attempts must be >= 1",
            operation_name
        )));
    }
    // 上限を超える max_attempts はクランプして無制限リトライを防止する（H-09 監査対応）
    let clamped = max_attempts.min(MAX_RETRY_LIMIT);
    if clamped < max_attempts {
        warn!(
            operation = operation_name,
            requested = max_attempts,
            clamped = clamped,
            "max_attempts が上限 {} を超えたためクランプしました",
            MAX_RETRY_LIMIT
        );
    }
    let mut delay_ms = 100u64;
    for attempt in 1..=clamped {
        match op().await {
            Ok(val) => return Ok(val),
            Err(status) if attempt < clamped && is_transient(&status) => {
                warn!(
                    operation = operation_name,
                    attempt = attempt,
                    max_attempts = clamped,
                    error = %status,
                    delay_ms = delay_ms,
                    "一時的な gRPC エラー、リトライします"
                );
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                delay_ms = (delay_ms * 2).min(5000);
            }
            Err(status) => return Err(status),
        }
    }
    // ループ内の全パスが return するため論理的には到達しないが、
    // unreachable! の代わりに安全なエラーを返す（H-09 監査対応）
    Err(Status::internal(format!(
        "{}: retry exhausted after {} attempts",
        operation_name, clamped
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_retry_on_unavailable() {
        // UNAVAILABLE エラーが max_attempts-1 回発生した後に成功するケース
        let call_count = Arc::new(AtomicU32::new(0));
        let count_clone = call_count.clone();

        let result = with_retry("test-op", 3, || {
            let count = count_clone.clone();
            async move {
                let n = count.fetch_add(1, Ordering::SeqCst) + 1;
                if n < 3 {
                    Err(Status::unavailable("server not ready"))
                } else {
                    Ok(42u32)
                }
            }
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(call_count.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_no_retry_on_not_found() {
        // NOT_FOUND は一時的エラーではないのでリトライしない
        let call_count = Arc::new(AtomicU32::new(0));
        let count_clone = call_count.clone();

        let result = with_retry("test-op", 3, || {
            let count = count_clone.clone();
            async move {
                count.fetch_add(1, Ordering::SeqCst);
                Err::<u32, _>(Status::not_found("resource not found"))
            }
        })
        .await;

        assert!(result.is_err());
        // NOT_FOUND はリトライしないため1回のみ呼ばれる
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_max_attempts_exhausted() {
        // 全試行が UNAVAILABLE で失敗するケース
        let call_count = Arc::new(AtomicU32::new(0));
        let count_clone = call_count.clone();

        let result = with_retry("test-op", 2, || {
            let count = count_clone.clone();
            async move {
                count.fetch_add(1, Ordering::SeqCst);
                Err::<u32, _>(Status::unavailable("always unavailable"))
            }
        })
        .await;

        assert!(result.is_err());
        // max_attempts=2 なので2回呼ばれる
        assert_eq!(call_count.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_zero_max_attempts_returns_error() {
        // max_attempts = 0 の場合はパニックせずに即座にエラーを返すことを確認する
        let result = with_retry("test-op", 0, || async {
            Ok::<u32, _>(42)
        })
        .await;

        // クロージャは一切呼ばれず、internal エラーが返されることを検証する
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), tonic::Code::Internal);
    }

    #[tokio::test]
    async fn test_max_attempts_clamped_to_limit() {
        // max_attempts が MAX_RETRY_LIMIT を超える場合、上限にクランプされることを確認する（H-09 監査対応）
        let call_count = Arc::new(AtomicU32::new(0));
        let count_clone = call_count.clone();

        let result = with_retry("test-op", 100, || {
            let count = count_clone.clone();
            async move {
                count.fetch_add(1, Ordering::SeqCst);
                Err::<u32, _>(Status::unavailable("always unavailable"))
            }
        })
        .await;

        assert!(result.is_err());
        // MAX_RETRY_LIMIT=10 にクランプされるため、100 回ではなく 10 回のみ呼ばれる
        assert_eq!(call_count.load(Ordering::SeqCst), super::MAX_RETRY_LIMIT);
    }
}

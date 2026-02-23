use thiserror::Error;

#[derive(Debug, Error)]
pub enum RetryError<E> {
    #[error("すべてのリトライが失敗しました ({attempts} 回): {last_error}")]
    ExhaustedRetries { attempts: u32, last_error: E },
    #[error("サーキットブレーカーがオープン状態です")]
    CircuitBreakerOpen,
    #[error("操作がタイムアウトしました")]
    Timeout,
}

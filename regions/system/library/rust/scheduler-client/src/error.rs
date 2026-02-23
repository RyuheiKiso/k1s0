use thiserror::Error;

#[derive(Debug, Error)]
pub enum SchedulerError {
    #[error("ジョブが見つかりません: {0}")]
    JobNotFound(String),
    #[error("無効なスケジュール: {0}")]
    InvalidSchedule(String),
    #[error("サーバーエラー: {0}")]
    ServerError(String),
    #[error("タイムアウト")]
    Timeout,
}

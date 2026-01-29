//! 進捗コールバック
//!
//! テンプレート展開中の進捗通知を提供する。

/// 進捗通知を受け取るトレイト
pub trait ProgressCallback: Send + Sync {
    /// 総ファイル数を通知
    fn on_total(&self, total: usize);
    /// ファイル処理完了を通知
    fn on_file_done(&self, path: &str);
}

/// 何もしない進捗コールバック
pub struct NoopProgress;

impl ProgressCallback for NoopProgress {
    fn on_total(&self, _total: usize) {}
    fn on_file_done(&self, _path: &str) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noop_progress() {
        let progress = NoopProgress;
        progress.on_total(10);
        progress.on_file_done("test.rs");
    }
}

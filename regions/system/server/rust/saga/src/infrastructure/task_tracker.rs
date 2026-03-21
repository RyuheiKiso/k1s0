/// Saga 実行タスクの生存管理モジュール。
///
/// `StartSagaUseCase` が `tokio::spawn` するバックグラウンドタスクを追跡し、
/// シャットダウン時に実行中タスクが完了するまで待機するグレースフルシャットダウンを実現する。
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use tokio::sync::Notify;

/// SagaTaskTracker は実行中の Saga タスク数を追跡する。
///
/// `spawn` でタスクを登録し、シャットダウン時に `wait_for_completion` で全タスクの終了を待つ。
#[derive(Clone)]
pub struct SagaTaskTracker {
    /// 現在実行中のタスク数（spawn 時に+1、完了時に-1）
    active: Arc<AtomicUsize>,
    /// 全タスク完了時に notify するためのハンドル
    all_done: Arc<Notify>,
}

impl SagaTaskTracker {
    /// 新しい SagaTaskTracker を生成する。
    pub fn new() -> Self {
        Self {
            active: Arc::new(AtomicUsize::new(0)),
            all_done: Arc::new(Notify::new()),
        }
    }

    /// タスクをバックグラウンドで起動し、完了時に追跡カウントを更新する。
    pub fn spawn<F>(&self, fut: F)
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        let active = self.active.clone();
        let all_done = self.all_done.clone();
        // タスク開始前にカウントを増やし、完了後にカウントを減らす
        active.fetch_add(1, Ordering::AcqRel);
        tokio::spawn(async move {
            fut.await;
            // カウントが 0 になったら全待機者に通知する
            if active.fetch_sub(1, Ordering::AcqRel) == 1 {
                all_done.notify_waiters();
            }
        });
    }

    /// 実行中のタスク数を返す。
    pub fn active_count(&self) -> usize {
        self.active.load(Ordering::Acquire)
    }

    /// 全タスクが完了するまで待機する。
    ///
    /// シャットダウンシグナルを受信後に呼び出し、実行中 Saga の完了を待ってから終了する。
    /// タイムアウトは呼び出し元で `tokio::time::timeout` を使って設定すること。
    pub async fn wait_for_completion(&self) {
        // アクティブタスクが残っている間は通知を待つ
        while self.active.load(Ordering::Acquire) > 0 {
            self.all_done.notified().await;
        }
    }
}

impl Default for SagaTaskTracker {
    fn default() -> Self {
        Self::new()
    }
}

use async_trait::async_trait;

use crate::envelope::EventEnvelope;
use crate::error::{HandlerError, SubscribeError};

/// イベントを処理するハンドラ。
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// ハンドラが処理対象とするイベント型を返す。
    fn event_type(&self) -> &str;

    /// イベントを処理する。
    async fn handle(&self, envelope: &EventEnvelope) -> Result<(), HandlerError>;
}

/// イベントを購読する trait。
#[async_trait]
pub trait EventSubscriber: Send + Sync {
    /// ハンドラを登録し、購読を開始する。
    async fn subscribe(
        &self,
        handler: Box<dyn EventHandler>,
    ) -> Result<SubscriptionHandle, SubscribeError>;
}

/// 購読のライフサイクルを管理するハンドル。
pub struct SubscriptionHandle {
    cancel_tx: tokio::sync::oneshot::Sender<()>,
}

impl SubscriptionHandle {
    /// 新しいハンドルを作成する。
    #[must_use]
    pub fn new(cancel_tx: tokio::sync::oneshot::Sender<()>) -> Self {
        Self { cancel_tx }
    }

    /// 購読をキャンセルする。
    pub fn cancel(self) {
        // 送信失敗 = 既に終了済みなので無視
        let _ = self.cancel_tx.send(());
    }
}

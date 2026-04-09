use tokio::sync::broadcast;

/// `FeatureFlagChangeEvent` はフラグ変更イベント。broadcast チャンネル経由で配信される。
#[derive(Debug, Clone)]
pub struct FeatureFlagChangeEvent {
    pub flag_key: String,
    pub change_type: String,
    pub enabled: bool,
    pub description: String,
}

/// `WatchFeatureFlagUseCase` はフラグ変更の publish/subscribe を管理するユースケース。
#[allow(dead_code)]
pub struct WatchFeatureFlagUseCase {
    sender: broadcast::Sender<FeatureFlagChangeEvent>,
}

impl WatchFeatureFlagUseCase {
    /// 新しい `WatchFeatureFlagUseCase` を生成する。
    /// `broadcast::Sender` も返し、更新系ユースケースが変更通知を発行できるようにする。
    #[must_use]
    pub fn new() -> (Self, broadcast::Sender<FeatureFlagChangeEvent>) {
        let (tx, _) = broadcast::channel(256);
        let sender = tx.clone();
        (Self { sender: tx }, sender)
    }

    /// 新しい Receiver を返す。
    #[allow(dead_code)]
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<FeatureFlagChangeEvent> {
        self.sender.subscribe()
    }

    /// 変更イベントをブロードキャストする。
    #[allow(dead_code)]
    pub fn notify(&self, event: FeatureFlagChangeEvent) {
        let _ = self.sender.send(event);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_subscribe_and_notify() {
        let (uc, _tx) = WatchFeatureFlagUseCase::new();
        let mut rx = uc.subscribe();

        uc.notify(FeatureFlagChangeEvent {
            flag_key: "dark-mode".to_string(),
            change_type: "UPDATED".to_string(),
            enabled: true,
            description: "Dark mode".to_string(),
        });

        let event = rx.recv().await.unwrap();
        assert_eq!(event.flag_key, "dark-mode");
        assert_eq!(event.change_type, "UPDATED");
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let (uc, _tx) = WatchFeatureFlagUseCase::new();
        let mut rx1 = uc.subscribe();
        let mut rx2 = uc.subscribe();

        uc.notify(FeatureFlagChangeEvent {
            flag_key: "beta-feature".to_string(),
            change_type: "CREATED".to_string(),
            enabled: false,
            description: "Beta".to_string(),
        });

        let e1 = rx1.recv().await.unwrap();
        let e2 = rx2.recv().await.unwrap();
        assert_eq!(e1.flag_key, e2.flag_key);
    }

    #[tokio::test]
    async fn test_closed_channel() {
        let (tx, _) = broadcast::channel::<FeatureFlagChangeEvent>(4);
        let mut rx = tx.subscribe();
        drop(tx);
        assert!(rx.recv().await.is_err());
    }
}

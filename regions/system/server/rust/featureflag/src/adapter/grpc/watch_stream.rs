use tokio::sync::broadcast;

use crate::usecase::watch_feature_flag::FeatureFlagChangeEvent;

/// `FeatureFlagChangeNotification` は gRPC ストリーミングレスポンスとして返す変更通知。
#[derive(Debug, Clone)]
pub struct FeatureFlagChangeNotification {
    pub flag_key: String,
    pub change_type: String,
    pub enabled: bool,
    pub description: String,
}

/// `WatchFeatureFlagStreamHandler` は `broadcast::Receiver` をラップし、
/// `flag_key` フィルタを適用しながら次の変更通知を非同期で返す。
pub struct WatchFeatureFlagStreamHandler {
    receiver: broadcast::Receiver<FeatureFlagChangeEvent>,
    flag_key_filter: Option<String>,
}

impl std::fmt::Debug for WatchFeatureFlagStreamHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // receiver は Debug 非対応のため省略し、フィルタ情報のみ出力する
        f.debug_struct("WatchFeatureFlagStreamHandler")
            .field("flag_key_filter", &self.flag_key_filter)
            .finish_non_exhaustive()
    }
}

impl WatchFeatureFlagStreamHandler {
    /// 新しいハンドラを生成する。
    ///
    /// - `receiver`: `WatchFeatureFlagUseCase::subscribe()` で得た Receiver。
    /// - `flag_key_filter`: 指定した場合、そのフラグキーの変更通知のみを返す。
    #[must_use]
    pub fn new(
        receiver: broadcast::Receiver<FeatureFlagChangeEvent>,
        flag_key_filter: Option<String>,
    ) -> Self {
        Self {
            receiver,
            flag_key_filter,
        }
    }

    /// 次の変更通知を受信して返す（非同期）。
    pub async fn next(&mut self) -> Option<FeatureFlagChangeNotification> {
        loop {
            match self.receiver.recv().await {
                Ok(event) => {
                    if let Some(ref filter) = self.flag_key_filter {
                        if !filter.is_empty() && event.flag_key != *filter {
                            continue;
                        }
                    }
                    return Some(FeatureFlagChangeNotification {
                        flag_key: event.flag_key,
                        change_type: event.change_type,
                        enabled: event.enabled,
                        description: event.description,
                    });
                }
                Err(broadcast::error::RecvError::Lagged(_)) => {}
                Err(broadcast::error::RecvError::Closed) => {
                    return None;
                }
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::usecase::watch_feature_flag::WatchFeatureFlagUseCase;

    fn make_event(flag_key: &str, change_type: &str) -> FeatureFlagChangeEvent {
        FeatureFlagChangeEvent {
            flag_key: flag_key.to_string(),
            change_type: change_type.to_string(),
            enabled: true,
            description: "test".to_string(),
        }
    }

    #[tokio::test]
    async fn test_next_returns_notification() {
        let (uc, _tx) = WatchFeatureFlagUseCase::new();
        let rx = uc.subscribe();
        let mut handler = WatchFeatureFlagStreamHandler::new(rx, None);

        uc.notify(make_event("dark-mode", "UPDATED"));

        let notif = handler.next().await.unwrap();
        assert_eq!(notif.flag_key, "dark-mode");
        assert_eq!(notif.change_type, "UPDATED");
    }

    #[tokio::test]
    async fn test_flag_key_filter() {
        let (uc, _tx) = WatchFeatureFlagUseCase::new();
        let rx = uc.subscribe();
        let mut handler = WatchFeatureFlagStreamHandler::new(rx, Some("beta".to_string()));

        uc.notify(make_event("dark-mode", "UPDATED"));
        uc.notify(make_event("beta", "CREATED"));

        let notif = handler.next().await.unwrap();
        assert_eq!(notif.flag_key, "beta");
    }

    #[tokio::test]
    async fn test_no_filter_receives_all() {
        let (uc, _tx) = WatchFeatureFlagUseCase::new();
        let rx = uc.subscribe();
        let mut handler = WatchFeatureFlagStreamHandler::new(rx, None);

        uc.notify(make_event("any-flag", "DELETED"));

        let notif = handler.next().await.unwrap();
        assert_eq!(notif.flag_key, "any-flag");
    }

    #[tokio::test]
    async fn test_closed_channel_returns_none() {
        let (tx, rx) = broadcast::channel::<FeatureFlagChangeEvent>(4);
        let mut handler = WatchFeatureFlagStreamHandler::new(rx, None);
        drop(tx);
        assert!(handler.next().await.is_none());
    }

    #[tokio::test]
    async fn test_lagged_receiver_continues() {
        let (tx, rx) = broadcast::channel::<FeatureFlagChangeEvent>(1);
        let mut handler = WatchFeatureFlagStreamHandler::new(rx, None);

        let _ = tx.send(make_event("f1", "UPDATED"));
        let _ = tx.send(make_event("f2", "CREATED"));

        let notif = handler.next().await.unwrap();
        assert_eq!(notif.flag_key, "f2");
    }
}

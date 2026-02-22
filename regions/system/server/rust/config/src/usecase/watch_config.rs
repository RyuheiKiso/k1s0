use tokio::sync::broadcast;

/// ConfigChangeEvent は設定値変更イベントを表す。
/// UpdateConfigUseCase から broadcast channel 経由で送信される。
#[derive(Debug, Clone)]
pub struct ConfigChangeEvent {
    pub namespace: String,
    pub key: String,
    pub value_json: serde_json::Value,
    pub updated_by: String,
    pub version: i32,
}

/// WatchConfigUseCase は broadcast channel を使って設定変更を監視するユースケース。
/// UpdateConfigUseCase の notify() 呼び出しにより、すべての subscribe() 受信者に
/// 変更通知が届く。
pub struct WatchConfigUseCase {
    sender: broadcast::Sender<ConfigChangeEvent>,
}

impl WatchConfigUseCase {
    /// 新しい WatchConfigUseCase を生成し、外部から notify() するための
    /// broadcast::Sender もあわせて返す。
    pub fn new() -> (Self, broadcast::Sender<ConfigChangeEvent>) {
        let (tx, _) = broadcast::channel(256);
        (Self { sender: tx.clone() }, tx)
    }

    /// 変更通知を受け取る Receiver を購読する。
    pub fn subscribe(&self) -> broadcast::Receiver<ConfigChangeEvent> {
        self.sender.subscribe()
    }

    /// 変更イベントをすべての購読者に送信する。
    /// 受信者がいない場合のエラーはベストエフォートとして無視する。
    pub fn notify(&self, event: ConfigChangeEvent) {
        let _ = self.sender.send(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_subscribe_and_notify() {
        let (uc, _tx) = WatchConfigUseCase::new();
        let mut rx = uc.subscribe();

        let event = ConfigChangeEvent {
            namespace: "system.auth.database".to_string(),
            key: "max_connections".to_string(),
            value_json: serde_json::json!(50),
            updated_by: "operator@example.com".to_string(),
            version: 4,
        };

        uc.notify(event.clone());

        let received = rx.recv().await.unwrap();
        assert_eq!(received.namespace, "system.auth.database");
        assert_eq!(received.key, "max_connections");
        assert_eq!(received.value_json, serde_json::json!(50));
        assert_eq!(received.updated_by, "operator@example.com");
        assert_eq!(received.version, 4);
    }

    #[tokio::test]
    async fn test_notify_multiple_subscribers() {
        let (uc, _tx) = WatchConfigUseCase::new();
        let mut rx1 = uc.subscribe();
        let mut rx2 = uc.subscribe();

        let event = ConfigChangeEvent {
            namespace: "system.auth".to_string(),
            key: "timeout".to_string(),
            value_json: serde_json::json!(30),
            updated_by: "admin@example.com".to_string(),
            version: 2,
        };

        uc.notify(event);

        let r1 = rx1.recv().await.unwrap();
        let r2 = rx2.recv().await.unwrap();

        assert_eq!(r1.key, "timeout");
        assert_eq!(r2.key, "timeout");
        assert_eq!(r1.version, 2);
        assert_eq!(r2.version, 2);
    }

    #[tokio::test]
    async fn test_notify_no_receivers_is_ok() {
        // 受信者がいない状態で notify() してもパニックしない
        let (uc, _tx) = WatchConfigUseCase::new();
        let event = ConfigChangeEvent {
            namespace: "system.test".to_string(),
            key: "key".to_string(),
            value_json: serde_json::json!(true),
            updated_by: "user".to_string(),
            version: 1,
        };
        // drop _tx が先に走っても大丈夫であることを確認
        uc.notify(event); // should not panic
    }

    #[tokio::test]
    async fn test_lagged_receiver_recovers() {
        // キャパシティ 1 の channel でラグが発生してもクラッシュしない
        let (tx, _) = broadcast::channel::<ConfigChangeEvent>(1);
        let uc = WatchConfigUseCase { sender: tx };
        let mut rx = uc.subscribe();

        // 2 件送信して受信者をラグさせる
        uc.notify(ConfigChangeEvent {
            namespace: "a".to_string(),
            key: "k".to_string(),
            value_json: serde_json::json!(1),
            updated_by: "u".to_string(),
            version: 1,
        });
        uc.notify(ConfigChangeEvent {
            namespace: "b".to_string(),
            key: "k".to_string(),
            value_json: serde_json::json!(2),
            updated_by: "u".to_string(),
            version: 2,
        });

        // ラグエラーまたは最新値が返ってくる
        let result = rx.recv().await;
        // Lagged または値 — どちらでもパニックしない
        let _ = result;
    }

    #[tokio::test]
    async fn test_sender_returned_by_new_can_also_send() {
        // new() が返す Sender から直接送信しても受信できる
        let (uc, tx) = WatchConfigUseCase::new();
        let mut rx = uc.subscribe();

        let event = ConfigChangeEvent {
            namespace: "system.billing".to_string(),
            key: "invoice_limit".to_string(),
            value_json: serde_json::json!(1000),
            updated_by: "billing-svc".to_string(),
            version: 5,
        };
        let _ = tx.send(event);

        let received = rx.recv().await.unwrap();
        assert_eq!(received.namespace, "system.billing");
        assert_eq!(received.version, 5);
    }

    #[test]
    fn test_config_change_event_is_clone_and_debug() {
        let event = ConfigChangeEvent {
            namespace: "ns".to_string(),
            key: "k".to_string(),
            value_json: serde_json::json!(null),
            updated_by: "u".to_string(),
            version: 0,
        };
        let cloned = event.clone();
        assert_eq!(cloned.namespace, "ns");
        // Debug フォーマットが動作する
        let debug_str = format!("{:?}", cloned);
        assert!(debug_str.contains("ConfigChangeEvent"));
    }
}

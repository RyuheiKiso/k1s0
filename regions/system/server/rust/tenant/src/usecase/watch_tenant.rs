use tokio::sync::broadcast;

/// TenantChangeEvent はテナント変更イベント。broadcast チャンネル経由で配信される。
#[derive(Debug, Clone)]
pub struct TenantChangeEvent {
    pub tenant_id: String,
    pub change_type: String,
    pub tenant_name: String,
    pub tenant_display_name: String,
    pub tenant_status: String,
    pub tenant_plan: String,
}

/// WatchTenantUseCase はテナント変更の publish/subscribe を管理するユースケース。
#[allow(dead_code)]
pub struct WatchTenantUseCase {
    sender: broadcast::Sender<TenantChangeEvent>,
}

impl WatchTenantUseCase {
    /// 新しい WatchTenantUseCase を生成する。
    /// broadcast::Sender も返し、更新系ユースケースが変更通知を発行できるようにする。
    pub fn new() -> (Self, broadcast::Sender<TenantChangeEvent>) {
        let (tx, _) = broadcast::channel(256);
        let sender = tx.clone();
        (Self { sender: tx }, sender)
    }

    /// 新しい Receiver を返す。各ストリーミング接続が個別に subscribe する。
    #[allow(dead_code)]
    pub fn subscribe(&self) -> broadcast::Receiver<TenantChangeEvent> {
        self.sender.subscribe()
    }

    /// 変更イベントをブロードキャストする。
    #[allow(dead_code)]
    pub fn notify(&self, event: TenantChangeEvent) {
        let _ = self.sender.send(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_subscribe_and_notify() {
        let (uc, _tx) = WatchTenantUseCase::new();
        let mut rx = uc.subscribe();

        uc.notify(TenantChangeEvent {
            tenant_id: "t-1".to_string(),
            change_type: "UPDATED".to_string(),
            tenant_name: "acme".to_string(),
            tenant_display_name: "ACME Corp".to_string(),
            tenant_status: "active".to_string(),
            tenant_plan: "professional".to_string(),
        });

        let event = rx.recv().await.unwrap();
        assert_eq!(event.tenant_id, "t-1");
        assert_eq!(event.change_type, "UPDATED");
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let (uc, _tx) = WatchTenantUseCase::new();
        let mut rx1 = uc.subscribe();
        let mut rx2 = uc.subscribe();

        uc.notify(TenantChangeEvent {
            tenant_id: "t-2".to_string(),
            change_type: "CREATED".to_string(),
            tenant_name: "beta".to_string(),
            tenant_display_name: "Beta Inc".to_string(),
            tenant_status: "provisioning".to_string(),
            tenant_plan: "standard".to_string(),
        });

        let e1 = rx1.recv().await.unwrap();
        let e2 = rx2.recv().await.unwrap();
        assert_eq!(e1.tenant_id, e2.tenant_id);
    }

    #[tokio::test]
    async fn test_closed_channel() {
        let (tx, _) = broadcast::channel::<TenantChangeEvent>(4);
        let mut rx = tx.subscribe();
        drop(tx);
        assert!(rx.recv().await.is_err());
    }
}

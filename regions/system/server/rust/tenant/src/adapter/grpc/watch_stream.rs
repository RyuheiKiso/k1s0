use tokio::sync::broadcast;

use crate::usecase::watch_tenant::TenantChangeEvent;

/// `TenantChangeNotification` は gRPC ストリーミングレスポンスとして返す変更通知。
#[derive(Debug, Clone)]
pub struct TenantChangeNotification {
    pub tenant_id: String,
    pub change_type: String,
    pub tenant_name: String,
    pub tenant_display_name: String,
    pub tenant_status: String,
    pub tenant_plan: String,
}

/// `WatchTenantStreamHandler` は `broadcast::Receiver` をラップし、
/// `tenant_id` フィルタを適用しながら次の変更通知を非同期で返す。
pub struct WatchTenantStreamHandler {
    receiver: broadcast::Receiver<TenantChangeEvent>,
    tenant_id_filter: Option<String>,
}

impl std::fmt::Debug for WatchTenantStreamHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WatchTenantStreamHandler")
            .field("tenant_id_filter", &self.tenant_id_filter)
            .finish()
    }
}

impl WatchTenantStreamHandler {
    /// 新しいハンドラを生成する。
    ///
    /// - `receiver`: `WatchTenantUseCase::subscribe()` で得た Receiver。
    /// - `tenant_id_filter`: 指定した場合、そのテナント ID の変更通知のみを返す。
    ///   空文字列または None の場合は全通知を返す。
    #[must_use] 
    pub fn new(
        receiver: broadcast::Receiver<TenantChangeEvent>,
        tenant_id_filter: Option<String>,
    ) -> Self {
        Self {
            receiver,
            tenant_id_filter,
        }
    }

    /// 次の変更通知を受信して返す（非同期）。
    pub async fn next(&mut self) -> Option<TenantChangeNotification> {
        loop {
            match self.receiver.recv().await {
                Ok(event) => {
                    if let Some(ref filter) = self.tenant_id_filter {
                        if !filter.is_empty() && event.tenant_id != *filter {
                            continue;
                        }
                    }
                    return Some(TenantChangeNotification {
                        tenant_id: event.tenant_id,
                        change_type: event.change_type,
                        tenant_name: event.tenant_name,
                        tenant_display_name: event.tenant_display_name,
                        tenant_status: event.tenant_status,
                        tenant_plan: event.tenant_plan,
                    });
                }
                Err(broadcast::error::RecvError::Lagged(_)) => {
                    continue;
                }
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
    use crate::usecase::watch_tenant::WatchTenantUseCase;

    fn make_event(tenant_id: &str, change_type: &str) -> TenantChangeEvent {
        TenantChangeEvent {
            tenant_id: tenant_id.to_string(),
            change_type: change_type.to_string(),
            tenant_name: "test".to_string(),
            tenant_display_name: "Test".to_string(),
            tenant_status: "active".to_string(),
            tenant_plan: "standard".to_string(),
        }
    }

    #[tokio::test]
    async fn test_next_returns_notification() {
        let (uc, _tx) = WatchTenantUseCase::new();
        let rx = uc.subscribe();
        let mut handler = WatchTenantStreamHandler::new(rx, None);

        uc.notify(make_event("t-1", "UPDATED"));

        let notif = handler.next().await.unwrap();
        assert_eq!(notif.tenant_id, "t-1");
        assert_eq!(notif.change_type, "UPDATED");
    }

    #[tokio::test]
    async fn test_tenant_id_filter() {
        let (uc, _tx) = WatchTenantUseCase::new();
        let rx = uc.subscribe();
        let mut handler = WatchTenantStreamHandler::new(rx, Some("t-2".to_string()));

        uc.notify(make_event("t-1", "UPDATED"));
        uc.notify(make_event("t-2", "SUSPENDED"));

        let notif = handler.next().await.unwrap();
        assert_eq!(notif.tenant_id, "t-2");
        assert_eq!(notif.change_type, "SUSPENDED");
    }

    #[tokio::test]
    async fn test_no_filter_receives_all() {
        let (uc, _tx) = WatchTenantUseCase::new();
        let rx = uc.subscribe();
        let mut handler = WatchTenantStreamHandler::new(rx, None);

        uc.notify(make_event("t-1", "CREATED"));

        let notif = handler.next().await.unwrap();
        assert_eq!(notif.tenant_id, "t-1");
    }

    #[tokio::test]
    async fn test_closed_channel_returns_none() {
        let (tx, rx) = broadcast::channel::<TenantChangeEvent>(4);
        let mut handler = WatchTenantStreamHandler::new(rx, None);
        drop(tx);
        assert!(handler.next().await.is_none());
    }

    #[tokio::test]
    async fn test_lagged_receiver_continues() {
        let (tx, rx) = broadcast::channel::<TenantChangeEvent>(1);
        let mut handler = WatchTenantStreamHandler::new(rx, None);

        let _ = tx.send(make_event("t-1", "UPDATED"));
        let _ = tx.send(make_event("t-2", "CREATED"));

        let notif = handler.next().await.unwrap();
        assert_eq!(notif.tenant_id, "t-2");
    }
}

use tokio::sync::broadcast;

use crate::usecase::watch_config::ConfigChangeEvent;

/// WatchConfigRequest はクライアントから config 変更監視リクエストを表す。
#[derive(Debug, Clone)]
pub struct WatchConfigRequest {
    /// 監視対象の namespace プレフィックス。
    /// 空文字列の場合はすべての namespace の変更を受け取る。
    pub namespace: String,
}

/// ConfigChangeNotification は gRPC ストリーミングレスポンスとして返す変更通知。
#[derive(Debug, Clone)]
pub struct ConfigChangeNotification {
    pub namespace: String,
    pub key: String,
    pub value_json: String,
    pub version: i32,
    pub updated_by: String,
}

/// WatchConfigStreamHandler は broadcast::Receiver をラップし、
/// namespace フィルタを適用しながら次の変更通知を非同期で返す。
pub struct WatchConfigStreamHandler {
    receiver: broadcast::Receiver<ConfigChangeEvent>,
    namespace_filter: Option<String>,
}

impl std::fmt::Debug for WatchConfigStreamHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WatchConfigStreamHandler")
            .field("namespace_filter", &self.namespace_filter)
            .finish()
    }
}

impl WatchConfigStreamHandler {
    /// 新しいハンドラを生成する。
    ///
    /// - `receiver`: `WatchConfigUseCase::subscribe()` で得た Receiver。
    /// - `namespace_filter`: `Some(prefix)` を指定すると、そのプレフィックスに
    ///   一致する namespace の変更通知のみを返す。`None` の場合は全通知を返す。
    pub fn new(
        receiver: broadcast::Receiver<ConfigChangeEvent>,
        namespace_filter: Option<String>,
    ) -> Self {
        Self {
            receiver,
            namespace_filter,
        }
    }

    /// 次の変更通知を受信して返す（非同期）。
    ///
    /// - フィルタに一致しないイベントはスキップして次を待つ。
    /// - `broadcast::error::RecvError::Lagged` の場合はスキップして次を待つ。
    /// - チャンネルが閉じられた場合は `None` を返す。
    pub async fn next(&mut self) -> Option<ConfigChangeNotification> {
        loop {
            match self.receiver.recv().await {
                Ok(event) => {
                    if let Some(ref ns_prefix) = self.namespace_filter {
                        if !event.namespace.starts_with(ns_prefix.as_str()) {
                            continue;
                        }
                    }
                    return Some(ConfigChangeNotification {
                        namespace: event.namespace,
                        key: event.key,
                        value_json: event.value_json.to_string(),
                        version: event.version,
                        updated_by: event.updated_by,
                    });
                }
                Err(broadcast::error::RecvError::Lagged(_)) => {
                    // 遅延分のメッセージを読み飛ばして次を待つ
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
mod tests {
    use super::*;
    use crate::usecase::watch_config::WatchConfigUseCase;
    use tokio::sync::broadcast;

    fn make_event(namespace: &str, key: &str, version: i32) -> ConfigChangeEvent {
        ConfigChangeEvent {
            namespace: namespace.to_string(),
            key: key.to_string(),
            value_json: serde_json::json!(version),
            updated_by: "tester".to_string(),
            version,
        }
    }

    #[tokio::test]
    async fn test_next_returns_notification() {
        let (uc, _tx) = WatchConfigUseCase::new();
        let rx = uc.subscribe();
        let mut handler = WatchConfigStreamHandler::new(rx, None);

        uc.notify(make_event("system.auth", "timeout", 1));

        let notif = handler.next().await.unwrap();
        assert_eq!(notif.namespace, "system.auth");
        assert_eq!(notif.key, "timeout");
        assert_eq!(notif.version, 1);
        assert_eq!(notif.updated_by, "tester");
    }

    #[tokio::test]
    async fn test_namespace_filter_passes_matching() {
        let (uc, _tx) = WatchConfigUseCase::new();
        let rx = uc.subscribe();
        let mut handler = WatchConfigStreamHandler::new(rx, Some("system.auth".to_string()));

        uc.notify(make_event("system.auth.database", "max_connections", 3));

        let notif = handler.next().await.unwrap();
        assert_eq!(notif.namespace, "system.auth.database");
        assert_eq!(notif.key, "max_connections");
    }

    #[tokio::test]
    async fn test_namespace_filter_skips_non_matching_and_returns_next() {
        let (uc, _tx) = WatchConfigUseCase::new();
        let rx = uc.subscribe();
        let mut handler = WatchConfigStreamHandler::new(rx, Some("system.auth".to_string()));

        // 最初の通知はフィルタ対象外（スキップされる）
        uc.notify(make_event("business.billing", "rate", 1));
        // 2 番目の通知がフィルタに一致する
        uc.notify(make_event("system.auth.jwt", "issuer", 2));

        let notif = handler.next().await.unwrap();
        assert_eq!(notif.namespace, "system.auth.jwt");
        assert_eq!(notif.version, 2);
    }

    #[tokio::test]
    async fn test_no_filter_receives_all_namespaces() {
        let (uc, _tx) = WatchConfigUseCase::new();
        let rx = uc.subscribe();
        let mut handler = WatchConfigStreamHandler::new(rx, None);

        uc.notify(make_event("business.billing", "rate", 10));

        let notif = handler.next().await.unwrap();
        assert_eq!(notif.namespace, "business.billing");
        assert_eq!(notif.version, 10);
    }

    #[tokio::test]
    async fn test_closed_channel_returns_none() {
        let (tx, rx) = broadcast::channel::<ConfigChangeEvent>(4);
        let mut handler = WatchConfigStreamHandler::new(rx, None);

        // 送信側を drop してチャンネルを閉じる
        drop(tx);

        let result = handler.next().await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_lagged_receiver_continues_after_lag() {
        // キャパシティ 1 にして lag を発生させ、次のメッセージを正常受信できることを確認
        let (tx, rx) = broadcast::channel::<ConfigChangeEvent>(1);
        let mut handler = WatchConfigStreamHandler::new(rx, None);

        // 2 件送信して lag を発生させる
        let _ = tx.send(make_event("system.x", "k", 1));
        let _ = tx.send(make_event("system.y", "k", 2));

        // Lagged エラーをスキップして最新値が取れるはず
        let notif = handler.next().await.unwrap();
        // lag 後に届くのは最新のイベント
        assert_eq!(notif.version, 2);
    }

    #[tokio::test]
    async fn test_value_json_serialized_to_string() {
        let (uc, _tx) = WatchConfigUseCase::new();
        let rx = uc.subscribe();
        let mut handler = WatchConfigStreamHandler::new(rx, None);

        let event = ConfigChangeEvent {
            namespace: "system.test".to_string(),
            key: "config".to_string(),
            value_json: serde_json::json!({"host": "localhost", "port": 5432}),
            updated_by: "admin".to_string(),
            version: 1,
        };
        uc.notify(event);

        let notif = handler.next().await.unwrap();
        // value_json は JSON 文字列として格納される
        assert!(notif.value_json.contains("localhost"));
        assert!(notif.value_json.contains("5432"));
    }
}

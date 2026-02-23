//! WatchConfig gRPC ストリームの統合テスト（外部テストファイル）。
//! watch_stream.rs の inline テストを補完する。

use k1s0_config_server::adapter::grpc::watch_stream::WatchConfigStreamHandler;
use k1s0_config_server::usecase::watch_config::{ConfigChangeEvent, WatchConfigUseCase};

fn make_event(namespace: &str, key: &str, version: i32) -> ConfigChangeEvent {
    ConfigChangeEvent {
        namespace: namespace.to_string(),
        key: key.to_string(),
        value_json: serde_json::json!(version),
        updated_by: "tester".to_string(),
        version,
    }
}

// ---------------------------------------------------------------------------
// 複数クライアント
// ---------------------------------------------------------------------------

/// 複数クライアントが同じイベントをそれぞれ受け取れること。
#[tokio::test]
async fn test_watch_config_multiple_clients_receive_same_event() {
    let (uc, _tx) = WatchConfigUseCase::new();

    // 2 つのクライアントが同じ broadcast channel を購読
    let rx1 = uc.subscribe();
    let rx2 = uc.subscribe();

    let mut handler1 = WatchConfigStreamHandler::new(rx1, None);
    let mut handler2 = WatchConfigStreamHandler::new(rx2, None);

    uc.notify(make_event("system.auth", "timeout", 42));

    let notif1 = handler1.next().await.unwrap();
    let notif2 = handler2.next().await.unwrap();

    assert_eq!(notif1.namespace, "system.auth");
    assert_eq!(notif1.version, 42);
    assert_eq!(notif2.namespace, "system.auth");
    assert_eq!(notif2.version, 42);
}

/// 複数クライアントが独立した namespace フィルタを持てること。
#[tokio::test]
async fn test_watch_config_multiple_clients_independent_namespace_filters() {
    let (uc, _tx) = WatchConfigUseCase::new();

    // クライアント1: system フィルタ / クライアント2: business フィルタ
    let rx1 = uc.subscribe();
    let rx2 = uc.subscribe();

    let mut handler1 = WatchConfigStreamHandler::new(rx1, Some("system".to_string()));
    let mut handler2 = WatchConfigStreamHandler::new(rx2, Some("business".to_string()));

    // system にマッチするイベントと business にマッチするイベントを連続して送る
    uc.notify(make_event("system.config", "key1", 1));
    uc.notify(make_event("business.billing", "rate", 2));

    // handler1 は system.config を受け取る（business は無視）
    let notif1 = handler1.next().await.unwrap();
    assert_eq!(notif1.namespace, "system.config");
    assert_eq!(notif1.version, 1);

    // handler2 は business.billing を受け取る（system は無視）
    let notif2 = handler2.next().await.unwrap();
    assert_eq!(notif2.namespace, "business.billing");
    assert_eq!(notif2.version, 2);
}

// ---------------------------------------------------------------------------
// 購読タイミング
// ---------------------------------------------------------------------------

/// 購読後に発行されたイベントのみ受け取れること（購読前イベントは受け取らない）。
#[tokio::test]
async fn test_watch_config_subscriber_receives_only_post_subscription_events() {
    let (uc, _tx) = WatchConfigUseCase::new();

    // 購読前にイベントを発行（このイベントは受け取れない）
    uc.notify(make_event("system.auth", "old-key", 1));

    // 発行後に購読
    let rx = uc.subscribe();
    let mut handler = WatchConfigStreamHandler::new(rx, None);

    // 購読後に新しいイベントを発行
    uc.notify(make_event("system.auth", "new-key", 2));

    // 購読後に発行されたイベントのみ受け取る
    let notif = handler.next().await.unwrap();
    assert_eq!(notif.key, "new-key");
    assert_eq!(notif.version, 2);
}

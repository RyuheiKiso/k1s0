use k1s0_websocket::{
    CloseFrame, ConnectionState, InMemoryWsClient, WsClient, WsConfig, WsError, WsMessage,
};

// ===========================================================================
// Connection state transition tests
// ===========================================================================

// 新規クライアントの初期状態が Disconnected であることを確認する。
#[tokio::test]
async fn initial_state_is_disconnected() {
    let client = InMemoryWsClient::new();
    assert_eq!(client.state(), ConnectionState::Disconnected);
}

// connect 呼び出し後に状態が Connected に遷移することを確認する。
#[tokio::test]
async fn connect_transitions_to_connected() {
    let mut client = InMemoryWsClient::new();
    client.connect().await.unwrap();
    assert_eq!(client.state(), ConnectionState::Connected);
}

// disconnect 呼び出し後に状態が Disconnected に遷移することを確認する。
#[tokio::test]
async fn disconnect_transitions_to_disconnected() {
    let mut client = InMemoryWsClient::new();
    client.connect().await.unwrap();
    client.disconnect().await.unwrap();
    assert_eq!(client.state(), ConnectionState::Disconnected);
}

// 接続済み状態で再接続すると AlreadyConnected エラーが返されることを確認する。
#[tokio::test]
async fn connect_when_already_connected_returns_error() {
    let mut client = InMemoryWsClient::new();
    client.connect().await.unwrap();
    let result = client.connect().await;
    assert!(matches!(result, Err(WsError::AlreadyConnected)));
    // State should remain connected
    assert_eq!(client.state(), ConnectionState::Connected);
}

// 未接続状態で切断すると NotConnected エラーが返されることを確認する。
#[tokio::test]
async fn disconnect_when_not_connected_returns_error() {
    let mut client = InMemoryWsClient::new();
    let result = client.disconnect().await;
    assert!(matches!(result, Err(WsError::NotConnected)));
    assert_eq!(client.state(), ConnectionState::Disconnected);
}

// 切断後に再接続すると Connected 状態に戻ることを確認する。
#[tokio::test]
async fn reconnect_after_disconnect() {
    let mut client = InMemoryWsClient::new();
    client.connect().await.unwrap();
    client.disconnect().await.unwrap();
    client.connect().await.unwrap();
    assert_eq!(client.state(), ConnectionState::Connected);
}

// ===========================================================================
// Send / Receive tests
// ===========================================================================

// テキストメッセージを送信し送信バッファに正しく記録されることを確認する。
#[tokio::test]
async fn send_text_message() {
    let mut client = InMemoryWsClient::new();
    client.connect().await.unwrap();

    client
        .send(WsMessage::Text("hello".to_string()))
        .await
        .unwrap();

    let sent = client.pop_sent().await.unwrap();
    assert_eq!(sent, WsMessage::Text("hello".to_string()));
}

// バイナリメッセージを送信し送信バッファに正しく記録されることを確認する。
#[tokio::test]
async fn send_binary_message() {
    let mut client = InMemoryWsClient::new();
    client.connect().await.unwrap();

    let data = vec![0x01, 0x02, 0x03, 0xFF];
    client
        .send(WsMessage::Binary(data.clone()))
        .await
        .unwrap();

    let sent = client.pop_sent().await.unwrap();
    assert_eq!(sent, WsMessage::Binary(data));
}

// 受信バッファに追加したテキストメッセージが正しく受信できることを確認する。
#[tokio::test]
async fn receive_text_message() {
    let mut client = InMemoryWsClient::new();
    client.connect().await.unwrap();

    client
        .push_receive(WsMessage::Text("world".to_string()))
        .await;

    let msg = client.receive().await.unwrap();
    assert_eq!(msg, WsMessage::Text("world".to_string()));
}

// 受信バッファに追加したバイナリメッセージが正しく受信できることを確認する。
#[tokio::test]
async fn receive_binary_message() {
    let mut client = InMemoryWsClient::new();
    client.connect().await.unwrap();

    let data = vec![10, 20, 30];
    client.push_receive(WsMessage::Binary(data.clone())).await;

    let msg = client.receive().await.unwrap();
    assert_eq!(msg, WsMessage::Binary(data));
}

// 複数のメッセージを送信した場合に FIFO 順で取得できることを確認する。
#[tokio::test]
async fn send_and_receive_multiple_messages_in_order() {
    let mut client = InMemoryWsClient::new();
    client.connect().await.unwrap();

    // Send multiple messages
    for i in 0..3 {
        client
            .send(WsMessage::Text(format!("msg-{}", i)))
            .await
            .unwrap();
    }

    // Verify FIFO order
    for i in 0..3 {
        let sent = client.pop_sent().await.unwrap();
        assert_eq!(sent, WsMessage::Text(format!("msg-{}", i)));
    }

    // Buffer is now empty
    assert!(client.pop_sent().await.is_none());
}

// 複数のメッセージを受信バッファに追加した場合に FIFO 順で受信できることを確認する。
#[tokio::test]
async fn receive_multiple_messages_in_order() {
    let mut client = InMemoryWsClient::new();
    client.connect().await.unwrap();

    for i in 0..3 {
        client
            .push_receive(WsMessage::Text(format!("recv-{}", i)))
            .await;
    }

    for i in 0..3 {
        let msg = client.receive().await.unwrap();
        assert_eq!(msg, WsMessage::Text(format!("recv-{}", i)));
    }
}

// ===========================================================================
// Buffer behavior tests
// ===========================================================================

// 初期状態では送信バッファが空であることを確認する。
#[tokio::test]
async fn send_buffer_starts_empty() {
    let client = InMemoryWsClient::new();
    assert!(client.pop_sent().await.is_none());
}

// 受信バッファが空のときに receive がエラーを返すことを確認する。
#[tokio::test]
async fn receive_empty_buffer_returns_error() {
    let mut client = InMemoryWsClient::new();
    client.connect().await.unwrap();

    let result = client.receive().await;
    assert!(matches!(result, Err(WsError::ReceiveError(_))));
}

// 1 件のメッセージ受信後はバッファが空になり次の receive がエラーになることを確認する。
#[tokio::test]
async fn buffer_drains_correctly() {
    let mut client = InMemoryWsClient::new();
    client.connect().await.unwrap();

    client
        .push_receive(WsMessage::Text("only-one".to_string()))
        .await;

    // First receive succeeds
    let msg = client.receive().await.unwrap();
    assert_eq!(msg, WsMessage::Text("only-one".to_string()));

    // Second receive fails (buffer empty)
    let result = client.receive().await;
    assert!(matches!(result, Err(WsError::ReceiveError(_))));
}

// ===========================================================================
// Error cases
// ===========================================================================

// 未接続状態で送信すると NotConnected エラーが返されることを確認する。
#[tokio::test]
async fn send_when_disconnected_returns_error() {
    let client = InMemoryWsClient::new();
    let result = client.send(WsMessage::Text("test".to_string())).await;
    assert!(matches!(result, Err(WsError::NotConnected)));
}

// 未接続状態で受信すると NotConnected エラーが返されることを確認する。
#[tokio::test]
async fn receive_when_disconnected_returns_error() {
    let client = InMemoryWsClient::new();
    let result = client.receive().await;
    assert!(matches!(result, Err(WsError::NotConnected)));
}

// 切断後に送信すると NotConnected エラーが返されることを確認する。
#[tokio::test]
async fn send_after_disconnect_returns_error() {
    let mut client = InMemoryWsClient::new();
    client.connect().await.unwrap();
    client.disconnect().await.unwrap();

    let result = client.send(WsMessage::Text("after-dc".to_string())).await;
    assert!(matches!(result, Err(WsError::NotConnected)));
}

// バッファにメッセージがあっても切断後は NotConnected エラーが返されることを確認する。
#[tokio::test]
async fn receive_after_disconnect_returns_error() {
    let mut client = InMemoryWsClient::new();
    client.connect().await.unwrap();
    // Push a message while connected
    client
        .push_receive(WsMessage::Text("pending".to_string()))
        .await;
    client.disconnect().await.unwrap();

    // Even though there's a message in the buffer, receive fails because disconnected
    let result = client.receive().await;
    assert!(matches!(result, Err(WsError::NotConnected)));
}

// ===========================================================================
// Close frame handling tests
// ===========================================================================

// 理由付きクローズフレームを送信し正しいコードと理由が記録されることを確認する。
#[tokio::test]
async fn send_close_frame_with_reason() {
    let mut client = InMemoryWsClient::new();
    client.connect().await.unwrap();

    let close_msg = WsMessage::Close(Some(CloseFrame {
        code: 1000,
        reason: "normal closure".to_string(),
    }));
    client.send(close_msg).await.unwrap();

    let sent = client.pop_sent().await.unwrap();
    if let WsMessage::Close(Some(frame)) = sent {
        assert_eq!(frame.code, 1000);
        assert_eq!(frame.reason, "normal closure");
    } else {
        panic!("expected Close frame with payload");
    }
}

// 理由なしのクローズフレームを送信し None として記録されることを確認する。
#[tokio::test]
async fn send_close_frame_without_reason() {
    let mut client = InMemoryWsClient::new();
    client.connect().await.unwrap();

    client.send(WsMessage::Close(None)).await.unwrap();

    let sent = client.pop_sent().await.unwrap();
    assert_eq!(sent, WsMessage::Close(None));
}

// クローズフレームを受信バッファに追加し正しく受信できることを確認する。
#[tokio::test]
async fn receive_close_frame() {
    let mut client = InMemoryWsClient::new();
    client.connect().await.unwrap();

    client
        .push_receive(WsMessage::Close(Some(CloseFrame {
            code: 1001,
            reason: "going away".to_string(),
        })))
        .await;

    let msg = client.receive().await.unwrap();
    if let WsMessage::Close(Some(frame)) = msg {
        assert_eq!(frame.code, 1001);
        assert_eq!(frame.reason, "going away");
    } else {
        panic!("expected Close frame");
    }
}

// ===========================================================================
// Ping / Pong tests
// ===========================================================================

// Ping メッセージを送信し送信バッファに正しく記録されることを確認する。
#[tokio::test]
async fn send_ping_message() {
    let mut client = InMemoryWsClient::new();
    client.connect().await.unwrap();

    client
        .send(WsMessage::Ping(vec![1, 2, 3]))
        .await
        .unwrap();

    let sent = client.pop_sent().await.unwrap();
    assert_eq!(sent, WsMessage::Ping(vec![1, 2, 3]));
}

// Pong メッセージを受信バッファに追加し正しく受信できることを確認する。
#[tokio::test]
async fn receive_pong_message() {
    let mut client = InMemoryWsClient::new();
    client.connect().await.unwrap();

    client
        .push_receive(WsMessage::Pong(vec![1, 2, 3]))
        .await;

    let msg = client.receive().await.unwrap();
    assert_eq!(msg, WsMessage::Pong(vec![1, 2, 3]));
}

// ===========================================================================
// WsConfig tests
// ===========================================================================

// WsConfig::new でデフォルト値が正しく設定されていることを確認する。
#[test]
fn config_default_values() {
    let config = WsConfig::new("ws://test.example.com");
    assert_eq!(config.url, "ws://test.example.com");
    assert!(config.reconnect);
    assert_eq!(config.max_reconnect_attempts, 5);
    assert_eq!(config.reconnect_delay_ms, 1000);
    assert!(config.ping_interval_ms.is_none());
}

// WsConfig::new で URL が正しく設定され他のフィールドがデフォルト値であることを確認する。
#[test]
fn config_new_with_url() {
    let config = WsConfig::new("wss://example.com/ws");
    assert_eq!(config.url, "wss://example.com/ws");
    // Other fields should be defaults
    assert!(config.reconnect);
    assert_eq!(config.max_reconnect_attempts, 5);
}

// メソッドチェーンで設定した全 WsConfig フィールドが正しく反映されることを確認する。
#[test]
fn config_builder_chain() {
    let config = WsConfig::new("ws://test.local")
        .reconnect(false)
        .max_reconnect_attempts(10)
        .reconnect_delay_ms(2000)
        .ping_interval_ms(15000);

    assert_eq!(config.url, "ws://test.local");
    assert!(!config.reconnect);
    assert_eq!(config.max_reconnect_attempts, 10);
    assert_eq!(config.reconnect_delay_ms, 2000);
    assert_eq!(config.ping_interval_ms, Some(15000));
}

// String 型の URL を WsConfig::new に渡せることを確認する。
#[test]
fn config_accepts_string_type() {
    let url = String::from("ws://from-string");
    let config = WsConfig::new(url);
    assert_eq!(config.url, "ws://from-string");
}

// ===========================================================================
// WsMessage tests
// ===========================================================================

// 同じテキストを持つ WsMessage が等しいことを確認する。
#[test]
fn message_text_equality() {
    let a = WsMessage::Text("same".to_string());
    let b = WsMessage::Text("same".to_string());
    assert_eq!(a, b);
}

// 異なるテキストを持つ WsMessage が不等であることを確認する。
#[test]
fn message_text_inequality() {
    let a = WsMessage::Text("hello".to_string());
    let b = WsMessage::Text("world".to_string());
    assert_ne!(a, b);
}

// 同じバイト列を持つバイナリメッセージが等しいことを確認する。
#[test]
fn message_binary_equality() {
    let a = WsMessage::Binary(vec![1, 2, 3]);
    let b = WsMessage::Binary(vec![1, 2, 3]);
    assert_eq!(a, b);
}

// 異なるバリアントの WsMessage が不等であることを確認する。
#[test]
fn message_different_variants_not_equal() {
    let text = WsMessage::Text("hello".to_string());
    let binary = WsMessage::Binary(b"hello".to_vec());
    assert_ne!(text, binary);
}

// WsMessage のクローンが元と等しいことを確認する。
#[test]
fn message_clone() {
    let original = WsMessage::Text("clone me".to_string());
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

// 同じコードと理由を持つ CloseFrame が等しいことを確認する。
#[test]
fn close_frame_equality() {
    let a = CloseFrame {
        code: 1000,
        reason: "ok".to_string(),
    };
    let b = CloseFrame {
        code: 1000,
        reason: "ok".to_string(),
    };
    assert_eq!(a, b);
}

// コードが異なる CloseFrame が不等であることを確認する。
#[test]
fn close_frame_different_codes() {
    let a = CloseFrame {
        code: 1000,
        reason: "ok".to_string(),
    };
    let b = CloseFrame {
        code: 1001,
        reason: "ok".to_string(),
    };
    assert_ne!(a, b);
}

// ===========================================================================
// ConnectionState tests
// ===========================================================================

// 同じ ConnectionState が等しいことを確認する。
#[test]
fn connection_state_equality() {
    assert_eq!(ConnectionState::Connected, ConnectionState::Connected);
    assert_eq!(
        ConnectionState::Disconnected,
        ConnectionState::Disconnected
    );
}

// 異なる ConnectionState が不等であることを確認する。
#[test]
fn connection_state_inequality() {
    assert_ne!(ConnectionState::Connected, ConnectionState::Disconnected);
    assert_ne!(ConnectionState::Connecting, ConnectionState::Reconnecting);
}

// 全 ConnectionState バリアントが Debug トレイトを実装し空でない文字列を返すことを確認する。
#[test]
fn connection_state_all_variants_debug() {
    // Ensure all variants exist and have Debug
    let states = [
        ConnectionState::Disconnected,
        ConnectionState::Connecting,
        ConnectionState::Connected,
        ConnectionState::Reconnecting,
        ConnectionState::Closing,
    ];
    for state in &states {
        let debug = format!("{:?}", state);
        assert!(!debug.is_empty());
    }
}

// ConnectionState が Copy トレイトを実装していることを確認する。
#[test]
fn connection_state_copy() {
    let state = ConnectionState::Connected;
    let copied = state; // Copy, not move
    assert_eq!(state, copied);
}

// ===========================================================================
// WsError tests
// ===========================================================================

// 各 WsError バリアントの表示メッセージが正しいことを確認する。
#[test]
fn error_display_messages() {
    let err = WsError::ConnectionError("refused".to_string());
    assert_eq!(format!("{}", err), "connection error: refused");

    let err = WsError::SendError("broken pipe".to_string());
    assert_eq!(format!("{}", err), "send error: broken pipe");

    let err = WsError::ReceiveError("timeout".to_string());
    assert_eq!(format!("{}", err), "receive error: timeout");

    let err = WsError::NotConnected;
    assert_eq!(format!("{}", err), "not connected");

    let err = WsError::AlreadyConnected;
    assert_eq!(format!("{}", err), "already connected");

    let err = WsError::Closed("bye".to_string());
    assert_eq!(format!("{}", err), "closed: bye");
}

// WsError の Debug フォーマットにバリアント名が含まれることを確認する。
#[test]
fn error_debug_format() {
    let err = WsError::NotConnected;
    let debug = format!("{:?}", err);
    assert!(debug.contains("NotConnected"));
}

// ===========================================================================
// Default trait tests
// ===========================================================================

// InMemoryWsClient::default が初期状態 Disconnected で生成されることを確認する。
#[test]
fn in_memory_ws_client_default() {
    let client = InMemoryWsClient::default();
    assert_eq!(client.state(), ConnectionState::Disconnected);
}

use std::collections::VecDeque;
use std::sync::Arc;

use async_trait::async_trait;

use crate::error::WsError;
use crate::message::WsMessage;
use crate::state::ConnectionState;

#[async_trait]
#[cfg_attr(feature = "mock", mockall::automock)]
pub trait WsClient: Send + Sync {
    async fn connect(&mut self) -> Result<(), WsError>;
    async fn disconnect(&mut self) -> Result<(), WsError>;
    async fn send(&self, message: WsMessage) -> Result<(), WsError>;
    async fn receive(&self) -> Result<WsMessage, WsError>;
    fn state(&self) -> ConnectionState;
}

pub struct InMemoryWsClient {
    connection_state: ConnectionState,
    send_buffer: Arc<tokio::sync::Mutex<VecDeque<WsMessage>>>,
    receive_buffer: Arc<tokio::sync::Mutex<VecDeque<WsMessage>>>,
}

impl InMemoryWsClient {
    #[must_use] 
    pub fn new() -> Self {
        Self {
            connection_state: ConnectionState::Disconnected,
            send_buffer: Arc::new(tokio::sync::Mutex::new(VecDeque::new())),
            receive_buffer: Arc::new(tokio::sync::Mutex::new(VecDeque::new())),
        }
    }

    pub async fn push_receive(&self, msg: WsMessage) {
        let mut buf = self.receive_buffer.lock().await;
        buf.push_back(msg);
    }

    pub async fn pop_sent(&self) -> Option<WsMessage> {
        let mut buf = self.send_buffer.lock().await;
        buf.pop_front()
    }
}

impl Default for InMemoryWsClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl WsClient for InMemoryWsClient {
    async fn connect(&mut self) -> Result<(), WsError> {
        if self.connection_state == ConnectionState::Connected {
            return Err(WsError::AlreadyConnected);
        }
        self.connection_state = ConnectionState::Connected;
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), WsError> {
        if self.connection_state == ConnectionState::Disconnected {
            return Err(WsError::NotConnected);
        }
        self.connection_state = ConnectionState::Disconnected;
        Ok(())
    }

    async fn send(&self, message: WsMessage) -> Result<(), WsError> {
        if self.connection_state != ConnectionState::Connected {
            return Err(WsError::NotConnected);
        }
        let mut buf = self.send_buffer.lock().await;
        buf.push_back(message);
        Ok(())
    }

    async fn receive(&self) -> Result<WsMessage, WsError> {
        if self.connection_state != ConnectionState::Connected {
            return Err(WsError::NotConnected);
        }
        let mut buf = self.receive_buffer.lock().await;
        buf.pop_front()
            .ok_or_else(|| WsError::ReceiveError("no messages available".to_string()))
    }

    fn state(&self) -> ConnectionState {
        self.connection_state
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // 接続・切断の状態遷移が Disconnected → Connected → Disconnected の順で正しいことを確認する。
    #[tokio::test]
    async fn test_connect_disconnect() {
        let mut client = InMemoryWsClient::new();
        assert_eq!(client.state(), ConnectionState::Disconnected);

        client.connect().await.unwrap();
        assert_eq!(client.state(), ConnectionState::Connected);

        client.disconnect().await.unwrap();
        assert_eq!(client.state(), ConnectionState::Disconnected);
    }

    // 接続済み状態で再接続すると AlreadyConnected エラーが返されることを確認する。
    #[tokio::test]
    async fn test_double_connect() {
        let mut client = InMemoryWsClient::new();
        client.connect().await.unwrap();
        let result = client.connect().await;
        assert!(matches!(result, Err(WsError::AlreadyConnected)));
    }

    // 未接続状態で切断すると NotConnected エラーが返されることを確認する。
    #[tokio::test]
    async fn test_disconnect_when_not_connected() {
        let mut client = InMemoryWsClient::new();
        let result = client.disconnect().await;
        assert!(matches!(result, Err(WsError::NotConnected)));
    }

    // メッセージの受信と送信が正しく機能することを確認する。
    #[tokio::test]
    async fn test_send_receive() {
        let mut client = InMemoryWsClient::new();
        client.connect().await.unwrap();

        client
            .push_receive(WsMessage::Text("hello".to_string()))
            .await;

        let msg = client.receive().await.unwrap();
        assert_eq!(msg, WsMessage::Text("hello".to_string()));

        client
            .send(WsMessage::Text("world".to_string()))
            .await
            .unwrap();
        let sent = client.pop_sent().await.unwrap();
        assert_eq!(sent, WsMessage::Text("world".to_string()));
    }

    // 未接続状態で送信すると NotConnected エラーが返されることを確認する。
    #[tokio::test]
    async fn test_send_when_disconnected() {
        let client = InMemoryWsClient::new();
        let result = client.send(WsMessage::Text("test".to_string())).await;
        assert!(matches!(result, Err(WsError::NotConnected)));
    }

    // 未接続状態で受信すると NotConnected エラーが返されることを確認する。
    #[tokio::test]
    async fn test_receive_when_disconnected() {
        let client = InMemoryWsClient::new();
        let result = client.receive().await;
        assert!(matches!(result, Err(WsError::NotConnected)));
    }

    // 受信バッファが空のときに receive が ReceiveError を返すことを確認する。
    #[tokio::test]
    async fn test_receive_empty_buffer() {
        let mut client = InMemoryWsClient::new();
        client.connect().await.unwrap();
        let result = client.receive().await;
        assert!(matches!(result, Err(WsError::ReceiveError(_))));
    }

    // 各 WsError バリアントがパターンマッチで正しく識別できることを確認する。
    #[test]
    fn test_error_variants() {
        let err = WsError::ConnectionError("refused".to_string());
        assert!(matches!(err, WsError::ConnectionError(_)));

        let err = WsError::NotConnected;
        assert!(matches!(err, WsError::NotConnected));

        let err = WsError::AlreadyConnected;
        assert!(matches!(err, WsError::AlreadyConnected));

        let err = WsError::Closed("bye".to_string());
        assert!(matches!(err, WsError::Closed(_)));
    }

    // InMemoryWsClient::default が初期状態 Disconnected で生成されることを確認する。
    #[test]
    fn test_default() {
        let client = InMemoryWsClient::default();
        assert_eq!(client.state(), ConnectionState::Disconnected);
    }
}

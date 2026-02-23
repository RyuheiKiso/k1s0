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
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connect_disconnect() {
        let mut client = InMemoryWsClient::new();
        assert_eq!(client.state(), ConnectionState::Disconnected);

        client.connect().await.unwrap();
        assert_eq!(client.state(), ConnectionState::Connected);

        client.disconnect().await.unwrap();
        assert_eq!(client.state(), ConnectionState::Disconnected);
    }

    #[tokio::test]
    async fn test_double_connect() {
        let mut client = InMemoryWsClient::new();
        client.connect().await.unwrap();
        let result = client.connect().await;
        assert!(matches!(result, Err(WsError::AlreadyConnected)));
    }

    #[tokio::test]
    async fn test_disconnect_when_not_connected() {
        let mut client = InMemoryWsClient::new();
        let result = client.disconnect().await;
        assert!(matches!(result, Err(WsError::NotConnected)));
    }

    #[tokio::test]
    async fn test_send_receive() {
        let mut client = InMemoryWsClient::new();
        client.connect().await.unwrap();

        client.push_receive(WsMessage::Text("hello".to_string())).await;

        let msg = client.receive().await.unwrap();
        assert_eq!(msg, WsMessage::Text("hello".to_string()));

        client
            .send(WsMessage::Text("world".to_string()))
            .await
            .unwrap();
        let sent = client.pop_sent().await.unwrap();
        assert_eq!(sent, WsMessage::Text("world".to_string()));
    }

    #[tokio::test]
    async fn test_send_when_disconnected() {
        let client = InMemoryWsClient::new();
        let result = client.send(WsMessage::Text("test".to_string())).await;
        assert!(matches!(result, Err(WsError::NotConnected)));
    }

    #[tokio::test]
    async fn test_receive_when_disconnected() {
        let client = InMemoryWsClient::new();
        let result = client.receive().await;
        assert!(matches!(result, Err(WsError::NotConnected)));
    }

    #[tokio::test]
    async fn test_receive_empty_buffer() {
        let mut client = InMemoryWsClient::new();
        client.connect().await.unwrap();
        let result = client.receive().await;
        assert!(matches!(result, Err(WsError::ReceiveError(_))));
    }

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

    #[test]
    fn test_default() {
        let client = InMemoryWsClient::default();
        assert_eq!(client.state(), ConnectionState::Disconnected);
    }
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WsMessage {
    Text(String),
    Binary(Vec<u8>),
    Ping(Vec<u8>),
    Pong(Vec<u8>),
    Close(Option<CloseFrame>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CloseFrame {
    pub code: u16,
    pub reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_message() {
        let msg = WsMessage::Text("hello".to_string());
        assert!(matches!(msg, WsMessage::Text(ref s) if s == "hello"));
    }

    #[test]
    fn test_binary_message() {
        let msg = WsMessage::Binary(vec![1, 2, 3]);
        assert!(matches!(msg, WsMessage::Binary(ref b) if b.len() == 3));
    }

    #[test]
    fn test_ping_pong() {
        let ping = WsMessage::Ping(vec![0]);
        let pong = WsMessage::Pong(vec![0]);
        assert!(matches!(ping, WsMessage::Ping(_)));
        assert!(matches!(pong, WsMessage::Pong(_)));
    }

    #[test]
    fn test_close_frame() {
        let msg = WsMessage::Close(Some(CloseFrame {
            code: 1000,
            reason: "normal".to_string(),
        }));
        if let WsMessage::Close(Some(frame)) = msg {
            assert_eq!(frame.code, 1000);
            assert_eq!(frame.reason, "normal");
        } else {
            panic!("expected close frame");
        }
    }

    #[test]
    fn test_close_without_frame() {
        let msg = WsMessage::Close(None);
        assert!(matches!(msg, WsMessage::Close(None)));
    }
}

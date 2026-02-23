#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Closing,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_states() {
        assert_eq!(ConnectionState::Disconnected, ConnectionState::Disconnected);
        assert_ne!(ConnectionState::Connected, ConnectionState::Disconnected);
    }

    #[test]
    fn test_clone() {
        let state = ConnectionState::Connected;
        let cloned = state;
        assert_eq!(state, cloned);
    }

    #[test]
    fn test_debug() {
        let s = format!("{:?}", ConnectionState::Reconnecting);
        assert_eq!(s, "Reconnecting");
    }
}

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

    // ConnectionState の等値比較と不等値比較が正しく機能することを確認する。
    #[test]
    fn test_states() {
        assert_eq!(ConnectionState::Disconnected, ConnectionState::Disconnected);
        assert_ne!(ConnectionState::Connected, ConnectionState::Disconnected);
    }

    // ConnectionState が Copy トレイトを実装し値のコピーが元と等しいことを確認する。
    #[test]
    fn test_clone() {
        let state = ConnectionState::Connected;
        let cloned = state;
        assert_eq!(state, cloned);
    }

    // ConnectionState の Debug 出力がバリアント名と一致することを確認する。
    #[test]
    fn test_debug() {
        let s = format!("{:?}", ConnectionState::Reconnecting);
        assert_eq!(s, "Reconnecting");
    }
}

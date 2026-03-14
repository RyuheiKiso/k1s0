//! サービス間認証エラー定義。

/// ServiceAuthError はサービス間認証処理で発生するエラーを表す。
#[derive(thiserror::Error, Debug)]
pub enum ServiceAuthError {
    /// トークン取得に失敗した。
    #[error("トークン取得失敗: {0}")]
    TokenAcquisition(String),

    /// トークンの有効期限が切れた。
    #[error("トークンの有効期限が切れました")]
    TokenExpired,

    /// トークンが無効または検証に失敗した。
    #[error("無効なトークン: {0}")]
    InvalidToken(String),

    /// SPIFFE ID の検証に失敗した。
    #[error("SPIFFE ID 検証失敗: {0}")]
    SpiffeValidationFailed(String),

    /// OIDC ディスカバリーに失敗した。
    #[error("OIDC ディスカバリー失敗: {0}")]
    OidcDiscovery(String),

    /// HTTP リクエストに失敗した。
    #[error("HTTP リクエスト失敗: {0}")]
    Http(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    // TokenAcquisition エラーの表示文字列にメッセージが含まれることを確認する。
    #[test]
    fn test_token_acquisition_display() {
        let err = ServiceAuthError::TokenAcquisition("HTTP 401 - Unauthorized".to_string());
        assert!(err.to_string().contains("HTTP 401 - Unauthorized"));
        assert!(err.to_string().contains("トークン取得失敗"));
    }

    // TokenExpired エラーの表示文字列が期待通りであることを確認する。
    #[test]
    fn test_token_expired_display() {
        let err = ServiceAuthError::TokenExpired;
        assert!(err.to_string().contains("有効期限が切れました"));
    }

    // InvalidToken エラーの表示文字列にメッセージが含まれることを確認する。
    #[test]
    fn test_invalid_token_display() {
        let err = ServiceAuthError::InvalidToken("JWT signature mismatch".to_string());
        assert!(err.to_string().contains("JWT signature mismatch"));
        assert!(err.to_string().contains("無効なトークン"));
    }

    // SpiffeValidationFailed エラーの表示文字列にメッセージが含まれることを確認する。
    #[test]
    fn test_spiffe_validation_failed_display() {
        let err = ServiceAuthError::SpiffeValidationFailed("namespace mismatch".to_string());
        assert!(err.to_string().contains("namespace mismatch"));
        assert!(err.to_string().contains("SPIFFE ID 検証失敗"));
    }

    // OidcDiscovery エラーの表示文字列にメッセージが含まれることを確認する。
    #[test]
    fn test_oidc_discovery_display() {
        let err = ServiceAuthError::OidcDiscovery("well-known endpoint unreachable".to_string());
        assert!(err.to_string().contains("well-known endpoint unreachable"));
        assert!(err.to_string().contains("OIDC ディスカバリー失敗"));
    }

    // Http エラーの表示文字列にメッセージが含まれることを確認する。
    #[test]
    fn test_http_error_display() {
        let err = ServiceAuthError::Http("connection refused".to_string());
        assert!(err.to_string().contains("connection refused"));
        assert!(err.to_string().contains("HTTP リクエスト失敗"));
    }

    // すべてのエラーバリアントが std::error::Error トレイトを実装していることを確認する。
    #[test]
    fn test_all_variants_implement_error_trait() {
        fn assert_error<E: std::error::Error>(_: &E) {}

        assert_error(&ServiceAuthError::TokenAcquisition("test".to_string()));
        assert_error(&ServiceAuthError::TokenExpired);
        assert_error(&ServiceAuthError::InvalidToken("test".to_string()));
        assert_error(&ServiceAuthError::SpiffeValidationFailed("test".to_string()));
        assert_error(&ServiceAuthError::OidcDiscovery("test".to_string()));
        assert_error(&ServiceAuthError::Http("test".to_string()));
    }

    // 各エラーバリアントの Debug 出力にバリアント名が含まれることを確認する。
    #[test]
    fn test_all_variants_debug() {
        let variants: Vec<(&str, ServiceAuthError)> = vec![
            ("TokenAcquisition", ServiceAuthError::TokenAcquisition("x".to_string())),
            ("TokenExpired", ServiceAuthError::TokenExpired),
            ("InvalidToken", ServiceAuthError::InvalidToken("x".to_string())),
            ("SpiffeValidationFailed", ServiceAuthError::SpiffeValidationFailed("x".to_string())),
            ("OidcDiscovery", ServiceAuthError::OidcDiscovery("x".to_string())),
            ("Http", ServiceAuthError::Http("x".to_string())),
        ];

        for (name, err) in variants {
            let debug = format!("{:?}", err);
            assert!(debug.contains(name), "Debug for {} should contain variant name", name);
        }
    }
}

/// Severity classification for errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    /// Transient errors that may succeed on retry (timeouts, connection issues).
    Transient,
    /// Permanent errors that will not succeed on retry (not found, invalid input).
    Permanent,
    /// Errors that cannot be classified.
    Unknown,
}

/// Classifies an error's severity based on its message content.
pub fn classify_error(err: &dyn std::error::Error) -> ErrorSeverity {
    let msg = err.to_string().to_lowercase();
    if msg.contains("timeout") || msg.contains("connection") || msg.contains("unavailable") {
        ErrorSeverity::Transient
    } else if msg.contains("not found") || msg.contains("invalid") || msg.contains("unauthorized") {
        ErrorSeverity::Permanent
    } else {
        ErrorSeverity::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt;

    #[derive(Debug)]
    struct TestError(String);

    impl fmt::Display for TestError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl std::error::Error for TestError {}

    // timeout キーワードを含むエラーが Transient と分類されることを確認する。
    #[test]
    fn test_transient_timeout() {
        let err = TestError("connection timeout after 30s".to_string());
        assert_eq!(classify_error(&err), ErrorSeverity::Transient);
    }

    // connection キーワードを含むエラーが Transient と分類されることを確認する。
    #[test]
    fn test_transient_connection() {
        let err = TestError("connection refused".to_string());
        assert_eq!(classify_error(&err), ErrorSeverity::Transient);
    }

    // unavailable キーワードを含むエラーが Transient と分類されることを確認する。
    #[test]
    fn test_transient_unavailable() {
        let err = TestError("service unavailable".to_string());
        assert_eq!(classify_error(&err), ErrorSeverity::Transient);
    }

    // "not found" キーワードを含むエラーが Permanent と分類されることを確認する。
    #[test]
    fn test_permanent_not_found() {
        let err = TestError("resource not found".to_string());
        assert_eq!(classify_error(&err), ErrorSeverity::Permanent);
    }

    // invalid キーワードを含むエラーが Permanent と分類されることを確認する。
    #[test]
    fn test_permanent_invalid() {
        let err = TestError("invalid request parameter".to_string());
        assert_eq!(classify_error(&err), ErrorSeverity::Permanent);
    }

    // unauthorized キーワードを含むエラーが Permanent と分類されることを確認する。
    #[test]
    fn test_permanent_unauthorized() {
        let err = TestError("unauthorized access".to_string());
        assert_eq!(classify_error(&err), ErrorSeverity::Permanent);
    }

    // 分類キーワードを含まないエラーが Unknown と分類されることを確認する。
    #[test]
    fn test_unknown() {
        let err = TestError("something went wrong".to_string());
        assert_eq!(classify_error(&err), ErrorSeverity::Unknown);
    }

    // キーワードの大文字小文字を区別せずに分類されることを確認する。
    #[test]
    fn test_case_insensitive() {
        let err = TestError("CONNECTION TIMEOUT".to_string());
        assert_eq!(classify_error(&err), ErrorSeverity::Transient);
    }
}

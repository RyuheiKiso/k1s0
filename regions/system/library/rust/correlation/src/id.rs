use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// CorrelationId は業務上の相関 ID を表す。
/// 同一業務トランザクションを跨ぐリクエストの追跡に使用する。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CorrelationId(String);

impl CorrelationId {
    /// 新しい CorrelationId を UUID v4 で生成する。
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// 既存の文字列から CorrelationId を生成する。
    pub fn from_string(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// 文字列として取得する。
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for CorrelationId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for CorrelationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// TraceId は OpenTelemetry のトレース ID を表す。
/// 16 バイト（32文字の16進数）の形式で管理する。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TraceId(String);

impl TraceId {
    /// 新しい TraceId を生成する（UUID ベース）。
    pub fn new() -> Self {
        let uuid = Uuid::new_v4();
        // UUID のハイフンを除いた32文字の16進数
        Self(uuid.simple().to_string())
    }

    /// 既存のトレース ID 文字列から TraceId を生成する。
    pub fn from_string(s: impl Into<String>) -> Option<Self> {
        let s = s.into();
        if s.len() == 32 && s.chars().all(|c| c.is_ascii_hexdigit()) {
            Some(Self(s))
        } else {
            None
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for TraceId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for TraceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correlation_id_new() {
        let id = CorrelationId::new();
        assert!(!id.as_str().is_empty());
    }

    #[test]
    fn test_correlation_id_unique() {
        let id1 = CorrelationId::new();
        let id2 = CorrelationId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_correlation_id_from_string() {
        let id = CorrelationId::from_string("req-abc-123");
        assert_eq!(id.as_str(), "req-abc-123");
    }

    #[test]
    fn test_correlation_id_display() {
        let id = CorrelationId::from_string("test-id-001");
        assert_eq!(format!("{}", id), "test-id-001");
    }

    #[test]
    fn test_trace_id_new() {
        let id = TraceId::new();
        assert_eq!(id.as_str().len(), 32);
        assert!(id.as_str().chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_trace_id_unique() {
        let id1 = TraceId::new();
        let id2 = TraceId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_trace_id_from_valid_string() {
        let valid = "4bf92f3577b34da6a3ce929d0e0e4736";
        let id = TraceId::from_string(valid);
        assert!(id.is_some());
        assert_eq!(id.unwrap().as_str(), valid);
    }

    #[test]
    fn test_trace_id_from_invalid_string() {
        assert!(TraceId::from_string("too-short").is_none());
        assert!(TraceId::from_string("not-valid-hexadecimal-characters-!").is_none());
    }

    #[test]
    fn test_serialization_roundtrip() {
        let id = CorrelationId::from_string("corr-001");
        let json = serde_json::to_string(&id).unwrap();
        let deserialized: CorrelationId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, deserialized);
    }
}

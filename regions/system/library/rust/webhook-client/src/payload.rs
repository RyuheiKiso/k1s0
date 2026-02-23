use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub event_type: String,
    pub timestamp: String,
    pub data: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_serialize_deserialize_roundtrip() {
        let payload = WebhookPayload {
            event_type: "user.created".to_string(),
            timestamp: "2026-01-01T00:00:00Z".to_string(),
            data: json!({"user_id": "123"}),
        };
        let json_str = serde_json::to_string(&payload).unwrap();
        let deserialized: WebhookPayload = serde_json::from_str(&json_str).unwrap();
        assert_eq!(deserialized.event_type, "user.created");
        assert_eq!(deserialized.timestamp, "2026-01-01T00:00:00Z");
        assert_eq!(deserialized.data["user_id"], "123");
    }
}

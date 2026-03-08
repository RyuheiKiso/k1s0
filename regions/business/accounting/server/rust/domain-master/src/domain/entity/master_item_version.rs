use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterItemVersion {
    pub id: Uuid,
    pub item_id: Uuid,
    pub version_number: i32,
    pub before_data: Option<serde_json::Value>,
    pub after_data: Option<serde_json::Value>,
    pub changed_by: String,
    pub change_reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_master_item_version_construction() {
        let item_id = Uuid::new_v4();
        let version = MasterItemVersion {
            id: Uuid::new_v4(),
            item_id,
            version_number: 1,
            before_data: None,
            after_data: Some(serde_json::json!({"code": "JPY"})),
            changed_by: "admin".to_string(),
            change_reason: Some("Initial creation".to_string()),
            created_at: Utc::now(),
        };
        assert_eq!(version.item_id, item_id);
        assert_eq!(version.version_number, 1);
        assert!(version.before_data.is_none());
        assert!(version.after_data.is_some());
        assert_eq!(version.changed_by, "admin");
    }

    #[test]
    fn test_master_item_version_serialization_roundtrip() {
        let version = MasterItemVersion {
            id: Uuid::new_v4(),
            item_id: Uuid::new_v4(),
            version_number: 2,
            before_data: Some(serde_json::json!({"name": "old"})),
            after_data: Some(serde_json::json!({"name": "new"})),
            changed_by: "user1".to_string(),
            change_reason: None,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&version).unwrap();
        let deserialized: MasterItemVersion = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.version_number, 2);
        assert_eq!(deserialized.changed_by, "user1");
    }
}

use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PolicyBundle {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub policy_ids: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// テナント ID: CRIT-005 対応。RLS によるテナント分離のために使用する。
    pub tenant_id: String,
}

impl PolicyBundle {
    #[must_use] 
    pub fn new(
        name: String,
        description: Option<String>,
        enabled: bool,
        policy_ids: Vec<Uuid>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            enabled,
            policy_ids,
            created_at: now,
            updated_at: now,
            tenant_id: "system".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// PolicyBundle::new が指定した policy_ids を保持する
    #[test]
    fn new_with_policy_ids() {
        let ids = vec![Uuid::new_v4(), Uuid::new_v4()];
        let bundle = PolicyBundle::new(
            "auth-bundle".to_string(),
            Some("Authentication policies".to_string()),
            true,
            ids.clone(),
        );
        assert_eq!(bundle.name, "auth-bundle");
        assert!(bundle.enabled);
        assert_eq!(bundle.policy_ids.len(), 2);
        assert_eq!(bundle.policy_ids, ids);
    }

    /// description が None の場合も正常に生成される
    #[test]
    fn new_without_description() {
        let bundle = PolicyBundle::new("b".to_string(), None, false, vec![]);
        assert!(bundle.description.is_none());
        assert!(!bundle.enabled);
    }
}

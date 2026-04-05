use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Policy {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub rego_content: String,
    pub package_path: String,
    pub bundle_id: Option<Uuid>,
    pub version: u32,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// テナント ID: CRIT-005 対応。RLS によるテナント分離のために使用する。
    pub tenant_id: String,
}

impl Policy {
    pub fn new(name: String, description: String, rego_content: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            rego_content,
            package_path: String::new(),
            bundle_id: None,
            version: 1,
            enabled: true,
            created_at: now,
            updated_at: now,
            tenant_id: "system".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Policy::new がデフォルト値（version=1, enabled=true）で生成される
    #[test]
    fn new_with_defaults() {
        let p = Policy::new(
            "rbac".to_string(),
            "Role-based access control".to_string(),
            "package rbac\nallow { true }".to_string(),
        );
        assert_eq!(p.name, "rbac");
        assert_eq!(p.version, 1);
        assert!(p.enabled);
        assert!(p.bundle_id.is_none());
        assert!(p.package_path.is_empty());
    }

    /// 複数の Policy は異なる ID を持つ
    #[test]
    fn unique_ids() {
        let p1 = Policy::new("p1".to_string(), "".to_string(), "".to_string());
        let p2 = Policy::new("p2".to_string(), "".to_string(), "".to_string());
        assert_ne!(p1.id, p2.id);
    }
}

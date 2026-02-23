use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum MemberRole {
    Owner,
    Admin,
    Member,
    Viewer,
}

impl MemberRole {
    pub fn as_str(&self) -> &str {
        match self {
            MemberRole::Owner => "owner",
            MemberRole::Admin => "admin",
            MemberRole::Member => "member",
            MemberRole::Viewer => "viewer",
        }
    }
}

#[derive(Debug, Clone)]
pub struct TenantMember {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub joined_at: DateTime<Utc>,
}

impl TenantMember {
    pub fn new(tenant_id: Uuid, user_id: Uuid, role: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            user_id,
            role,
            joined_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tenant_member_new() {
        let tid = Uuid::new_v4();
        let uid = Uuid::new_v4();
        let m = TenantMember::new(tid, uid, MemberRole::Admin.as_str().to_string());
        assert_eq!(m.tenant_id, tid);
        assert_eq!(m.user_id, uid);
        assert_eq!(m.role, "admin");
    }

    #[test]
    fn test_member_role_as_str() {
        assert_eq!(MemberRole::Owner.as_str(), "owner");
        assert_eq!(MemberRole::Admin.as_str(), "admin");
        assert_eq!(MemberRole::Member.as_str(), "member");
        assert_eq!(MemberRole::Viewer.as_str(), "viewer");
    }
}

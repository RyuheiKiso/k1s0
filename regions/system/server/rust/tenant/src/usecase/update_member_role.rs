use std::sync::Arc;

use uuid::Uuid;

use crate::domain::entity::tenant_member::MemberRole;
use crate::domain::entity::TenantMember;
use crate::domain::repository::{MemberRepository, TenantRepository};

/// UpdateMemberRoleInput はメンバーロール更新の入力パラメータ。
pub struct UpdateMemberRoleInput {
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
}

/// UpdateMemberRoleError はメンバーロール更新に関するエラーを表す。
#[derive(Debug, thiserror::Error)]
pub enum UpdateMemberRoleError {
    #[error("member not found")]
    NotFound,

    #[error("invalid role: {0}")]
    InvalidRole(String),

    #[error("tenant not found")]
    TenantNotFound,

    #[error("internal error: {0}")]
    Internal(String),
}

/// UpdateMemberRoleUseCase はメンバーロール更新ユースケース。
pub struct UpdateMemberRoleUseCase {
    member_repo: Arc<dyn MemberRepository>,
    tenant_repo: Arc<dyn TenantRepository>,
}

impl UpdateMemberRoleUseCase {
    pub fn new(
        member_repo: Arc<dyn MemberRepository>,
        tenant_repo: Arc<dyn TenantRepository>,
    ) -> Self {
        Self {
            member_repo,
            tenant_repo,
        }
    }

    pub async fn execute(
        &self,
        input: UpdateMemberRoleInput,
    ) -> Result<TenantMember, UpdateMemberRoleError> {
        // Validate role
        let valid_roles = [
            MemberRole::Owner.as_str(),
            MemberRole::Admin.as_str(),
            MemberRole::Member.as_str(),
            MemberRole::Viewer.as_str(),
        ];
        if !valid_roles.contains(&input.role.as_str()) {
            return Err(UpdateMemberRoleError::InvalidRole(input.role));
        }

        // Verify tenant exists
        self.tenant_repo
            .find_by_id(&input.tenant_id)
            .await
            .map_err(|e| UpdateMemberRoleError::Internal(e.to_string()))?
            .ok_or(UpdateMemberRoleError::TenantNotFound)?;

        // Update role
        let member = self
            .member_repo
            .update_role(&input.tenant_id, &input.user_id, &input.role)
            .await
            .map_err(|e| UpdateMemberRoleError::Internal(e.to_string()))?
            .ok_or(UpdateMemberRoleError::NotFound)?;

        Ok(member)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::{Plan, Tenant, TenantStatus};
    use crate::domain::repository::member_repository::MockMemberRepository;
    use crate::domain::repository::tenant_repository::MockTenantRepository;
    use chrono::Utc;

    fn make_tenant(id: Uuid) -> Tenant {
        Tenant {
            id,
            name: "acme".to_string(),
            display_name: "ACME".to_string(),
            status: TenantStatus::Active,
            plan: Plan::Professional,
            owner_id: None,
            settings: serde_json::json!({}),
            keycloak_realm: None,
            db_schema: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_update_member_role_success() {
        let tenant_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let mut mock_tenant = MockTenantRepository::new();
        let tid = tenant_id;
        mock_tenant
            .expect_find_by_id()
            .withf(move |id| *id == tid)
            .returning(move |_| Ok(Some(make_tenant(tenant_id))));

        let mut mock_member = MockMemberRepository::new();
        let ctid = tenant_id;
        let cuid = user_id;
        mock_member
            .expect_update_role()
            .withf(move |t, u, r| *t == ctid && *u == cuid && r == "admin")
            .returning(move |_, _, _| {
                Ok(Some(TenantMember::new(
                    tenant_id,
                    user_id,
                    "admin".to_string(),
                )))
            });

        let uc = UpdateMemberRoleUseCase::new(Arc::new(mock_member), Arc::new(mock_tenant));
        let result = uc
            .execute(UpdateMemberRoleInput {
                tenant_id,
                user_id,
                role: "admin".to_string(),
            })
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().role, "admin");
    }

    #[tokio::test]
    async fn test_update_member_role_invalid_role() {
        let mock_tenant = MockTenantRepository::new();
        let mock_member = MockMemberRepository::new();

        let uc = UpdateMemberRoleUseCase::new(Arc::new(mock_member), Arc::new(mock_tenant));
        let result = uc
            .execute(UpdateMemberRoleInput {
                tenant_id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                role: "superadmin".to_string(),
            })
            .await;
        assert!(matches!(result, Err(UpdateMemberRoleError::InvalidRole(_))));
    }

    #[tokio::test]
    async fn test_update_member_role_tenant_not_found() {
        let mut mock_tenant = MockTenantRepository::new();
        mock_tenant.expect_find_by_id().returning(|_| Ok(None));

        let mock_member = MockMemberRepository::new();

        let uc = UpdateMemberRoleUseCase::new(Arc::new(mock_member), Arc::new(mock_tenant));
        let result = uc
            .execute(UpdateMemberRoleInput {
                tenant_id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                role: "admin".to_string(),
            })
            .await;
        assert!(matches!(result, Err(UpdateMemberRoleError::TenantNotFound)));
    }

    #[tokio::test]
    async fn test_update_member_role_member_not_found() {
        let tenant_id = Uuid::new_v4();

        let mut mock_tenant = MockTenantRepository::new();
        let tid = tenant_id;
        mock_tenant
            .expect_find_by_id()
            .withf(move |id| *id == tid)
            .returning(move |_| Ok(Some(make_tenant(tenant_id))));

        let mut mock_member = MockMemberRepository::new();
        mock_member
            .expect_update_role()
            .returning(|_, _, _| Ok(None));

        let uc = UpdateMemberRoleUseCase::new(Arc::new(mock_member), Arc::new(mock_tenant));
        let result = uc
            .execute(UpdateMemberRoleInput {
                tenant_id,
                user_id: Uuid::new_v4(),
                role: "member".to_string(),
            })
            .await;
        assert!(matches!(result, Err(UpdateMemberRoleError::NotFound)));
    }
}

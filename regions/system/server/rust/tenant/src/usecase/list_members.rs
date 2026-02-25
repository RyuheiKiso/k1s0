use std::sync::Arc;
use uuid::Uuid;

use crate::domain::entity::TenantMember;
use crate::domain::repository::MemberRepository;

#[derive(Debug, thiserror::Error)]
pub enum ListMembersError {
    #[error("tenant not found: {0}")]
    NotFound(String),
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct ListMembersUseCase {
    member_repo: Arc<dyn MemberRepository>,
}

impl ListMembersUseCase {
    pub fn new(member_repo: Arc<dyn MemberRepository>) -> Self {
        Self { member_repo }
    }

    pub async fn execute(
        &self,
        tenant_id: Uuid,
    ) -> Result<Vec<TenantMember>, ListMembersError> {
        self.member_repo
            .find_by_tenant(&tenant_id)
            .await
            .map_err(|e| ListMembersError::Internal(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::MemberRole;
    use crate::domain::repository::member_repository::MockMemberRepository;

    #[tokio::test]
    async fn test_list_members_success() {
        let mut mock = MockMemberRepository::new();
        let tenant_id = Uuid::new_v4();
        let tid = tenant_id;
        mock.expect_find_by_tenant()
            .withf(move |id| *id == tid)
            .returning(move |_| {
                Ok(vec![
                    TenantMember::new(
                        tenant_id,
                        Uuid::new_v4(),
                        MemberRole::Owner.as_str().to_string(),
                    ),
                    TenantMember::new(
                        tenant_id,
                        Uuid::new_v4(),
                        MemberRole::Member.as_str().to_string(),
                    ),
                ])
            });

        let uc = ListMembersUseCase::new(Arc::new(mock));
        let members = uc.execute(tenant_id).await.unwrap();
        assert_eq!(members.len(), 2);
    }

    #[tokio::test]
    async fn test_list_members_empty() {
        let mut mock = MockMemberRepository::new();
        mock.expect_find_by_tenant().returning(|_| Ok(vec![]));

        let uc = ListMembersUseCase::new(Arc::new(mock));
        let members = uc.execute(Uuid::new_v4()).await.unwrap();
        assert!(members.is_empty());
    }
}

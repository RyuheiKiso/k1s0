use std::sync::Arc;
use uuid::Uuid;

use crate::domain::entity::TenantMember;
use crate::domain::repository::MemberRepository;

#[derive(Debug, thiserror::Error)]
pub enum AddMemberError {
    #[error("already a member")]
    AlreadyMember,
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct AddMemberInput {
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
}

pub struct AddMemberUseCase {
    member_repo: Arc<dyn MemberRepository>,
}

impl AddMemberUseCase {
    pub fn new(member_repo: Arc<dyn MemberRepository>) -> Self {
        Self { member_repo }
    }

    pub async fn execute(&self, input: AddMemberInput) -> Result<TenantMember, AddMemberError> {
        // Check if already a member
        if let Some(_existing) = self
            .member_repo
            .find_member(&input.tenant_id, &input.user_id)
            .await
            .map_err(|e| AddMemberError::Internal(e.to_string()))?
        {
            return Err(AddMemberError::AlreadyMember);
        }

        let member = TenantMember::new(input.tenant_id, input.user_id, input.role);

        self.member_repo
            .add(&member)
            .await
            .map_err(|e| AddMemberError::Internal(e.to_string()))?;

        Ok(member)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::MemberRole;
    use crate::domain::repository::member_repository::MockMemberRepository;

    #[tokio::test]
    async fn test_add_member_success() {
        let mut mock = MockMemberRepository::new();
        mock.expect_find_member().returning(|_, _| Ok(None));
        mock.expect_add().returning(|_| Ok(()));

        let uc = AddMemberUseCase::new(Arc::new(mock));
        let input = AddMemberInput {
            tenant_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            role: MemberRole::Member.as_str().to_string(),
        };

        let member = uc.execute(input).await.unwrap();
        assert_eq!(member.role, "member");
    }

    #[tokio::test]
    async fn test_add_member_already_member() {
        let mut mock = MockMemberRepository::new();
        let tid = Uuid::new_v4();
        let uid = Uuid::new_v4();
        mock.expect_find_member().returning(move |_, _| {
            Ok(Some(TenantMember::new(
                tid,
                uid,
                MemberRole::Member.as_str().to_string(),
            )))
        });

        let uc = AddMemberUseCase::new(Arc::new(mock));
        let input = AddMemberInput {
            tenant_id: tid,
            user_id: uid,
            role: MemberRole::Admin.as_str().to_string(),
        };

        let result = uc.execute(input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AddMemberError::AlreadyMember => {}
            e => panic!("unexpected error: {:?}", e),
        }
    }
}

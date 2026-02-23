use std::sync::Arc;
use uuid::Uuid;

use crate::domain::repository::MemberRepository;

#[derive(Debug, thiserror::Error)]
pub enum RemoveMemberError {
    #[error("member not found")]
    NotFound,
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct RemoveMemberUseCase {
    member_repo: Arc<dyn MemberRepository>,
}

impl RemoveMemberUseCase {
    pub fn new(member_repo: Arc<dyn MemberRepository>) -> Self {
        Self { member_repo }
    }

    pub async fn execute(
        &self,
        tenant_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, RemoveMemberError> {
        let removed = self
            .member_repo
            .remove(&tenant_id, &user_id)
            .await
            .map_err(|e| RemoveMemberError::Internal(e.to_string()))?;

        if !removed {
            return Err(RemoveMemberError::NotFound);
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::member_repository::MockMemberRepository;

    #[tokio::test]
    async fn test_remove_member_success() {
        let mut mock = MockMemberRepository::new();
        mock.expect_remove().returning(|_, _| Ok(true));

        let uc = RemoveMemberUseCase::new(Arc::new(mock));
        let result = uc
            .execute(Uuid::new_v4(), Uuid::new_v4())
            .await
            .unwrap();
        assert!(result);
    }

    #[tokio::test]
    async fn test_remove_member_not_found() {
        let mut mock = MockMemberRepository::new();
        mock.expect_remove().returning(|_, _| Ok(false));

        let uc = RemoveMemberUseCase::new(Arc::new(mock));
        let result = uc.execute(Uuid::new_v4(), Uuid::new_v4()).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            RemoveMemberError::NotFound => {}
            e => panic!("unexpected error: {:?}", e),
        }
    }
}

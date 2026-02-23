use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entity::{ProvisioningJob, TenantMember};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait MemberRepository: Send + Sync {
    async fn find_by_tenant(&self, tenant_id: &Uuid) -> anyhow::Result<Vec<TenantMember>>;
    async fn find_member(
        &self,
        tenant_id: &Uuid,
        user_id: &Uuid,
    ) -> anyhow::Result<Option<TenantMember>>;
    async fn add(&self, member: &TenantMember) -> anyhow::Result<()>;
    async fn remove(&self, tenant_id: &Uuid, user_id: &Uuid) -> anyhow::Result<bool>;
    async fn find_job(&self, job_id: &Uuid) -> anyhow::Result<Option<ProvisioningJob>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::MemberRole;

    #[tokio::test]
    async fn test_mock_find_by_tenant() {
        let mut mock = MockMemberRepository::new();
        mock.expect_find_by_tenant().returning(|_| Ok(vec![]));

        let result = mock.find_by_tenant(&Uuid::new_v4()).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_mock_find_member() {
        let mut mock = MockMemberRepository::new();
        let tid = Uuid::new_v4();
        let uid = Uuid::new_v4();
        let ctid = tid;
        let cuid = uid;
        mock.expect_find_member()
            .withf(move |t, u| *t == ctid && *u == cuid)
            .returning(move |_, _| {
                Ok(Some(TenantMember::new(
                    tid,
                    uid,
                    MemberRole::Member.as_str().to_string(),
                )))
            });

        let result = mock.find_member(&tid, &uid).await.unwrap();
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_mock_remove() {
        let mut mock = MockMemberRepository::new();
        mock.expect_remove().returning(|_, _| Ok(true));

        let result = mock
            .remove(&Uuid::new_v4(), &Uuid::new_v4())
            .await
            .unwrap();
        assert!(result);
    }

    #[tokio::test]
    async fn test_mock_find_job() {
        let mut mock = MockMemberRepository::new();
        mock.expect_find_job().returning(|_| Ok(None));

        let result = mock.find_job(&Uuid::new_v4()).await.unwrap();
        assert!(result.is_none());
    }
}

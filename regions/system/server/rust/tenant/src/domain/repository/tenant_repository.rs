use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entity::Tenant;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait TenantRepository: Send + Sync {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<Tenant>>;
    async fn find_by_name(&self, name: &str) -> anyhow::Result<Option<Tenant>>;
    async fn list(&self, page: i32, page_size: i32) -> anyhow::Result<(Vec<Tenant>, i64)>;
    async fn create(&self, tenant: &Tenant) -> anyhow::Result<()>;
    async fn update(&self, tenant: &Tenant) -> anyhow::Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::TenantStatus;

    #[tokio::test]
    async fn test_mock_find_by_id() {
        let mut mock = MockTenantRepository::new();
        let id = Uuid::new_v4();
        let cid = id;
        mock.expect_find_by_id()
            .withf(move |i| *i == cid)
            .returning(move |_| {
                Ok(Some(Tenant {
                    id,
                    name: "acme".to_string(),
                    display_name: "ACME".to_string(),
                    status: TenantStatus::Active,
                    plan: "professional".to_string(),
                    created_at: chrono::Utc::now(),
                }))
            });

        let result = mock.find_by_id(&id).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "acme");
    }

    #[tokio::test]
    async fn test_mock_find_by_name() {
        let mut mock = MockTenantRepository::new();
        mock.expect_find_by_name()
            .withf(|n| n == "acme")
            .returning(|_| Ok(None));

        let result = mock.find_by_name("acme").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_mock_list() {
        let mut mock = MockTenantRepository::new();
        mock.expect_list().returning(|_, _| Ok((vec![], 0)));

        let (tenants, total) = mock.list(1, 20).await.unwrap();
        assert!(tenants.is_empty());
        assert_eq!(total, 0);
    }
}

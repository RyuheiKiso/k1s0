use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use tokio::sync::RwLock;
use uuid::Uuid;

use k1s0_tenant_server::domain::entity::tenant::{Plan, TenantStatus};
use k1s0_tenant_server::domain::entity::tenant_member::MemberRole;
use k1s0_tenant_server::domain::entity::{
    ProvisioningJob, ProvisioningStatus, Tenant, TenantMember,
};
use k1s0_tenant_server::domain::repository::{MemberRepository, TenantRepository};
use k1s0_tenant_server::infrastructure::kafka_producer::TenantEventPublisher;
use k1s0_tenant_server::infrastructure::keycloak_admin::KeycloakAdmin;
use k1s0_tenant_server::infrastructure::saga_client::SagaClient;
use k1s0_tenant_server::usecase::*;

// ---------------------------------------------------------------------------
// In-memory stub: TenantRepository
// ---------------------------------------------------------------------------
struct StubTenantRepository {
    tenants: RwLock<Vec<Tenant>>,
}

impl StubTenantRepository {
    fn new() -> Self {
        Self {
            tenants: RwLock::new(Vec::new()),
        }
    }

    fn with_tenants(tenants: Vec<Tenant>) -> Self {
        Self {
            tenants: RwLock::new(tenants),
        }
    }
}

#[async_trait]
impl TenantRepository for StubTenantRepository {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<Tenant>> {
        let tenants = self.tenants.read().await;
        Ok(tenants.iter().find(|t| t.id == *id).cloned())
    }

    async fn find_by_name(&self, name: &str) -> anyhow::Result<Option<Tenant>> {
        let tenants = self.tenants.read().await;
        Ok(tenants.iter().find(|t| t.name == name).cloned())
    }

    async fn list(&self, page: i32, page_size: i32) -> anyhow::Result<(Vec<Tenant>, i64)> {
        let tenants = self.tenants.read().await;
        let total = tenants.len() as i64;
        let start = ((page - 1) * page_size) as usize;
        let end = (start + page_size as usize).min(tenants.len());
        if start >= tenants.len() {
            return Ok((vec![], total));
        }
        Ok((tenants[start..end].to_vec(), total))
    }

    async fn create(&self, tenant: &Tenant) -> anyhow::Result<()> {
        self.tenants.write().await.push(tenant.clone());
        Ok(())
    }

    async fn update(&self, tenant: &Tenant) -> anyhow::Result<()> {
        let mut tenants = self.tenants.write().await;
        if let Some(pos) = tenants.iter().position(|t| t.id == tenant.id) {
            tenants[pos] = tenant.clone();
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// In-memory stub: MemberRepository
// ---------------------------------------------------------------------------
struct StubMemberRepository {
    members: RwLock<Vec<TenantMember>>,
    jobs: RwLock<Vec<ProvisioningJob>>,
}

impl StubMemberRepository {
    fn new() -> Self {
        Self {
            members: RwLock::new(Vec::new()),
            jobs: RwLock::new(Vec::new()),
        }
    }

    fn with_members(members: Vec<TenantMember>) -> Self {
        Self {
            members: RwLock::new(members),
            jobs: RwLock::new(Vec::new()),
        }
    }

    fn with_jobs(jobs: Vec<ProvisioningJob>) -> Self {
        Self {
            members: RwLock::new(Vec::new()),
            jobs: RwLock::new(jobs),
        }
    }
}

#[async_trait]
impl MemberRepository for StubMemberRepository {
    async fn find_by_tenant(&self, tenant_id: &Uuid) -> anyhow::Result<Vec<TenantMember>> {
        let members = self.members.read().await;
        Ok(members
            .iter()
            .filter(|m| m.tenant_id == *tenant_id)
            .cloned()
            .collect())
    }

    async fn find_member(
        &self,
        tenant_id: &Uuid,
        user_id: &Uuid,
    ) -> anyhow::Result<Option<TenantMember>> {
        let members = self.members.read().await;
        Ok(members
            .iter()
            .find(|m| m.tenant_id == *tenant_id && m.user_id == *user_id)
            .cloned())
    }

    async fn add(&self, member: &TenantMember) -> anyhow::Result<()> {
        self.members.write().await.push(member.clone());
        Ok(())
    }

    async fn remove(&self, tenant_id: &Uuid, user_id: &Uuid) -> anyhow::Result<bool> {
        let mut members = self.members.write().await;
        let len_before = members.len();
        members.retain(|m| !(m.tenant_id == *tenant_id && m.user_id == *user_id));
        Ok(members.len() < len_before)
    }

    async fn update_role(
        &self,
        tenant_id: &Uuid,
        user_id: &Uuid,
        role: &str,
    ) -> anyhow::Result<Option<TenantMember>> {
        let mut members = self.members.write().await;
        if let Some(member) = members
            .iter_mut()
            .find(|m| m.tenant_id == *tenant_id && m.user_id == *user_id)
        {
            member.role = role.to_string();
            Ok(Some(member.clone()))
        } else {
            Ok(None)
        }
    }

    async fn find_job(&self, job_id: &Uuid) -> anyhow::Result<Option<ProvisioningJob>> {
        let jobs = self.jobs.read().await;
        Ok(jobs.iter().find(|j| j.id == *job_id).cloned())
    }
}

// ---------------------------------------------------------------------------
// Stub: SagaClient
// ---------------------------------------------------------------------------
struct StubSagaClient;

#[async_trait]
impl SagaClient for StubSagaClient {
    async fn start_provisioning_saga(
        &self,
        _tenant_id: &str,
        _tenant_name: &str,
    ) -> anyhow::Result<()> {
        Ok(())
    }
    async fn start_deprovisioning_saga(
        &self,
        _tenant_id: &str,
        _tenant_name: &str,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Stub: TenantEventPublisher
// ---------------------------------------------------------------------------
struct StubEventPublisher {
    events: RwLock<Vec<String>>,
}

impl StubEventPublisher {
    fn new() -> Self {
        Self {
            events: RwLock::new(Vec::new()),
        }
    }
}

#[async_trait]
impl TenantEventPublisher for StubEventPublisher {
    async fn publish_tenant_created(&self, _tenant: &Tenant) -> anyhow::Result<()> {
        self.events.write().await.push("created".to_string());
        Ok(())
    }
    async fn publish_tenant_updated(&self, _tenant: &Tenant) -> anyhow::Result<()> {
        self.events.write().await.push("updated".to_string());
        Ok(())
    }
    async fn publish_tenant_suspended(&self, _tenant: &Tenant) -> anyhow::Result<()> {
        self.events.write().await.push("suspended".to_string());
        Ok(())
    }
    async fn publish_tenant_deleted(&self, _tenant: &Tenant) -> anyhow::Result<()> {
        self.events.write().await.push("deleted".to_string());
        Ok(())
    }
    async fn close(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Stub: KeycloakAdmin
// ---------------------------------------------------------------------------
struct StubKeycloakAdmin;

#[async_trait]
impl KeycloakAdmin for StubKeycloakAdmin {
    async fn create_realm(&self, _realm_name: &str) -> anyhow::Result<()> {
        Ok(())
    }
    async fn delete_realm(&self, _realm_name: &str) -> anyhow::Result<()> {
        Ok(())
    }
    async fn add_user(&self, _realm_name: &str, _user_id: &str) -> anyhow::Result<()> {
        Ok(())
    }
    async fn remove_user(&self, _realm_name: &str, _user_id: &str) -> anyhow::Result<()> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Helper: build a Tenant with a specific status
// ---------------------------------------------------------------------------
fn make_tenant(name: &str, status: TenantStatus, plan: Plan) -> Tenant {
    Tenant {
        id: Uuid::new_v4(),
        name: name.to_string(),
        display_name: name.to_uppercase(),
        status,
        plan,
        owner_id: None,
        settings: serde_json::json!({}),
        keycloak_realm: None,
        db_schema: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

fn make_tenant_with_id(id: Uuid, name: &str, status: TenantStatus, plan: Plan) -> Tenant {
    Tenant {
        id,
        name: name.to_string(),
        display_name: name.to_uppercase(),
        status,
        plan,
        owner_id: None,
        settings: serde_json::json!({}),
        keycloak_realm: None,
        db_schema: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

// ===========================================================================
// CreateTenant
// ===========================================================================
mod create_tenant {
    use super::*;

    #[tokio::test]
    async fn success_creates_provisioning_tenant() {
        let repo = Arc::new(StubTenantRepository::new());
        let uc = CreateTenantUseCase::new(repo.clone())
            .with_saga_client(Arc::new(StubSagaClient))
            .with_event_publisher(Arc::new(StubEventPublisher::new()))
            .with_keycloak_admin(Arc::new(StubKeycloakAdmin));

        let input = CreateTenantInput {
            name: "acme-corp".to_string(),
            display_name: "ACME Corporation".to_string(),
            plan: Plan::Professional,
            owner_id: Some(Uuid::new_v4()),
        };

        let tenant = uc.execute(input).await.unwrap();
        assert_eq!(tenant.name, "acme-corp");
        assert_eq!(tenant.display_name, "ACME Corporation");
        assert_eq!(tenant.status, TenantStatus::Provisioning);
        assert_eq!(tenant.plan, Plan::Professional);

        // Verify persisted
        let stored = repo.find_by_name("acme-corp").await.unwrap();
        assert!(stored.is_some());
    }

    #[tokio::test]
    async fn name_conflict_returns_error() {
        let existing = make_tenant("acme-corp", TenantStatus::Active, Plan::Free);
        let repo = Arc::new(StubTenantRepository::with_tenants(vec![existing]));
        let uc = CreateTenantUseCase::new(repo);

        let input = CreateTenantInput {
            name: "acme-corp".to_string(),
            display_name: "ACME Corporation".to_string(),
            plan: Plan::Professional,
            owner_id: None,
        };

        let result = uc.execute(input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            CreateTenantError::NameConflict(name) => assert_eq!(name, "acme-corp"),
            e => panic!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn without_owner_id() {
        let repo = Arc::new(StubTenantRepository::new());
        let uc = CreateTenantUseCase::new(repo);

        let input = CreateTenantInput {
            name: "no-owner".to_string(),
            display_name: "No Owner".to_string(),
            plan: Plan::Free,
            owner_id: None,
        };

        let tenant = uc.execute(input).await.unwrap();
        assert!(tenant.owner_id.is_none());
    }

    #[tokio::test]
    async fn with_watch_sender_broadcasts_event() {
        let repo = Arc::new(StubTenantRepository::new());
        let (tx, mut rx) = tokio::sync::broadcast::channel(16);
        let uc = CreateTenantUseCase::new(repo).with_watch_sender(tx);

        let input = CreateTenantInput {
            name: "watch-test".to_string(),
            display_name: "Watch Test".to_string(),
            plan: Plan::Starter,
            owner_id: None,
        };

        uc.execute(input).await.unwrap();
        let event = rx.recv().await.unwrap();
        assert_eq!(event.change_type, "CREATED");
        assert_eq!(event.tenant_name, "watch-test");
    }
}

// ===========================================================================
// ActivateTenant
// ===========================================================================
mod activate_tenant {
    use super::*;

    #[tokio::test]
    async fn activates_from_provisioning() {
        let id = Uuid::new_v4();
        let tenant = make_tenant_with_id(id, "t1", TenantStatus::Provisioning, Plan::Free);
        let repo = Arc::new(StubTenantRepository::with_tenants(vec![tenant]));
        let uc = ActivateTenantUseCase::new(repo.clone());

        let result = uc.execute(id).await.unwrap();
        assert_eq!(result.status, TenantStatus::Active);

        // Verify persisted
        let stored = repo.find_by_id(&id).await.unwrap().unwrap();
        assert_eq!(stored.status, TenantStatus::Active);
    }

    #[tokio::test]
    async fn activates_from_suspended() {
        let id = Uuid::new_v4();
        let tenant = make_tenant_with_id(id, "t1", TenantStatus::Suspended, Plan::Free);
        let repo = Arc::new(StubTenantRepository::with_tenants(vec![tenant]));
        let uc = ActivateTenantUseCase::new(repo);

        let result = uc.execute(id).await.unwrap();
        assert_eq!(result.status, TenantStatus::Active);
    }

    #[tokio::test]
    async fn rejects_deleted_tenant() {
        let id = Uuid::new_v4();
        let tenant = make_tenant_with_id(id, "t1", TenantStatus::Deleted, Plan::Free);
        let repo = Arc::new(StubTenantRepository::with_tenants(vec![tenant]));
        let uc = ActivateTenantUseCase::new(repo);

        let result = uc.execute(id).await;
        assert!(matches!(result, Err(ActivateTenantError::InvalidStatus(_))));
    }

    #[tokio::test]
    async fn rejects_already_active() {
        let id = Uuid::new_v4();
        let tenant = make_tenant_with_id(id, "t1", TenantStatus::Active, Plan::Free);
        let repo = Arc::new(StubTenantRepository::with_tenants(vec![tenant]));
        let uc = ActivateTenantUseCase::new(repo);

        let result = uc.execute(id).await;
        assert!(matches!(result, Err(ActivateTenantError::InvalidStatus(_))));
    }

    #[tokio::test]
    async fn not_found() {
        let repo = Arc::new(StubTenantRepository::new());
        let uc = ActivateTenantUseCase::new(repo);

        let result = uc.execute(Uuid::new_v4()).await;
        assert!(matches!(result, Err(ActivateTenantError::NotFound(_))));
    }

    #[tokio::test]
    async fn broadcasts_watch_event() {
        let id = Uuid::new_v4();
        let tenant = make_tenant_with_id(id, "t1", TenantStatus::Suspended, Plan::Free);
        let repo = Arc::new(StubTenantRepository::with_tenants(vec![tenant]));
        let (tx, mut rx) = tokio::sync::broadcast::channel(16);
        let uc = ActivateTenantUseCase::new(repo).with_watch_sender(tx);

        uc.execute(id).await.unwrap();
        let event = rx.recv().await.unwrap();
        assert_eq!(event.change_type, "ACTIVATED");
    }
}

// ===========================================================================
// SuspendTenant
// ===========================================================================
mod suspend_tenant {
    use super::*;

    #[tokio::test]
    async fn success_from_active() {
        let id = Uuid::new_v4();
        let tenant = make_tenant_with_id(id, "t1", TenantStatus::Active, Plan::Professional);
        let repo = Arc::new(StubTenantRepository::with_tenants(vec![tenant]));
        let uc = SuspendTenantUseCase::new(repo.clone());

        let result = uc.execute(id).await.unwrap();
        assert_eq!(result.status, TenantStatus::Suspended);

        let stored = repo.find_by_id(&id).await.unwrap().unwrap();
        assert_eq!(stored.status, TenantStatus::Suspended);
    }

    #[tokio::test]
    async fn rejects_provisioning_tenant() {
        let id = Uuid::new_v4();
        let tenant = make_tenant_with_id(id, "t1", TenantStatus::Provisioning, Plan::Free);
        let repo = Arc::new(StubTenantRepository::with_tenants(vec![tenant]));
        let uc = SuspendTenantUseCase::new(repo);

        let result = uc.execute(id).await;
        assert!(matches!(result, Err(SuspendTenantError::InvalidStatus(_))));
    }

    #[tokio::test]
    async fn rejects_deleted_tenant() {
        let id = Uuid::new_v4();
        let tenant = make_tenant_with_id(id, "t1", TenantStatus::Deleted, Plan::Free);
        let repo = Arc::new(StubTenantRepository::with_tenants(vec![tenant]));
        let uc = SuspendTenantUseCase::new(repo);

        let result = uc.execute(id).await;
        assert!(matches!(result, Err(SuspendTenantError::InvalidStatus(_))));
    }

    #[tokio::test]
    async fn not_found() {
        let repo = Arc::new(StubTenantRepository::new());
        let uc = SuspendTenantUseCase::new(repo);

        let result = uc.execute(Uuid::new_v4()).await;
        assert!(matches!(result, Err(SuspendTenantError::NotFound(_))));
    }

    #[tokio::test]
    async fn broadcasts_watch_event() {
        let id = Uuid::new_v4();
        let tenant = make_tenant_with_id(id, "t1", TenantStatus::Active, Plan::Free);
        let repo = Arc::new(StubTenantRepository::with_tenants(vec![tenant]));
        let (tx, mut rx) = tokio::sync::broadcast::channel(16);
        let uc = SuspendTenantUseCase::new(repo).with_watch_sender(tx);

        uc.execute(id).await.unwrap();
        let event = rx.recv().await.unwrap();
        assert_eq!(event.change_type, "SUSPENDED");
    }
}

// ===========================================================================
// DeleteTenant
// ===========================================================================
mod delete_tenant {
    use super::*;

    #[tokio::test]
    async fn success_marks_deleted() {
        let id = Uuid::new_v4();
        let tenant = make_tenant_with_id(id, "t1", TenantStatus::Active, Plan::Free);
        let repo = Arc::new(StubTenantRepository::with_tenants(vec![tenant]));
        let uc = DeleteTenantUseCase::new(repo.clone())
            .with_saga_client(Arc::new(StubSagaClient))
            .with_event_publisher(Arc::new(StubEventPublisher::new()))
            .with_keycloak_admin(Arc::new(StubKeycloakAdmin));

        let result = uc.execute(id).await.unwrap();
        assert_eq!(result.status, TenantStatus::Deleted);

        let stored = repo.find_by_id(&id).await.unwrap().unwrap();
        assert_eq!(stored.status, TenantStatus::Deleted);
    }

    #[tokio::test]
    async fn delete_from_suspended() {
        let id = Uuid::new_v4();
        let tenant = make_tenant_with_id(id, "t1", TenantStatus::Suspended, Plan::Free);
        let repo = Arc::new(StubTenantRepository::with_tenants(vec![tenant]));
        let uc = DeleteTenantUseCase::new(repo);

        let result = uc.execute(id).await.unwrap();
        assert_eq!(result.status, TenantStatus::Deleted);
    }

    #[tokio::test]
    async fn delete_from_provisioning() {
        let id = Uuid::new_v4();
        let tenant = make_tenant_with_id(id, "t1", TenantStatus::Provisioning, Plan::Free);
        let repo = Arc::new(StubTenantRepository::with_tenants(vec![tenant]));
        let uc = DeleteTenantUseCase::new(repo);

        let result = uc.execute(id).await.unwrap();
        assert_eq!(result.status, TenantStatus::Deleted);
    }

    #[tokio::test]
    async fn rejects_already_deleted() {
        let id = Uuid::new_v4();
        let tenant = make_tenant_with_id(id, "t1", TenantStatus::Deleted, Plan::Free);
        let repo = Arc::new(StubTenantRepository::with_tenants(vec![tenant]));
        let uc = DeleteTenantUseCase::new(repo);

        let result = uc.execute(id).await;
        assert!(matches!(result, Err(DeleteTenantError::InvalidStatus(_))));
    }

    #[tokio::test]
    async fn not_found() {
        let repo = Arc::new(StubTenantRepository::new());
        let uc = DeleteTenantUseCase::new(repo);

        let result = uc.execute(Uuid::new_v4()).await;
        assert!(matches!(result, Err(DeleteTenantError::NotFound(_))));
    }

    #[tokio::test]
    async fn broadcasts_watch_event() {
        let id = Uuid::new_v4();
        let tenant = make_tenant_with_id(id, "t1", TenantStatus::Active, Plan::Free);
        let repo = Arc::new(StubTenantRepository::with_tenants(vec![tenant]));
        let (tx, mut rx) = tokio::sync::broadcast::channel(16);
        let uc = DeleteTenantUseCase::new(repo).with_watch_sender(tx);

        uc.execute(id).await.unwrap();
        let event = rx.recv().await.unwrap();
        assert_eq!(event.change_type, "DELETED");
    }
}

// ===========================================================================
// UpdateTenant
// ===========================================================================
mod update_tenant {
    use super::*;

    #[tokio::test]
    async fn success_updates_display_name_and_plan() {
        let id = Uuid::new_v4();
        let tenant = make_tenant_with_id(id, "t1", TenantStatus::Active, Plan::Free);
        let repo = Arc::new(StubTenantRepository::with_tenants(vec![tenant]));
        let uc = UpdateTenantUseCase::new(repo.clone());

        let input = UpdateTenantInput {
            id,
            display_name: "Updated Name".to_string(),
            plan: Plan::Enterprise,
        };

        let result = uc.execute(input).await.unwrap();
        assert_eq!(result.display_name, "Updated Name");
        assert_eq!(result.plan, Plan::Enterprise);

        let stored = repo.find_by_id(&id).await.unwrap().unwrap();
        assert_eq!(stored.display_name, "Updated Name");
        assert_eq!(stored.plan, Plan::Enterprise);
    }

    #[tokio::test]
    async fn not_found() {
        let repo = Arc::new(StubTenantRepository::new());
        let uc = UpdateTenantUseCase::new(repo);

        let input = UpdateTenantInput {
            id: Uuid::new_v4(),
            display_name: "x".to_string(),
            plan: Plan::Free,
        };

        let result = uc.execute(input).await;
        assert!(matches!(result, Err(UpdateTenantError::NotFound(_))));
    }

    #[tokio::test]
    async fn broadcasts_watch_event() {
        let id = Uuid::new_v4();
        let tenant = make_tenant_with_id(id, "t1", TenantStatus::Active, Plan::Free);
        let repo = Arc::new(StubTenantRepository::with_tenants(vec![tenant]));
        let (tx, mut rx) = tokio::sync::broadcast::channel(16);
        let uc = UpdateTenantUseCase::new(repo).with_watch_sender(tx);

        let input = UpdateTenantInput {
            id,
            display_name: "Updated".to_string(),
            plan: Plan::Starter,
        };

        uc.execute(input).await.unwrap();
        let event = rx.recv().await.unwrap();
        assert_eq!(event.change_type, "UPDATED");
        assert_eq!(event.tenant_plan, "starter");
    }
}

// ===========================================================================
// GetTenant
// ===========================================================================
mod get_tenant {
    use super::*;

    #[tokio::test]
    async fn found() {
        let id = Uuid::new_v4();
        let tenant = make_tenant_with_id(id, "acme", TenantStatus::Active, Plan::Professional);
        let repo = Arc::new(StubTenantRepository::with_tenants(vec![tenant]));
        let uc = GetTenantUseCase::new(repo);

        let result = uc.execute(id).await.unwrap();
        assert_eq!(result.id, id);
        assert_eq!(result.name, "acme");
    }

    #[tokio::test]
    async fn not_found() {
        let repo = Arc::new(StubTenantRepository::new());
        let uc = GetTenantUseCase::new(repo);

        let result = uc.execute(Uuid::new_v4()).await;
        assert!(matches!(result, Err(GetTenantError::NotFound(_))));
    }
}

// ===========================================================================
// ListTenants
// ===========================================================================
mod list_tenants {
    use super::*;

    #[tokio::test]
    async fn returns_all_tenants() {
        let tenants = vec![
            make_tenant("t1", TenantStatus::Active, Plan::Free),
            make_tenant("t2", TenantStatus::Active, Plan::Professional),
            make_tenant("t3", TenantStatus::Suspended, Plan::Enterprise),
        ];
        let repo = Arc::new(StubTenantRepository::with_tenants(tenants));
        let uc = ListTenantsUseCase::new(repo);

        let (list, total) = uc.execute(1, 20).await.unwrap();
        assert_eq!(list.len(), 3);
        assert_eq!(total, 3);
    }

    #[tokio::test]
    async fn empty_list() {
        let repo = Arc::new(StubTenantRepository::new());
        let uc = ListTenantsUseCase::new(repo);

        let (list, total) = uc.execute(1, 20).await.unwrap();
        assert!(list.is_empty());
        assert_eq!(total, 0);
    }

    #[tokio::test]
    async fn pagination_with_page_and_size() {
        let tenants = vec![
            make_tenant("t1", TenantStatus::Active, Plan::Free),
            make_tenant("t2", TenantStatus::Active, Plan::Free),
            make_tenant("t3", TenantStatus::Active, Plan::Free),
        ];
        let repo = Arc::new(StubTenantRepository::with_tenants(tenants));
        let uc = ListTenantsUseCase::new(repo);

        let (list, total) = uc.execute(1, 2).await.unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(total, 3);

        // The usecase normalizes page < 1 to 1 and page_size < 1 to 20
    }

    #[tokio::test]
    async fn invalid_page_defaults_to_first() {
        let tenants = vec![make_tenant("t1", TenantStatus::Active, Plan::Free)];
        let repo = Arc::new(StubTenantRepository::with_tenants(tenants));
        let uc = ListTenantsUseCase::new(repo);

        let (list, _) = uc.execute(0, 20).await.unwrap();
        assert_eq!(list.len(), 1);
    }
}

// ===========================================================================
// GetProvisioningStatus
// ===========================================================================
mod get_provisioning_status {
    use super::*;

    #[tokio::test]
    async fn found_running_job() {
        let job_id = Uuid::new_v4();
        let job = ProvisioningJob {
            id: job_id,
            tenant_id: Uuid::new_v4(),
            status: ProvisioningStatus::Running,
            current_step: Some("creating_realm".to_string()),
            error_message: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let repo = Arc::new(StubMemberRepository::with_jobs(vec![job]));
        let uc = GetProvisioningStatusUseCase::new(repo);

        let result = uc.execute(job_id).await.unwrap();
        assert_eq!(result.id, job_id);
        assert_eq!(result.status, ProvisioningStatus::Running);
        assert_eq!(result.current_step.unwrap(), "creating_realm");
    }

    #[tokio::test]
    async fn found_completed_job() {
        let job_id = Uuid::new_v4();
        let job = ProvisioningJob {
            id: job_id,
            tenant_id: Uuid::new_v4(),
            status: ProvisioningStatus::Completed,
            current_step: None,
            error_message: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let repo = Arc::new(StubMemberRepository::with_jobs(vec![job]));
        let uc = GetProvisioningStatusUseCase::new(repo);

        let result = uc.execute(job_id).await.unwrap();
        assert_eq!(result.status, ProvisioningStatus::Completed);
    }

    #[tokio::test]
    async fn found_failed_job() {
        let job_id = Uuid::new_v4();
        let job = ProvisioningJob {
            id: job_id,
            tenant_id: Uuid::new_v4(),
            status: ProvisioningStatus::Failed,
            current_step: Some("db_migration".to_string()),
            error_message: Some("connection refused".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let repo = Arc::new(StubMemberRepository::with_jobs(vec![job]));
        let uc = GetProvisioningStatusUseCase::new(repo);

        let result = uc.execute(job_id).await.unwrap();
        assert_eq!(result.status, ProvisioningStatus::Failed);
        assert_eq!(result.error_message.unwrap(), "connection refused");
    }

    #[tokio::test]
    async fn not_found() {
        let repo = Arc::new(StubMemberRepository::new());
        let uc = GetProvisioningStatusUseCase::new(repo);

        let result = uc.execute(Uuid::new_v4()).await;
        assert!(matches!(
            result,
            Err(GetProvisioningStatusError::NotFound(_))
        ));
    }
}

// ===========================================================================
// AddMember
// ===========================================================================
mod add_member {
    use super::*;

    #[tokio::test]
    async fn success() {
        let repo = Arc::new(StubMemberRepository::new());
        let uc = AddMemberUseCase::new(repo.clone());
        let tid = Uuid::new_v4();
        let uid = Uuid::new_v4();

        let input = AddMemberInput {
            tenant_id: tid,
            user_id: uid,
            role: MemberRole::Admin.as_str().to_string(),
        };

        let member = uc.execute(input).await.unwrap();
        assert_eq!(member.tenant_id, tid);
        assert_eq!(member.user_id, uid);
        assert_eq!(member.role, "admin");

        // Verify persisted
        let members = repo.find_by_tenant(&tid).await.unwrap();
        assert_eq!(members.len(), 1);
    }

    #[tokio::test]
    async fn already_member_returns_error() {
        let tid = Uuid::new_v4();
        let uid = Uuid::new_v4();
        let existing = TenantMember::new(tid, uid, MemberRole::Member.as_str().to_string());
        let repo = Arc::new(StubMemberRepository::with_members(vec![existing]));
        let uc = AddMemberUseCase::new(repo);

        let input = AddMemberInput {
            tenant_id: tid,
            user_id: uid,
            role: MemberRole::Admin.as_str().to_string(),
        };

        let result = uc.execute(input).await;
        assert!(matches!(result, Err(AddMemberError::AlreadyMember)));
    }

    #[tokio::test]
    async fn add_member_with_owner_role() {
        let repo = Arc::new(StubMemberRepository::new());
        let uc = AddMemberUseCase::new(repo);

        let input = AddMemberInput {
            tenant_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            role: MemberRole::Owner.as_str().to_string(),
        };

        let member = uc.execute(input).await.unwrap();
        assert_eq!(member.role, "owner");
    }

    #[tokio::test]
    async fn add_member_with_viewer_role() {
        let repo = Arc::new(StubMemberRepository::new());
        let uc = AddMemberUseCase::new(repo);

        let input = AddMemberInput {
            tenant_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            role: MemberRole::Viewer.as_str().to_string(),
        };

        let member = uc.execute(input).await.unwrap();
        assert_eq!(member.role, "viewer");
    }
}

// ===========================================================================
// RemoveMember
// ===========================================================================
mod remove_member {
    use super::*;

    #[tokio::test]
    async fn success() {
        let tid = Uuid::new_v4();
        let uid = Uuid::new_v4();
        let member = TenantMember::new(tid, uid, MemberRole::Member.as_str().to_string());
        let repo = Arc::new(StubMemberRepository::with_members(vec![member]));
        let uc = RemoveMemberUseCase::new(repo.clone());

        let result = uc.execute(tid, uid).await.unwrap();
        assert!(result);

        // Verify removed
        let members = repo.find_by_tenant(&tid).await.unwrap();
        assert!(members.is_empty());
    }

    #[tokio::test]
    async fn not_found() {
        let repo = Arc::new(StubMemberRepository::new());
        let uc = RemoveMemberUseCase::new(repo);

        let result = uc.execute(Uuid::new_v4(), Uuid::new_v4()).await;
        assert!(matches!(result, Err(RemoveMemberError::NotFound)));
    }
}

// ===========================================================================
// ListMembers
// ===========================================================================
mod list_members {
    use super::*;

    #[tokio::test]
    async fn returns_members_for_tenant() {
        let tid = Uuid::new_v4();
        let members = vec![
            TenantMember::new(tid, Uuid::new_v4(), MemberRole::Owner.as_str().to_string()),
            TenantMember::new(tid, Uuid::new_v4(), MemberRole::Member.as_str().to_string()),
            TenantMember::new(tid, Uuid::new_v4(), MemberRole::Viewer.as_str().to_string()),
        ];
        let repo = Arc::new(StubMemberRepository::with_members(members));
        let uc = ListMembersUseCase::new(repo);

        let result = uc.execute(tid).await.unwrap();
        assert_eq!(result.len(), 3);
    }

    #[tokio::test]
    async fn empty_when_no_members() {
        let repo = Arc::new(StubMemberRepository::new());
        let uc = ListMembersUseCase::new(repo);

        let result = uc.execute(Uuid::new_v4()).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn filters_by_tenant_id() {
        let tid1 = Uuid::new_v4();
        let tid2 = Uuid::new_v4();
        let members = vec![
            TenantMember::new(tid1, Uuid::new_v4(), MemberRole::Owner.as_str().to_string()),
            TenantMember::new(
                tid2,
                Uuid::new_v4(),
                MemberRole::Member.as_str().to_string(),
            ),
        ];
        let repo = Arc::new(StubMemberRepository::with_members(members));
        let uc = ListMembersUseCase::new(repo);

        let result = uc.execute(tid1).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].tenant_id, tid1);
    }
}

// ===========================================================================
// WatchTenant
// ===========================================================================
mod watch_tenant {
    use super::*;

    #[tokio::test]
    async fn subscribe_and_notify() {
        let (uc, _tx) = WatchTenantUseCase::new();
        let mut rx = uc.subscribe();

        uc.notify(TenantChangeEvent {
            tenant_id: "t-1".to_string(),
            change_type: "UPDATED".to_string(),
            tenant_name: "acme".to_string(),
            tenant_display_name: "ACME Corp".to_string(),
            tenant_status: "active".to_string(),
            tenant_plan: "professional".to_string(),
        });

        let event = rx.recv().await.unwrap();
        assert_eq!(event.tenant_id, "t-1");
        assert_eq!(event.change_type, "UPDATED");
    }

    #[tokio::test]
    async fn multiple_subscribers_receive_same_event() {
        let (uc, _tx) = WatchTenantUseCase::new();
        let mut rx1 = uc.subscribe();
        let mut rx2 = uc.subscribe();

        uc.notify(TenantChangeEvent {
            tenant_id: "t-2".to_string(),
            change_type: "CREATED".to_string(),
            tenant_name: "beta".to_string(),
            tenant_display_name: "Beta".to_string(),
            tenant_status: "provisioning".to_string(),
            tenant_plan: "free".to_string(),
        });

        let e1 = rx1.recv().await.unwrap();
        let e2 = rx2.recv().await.unwrap();
        assert_eq!(e1.tenant_id, e2.tenant_id);
    }

    #[tokio::test]
    async fn closed_channel_returns_error() {
        let (tx, _) = tokio::sync::broadcast::channel::<TenantChangeEvent>(4);
        let mut rx = tx.subscribe();
        drop(tx);
        assert!(rx.recv().await.is_err());
    }
}

// ===========================================================================
// Cross-cutting: full lifecycle integration
// ===========================================================================
mod lifecycle {
    use super::*;

    #[tokio::test]
    async fn full_tenant_lifecycle() {
        let repo = Arc::new(StubTenantRepository::new());
        let event_pub = Arc::new(StubEventPublisher::new());

        // 1. Create
        let create_uc =
            CreateTenantUseCase::new(repo.clone()).with_event_publisher(event_pub.clone());
        let tenant = create_uc
            .execute(CreateTenantInput {
                name: "lifecycle-test".to_string(),
                display_name: "Lifecycle Test".to_string(),
                plan: Plan::Professional,
                owner_id: None,
            })
            .await
            .unwrap();
        let id = tenant.id;
        assert_eq!(tenant.status, TenantStatus::Provisioning);

        // 2. Activate
        let activate_uc = ActivateTenantUseCase::new(repo.clone());
        let tenant = activate_uc.execute(id).await.unwrap();
        assert_eq!(tenant.status, TenantStatus::Active);

        // 3. Update
        let update_uc =
            UpdateTenantUseCase::new(repo.clone()).with_event_publisher(event_pub.clone());
        let tenant = update_uc
            .execute(UpdateTenantInput {
                id,
                display_name: "Lifecycle Updated".to_string(),
                plan: Plan::Enterprise,
            })
            .await
            .unwrap();
        assert_eq!(tenant.display_name, "Lifecycle Updated");
        assert_eq!(tenant.plan, Plan::Enterprise);

        // 4. Suspend
        let suspend_uc =
            SuspendTenantUseCase::new(repo.clone()).with_event_publisher(event_pub.clone());
        let tenant = suspend_uc.execute(id).await.unwrap();
        assert_eq!(tenant.status, TenantStatus::Suspended);

        // 5. Re-activate
        let tenant = activate_uc.execute(id).await.unwrap();
        assert_eq!(tenant.status, TenantStatus::Active);

        // 6. Delete
        let delete_uc =
            DeleteTenantUseCase::new(repo.clone()).with_event_publisher(event_pub.clone());
        let tenant = delete_uc.execute(id).await.unwrap();
        assert_eq!(tenant.status, TenantStatus::Deleted);

        // 7. Verify delete prevents further activation
        let result = activate_uc.execute(id).await;
        assert!(matches!(result, Err(ActivateTenantError::InvalidStatus(_))));
    }

    #[tokio::test]
    async fn member_lifecycle() {
        let member_repo = Arc::new(StubMemberRepository::new());
        let tid = Uuid::new_v4();
        let uid = Uuid::new_v4();

        // Add member
        let add_uc = AddMemberUseCase::new(member_repo.clone());
        let member = add_uc
            .execute(AddMemberInput {
                tenant_id: tid,
                user_id: uid,
                role: MemberRole::Member.as_str().to_string(),
            })
            .await
            .unwrap();
        assert_eq!(member.role, "member");

        // List members
        let list_uc = ListMembersUseCase::new(member_repo.clone());
        let members = list_uc.execute(tid).await.unwrap();
        assert_eq!(members.len(), 1);

        // Cannot add same member again
        let result = add_uc
            .execute(AddMemberInput {
                tenant_id: tid,
                user_id: uid,
                role: MemberRole::Admin.as_str().to_string(),
            })
            .await;
        assert!(matches!(result, Err(AddMemberError::AlreadyMember)));

        // Remove member
        let remove_uc = RemoveMemberUseCase::new(member_repo.clone());
        let removed = remove_uc.execute(tid, uid).await.unwrap();
        assert!(removed);

        // Verify removed
        let members = list_uc.execute(tid).await.unwrap();
        assert!(members.is_empty());

        // Can re-add after removal
        let member = add_uc
            .execute(AddMemberInput {
                tenant_id: tid,
                user_id: uid,
                role: MemberRole::Admin.as_str().to_string(),
            })
            .await
            .unwrap();
        assert_eq!(member.role, "admin");
    }
}

use std::sync::Arc;

use crate::domain::entity::{ProvisioningJob, Tenant, TenantMember};
use crate::usecase::{
    AddMemberInput, AddMemberUseCase, CreateTenantInput, CreateTenantUseCase,
    GetProvisioningStatusUseCase, GetTenantUseCase, ListTenantsUseCase, RemoveMemberUseCase,
};

// --- gRPC Request/Response Types (proto equivalent) ---

#[derive(Debug, Clone)]
pub struct PbTimestamp {
    pub seconds: i64,
    pub nanos: i32,
}

#[derive(Debug, Clone)]
pub struct PbTenant {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub status: String,
    pub plan: String,
    pub realm_name: String,
    pub owner_id: String,
    pub created_at: Option<PbTimestamp>,
    pub updated_at: Option<PbTimestamp>,
}

#[derive(Debug, Clone)]
pub struct PbTenantMember {
    pub id: String,
    pub tenant_id: String,
    pub user_id: String,
    pub role: String,
    pub joined_at: Option<PbTimestamp>,
}

#[derive(Debug, Clone)]
pub struct PbProvisioningJob {
    pub id: String,
    pub tenant_id: String,
    pub status: String,
    pub current_step: String,
    pub error_message: String,
    pub created_at: Option<PbTimestamp>,
    pub updated_at: Option<PbTimestamp>,
}

#[derive(Debug, Clone)]
pub struct CreateTenantRequest {
    pub name: String,
    pub display_name: String,
    pub plan: String,
    pub owner_id: String,
}

#[derive(Debug, Clone)]
pub struct CreateTenantResponse {
    pub tenant: Option<PbTenant>,
}

#[derive(Debug, Clone)]
pub struct GetTenantRequest {
    pub tenant_id: String,
}

#[derive(Debug, Clone)]
pub struct GetTenantResponse {
    pub tenant: Option<PbTenant>,
}

#[derive(Debug, Clone)]
pub struct ListTenantsRequest {
    pub page: i32,
    pub page_size: i32,
}

#[derive(Debug, Clone)]
pub struct ListTenantsResponse {
    pub tenants: Vec<PbTenant>,
    pub total_count: i64,
}

#[derive(Debug, Clone)]
pub struct AddMemberRequest {
    pub tenant_id: String,
    pub user_id: String,
    pub role: String,
}

#[derive(Debug, Clone)]
pub struct AddMemberResponse {
    pub member: Option<PbTenantMember>,
}

#[derive(Debug, Clone)]
pub struct RemoveMemberRequest {
    pub tenant_id: String,
    pub user_id: String,
}

#[derive(Debug, Clone)]
pub struct RemoveMemberResponse {
    pub success: bool,
}

#[derive(Debug, Clone)]
pub struct GetProvisioningStatusRequest {
    pub job_id: String,
}

#[derive(Debug, Clone)]
pub struct GetProvisioningStatusResponse {
    pub job: Option<PbProvisioningJob>,
}

// --- gRPC Error ---

#[derive(Debug, thiserror::Error)]
pub enum GrpcError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("invalid argument: {0}")]
    InvalidArgument(String),
    #[error("already exists: {0}")]
    AlreadyExists(String),
    #[error("internal: {0}")]
    Internal(String),
}

// --- TenantGrpcService ---

pub struct TenantGrpcService {
    create_tenant_uc: Arc<CreateTenantUseCase>,
    get_tenant_uc: Arc<GetTenantUseCase>,
    list_tenants_uc: Arc<ListTenantsUseCase>,
    add_member_uc: Arc<AddMemberUseCase>,
    remove_member_uc: Arc<RemoveMemberUseCase>,
    get_provisioning_status_uc: Arc<GetProvisioningStatusUseCase>,
}

impl TenantGrpcService {
    pub fn new(
        create_tenant_uc: Arc<CreateTenantUseCase>,
        get_tenant_uc: Arc<GetTenantUseCase>,
        list_tenants_uc: Arc<ListTenantsUseCase>,
        add_member_uc: Arc<AddMemberUseCase>,
        remove_member_uc: Arc<RemoveMemberUseCase>,
        get_provisioning_status_uc: Arc<GetProvisioningStatusUseCase>,
    ) -> Self {
        Self {
            create_tenant_uc,
            get_tenant_uc,
            list_tenants_uc,
            add_member_uc,
            remove_member_uc,
            get_provisioning_status_uc,
        }
    }

    pub async fn create_tenant(
        &self,
        req: CreateTenantRequest,
    ) -> Result<CreateTenantResponse, GrpcError> {
        validate_plan(&req.plan)?;
        let owner_id = if req.owner_id.is_empty() {
            None
        } else {
            Some(
                uuid::Uuid::parse_str(&req.owner_id)
                    .map_err(|e| GrpcError::InvalidArgument(format!("invalid owner_id: {}", e)))?,
            )
        };

        let input = CreateTenantInput {
            name: req.name,
            display_name: req.display_name,
            plan: req.plan,
            owner_id,
        };

        match self.create_tenant_uc.execute(input).await {
            Ok(tenant) => Ok(CreateTenantResponse {
                tenant: Some(domain_tenant_to_pb(&tenant)),
            }),
            Err(crate::usecase::CreateTenantError::NameConflict(name)) => {
                Err(GrpcError::AlreadyExists(format!("tenant name: {}", name)))
            }
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    pub async fn get_tenant(
        &self,
        req: GetTenantRequest,
    ) -> Result<GetTenantResponse, GrpcError> {
        let tenant_id = uuid::Uuid::parse_str(&req.tenant_id)
            .map_err(|e| GrpcError::InvalidArgument(format!("invalid tenant_id: {}", e)))?;

        match self.get_tenant_uc.execute(tenant_id).await {
            Ok(tenant) => Ok(GetTenantResponse {
                tenant: Some(domain_tenant_to_pb(&tenant)),
            }),
            Err(crate::usecase::GetTenantError::NotFound(id)) => {
                Err(GrpcError::NotFound(format!("tenant not found: {}", id)))
            }
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    pub async fn list_tenants(
        &self,
        req: ListTenantsRequest,
    ) -> Result<ListTenantsResponse, GrpcError> {
        let page = if req.page < 1 { 1 } else { req.page };
        let page_size = if req.page_size < 1 { 20 } else { req.page_size };

        match self.list_tenants_uc.execute(page, page_size).await {
            Ok((tenants, total_count)) => {
                let pb_tenants: Vec<PbTenant> =
                    tenants.iter().map(domain_tenant_to_pb).collect();
                Ok(ListTenantsResponse {
                    tenants: pb_tenants,
                    total_count,
                })
            }
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    pub async fn add_member(
        &self,
        req: AddMemberRequest,
    ) -> Result<AddMemberResponse, GrpcError> {
        let tenant_id = uuid::Uuid::parse_str(&req.tenant_id)
            .map_err(|e| GrpcError::InvalidArgument(format!("invalid tenant_id: {}", e)))?;
        let user_id = uuid::Uuid::parse_str(&req.user_id)
            .map_err(|e| GrpcError::InvalidArgument(format!("invalid user_id: {}", e)))?;
        validate_role(&req.role)?;

        let input = AddMemberInput {
            tenant_id,
            user_id,
            role: req.role,
        };

        match self.add_member_uc.execute(input).await {
            Ok(member) => Ok(AddMemberResponse {
                member: Some(domain_member_to_pb(&member)),
            }),
            Err(crate::usecase::AddMemberError::AlreadyMember) => {
                Err(GrpcError::AlreadyExists("member already exists".to_string()))
            }
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    pub async fn remove_member(
        &self,
        req: RemoveMemberRequest,
    ) -> Result<RemoveMemberResponse, GrpcError> {
        let tenant_id = uuid::Uuid::parse_str(&req.tenant_id)
            .map_err(|e| GrpcError::InvalidArgument(format!("invalid tenant_id: {}", e)))?;
        let user_id = uuid::Uuid::parse_str(&req.user_id)
            .map_err(|e| GrpcError::InvalidArgument(format!("invalid user_id: {}", e)))?;

        match self.remove_member_uc.execute(tenant_id, user_id).await {
            Ok(success) => Ok(RemoveMemberResponse { success }),
            Err(crate::usecase::RemoveMemberError::NotFound) => {
                Err(GrpcError::NotFound("member not found".to_string()))
            }
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    pub async fn get_provisioning_status(
        &self,
        req: GetProvisioningStatusRequest,
    ) -> Result<GetProvisioningStatusResponse, GrpcError> {
        let job_id = uuid::Uuid::parse_str(&req.job_id)
            .map_err(|e| GrpcError::InvalidArgument(format!("invalid job_id: {}", e)))?;

        match self.get_provisioning_status_uc.execute(job_id).await {
            Ok(job) => Ok(GetProvisioningStatusResponse {
                job: Some(domain_job_to_pb(&job)),
            }),
            Err(crate::usecase::GetProvisioningStatusError::NotFound(id)) => {
                Err(GrpcError::NotFound(format!("job not found: {}", id)))
            }
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }
}

// --- Validation helpers ---

fn validate_plan(s: &str) -> Result<(), GrpcError> {
    match s {
        "free" | "starter" | "professional" | "enterprise" => Ok(()),
        _ => Err(GrpcError::InvalidArgument(format!("unknown plan: {}", s))),
    }
}

fn validate_role(s: &str) -> Result<(), GrpcError> {
    match s {
        "owner" | "admin" | "member" | "viewer" => Ok(()),
        _ => Err(GrpcError::InvalidArgument(format!("unknown role: {}", s))),
    }
}

// --- Conversion helpers ---

fn domain_tenant_to_pb(t: &Tenant) -> PbTenant {
    PbTenant {
        id: t.id.to_string(),
        name: t.name.clone(),
        display_name: t.display_name.clone(),
        status: t.status.as_str().to_string(),
        plan: t.plan.clone(),
        realm_name: t.keycloak_realm.clone().unwrap_or_default(),
        owner_id: String::new(),
        created_at: Some(PbTimestamp {
            seconds: t.created_at.timestamp(),
            nanos: t.created_at.timestamp_subsec_nanos() as i32,
        }),
        updated_at: Some(PbTimestamp {
            seconds: t.updated_at.timestamp(),
            nanos: t.updated_at.timestamp_subsec_nanos() as i32,
        }),
    }
}

fn domain_member_to_pb(m: &TenantMember) -> PbTenantMember {
    PbTenantMember {
        id: m.id.to_string(),
        tenant_id: m.tenant_id.to_string(),
        user_id: m.user_id.to_string(),
        role: m.role.clone(),
        joined_at: Some(PbTimestamp {
            seconds: m.joined_at.timestamp(),
            nanos: m.joined_at.timestamp_subsec_nanos() as i32,
        }),
    }
}

fn domain_job_to_pb(j: &ProvisioningJob) -> PbProvisioningJob {
    PbProvisioningJob {
        id: j.id.to_string(),
        tenant_id: j.tenant_id.to_string(),
        status: j.status.as_str().to_string(),
        current_step: j.current_step.clone().unwrap_or_default(),
        error_message: j.error_message.clone().unwrap_or_default(),
        created_at: Some(PbTimestamp {
            seconds: j.created_at.timestamp(),
            nanos: j.created_at.timestamp_subsec_nanos() as i32,
        }),
        updated_at: Some(PbTimestamp {
            seconds: j.updated_at.timestamp(),
            nanos: j.updated_at.timestamp_subsec_nanos() as i32,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::{Plan, ProvisioningStatus, TenantStatus};
    use crate::domain::repository::member_repository::MockMemberRepository;
    use crate::domain::repository::tenant_repository::MockTenantRepository;

    fn setup_mocks() -> (MockTenantRepository, MockMemberRepository) {
        let mut tenant_mock = MockTenantRepository::new();
        tenant_mock.expect_find_by_name().returning(|_| Ok(None));
        tenant_mock.expect_create().returning(|_| Ok(()));
        tenant_mock.expect_find_by_id().returning(|id| {
            Ok(Some(Tenant {
                id: *id,
                name: "acme-corp".to_string(),
                display_name: "ACME Corporation".to_string(),
                status: TenantStatus::Active,
                plan: "pro".to_string(),
                settings: serde_json::json!({}),
                keycloak_realm: None,
                db_schema: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }))
        });
        tenant_mock.expect_list().returning(|_, _| {
            Ok((
                vec![Tenant::new(
                    "acme-corp".to_string(),
                    "ACME Corporation".to_string(),
                    Plan::Professional.as_str().to_string(),
                    None,
                )],
                1,
            ))
        });

        let mut member_mock = MockMemberRepository::new();
        member_mock.expect_find_member().returning(|_, _| Ok(None));
        member_mock.expect_add().returning(|_| Ok(()));
        member_mock.expect_remove().returning(|_, _| Ok(true));
        member_mock.expect_find_job().returning(|id| {
            Ok(Some(ProvisioningJob {
                id: *id,
                tenant_id: uuid::Uuid::new_v4(),
                status: ProvisioningStatus::Completed,
                current_step: None,
                error_message: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }))
        });

        (tenant_mock, member_mock)
    }

    fn make_grpc_service(
        tenant_mock: MockTenantRepository,
        member_mock: MockMemberRepository,
    ) -> TenantGrpcService {
        let tenant_repo = Arc::new(tenant_mock);
        let member_repo = Arc::new(member_mock);
        TenantGrpcService::new(
            Arc::new(CreateTenantUseCase::new(tenant_repo.clone())),
            Arc::new(GetTenantUseCase::new(tenant_repo.clone())),
            Arc::new(ListTenantsUseCase::new(tenant_repo)),
            Arc::new(AddMemberUseCase::new(member_repo.clone())),
            Arc::new(RemoveMemberUseCase::new(member_repo.clone())),
            Arc::new(GetProvisioningStatusUseCase::new(member_repo)),
        )
    }

    #[tokio::test]
    async fn test_grpc_create_tenant() {
        let (tm, mm) = setup_mocks();
        let svc = make_grpc_service(tm, mm);

        let req = CreateTenantRequest {
            name: "acme-corp".to_string(),
            display_name: "ACME Corporation".to_string(),
            plan: "professional".to_string(),
            owner_id: uuid::Uuid::new_v4().to_string(),
        };

        let resp = svc.create_tenant(req).await.unwrap();
        let tenant = resp.tenant.unwrap();
        assert_eq!(tenant.name, "acme-corp");
        assert_eq!(tenant.status, "provisioning");
        assert_eq!(tenant.plan, "professional");
    }

    #[tokio::test]
    async fn test_grpc_get_tenant() {
        let (tm, mm) = setup_mocks();
        let svc = make_grpc_service(tm, mm);

        let tenant_id = uuid::Uuid::new_v4();
        let req = GetTenantRequest {
            tenant_id: tenant_id.to_string(),
        };

        let resp = svc.get_tenant(req).await.unwrap();
        let tenant = resp.tenant.unwrap();
        assert_eq!(tenant.id, tenant_id.to_string());
    }

    #[tokio::test]
    async fn test_grpc_list_tenants() {
        let (tm, mm) = setup_mocks();
        let svc = make_grpc_service(tm, mm);

        let req = ListTenantsRequest {
            page: 1,
            page_size: 10,
        };

        let resp = svc.list_tenants(req).await.unwrap();
        assert_eq!(resp.tenants.len(), 1);
        assert_eq!(resp.total_count, 1);
    }

    #[tokio::test]
    async fn test_grpc_add_member() {
        let (tm, mm) = setup_mocks();
        let svc = make_grpc_service(tm, mm);

        let req = AddMemberRequest {
            tenant_id: uuid::Uuid::new_v4().to_string(),
            user_id: uuid::Uuid::new_v4().to_string(),
            role: "member".to_string(),
        };

        let resp = svc.add_member(req).await.unwrap();
        let member = resp.member.unwrap();
        assert_eq!(member.role, "member");
    }

    #[tokio::test]
    async fn test_grpc_remove_member() {
        let (tm, mm) = setup_mocks();
        let svc = make_grpc_service(tm, mm);

        let req = RemoveMemberRequest {
            tenant_id: uuid::Uuid::new_v4().to_string(),
            user_id: uuid::Uuid::new_v4().to_string(),
        };

        let resp = svc.remove_member(req).await.unwrap();
        assert!(resp.success);
    }

    #[tokio::test]
    async fn test_grpc_get_provisioning_status() {
        let (tm, mm) = setup_mocks();
        let svc = make_grpc_service(tm, mm);

        let req = GetProvisioningStatusRequest {
            job_id: uuid::Uuid::new_v4().to_string(),
        };

        let resp = svc.get_provisioning_status(req).await.unwrap();
        let job = resp.job.unwrap();
        assert_eq!(job.status, "completed");
    }

    #[tokio::test]
    async fn test_grpc_create_tenant_invalid_plan() {
        let (tm, mm) = setup_mocks();
        let svc = make_grpc_service(tm, mm);

        let req = CreateTenantRequest {
            name: "t".to_string(),
            display_name: "T".to_string(),
            plan: "invalid".to_string(),
            owner_id: "".to_string(),
        };

        let result = svc.create_tenant(req).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            GrpcError::InvalidArgument(msg) => assert!(msg.contains("plan")),
            e => panic!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_grpc_add_member_invalid_role() {
        let (tm, mm) = setup_mocks();
        let svc = make_grpc_service(tm, mm);

        let req = AddMemberRequest {
            tenant_id: uuid::Uuid::new_v4().to_string(),
            user_id: uuid::Uuid::new_v4().to_string(),
            role: "invalid".to_string(),
        };

        let result = svc.add_member(req).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            GrpcError::InvalidArgument(msg) => assert!(msg.contains("role")),
            e => panic!("unexpected error: {:?}", e),
        }
    }
}

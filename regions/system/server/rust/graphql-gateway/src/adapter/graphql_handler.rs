use std::collections::HashMap;
use std::sync::Arc;

use async_graphql::dataloader::DataLoader;
use async_graphql::futures_util::Stream;
use async_graphql::{Context, Data, ErrorExtensions, FieldResult, Object, Schema, Subscription};
use async_graphql_axum::{GraphQLProtocol, GraphQLRequest, GraphQLResponse, GraphQLWebSocket};
use axum::{
    extract::{State, WebSocketUpgrade},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Extension, Json, Router,
};

use crate::adapter::middleware::auth_middleware::{AuthMiddlewareLayer, Claims};
use crate::domain::model::graphql_context::{
    ConfigLoader, FeatureFlagLoader, GraphqlContext, TenantLoader,
};
use crate::domain::model::{
    ApproveTaskPayload, AuditLogConnection, CancelInstancePayload, CatalogService,
    CatalogServiceConnection, ConfigEntry, CreateChannelPayload, CreateJobPayload,
    CreateSessionPayload, CreateTemplatePayload, CreateTenantPayload, CreateWorkflowPayload,
    DeleteChannelPayload, DeleteJobPayload, DeleteSecretPayload, DeleteServicePayload,
    DeleteTemplatePayload, DeleteWorkflowPayload, FeatureFlag, Job, JobExecution, Navigation,
    NotificationChannel, NotificationLog, NotificationTemplate, PauseJobPayload, PermissionCheck,
    ReassignTaskPayload, RecordAuditLogPayload, RefreshSessionPayload, RegisterServicePayload,
    RejectTaskPayload, ResumeJobPayload, RetryNotificationPayload, RevokeAllSessionsPayload,
    RevokeSessionPayload, Role, RotateSecretPayload, SecretMetadata, SendNotificationPayload,
    ServiceHealth, Session, SetFeatureFlagPayload, SetSecretPayload, StartInstancePayload, Tenant,
    TenantConnection, TenantStatus, TriggerJobPayload, UpdateChannelPayload, UpdateJobPayload,
    UpdateServicePayload, UpdateTemplatePayload, UpdateTenantPayload, UpdateWorkflowPayload, User,
    UserError, VaultAuditLogEntry, WorkflowDefinition, WorkflowInstance, WorkflowTask,
};
use crate::infrastructure::auth::JwksVerifier;
use crate::infrastructure::config::GraphQLConfig;
use crate::infrastructure::grpc::{
    AuthGrpcClient, ConfigGrpcClient, FeatureFlagGrpcClient, NavigationGrpcClient,
    NotificationGrpcClient, SchedulerGrpcClient, ServiceCatalogGrpcClient, SessionGrpcClient,
    TenantGrpcClient, VaultGrpcClient, WorkflowGrpcClient,
};
use crate::usecase::{
    AuthMutationResolver, AuthQueryResolver, ConfigQueryResolver, FeatureFlagQueryResolver,
    NavigationQueryResolver, NotificationMutationResolver, NotificationQueryResolver,
    SchedulerMutationResolver, SchedulerQueryResolver, ServiceCatalogMutationResolver,
    ServiceCatalogQueryResolver, SessionMutationResolver, SessionQueryResolver,
    SubscriptionResolver, TenantMutationResolver, TenantQueryResolver, VaultMutationResolver,
    VaultQueryResolver, WorkflowMutationResolver, WorkflowQueryResolver,
};

const CODE_FORBIDDEN: &str = "FORBIDDEN";
const CODE_VALIDATION: &str = "VALIDATION_ERROR";
const CODE_BACKEND: &str = "BACKEND_ERROR";

fn gql_error(code: &'static str, message: impl Into<String>) -> async_graphql::Error {
    async_graphql::Error::new(message.into()).extend_with(|_, e| e.set("code", code))
}

fn classify_domain_error(message: &str) -> &'static str {
    let lower = message.to_ascii_lowercase();
    if lower.contains("validation")
        || lower.contains("invalid")
        || lower.contains("required")
        || lower.contains("unknown")
    {
        CODE_VALIDATION
    } else {
        CODE_BACKEND
    }
}

/// Tier はロールチェックのスコープを表現する。
/// k1s0_server_common::middleware::rbac::Tier と同等だが、axum バージョン差異のため
/// GraphQL Gateway 内で再定義している。
#[derive(Debug, Clone, Copy)]
enum Tier {
    /// System tier: sys_admin / sys_operator / sys_auditor
    System,
}

/// ロールベースの権限チェック。Tier に応じてロールプレフィックスを切り替える。
/// k1s0_server_common::middleware::rbac::check_permission と同等のロジック。
fn check_permission(tier: Tier, roles: &[String], action: &str) -> bool {
    for role in roles {
        match tier {
            Tier::System => match role.as_str() {
                "sys_admin" => return true,
                "sys_operator" => {
                    if matches!(action, "read" | "write") {
                        return true;
                    }
                }
                "sys_auditor" => {
                    if action == "read" {
                        return true;
                    }
                }
                _ => {}
            },
        }
    }
    false
}

fn ensure_write_permission(ctx: &Context<'_>) -> FieldResult<()> {
    let roles = if let Ok(gql_ctx) = ctx.data::<GraphqlContext>() {
        gql_ctx.roles.clone()
    } else if let Ok(claims) = ctx.data::<Claims>() {
        claims.roles()
    } else {
        vec![]
    };

    if check_permission(Tier::System, &roles, "write") {
        Ok(())
    } else {
        Err(gql_error(
            CODE_FORBIDDEN,
            "insufficient permissions for this operation",
        ))
    }
}

// --- Input types ---

#[derive(async_graphql::InputObject)]
pub struct CreateTenantInput {
    pub name: String,
}

#[derive(async_graphql::InputObject)]
pub struct UpdateTenantInput {
    pub name: Option<String>,
    pub status: Option<TenantStatus>,
}

#[derive(async_graphql::InputObject)]
pub struct SetFeatureFlagInput {
    pub enabled: bool,
    pub rollout_percentage: Option<i32>,
    pub target_environments: Option<Vec<String>>,
}

#[derive(async_graphql::InputObject)]
pub struct RegisterServiceInput {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub tier: String,
    pub version: String,
    pub base_url: String,
    pub grpc_endpoint: Option<String>,
    pub health_url: String,
}

#[derive(async_graphql::InputObject)]
pub struct UpdateServiceInput {
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub version: Option<String>,
    pub base_url: Option<String>,
    pub grpc_endpoint: Option<String>,
    pub health_url: Option<String>,
}

// --- Auth / Session Input types ---

#[derive(async_graphql::InputObject)]
pub struct RecordAuditLogInput {
    pub event_type: String,
    pub user_id: String,
    pub ip_address: String,
    pub user_agent: String,
    pub resource: String,
    pub action: String,
    pub result: String,
    pub resource_id: Option<String>,
    pub trace_id: Option<String>,
}

#[derive(async_graphql::InputObject)]
pub struct CreateSessionInput {
    pub user_id: String,
    pub device_id: String,
    pub device_name: Option<String>,
    pub device_type: Option<String>,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub ttl_seconds: Option<i32>,
}

#[derive(async_graphql::InputObject)]
pub struct RefreshSessionInput {
    pub session_id: String,
    pub ttl_seconds: Option<i32>,
}

// --- Vault Input types ---

#[derive(async_graphql::InputObject)]
pub struct SetSecretInput {
    pub path: String,
    pub data: Vec<SecretKeyValue>,
}

#[derive(async_graphql::InputObject)]
pub struct SecretKeyValue {
    pub key: String,
    pub value: String,
}

#[derive(async_graphql::InputObject)]
pub struct RotateSecretInput {
    pub path: String,
    pub data: Vec<SecretKeyValue>,
}

// --- Scheduler Input types ---

#[derive(async_graphql::InputObject)]
pub struct CreateJobInput {
    pub name: String,
    pub description: String,
    pub cron_expression: String,
    pub timezone: String,
    pub target_type: String,
    pub target: String,
}

#[derive(async_graphql::InputObject)]
pub struct UpdateJobInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub cron_expression: Option<String>,
    pub timezone: Option<String>,
    pub target_type: Option<String>,
    pub target: Option<String>,
}

// --- Notification Input types ---

#[derive(async_graphql::InputObject)]
pub struct SendNotificationInput {
    pub channel_id: String,
    pub template_id: Option<String>,
    pub recipient: String,
    pub subject: Option<String>,
    pub body: Option<String>,
    pub template_variables: Option<Vec<TemplateVariableInput>>,
}

#[derive(async_graphql::InputObject)]
pub struct TemplateVariableInput {
    pub key: String,
    pub value: String,
}

#[derive(async_graphql::InputObject)]
pub struct CreateChannelInput {
    pub name: String,
    pub channel_type: String,
    pub config_json: String,
    pub enabled: bool,
}

#[derive(async_graphql::InputObject)]
pub struct UpdateChannelInput {
    pub name: Option<String>,
    pub enabled: Option<bool>,
    pub config_json: Option<String>,
}

#[derive(async_graphql::InputObject)]
pub struct CreateTemplateInput {
    pub name: String,
    pub channel_type: String,
    pub subject_template: Option<String>,
    pub body_template: String,
}

#[derive(async_graphql::InputObject)]
pub struct UpdateTemplateInput {
    pub name: Option<String>,
    pub subject_template: Option<String>,
    pub body_template: Option<String>,
}

// --- Workflow Input types ---

#[derive(async_graphql::InputObject)]
pub struct CreateWorkflowInput {
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub steps: Vec<WorkflowStepInput>,
}

#[derive(Debug, async_graphql::InputObject)]
pub struct WorkflowStepInput {
    pub step_id: String,
    pub name: String,
    pub step_type: String,
    pub assignee_role: Option<String>,
    pub timeout_hours: Option<i32>,
    pub on_approve: Option<String>,
    pub on_reject: Option<String>,
}

#[derive(async_graphql::InputObject)]
pub struct UpdateWorkflowInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub enabled: Option<bool>,
    pub steps: Option<Vec<WorkflowStepInput>>,
}

#[derive(async_graphql::InputObject)]
pub struct StartInstanceInput {
    pub workflow_id: String,
    pub title: String,
    pub initiator_id: String,
    pub context_json: Option<String>,
}

#[derive(async_graphql::InputObject)]
pub struct ReassignTaskInput {
    pub task_id: String,
    pub new_assignee_id: String,
    pub reason: Option<String>,
    pub actor_id: String,
}

#[derive(async_graphql::InputObject)]
pub struct TaskDecisionInput {
    pub task_id: String,
    pub actor_id: String,
    pub comment: Option<String>,
}

// --- Gateway 構造体: router() の引数爆発を解消する ---

/// gRPC バックエンドクライアント群。readyz ヘルスチェックと DataLoader 生成に使用。
pub struct GatewayClients {
    pub tenant: Arc<TenantGrpcClient>,
    pub feature_flag: Arc<FeatureFlagGrpcClient>,
    pub config: Arc<ConfigGrpcClient>,
    pub navigation: Arc<NavigationGrpcClient>,
    pub service_catalog: Arc<ServiceCatalogGrpcClient>,
    pub auth: Arc<AuthGrpcClient>,
    pub session: Arc<SessionGrpcClient>,
    pub vault: Arc<VaultGrpcClient>,
    pub scheduler: Arc<SchedulerGrpcClient>,
    pub notification: Arc<NotificationGrpcClient>,
    pub workflow: Arc<WorkflowGrpcClient>,
}

/// GraphQL リゾルバ群。QueryRoot / MutationRoot / SubscriptionRoot を構成する。
pub struct GatewayResolvers {
    pub tenant_query: Arc<TenantQueryResolver>,
    pub feature_flag_query: Arc<FeatureFlagQueryResolver>,
    pub config_query: Arc<ConfigQueryResolver>,
    pub navigation_query: Arc<NavigationQueryResolver>,
    pub service_catalog_query: Arc<ServiceCatalogQueryResolver>,
    pub tenant_mutation: Arc<TenantMutationResolver>,
    pub service_catalog_mutation: Arc<ServiceCatalogMutationResolver>,
    pub subscription: Arc<SubscriptionResolver>,
    pub auth_query: Arc<AuthQueryResolver>,
    pub auth_mutation: Arc<AuthMutationResolver>,
    pub session_query: Arc<SessionQueryResolver>,
    pub session_mutation: Arc<SessionMutationResolver>,
    pub vault_query: Arc<VaultQueryResolver>,
    pub vault_mutation: Arc<VaultMutationResolver>,
    pub scheduler_query: Arc<SchedulerQueryResolver>,
    pub scheduler_mutation: Arc<SchedulerMutationResolver>,
    pub notification_query: Arc<NotificationQueryResolver>,
    pub notification_mutation: Arc<NotificationMutationResolver>,
    pub workflow_query: Arc<WorkflowQueryResolver>,
    pub workflow_mutation: Arc<WorkflowMutationResolver>,
}

// --- Query ---

pub struct QueryRoot {
    pub tenant_query: Arc<TenantQueryResolver>,
    pub feature_flag_query: Arc<FeatureFlagQueryResolver>,
    pub config_query: Arc<ConfigQueryResolver>,
    pub navigation_query: Arc<NavigationQueryResolver>,
    pub service_catalog_query: Arc<ServiceCatalogQueryResolver>,
    pub auth_query: Arc<AuthQueryResolver>,
    pub session_query: Arc<SessionQueryResolver>,
    pub vault_query: Arc<VaultQueryResolver>,
    pub scheduler_query: Arc<SchedulerQueryResolver>,
    pub notification_query: Arc<NotificationQueryResolver>,
    pub workflow_query: Arc<WorkflowQueryResolver>,
}

#[Object]
impl QueryRoot {
    async fn tenant(
        &self,
        _ctx: &Context<'_>,
        id: async_graphql::ID,
    ) -> FieldResult<Option<Tenant>> {
        self.tenant_query
            .get_tenant(id.as_str())
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    async fn tenants(
        &self,
        _ctx: &Context<'_>,
        first: Option<i32>,
        after: Option<String>,
    ) -> FieldResult<TenantConnection> {
        self.tenant_query
            .list_tenants(first, after)
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    async fn feature_flag(
        &self,
        _ctx: &Context<'_>,
        key: String,
    ) -> FieldResult<Option<FeatureFlag>> {
        self.feature_flag_query
            .get_feature_flag(&key)
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    async fn feature_flags(
        &self,
        _ctx: &Context<'_>,
        environment: Option<String>,
    ) -> FieldResult<Vec<FeatureFlag>> {
        self.feature_flag_query
            .list_feature_flags(environment.as_deref())
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    async fn config(&self, ctx: &Context<'_>, key: String) -> FieldResult<Option<ConfigEntry>> {
        if let Ok(gql_ctx) = ctx.data::<GraphqlContext>() {
            return gql_ctx
                .config_loader
                .load_one(key.clone())
                .await
                .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()));
        }

        self.config_query
            .get_config(&key)
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    async fn navigation(
        &self,
        _ctx: &Context<'_>,
        bearer_token: Option<String>,
    ) -> FieldResult<Navigation> {
        let token = bearer_token.unwrap_or_default();
        self.navigation_query
            .get_navigation(&token)
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    async fn catalog_service(
        &self,
        _ctx: &Context<'_>,
        id: async_graphql::ID,
    ) -> FieldResult<Option<CatalogService>> {
        self.service_catalog_query
            .get_service(id.as_str())
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    async fn catalog_services(
        &self,
        _ctx: &Context<'_>,
        first: Option<i32>,
        tier: Option<String>,
        status: Option<String>,
        search: Option<String>,
    ) -> FieldResult<CatalogServiceConnection> {
        self.service_catalog_query
            .list_services(first, tier.as_deref(), status.as_deref(), search.as_deref())
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    async fn service_health(
        &self,
        _ctx: &Context<'_>,
        service_id: Option<String>,
    ) -> FieldResult<Vec<ServiceHealth>> {
        self.service_catalog_query
            .health_check(service_id.as_deref())
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    // --- Auth queries ---

    async fn user(
        &self,
        _ctx: &Context<'_>,
        user_id: async_graphql::ID,
    ) -> FieldResult<Option<User>> {
        self.auth_query
            .get_user(user_id.as_str())
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    async fn users(
        &self,
        _ctx: &Context<'_>,
        first: Option<i32>,
        after: Option<i32>,
        search: Option<String>,
        enabled: Option<bool>,
    ) -> FieldResult<Vec<User>> {
        self.auth_query
            .list_users(first, after, search.as_deref(), enabled)
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    async fn user_roles(
        &self,
        _ctx: &Context<'_>,
        user_id: async_graphql::ID,
    ) -> FieldResult<Vec<Role>> {
        self.auth_query
            .get_user_roles(user_id.as_str())
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    async fn check_permission(
        &self,
        _ctx: &Context<'_>,
        permission: String,
        resource: String,
        roles: Vec<String>,
        user_id: Option<String>,
    ) -> FieldResult<PermissionCheck> {
        self.auth_query
            .check_permission(user_id.as_deref(), &permission, &resource, &roles)
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    async fn search_audit_logs(
        &self,
        _ctx: &Context<'_>,
        first: Option<i32>,
        after: Option<i32>,
        user_id: Option<String>,
        event_type: Option<String>,
        result: Option<String>,
    ) -> FieldResult<AuditLogConnection> {
        self.auth_query
            .search_audit_logs(
                first,
                after,
                user_id.as_deref(),
                event_type.as_deref(),
                result.as_deref(),
            )
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    // --- Session queries ---

    async fn session(
        &self,
        _ctx: &Context<'_>,
        session_id: async_graphql::ID,
    ) -> FieldResult<Option<Session>> {
        self.session_query
            .get_session(session_id.as_str())
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    async fn user_sessions(
        &self,
        _ctx: &Context<'_>,
        user_id: async_graphql::ID,
    ) -> FieldResult<Vec<Session>> {
        self.session_query
            .list_user_sessions(user_id.as_str())
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    // --- Vault queries ---

    async fn secret_metadata(
        &self,
        _ctx: &Context<'_>,
        path: String,
    ) -> FieldResult<Option<SecretMetadata>> {
        self.vault_query
            .get_secret_metadata(&path)
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    async fn secrets(
        &self,
        _ctx: &Context<'_>,
        prefix: Option<String>,
    ) -> FieldResult<Vec<String>> {
        self.vault_query
            .list_secrets(prefix.as_deref())
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    async fn vault_audit_logs(
        &self,
        _ctx: &Context<'_>,
        offset: Option<i32>,
        limit: Option<i32>,
    ) -> FieldResult<Vec<VaultAuditLogEntry>> {
        self.vault_query
            .list_audit_logs(offset, limit)
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    // --- Scheduler queries ---

    async fn job(&self, _ctx: &Context<'_>, job_id: async_graphql::ID) -> FieldResult<Option<Job>> {
        self.scheduler_query
            .get_job(job_id.as_str())
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    async fn jobs(
        &self,
        _ctx: &Context<'_>,
        status: Option<String>,
        first: Option<i32>,
        after: Option<i32>,
    ) -> FieldResult<Vec<Job>> {
        self.scheduler_query
            .list_jobs(status.as_deref(), first, after)
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    async fn job_execution(
        &self,
        _ctx: &Context<'_>,
        execution_id: async_graphql::ID,
    ) -> FieldResult<Option<JobExecution>> {
        self.scheduler_query
            .get_job_execution(execution_id.as_str())
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    async fn job_executions(
        &self,
        _ctx: &Context<'_>,
        job_id: async_graphql::ID,
        first: Option<i32>,
        after: Option<i32>,
        status: Option<String>,
    ) -> FieldResult<Vec<JobExecution>> {
        self.scheduler_query
            .list_executions(job_id.as_str(), first, after, status.as_deref())
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    // --- Notification queries ---

    async fn notification(
        &self,
        _ctx: &Context<'_>,
        notification_id: async_graphql::ID,
    ) -> FieldResult<Option<NotificationLog>> {
        self.notification_query
            .get_notification(notification_id.as_str())
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    async fn notifications(
        &self,
        _ctx: &Context<'_>,
        channel_id: Option<String>,
        status: Option<String>,
        page: Option<i32>,
        page_size: Option<i32>,
    ) -> FieldResult<Vec<NotificationLog>> {
        self.notification_query
            .list_notifications(channel_id.as_deref(), status.as_deref(), page, page_size)
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    async fn notification_channel(
        &self,
        _ctx: &Context<'_>,
        id: async_graphql::ID,
    ) -> FieldResult<Option<NotificationChannel>> {
        self.notification_query
            .get_channel(id.as_str())
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    async fn notification_channels(
        &self,
        _ctx: &Context<'_>,
        channel_type: Option<String>,
        enabled_only: Option<bool>,
        page: Option<i32>,
        page_size: Option<i32>,
    ) -> FieldResult<Vec<NotificationChannel>> {
        self.notification_query
            .list_channels(
                channel_type.as_deref(),
                enabled_only.unwrap_or(false),
                page,
                page_size,
            )
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    async fn notification_template(
        &self,
        _ctx: &Context<'_>,
        id: async_graphql::ID,
    ) -> FieldResult<Option<NotificationTemplate>> {
        self.notification_query
            .get_template(id.as_str())
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    async fn notification_templates(
        &self,
        _ctx: &Context<'_>,
        channel_type: Option<String>,
        page: Option<i32>,
        page_size: Option<i32>,
    ) -> FieldResult<Vec<NotificationTemplate>> {
        self.notification_query
            .list_templates(channel_type.as_deref(), page, page_size)
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    // --- Workflow queries ---

    async fn workflow(
        &self,
        _ctx: &Context<'_>,
        workflow_id: async_graphql::ID,
    ) -> FieldResult<Option<WorkflowDefinition>> {
        self.workflow_query
            .get_workflow(workflow_id.as_str())
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    async fn workflows(
        &self,
        _ctx: &Context<'_>,
        enabled_only: Option<bool>,
        first: Option<i32>,
        after: Option<i32>,
    ) -> FieldResult<Vec<WorkflowDefinition>> {
        self.workflow_query
            .list_workflows(enabled_only.unwrap_or(false), first, after)
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    async fn workflow_instance(
        &self,
        _ctx: &Context<'_>,
        instance_id: async_graphql::ID,
    ) -> FieldResult<Option<WorkflowInstance>> {
        self.workflow_query
            .get_instance(instance_id.as_str())
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    async fn workflow_instances(
        &self,
        _ctx: &Context<'_>,
        status: Option<String>,
        workflow_id: Option<String>,
        initiator_id: Option<String>,
        first: Option<i32>,
        after: Option<i32>,
    ) -> FieldResult<Vec<WorkflowInstance>> {
        self.workflow_query
            .list_instances(
                status.as_deref(),
                workflow_id.as_deref(),
                initiator_id.as_deref(),
                first,
                after,
            )
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    #[allow(clippy::too_many_arguments)]
    async fn workflow_tasks(
        &self,
        _ctx: &Context<'_>,
        assignee_id: Option<String>,
        status: Option<String>,
        instance_id: Option<String>,
        overdue_only: Option<bool>,
        first: Option<i32>,
        after: Option<i32>,
    ) -> FieldResult<Vec<WorkflowTask>> {
        self.workflow_query
            .list_tasks(
                assignee_id.as_deref(),
                status.as_deref(),
                instance_id.as_deref(),
                overdue_only.unwrap_or(false),
                first,
                after,
            )
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }
}

// --- Mutation ---

pub struct MutationRoot {
    pub tenant_mutation: Arc<TenantMutationResolver>,
    pub feature_flag_client: Arc<FeatureFlagGrpcClient>,
    pub service_catalog_mutation: Arc<ServiceCatalogMutationResolver>,
    pub auth_mutation: Arc<AuthMutationResolver>,
    pub session_mutation: Arc<SessionMutationResolver>,
    pub vault_mutation: Arc<VaultMutationResolver>,
    pub scheduler_mutation: Arc<SchedulerMutationResolver>,
    pub notification_mutation: Arc<NotificationMutationResolver>,
    pub workflow_mutation: Arc<WorkflowMutationResolver>,
}

#[Object]
impl MutationRoot {
    async fn create_tenant(
        &self,
        ctx: &Context<'_>,
        input: CreateTenantInput,
    ) -> FieldResult<CreateTenantPayload> {
        ensure_write_permission(ctx)?;
        let claims = ctx.data::<Claims>().ok();
        let owner_user_id = claims.map(|c| c.sub.as_str()).unwrap_or("unknown");
        Ok(self
            .tenant_mutation
            .create_tenant(&input.name, owner_user_id)
            .await)
    }

    async fn update_tenant(
        &self,
        ctx: &Context<'_>,
        id: async_graphql::ID,
        input: UpdateTenantInput,
    ) -> FieldResult<UpdateTenantPayload> {
        ensure_write_permission(ctx)?;
        let status_str = input.status.map(|s| match s {
            TenantStatus::Active => "ACTIVE".to_string(),
            TenantStatus::Suspended => "SUSPENDED".to_string(),
            TenantStatus::Deleted => "DELETED".to_string(),
        });
        Ok(self
            .tenant_mutation
            .update_tenant(id.as_str(), input.name.as_deref(), status_str.as_deref())
            .await)
    }

    async fn set_feature_flag(
        &self,
        ctx: &Context<'_>,
        key: String,
        input: SetFeatureFlagInput,
    ) -> FieldResult<SetFeatureFlagPayload> {
        ensure_write_permission(ctx)?;
        match self
            .feature_flag_client
            .set_flag(
                &key,
                input.enabled,
                input.rollout_percentage,
                input.target_environments,
            )
            .await
        {
            Ok(flag) => Ok(SetFeatureFlagPayload {
                feature_flag: Some(flag),
                errors: vec![],
            }),
            Err(e) => {
                let msg = e.to_string();
                Ok(SetFeatureFlagPayload {
                    feature_flag: None,
                    errors: vec![UserError {
                        field: None,
                        message: msg,
                    }],
                })
            }
        }
    }

    async fn register_service(
        &self,
        ctx: &Context<'_>,
        input: RegisterServiceInput,
    ) -> FieldResult<RegisterServicePayload> {
        ensure_write_permission(ctx)?;
        Ok(self
            .service_catalog_mutation
            .register_service(
                &input.name,
                &input.display_name,
                &input.description,
                &input.tier,
                &input.version,
                &input.base_url,
                input.grpc_endpoint.as_deref(),
                &input.health_url,
                HashMap::new(),
            )
            .await)
    }

    async fn update_service(
        &self,
        ctx: &Context<'_>,
        id: async_graphql::ID,
        input: UpdateServiceInput,
    ) -> FieldResult<UpdateServicePayload> {
        ensure_write_permission(ctx)?;
        Ok(self
            .service_catalog_mutation
            .update_service(
                id.as_str(),
                input.display_name.as_deref(),
                input.description.as_deref(),
                input.version.as_deref(),
                input.base_url.as_deref(),
                input.grpc_endpoint.as_deref(),
                input.health_url.as_deref(),
                HashMap::new(),
            )
            .await)
    }

    async fn delete_service(
        &self,
        ctx: &Context<'_>,
        id: async_graphql::ID,
    ) -> FieldResult<DeleteServicePayload> {
        ensure_write_permission(ctx)?;
        Ok(self
            .service_catalog_mutation
            .delete_service(id.as_str())
            .await)
    }

    // --- Auth mutations ---

    async fn record_audit_log(
        &self,
        ctx: &Context<'_>,
        input: RecordAuditLogInput,
    ) -> FieldResult<RecordAuditLogPayload> {
        ensure_write_permission(ctx)?;
        Ok(self
            .auth_mutation
            .record_audit_log(
                &input.event_type,
                &input.user_id,
                &input.ip_address,
                &input.user_agent,
                &input.resource,
                &input.action,
                &input.result,
                input.resource_id.as_deref(),
                input.trace_id.as_deref(),
            )
            .await)
    }

    // --- Session mutations ---

    async fn create_session(
        &self,
        ctx: &Context<'_>,
        input: CreateSessionInput,
    ) -> FieldResult<CreateSessionPayload> {
        ensure_write_permission(ctx)?;
        Ok(self
            .session_mutation
            .create_session(
                &input.user_id,
                &input.device_id,
                input.device_name.as_deref(),
                input.device_type.as_deref(),
                input.user_agent.as_deref(),
                input.ip_address.as_deref(),
                input.ttl_seconds,
            )
            .await)
    }

    async fn refresh_session(
        &self,
        ctx: &Context<'_>,
        input: RefreshSessionInput,
    ) -> FieldResult<RefreshSessionPayload> {
        ensure_write_permission(ctx)?;
        Ok(self
            .session_mutation
            .refresh_session(&input.session_id, input.ttl_seconds)
            .await)
    }

    async fn revoke_session(
        &self,
        ctx: &Context<'_>,
        session_id: async_graphql::ID,
    ) -> FieldResult<RevokeSessionPayload> {
        ensure_write_permission(ctx)?;
        Ok(self
            .session_mutation
            .revoke_session(session_id.as_str())
            .await)
    }

    async fn revoke_all_sessions(
        &self,
        ctx: &Context<'_>,
        user_id: async_graphql::ID,
    ) -> FieldResult<RevokeAllSessionsPayload> {
        ensure_write_permission(ctx)?;
        Ok(self
            .session_mutation
            .revoke_all_sessions(user_id.as_str())
            .await)
    }

    // --- Vault mutations ---

    async fn set_secret(
        &self,
        ctx: &Context<'_>,
        input: SetSecretInput,
    ) -> FieldResult<SetSecretPayload> {
        ensure_write_permission(ctx)?;
        let data: HashMap<String, String> = input
            .data
            .into_iter()
            .map(|kv| (kv.key, kv.value))
            .collect();
        Ok(self.vault_mutation.set_secret(&input.path, data).await)
    }

    async fn rotate_secret(
        &self,
        ctx: &Context<'_>,
        input: RotateSecretInput,
    ) -> FieldResult<RotateSecretPayload> {
        ensure_write_permission(ctx)?;
        let data: HashMap<String, String> = input
            .data
            .into_iter()
            .map(|kv| (kv.key, kv.value))
            .collect();
        Ok(self.vault_mutation.rotate_secret(&input.path, data).await)
    }

    async fn delete_secret(
        &self,
        ctx: &Context<'_>,
        path: String,
        versions: Option<Vec<i64>>,
    ) -> FieldResult<DeleteSecretPayload> {
        ensure_write_permission(ctx)?;
        Ok(self
            .vault_mutation
            .delete_secret(&path, versions.unwrap_or_default())
            .await)
    }

    // --- Scheduler mutations ---

    async fn create_job(
        &self,
        ctx: &Context<'_>,
        input: CreateJobInput,
    ) -> FieldResult<CreateJobPayload> {
        ensure_write_permission(ctx)?;
        Ok(self
            .scheduler_mutation
            .create_job(
                &input.name,
                &input.description,
                &input.cron_expression,
                &input.timezone,
                &input.target_type,
                &input.target,
            )
            .await)
    }

    async fn update_job(
        &self,
        ctx: &Context<'_>,
        job_id: async_graphql::ID,
        input: UpdateJobInput,
    ) -> FieldResult<UpdateJobPayload> {
        ensure_write_permission(ctx)?;
        Ok(self
            .scheduler_mutation
            .update_job(
                job_id.as_str(),
                input.name.as_deref(),
                input.description.as_deref(),
                input.cron_expression.as_deref(),
                input.timezone.as_deref(),
                input.target_type.as_deref(),
                input.target.as_deref(),
            )
            .await)
    }

    async fn delete_job(
        &self,
        ctx: &Context<'_>,
        job_id: async_graphql::ID,
    ) -> FieldResult<DeleteJobPayload> {
        ensure_write_permission(ctx)?;
        Ok(self.scheduler_mutation.delete_job(job_id.as_str()).await)
    }

    async fn pause_job(
        &self,
        ctx: &Context<'_>,
        job_id: async_graphql::ID,
    ) -> FieldResult<PauseJobPayload> {
        ensure_write_permission(ctx)?;
        Ok(self.scheduler_mutation.pause_job(job_id.as_str()).await)
    }

    async fn resume_job(
        &self,
        ctx: &Context<'_>,
        job_id: async_graphql::ID,
    ) -> FieldResult<ResumeJobPayload> {
        ensure_write_permission(ctx)?;
        Ok(self.scheduler_mutation.resume_job(job_id.as_str()).await)
    }

    async fn trigger_job(
        &self,
        ctx: &Context<'_>,
        job_id: async_graphql::ID,
    ) -> FieldResult<TriggerJobPayload> {
        ensure_write_permission(ctx)?;
        Ok(self.scheduler_mutation.trigger_job(job_id.as_str()).await)
    }

    // --- Notification mutations ---

    async fn send_notification(
        &self,
        ctx: &Context<'_>,
        input: SendNotificationInput,
    ) -> FieldResult<SendNotificationPayload> {
        ensure_write_permission(ctx)?;
        let vars: HashMap<String, String> = input
            .template_variables
            .unwrap_or_default()
            .into_iter()
            .map(|kv| (kv.key, kv.value))
            .collect();
        Ok(self
            .notification_mutation
            .send_notification(
                &input.channel_id,
                input.template_id.as_deref(),
                &vars,
                &input.recipient,
                input.subject.as_deref(),
                input.body.as_deref(),
            )
            .await)
    }

    async fn retry_notification(
        &self,
        ctx: &Context<'_>,
        notification_id: async_graphql::ID,
    ) -> FieldResult<RetryNotificationPayload> {
        ensure_write_permission(ctx)?;
        Ok(self
            .notification_mutation
            .retry_notification(notification_id.as_str())
            .await)
    }

    async fn create_channel(
        &self,
        ctx: &Context<'_>,
        input: CreateChannelInput,
    ) -> FieldResult<CreateChannelPayload> {
        ensure_write_permission(ctx)?;
        Ok(self
            .notification_mutation
            .create_channel(
                &input.name,
                &input.channel_type,
                &input.config_json,
                input.enabled,
            )
            .await)
    }

    async fn update_channel(
        &self,
        ctx: &Context<'_>,
        id: async_graphql::ID,
        input: UpdateChannelInput,
    ) -> FieldResult<UpdateChannelPayload> {
        ensure_write_permission(ctx)?;
        Ok(self
            .notification_mutation
            .update_channel(
                id.as_str(),
                input.name.as_deref(),
                input.enabled,
                input.config_json.as_deref(),
            )
            .await)
    }

    async fn delete_channel(
        &self,
        ctx: &Context<'_>,
        id: async_graphql::ID,
    ) -> FieldResult<DeleteChannelPayload> {
        ensure_write_permission(ctx)?;
        Ok(self.notification_mutation.delete_channel(id.as_str()).await)
    }

    async fn create_template(
        &self,
        ctx: &Context<'_>,
        input: CreateTemplateInput,
    ) -> FieldResult<CreateTemplatePayload> {
        ensure_write_permission(ctx)?;
        Ok(self
            .notification_mutation
            .create_template(
                &input.name,
                &input.channel_type,
                input.subject_template.as_deref(),
                &input.body_template,
            )
            .await)
    }

    async fn update_template(
        &self,
        ctx: &Context<'_>,
        id: async_graphql::ID,
        input: UpdateTemplateInput,
    ) -> FieldResult<UpdateTemplatePayload> {
        ensure_write_permission(ctx)?;
        Ok(self
            .notification_mutation
            .update_template(
                id.as_str(),
                input.name.as_deref(),
                input.subject_template.as_deref(),
                input.body_template.as_deref(),
            )
            .await)
    }

    async fn delete_template(
        &self,
        ctx: &Context<'_>,
        id: async_graphql::ID,
    ) -> FieldResult<DeleteTemplatePayload> {
        ensure_write_permission(ctx)?;
        Ok(self
            .notification_mutation
            .delete_template(id.as_str())
            .await)
    }

    // --- Workflow mutations ---

    async fn create_workflow(
        &self,
        ctx: &Context<'_>,
        input: CreateWorkflowInput,
    ) -> FieldResult<CreateWorkflowPayload> {
        ensure_write_permission(ctx)?;
        Ok(self
            .workflow_mutation
            .create_workflow(&input.name, &input.description, input.enabled, &input.steps)
            .await)
    }

    async fn update_workflow(
        &self,
        ctx: &Context<'_>,
        workflow_id: async_graphql::ID,
        input: UpdateWorkflowInput,
    ) -> FieldResult<UpdateWorkflowPayload> {
        ensure_write_permission(ctx)?;
        Ok(self
            .workflow_mutation
            .update_workflow(
                workflow_id.as_str(),
                input.name.as_deref(),
                input.description.as_deref(),
                input.enabled,
                input.steps.as_deref(),
            )
            .await)
    }

    async fn delete_workflow(
        &self,
        ctx: &Context<'_>,
        workflow_id: async_graphql::ID,
    ) -> FieldResult<DeleteWorkflowPayload> {
        ensure_write_permission(ctx)?;
        Ok(self
            .workflow_mutation
            .delete_workflow(workflow_id.as_str())
            .await)
    }

    async fn start_workflow_instance(
        &self,
        ctx: &Context<'_>,
        input: StartInstanceInput,
    ) -> FieldResult<StartInstancePayload> {
        ensure_write_permission(ctx)?;
        Ok(self
            .workflow_mutation
            .start_instance(
                &input.workflow_id,
                &input.title,
                &input.initiator_id,
                input.context_json.as_deref(),
            )
            .await)
    }

    async fn cancel_workflow_instance(
        &self,
        ctx: &Context<'_>,
        instance_id: async_graphql::ID,
        reason: Option<String>,
    ) -> FieldResult<CancelInstancePayload> {
        ensure_write_permission(ctx)?;
        Ok(self
            .workflow_mutation
            .cancel_instance(instance_id.as_str(), reason.as_deref())
            .await)
    }

    async fn reassign_task(
        &self,
        ctx: &Context<'_>,
        input: ReassignTaskInput,
    ) -> FieldResult<ReassignTaskPayload> {
        ensure_write_permission(ctx)?;
        Ok(self
            .workflow_mutation
            .reassign_task(
                &input.task_id,
                &input.new_assignee_id,
                input.reason.as_deref(),
                &input.actor_id,
            )
            .await)
    }

    async fn approve_task(
        &self,
        ctx: &Context<'_>,
        input: TaskDecisionInput,
    ) -> FieldResult<ApproveTaskPayload> {
        ensure_write_permission(ctx)?;
        Ok(self
            .workflow_mutation
            .approve_task(&input.task_id, &input.actor_id, input.comment.as_deref())
            .await)
    }

    async fn reject_task(
        &self,
        ctx: &Context<'_>,
        input: TaskDecisionInput,
    ) -> FieldResult<RejectTaskPayload> {
        ensure_write_permission(ctx)?;
        Ok(self
            .workflow_mutation
            .reject_task(&input.task_id, &input.actor_id, input.comment.as_deref())
            .await)
    }
}

// --- Subscription ---

pub struct SubscriptionRoot {
    pub subscription: Arc<SubscriptionResolver>,
}

#[Subscription]
impl SubscriptionRoot {
    #[graphql(name = "configChanged")]
    async fn config_changed(
        &self,
        _ctx: &Context<'_>,
        #[graphql(default)] namespaces: Vec<String>,
    ) -> impl Stream<Item = ConfigEntry> {
        self.subscription.watch_config(namespaces).await
    }

    #[graphql(name = "tenantUpdated")]
    async fn tenant_updated(
        &self,
        _ctx: &Context<'_>,
        tenant_id: async_graphql::ID,
    ) -> impl Stream<Item = Tenant> {
        self.subscription
            .watch_tenant_updated(tenant_id.to_string())
            .await
    }

    #[graphql(name = "featureFlagChanged")]
    async fn feature_flag_changed(
        &self,
        _ctx: &Context<'_>,
        key: String,
    ) -> impl Stream<Item = FeatureFlag> {
        self.subscription.watch_feature_flag_changed(key).await
    }
}

pub type AppSchema = Schema<QueryRoot, MutationRoot, SubscriptionRoot>;

/// アプリケーション共有状態。GraphQL スキーマと Prometheus メトリクスを保持する。
#[derive(Clone)]
pub struct AppState {
    pub schema: AppSchema,
    pub metrics: Arc<k1s0_telemetry::metrics::Metrics>,
    pub query_timeout: std::time::Duration,
    pub jwks_verifier: Arc<JwksVerifier>,
    pub tenant_client: Arc<TenantGrpcClient>,
    pub feature_flag_client: Arc<FeatureFlagGrpcClient>,
    pub config_client: Arc<ConfigGrpcClient>,
    pub navigation_client: Arc<NavigationGrpcClient>,
    pub service_catalog_client: Arc<ServiceCatalogGrpcClient>,
    pub auth_client: Arc<AuthGrpcClient>,
    pub session_client: Arc<SessionGrpcClient>,
    pub vault_client: Arc<VaultGrpcClient>,
    pub scheduler_client: Arc<SchedulerGrpcClient>,
    pub notification_client: Arc<NotificationGrpcClient>,
    pub workflow_client: Arc<WorkflowGrpcClient>,
    pub tenant_loader: Arc<DataLoader<TenantLoader>>,
    pub flag_loader: Arc<DataLoader<FeatureFlagLoader>>,
    pub config_loader: Arc<DataLoader<ConfigLoader>>,
}

pub fn router(
    jwks_verifier: Arc<JwksVerifier>,
    clients: GatewayClients,
    resolvers: GatewayResolvers,
    graphql_cfg: GraphQLConfig,
    metrics: Arc<k1s0_telemetry::metrics::Metrics>,
) -> Router {
    let mut builder = Schema::build(
        QueryRoot {
            tenant_query: resolvers.tenant_query,
            feature_flag_query: resolvers.feature_flag_query,
            config_query: resolvers.config_query,
            navigation_query: resolvers.navigation_query,
            service_catalog_query: resolvers.service_catalog_query,
            auth_query: resolvers.auth_query,
            session_query: resolvers.session_query,
            vault_query: resolvers.vault_query,
            scheduler_query: resolvers.scheduler_query,
            notification_query: resolvers.notification_query,
            workflow_query: resolvers.workflow_query,
        },
        MutationRoot {
            tenant_mutation: resolvers.tenant_mutation,
            feature_flag_client: clients.feature_flag.clone(),
            service_catalog_mutation: resolvers.service_catalog_mutation,
            auth_mutation: resolvers.auth_mutation,
            session_mutation: resolvers.session_mutation,
            vault_mutation: resolvers.vault_mutation,
            scheduler_mutation: resolvers.scheduler_mutation,
            notification_mutation: resolvers.notification_mutation,
            workflow_mutation: resolvers.workflow_mutation,
        },
        SubscriptionRoot {
            subscription: resolvers.subscription,
        },
    )
    .limit_depth(graphql_cfg.max_depth as usize)
    .limit_complexity(graphql_cfg.max_complexity as usize);

    if !graphql_cfg.introspection {
        builder = builder.disable_introspection();
    }

    let schema = builder.finish();

    let query_timeout = std::time::Duration::from_secs(graphql_cfg.query_timeout_seconds as u64);
    let tenant_loader = Arc::new(DataLoader::new(
        TenantLoader {
            client: clients.tenant.clone(),
        },
        tokio::spawn,
    ));
    let flag_loader = Arc::new(DataLoader::new(
        FeatureFlagLoader {
            client: clients.feature_flag.clone(),
        },
        tokio::spawn,
    ));
    let config_loader = Arc::new(DataLoader::new(
        ConfigLoader {
            client: clients.config.clone(),
        },
        tokio::spawn,
    ));

    let app_state = AppState {
        schema: schema.clone(),
        metrics,
        query_timeout,
        jwks_verifier: jwks_verifier.clone(),
        tenant_client: clients.tenant,
        feature_flag_client: clients.feature_flag,
        config_client: clients.config,
        navigation_client: clients.navigation,
        service_catalog_client: clients.service_catalog,
        auth_client: clients.auth,
        session_client: clients.session,
        vault_client: clients.vault,
        scheduler_client: clients.scheduler,
        notification_client: clients.notification,
        workflow_client: clients.workflow,
        tenant_loader,
        flag_loader,
        config_loader,
    };

    let graphql_post = post(graphql_handler).layer(AuthMiddlewareLayer::new(jwks_verifier));

    let mut router = Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/metrics", get(metrics_handler))
        .route("/graphql", graphql_post)
        .route("/graphql/ws", get(graphql_ws_handler))
        .with_state(app_state);

    // 開発環境のみ Playground を有効化
    if graphql_cfg.playground {
        router = router.route("/graphql", get(graphql_playground));
    }

    router
}

async fn graphql_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    req: GraphQLRequest,
) -> impl IntoResponse {
    let request = req
        .into_inner()
        .data(GraphqlContext {
            user_id: claims.sub.clone(),
            roles: claims.roles(),
            request_id: uuid::Uuid::new_v4().to_string(),
            tenant_loader: state.tenant_loader.clone(),
            flag_loader: state.flag_loader.clone(),
            config_loader: state.config_loader.clone(),
        })
        .data(claims);
    match tokio::time::timeout(state.query_timeout, state.schema.execute(request)).await {
        Ok(resp) => GraphQLResponse::from(resp).into_response(),
        Err(_) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "data": null,
                "errors": [{
                    "message": "query execution timed out",
                    "extensions": { "code": "TIMEOUT" }
                }]
            })),
        )
            .into_response(),
    }
}

async fn graphql_playground() -> impl IntoResponse {
    Html(async_graphql::http::playground_source(
        async_graphql::http::GraphQLPlaygroundConfig::new("/graphql")
            .subscription_endpoint("/graphql/ws"),
    ))
}

async fn healthz() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok"}))
}

async fn readyz(State(state): State<AppState>) -> impl IntoResponse {
    let tenant_status = match state.tenant_client.list_tenants(1, 1).await {
        Ok(_) => "ok".to_string(),
        Err(e) => format!("error: {}", e),
    };
    let featureflag_status = match state.feature_flag_client.list_flags(None).await {
        Ok(_) => "ok".to_string(),
        Err(e) => format!("error: {}", e),
    };
    let config_status = match state
        .config_client
        .get_config("__readyz__", "__readyz__")
        .await
    {
        Ok(_) => "ok".to_string(),
        Err(e) => format!("error: {}", e),
    };
    let navigation_status = match state.navigation_client.get_navigation("").await {
        Ok(_) => "ok".to_string(),
        Err(e) => format!("error: {}", e),
    };
    let service_catalog_status = match state
        .service_catalog_client
        .list_services(1, 1, None, None, None)
        .await
    {
        Ok(_) => "ok".to_string(),
        Err(e) => format!("error: {}", e),
    };
    let auth_status = match state.auth_client.health_check().await {
        Ok(_) => "ok".to_string(),
        Err(e) => format!("error: {}", e),
    };
    let session_status = match state.session_client.health_check().await {
        Ok(_) => "ok".to_string(),
        Err(e) => format!("error: {}", e),
    };
    let vault_status = match state.vault_client.health_check().await {
        Ok(_) => "ok".to_string(),
        Err(e) => format!("error: {}", e),
    };
    let scheduler_status = match state.scheduler_client.health_check().await {
        Ok(_) => "ok".to_string(),
        Err(e) => format!("error: {}", e),
    };
    let notification_status = match state.notification_client.health_check().await {
        Ok(_) => "ok".to_string(),
        Err(e) => format!("error: {}", e),
    };
    let workflow_status = match state.workflow_client.health_check().await {
        Ok(_) => "ok".to_string(),
        Err(e) => format!("error: {}", e),
    };

    let ready = tenant_status == "ok"
        && featureflag_status == "ok"
        && config_status == "ok"
        && navigation_status == "ok"
        && service_catalog_status == "ok"
        && auth_status == "ok"
        && session_status == "ok"
        && vault_status == "ok"
        && scheduler_status == "ok"
        && notification_status == "ok"
        && workflow_status == "ok";
    let status_code = if ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        status_code,
        Json(serde_json::json!({
            "status": if ready { "ready" } else { "not_ready" },
            "checks": {
                "tenant_grpc": tenant_status,
                "featureflag_grpc": featureflag_status,
                "config_grpc": config_status,
                "navigation_grpc": navigation_status,
                "service_catalog_grpc": service_catalog_status,
                "auth_grpc": auth_status,
                "session_grpc": session_status,
                "vault_grpc": vault_status,
                "scheduler_grpc": scheduler_status,
                "notification_grpc": notification_status,
                "workflow_grpc": workflow_status,
            }
        })),
    )
}

async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    let body = state.metrics.gather_metrics();
    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
        body,
    )
}

async fn graphql_ws_handler(
    State(state): State<AppState>,
    protocol: GraphQLProtocol,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    let schema = state.schema.clone();
    let verifier = state.jwks_verifier.clone();
    let tenant_loader = state.tenant_loader.clone();
    let flag_loader = state.flag_loader.clone();
    let config_loader = state.config_loader.clone();

    ws.protocols(async_graphql::http::ALL_WEBSOCKET_PROTOCOLS)
        .on_upgrade(move |socket| async move {
            GraphQLWebSocket::new(socket, schema, protocol)
                .on_connection_init(move |payload| {
                    let verifier = verifier.clone();
                    let tenant_loader = tenant_loader.clone();
                    let flag_loader = flag_loader.clone();
                    let config_loader = config_loader.clone();
                    async move {
                        let token = extract_bearer_token_from_connection_init(&payload)
                            .ok_or_else(|| {
                                gql_error(
                                    CODE_VALIDATION,
                                    "missing bearer token in connection_init payload",
                                )
                            })?;

                        let claims = verifier.verify_token(&token).await.map_err(|_| {
                            gql_error(CODE_FORBIDDEN, "invalid or expired JWT token")
                        })?;

                        let mut data = Data::default();
                        data.insert(GraphqlContext {
                            user_id: claims.sub.clone(),
                            roles: claims.roles(),
                            request_id: uuid::Uuid::new_v4().to_string(),
                            tenant_loader,
                            flag_loader,
                            config_loader,
                        });
                        data.insert(claims);
                        Ok(data)
                    }
                })
                .serve()
                .await;
        })
}

fn extract_bearer_token_from_connection_init(payload: &serde_json::Value) -> Option<String> {
    fn normalize(v: &str) -> String {
        v.trim().to_ascii_lowercase()
    }

    fn pick_token(value: &serde_json::Value) -> Option<String> {
        let token = value.as_str()?.trim();
        if token.is_empty() {
            return None;
        }
        if let Some(bearer) = token
            .strip_prefix("Bearer ")
            .or_else(|| token.strip_prefix("bearer "))
        {
            let trimmed = bearer.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        } else {
            Some(token.to_string())
        }
    }

    let obj = payload.as_object()?;

    for (key, value) in obj {
        let key_norm = normalize(key);
        if matches!(
            key_norm.as_str(),
            "authorization" | "authtoken" | "token" | "bearer_token"
        ) {
            if let Some(token) = pick_token(value) {
                return Some(token);
            }
        }
    }

    if let Some(headers) = obj.get("headers").and_then(|v| v.as_object()) {
        for (key, value) in headers {
            let key_norm = normalize(key);
            if key_norm == "authorization" {
                if let Some(token) = pick_token(value) {
                    return Some(token);
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::extract_bearer_token_from_connection_init;
    use serde_json::json;

    #[test]
    fn extracts_token_from_authorization_bearer() {
        let payload = json!({
            "authorization": "Bearer abc.def.ghi"
        });
        assert_eq!(
            extract_bearer_token_from_connection_init(&payload).as_deref(),
            Some("abc.def.ghi")
        );
    }

    #[test]
    fn extracts_token_from_headers_authorization() {
        let payload = json!({
            "headers": {
                "Authorization": "bearer token-123"
            }
        });
        assert_eq!(
            extract_bearer_token_from_connection_init(&payload).as_deref(),
            Some("token-123")
        );
    }

    #[test]
    fn returns_none_when_token_missing() {
        let payload = json!({
            "headers": {
                "x-request-id": "req-1"
            }
        });
        assert!(extract_bearer_token_from_connection_init(&payload).is_none());
    }
}

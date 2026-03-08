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
    ConfigEntry, CreateTenantPayload, FeatureFlag, SetFeatureFlagPayload, Tenant, TenantConnection,
    TenantStatus, UpdateTenantPayload, UserError,
};
use crate::infra::auth::JwksVerifier;
use crate::infra::config::GraphQLConfig;
use crate::infra::grpc::{ConfigGrpcClient, FeatureFlagGrpcClient, TenantGrpcClient};
use crate::usecase::{
    ConfigQueryResolver, FeatureFlagQueryResolver, SubscriptionResolver, TenantMutationResolver,
    TenantQueryResolver,
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

fn has_write_role(roles: &[String]) -> bool {
    roles
        .iter()
        .any(|r| r == "sys_admin" || r == "sys_operator")
}

fn ensure_write_permission(ctx: &Context<'_>) -> FieldResult<()> {
    let roles = if let Ok(gql_ctx) = ctx.data::<GraphqlContext>() {
        gql_ctx.roles.clone()
    } else if let Ok(claims) = ctx.data::<Claims>() {
        claims.roles()
    } else {
        vec![]
    };

    if has_write_role(&roles) {
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

// --- Query ---

pub struct QueryRoot {
    pub tenant_query: Arc<TenantQueryResolver>,
    pub feature_flag_query: Arc<FeatureFlagQueryResolver>,
    pub config_query: Arc<ConfigQueryResolver>,
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
}

// --- Mutation ---

pub struct MutationRoot {
    pub tenant_mutation: Arc<TenantMutationResolver>,
    pub feature_flag_client: Arc<FeatureFlagGrpcClient>,
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
    pub tenant_loader: Arc<DataLoader<TenantLoader>>,
    pub flag_loader: Arc<DataLoader<FeatureFlagLoader>>,
    pub config_loader: Arc<DataLoader<ConfigLoader>>,
}

pub fn router(
    jwks_verifier: Arc<JwksVerifier>,
    tenant_query: Arc<TenantQueryResolver>,
    feature_flag_query: Arc<FeatureFlagQueryResolver>,
    config_query: Arc<ConfigQueryResolver>,
    tenant_mutation: Arc<TenantMutationResolver>,
    subscription: Arc<SubscriptionResolver>,
    feature_flag_client: Arc<FeatureFlagGrpcClient>,
    tenant_client: Arc<TenantGrpcClient>,
    config_client: Arc<ConfigGrpcClient>,
    graphql_cfg: GraphQLConfig,
    metrics: Arc<k1s0_telemetry::metrics::Metrics>,
) -> Router {
    let mut builder = Schema::build(
        QueryRoot {
            tenant_query,
            feature_flag_query,
            config_query,
        },
        MutationRoot {
            tenant_mutation,
            feature_flag_client: feature_flag_client.clone(),
        },
        SubscriptionRoot { subscription },
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
            client: tenant_client.clone(),
        },
        tokio::spawn,
    ));
    let flag_loader = Arc::new(DataLoader::new(
        FeatureFlagLoader {
            client: feature_flag_client.clone(),
        },
        tokio::spawn,
    ));
    let config_loader = Arc::new(DataLoader::new(
        ConfigLoader {
            client: config_client.clone(),
        },
        tokio::spawn,
    ));

    let app_state = AppState {
        schema: schema.clone(),
        metrics,
        query_timeout,
        jwks_verifier: jwks_verifier.clone(),
        tenant_client: tenant_client.clone(),
        feature_flag_client: feature_flag_client.clone(),
        config_client: config_client.clone(),
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

    let ready = tenant_status == "ok" && featureflag_status == "ok" && config_status == "ok";
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

use std::sync::Arc;

use async_graphql::{Context, FieldResult, Object, Schema, Subscription};
use async_graphql::futures_util::Stream;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, get_service, post},
    Extension, Json, Router,
};

use crate::adapter::middleware::auth_middleware::{AuthMiddlewareLayer, Claims};
use crate::domain::model::{
    ConfigEntry, CreateTenantPayload, FeatureFlag, SetFeatureFlagPayload, Tenant,
    TenantConnection, TenantStatus, UpdateTenantPayload, UserError,
};
use crate::infra::auth::JwksVerifier;
use crate::infra::config::GraphQLConfig;
use crate::infra::grpc::FeatureFlagGrpcClient;
use crate::usecase::{
    ConfigQueryResolver, FeatureFlagQueryResolver, SubscriptionResolver, TenantMutationResolver,
    TenantQueryResolver,
};

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
            .map_err(|e| async_graphql::Error::new(e.to_string()))
    }

    async fn tenants(
        &self,
        _ctx: &Context<'_>,
        first: Option<i32>,
        after: Option<String>,
        last: Option<i32>,
        before: Option<String>,
    ) -> FieldResult<TenantConnection> {
        self.tenant_query
            .list_tenants(first, after, last, before)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))
    }

    async fn feature_flag(
        &self,
        _ctx: &Context<'_>,
        key: String,
    ) -> FieldResult<Option<FeatureFlag>> {
        self.feature_flag_query
            .get_feature_flag(&key)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))
    }

    async fn feature_flags(
        &self,
        _ctx: &Context<'_>,
        environment: Option<String>,
    ) -> FieldResult<Vec<FeatureFlag>> {
        self.feature_flag_query
            .list_feature_flags(environment.as_deref())
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))
    }

    async fn config(
        &self,
        _ctx: &Context<'_>,
        key: String,
    ) -> FieldResult<Option<ConfigEntry>> {
        self.config_query
            .get_config(&key)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))
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
        let claims = ctx.data::<Claims>().ok();
        let owner_user_id = claims.map(|c| c.sub.as_str()).unwrap_or("unknown");
        Ok(self
            .tenant_mutation
            .create_tenant(&input.name, owner_user_id)
            .await)
    }

    async fn update_tenant(
        &self,
        _ctx: &Context<'_>,
        id: async_graphql::ID,
        input: UpdateTenantInput,
    ) -> FieldResult<UpdateTenantPayload> {
        let status_str = input.status.map(|s| match s {
            TenantStatus::Active => "ACTIVE".to_string(),
            TenantStatus::Suspended => "SUSPENDED".to_string(),
            TenantStatus::Deleted => "DELETED".to_string(),
        });
        Ok(self
            .tenant_mutation
            .update_tenant(
                id.as_str(),
                input.name.as_deref(),
                status_str.as_deref(),
            )
            .await)
    }

    async fn set_feature_flag(
        &self,
        _ctx: &Context<'_>,
        key: String,
        input: SetFeatureFlagInput,
    ) -> FieldResult<SetFeatureFlagPayload> {
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
            Err(e) => Ok(SetFeatureFlagPayload {
                feature_flag: None,
                errors: vec![UserError {
                    field: None,
                    message: e.to_string(),
                }],
            }),
        }
    }
}

// --- Subscription ---

pub struct SubscriptionRoot {
    pub subscription: Arc<SubscriptionResolver>,
}

#[Subscription]
impl SubscriptionRoot {
    async fn config_changed(
        &self,
        _ctx: &Context<'_>,
        namespaces: Option<Vec<String>>,
    ) -> impl Stream<Item = ConfigEntry> {
        self.subscription
            .watch_config(namespaces.unwrap_or_default())
            .await
    }

    async fn tenant_updated(
        &self,
        _ctx: &Context<'_>,
        tenant_id: async_graphql::ID,
    ) -> impl Stream<Item = Tenant> {
        self.subscription
            .watch_tenant_updated(tenant_id.to_string())
    }

    async fn feature_flag_changed(
        &self,
        _ctx: &Context<'_>,
        key: String,
    ) -> impl Stream<Item = FeatureFlag> {
        self.subscription
            .watch_feature_flag_changed(key)
    }
}

pub type AppSchema = Schema<QueryRoot, MutationRoot, SubscriptionRoot>;

/// アプリケーション共有状態。GraphQL スキーマと Prometheus メトリクスを保持する。
#[derive(Clone)]
pub struct AppState {
    pub schema: AppSchema,
    pub metrics: Arc<k1s0_telemetry::metrics::Metrics>,
    pub query_timeout: std::time::Duration,
}

pub fn router(
    jwks_verifier: Arc<JwksVerifier>,
    tenant_query: Arc<TenantQueryResolver>,
    feature_flag_query: Arc<FeatureFlagQueryResolver>,
    config_query: Arc<ConfigQueryResolver>,
    tenant_mutation: Arc<TenantMutationResolver>,
    subscription: Arc<SubscriptionResolver>,
    feature_flag_client: Arc<FeatureFlagGrpcClient>,
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
            feature_flag_client,
        },
        SubscriptionRoot { subscription },
    )
    .limit_depth(graphql_cfg.max_depth as usize)
    .limit_complexity(graphql_cfg.max_complexity as usize);

    if !graphql_cfg.introspection {
        builder = builder.disable_introspection();
    }

    let schema = builder.finish();

    let query_timeout =
        std::time::Duration::from_secs(graphql_cfg.query_timeout_seconds as u64);

    let app_state = AppState {
        schema: schema.clone(),
        metrics,
        query_timeout,
    };

    let graphql_post = post(graphql_handler).layer(AuthMiddlewareLayer::new(jwks_verifier));

    let mut router = Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/metrics", get(metrics_handler))
        .route("/graphql", graphql_post)
        .route(
            "/graphql/ws",
            get_service(GraphQLSubscription::new(schema)),
        )
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
    let request = req.into_inner().data(claims);
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

async fn readyz() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ready"}))
}

async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    let body = state.metrics.gather_metrics();
    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
        body,
    )
}

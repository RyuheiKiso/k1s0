use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::RwLock;

use k1s0_graphql_gateway_server::domain::model::{
    decode_cursor, encode_cursor, CatalogService, CatalogServiceConnection, ConfigEntry,
    CreateTenantPayload, DeleteServicePayload, FeatureFlag, GuardType, MetadataEntry, Navigation,
    NavigationGuard, NavigationRoute, PageInfo, ParamType, RegisterServicePayload, RouteParam,
    ServiceHealth as CatalogServiceHealth, Tenant, TenantConnection, TenantEdge, TenantStatus,
    TransitionConfig as NavTransitionConfig, TransitionType, UpdateServicePayload,
    UpdateTenantPayload, UserError,
};

// ---------------------------------------------------------------------------
// Traits: Abstract gRPC client interfaces for testing resolver logic
// ---------------------------------------------------------------------------

#[async_trait]
trait TenantClient: Send + Sync {
    async fn get_tenant(&self, id: &str) -> anyhow::Result<Option<Tenant>>;
    async fn list_tenants(&self, page: i32, page_size: i32) -> anyhow::Result<TenantPage>;
    async fn create_tenant(&self, name: &str, owner_user_id: &str) -> anyhow::Result<Tenant>;
    async fn update_tenant(
        &self,
        id: &str,
        name: Option<&str>,
        status: Option<&str>,
    ) -> anyhow::Result<Tenant>;
}

#[async_trait]
trait ConfigClient: Send + Sync {
    async fn get_config(&self, namespace: &str, key: &str) -> anyhow::Result<Option<ConfigEntry>>;
}

#[async_trait]
trait FeatureFlagClient: Send + Sync {
    async fn get_flag(&self, key: &str) -> anyhow::Result<Option<FeatureFlag>>;
    async fn list_flags(&self, environment: Option<&str>) -> anyhow::Result<Vec<FeatureFlag>>;
}

#[async_trait]
trait NavigationClient: Send + Sync {
    async fn get_navigation(&self, bearer_token: &str) -> anyhow::Result<Navigation>;
}

#[allow(clippy::too_many_arguments)]
#[async_trait]
trait ServiceCatalogClient: Send + Sync {
    async fn get_service(&self, service_id: &str) -> anyhow::Result<Option<CatalogService>>;
    async fn list_services(
        &self,
        page: i32,
        page_size: i32,
        tier: Option<&str>,
        status: Option<&str>,
        search: Option<&str>,
    ) -> anyhow::Result<CatalogServiceConnection>;
    async fn register_service(
        &self,
        name: &str,
        display_name: &str,
        description: &str,
        tier: &str,
        version: &str,
        base_url: &str,
        grpc_endpoint: Option<&str>,
        health_url: &str,
        metadata: HashMap<String, String>,
    ) -> anyhow::Result<CatalogService>;
    async fn update_service(
        &self,
        service_id: &str,
        display_name: Option<&str>,
        description: Option<&str>,
        version: Option<&str>,
        base_url: Option<&str>,
        grpc_endpoint: Option<&str>,
        health_url: Option<&str>,
        metadata: HashMap<String, String>,
    ) -> anyhow::Result<CatalogService>;
    async fn delete_service(&self, service_id: &str) -> anyhow::Result<bool>;
    async fn health_check(
        &self,
        service_id: Option<&str>,
    ) -> anyhow::Result<Vec<CatalogServiceHealth>>;
}

/// gRPC TenantPage equivalent for test stub
struct TenantPage {
    nodes: Vec<Tenant>,
    total_count: i64,
    has_next: bool,
}

// ---------------------------------------------------------------------------
// Stub: In-memory TenantClient
// ---------------------------------------------------------------------------

struct StubTenantClient {
    tenants: RwLock<Vec<Tenant>>,
    should_fail: bool,
}

impl StubTenantClient {
    fn new() -> Self {
        Self {
            tenants: RwLock::new(Vec::new()),
            should_fail: false,
        }
    }

    fn failing() -> Self {
        Self {
            tenants: RwLock::new(Vec::new()),
            should_fail: true,
        }
    }

    async fn seed(&self, tenant: Tenant) {
        self.tenants.write().await.push(tenant);
    }
}

#[async_trait]
impl TenantClient for StubTenantClient {
    async fn get_tenant(&self, id: &str) -> anyhow::Result<Option<Tenant>> {
        if self.should_fail {
            return Err(anyhow::anyhow!("stub grpc error"));
        }
        let tenants = self.tenants.read().await;
        Ok(tenants.iter().find(|t| t.id == id).cloned())
    }

    async fn list_tenants(&self, page: i32, page_size: i32) -> anyhow::Result<TenantPage> {
        if self.should_fail {
            return Err(anyhow::anyhow!("stub grpc error"));
        }
        let tenants = self.tenants.read().await;
        let total = tenants.len() as i32;
        let start = ((page - 1) * page_size) as usize;
        let nodes: Vec<Tenant> = tenants
            .iter()
            .skip(start)
            .take(page_size as usize)
            .cloned()
            .collect();
        let has_next = (page * page_size) < total;
        Ok(TenantPage {
            nodes,
            total_count: i64::from(total),
            has_next,
        })
    }

    async fn create_tenant(&self, name: &str, _owner_user_id: &str) -> anyhow::Result<Tenant> {
        if self.should_fail {
            return Err(anyhow::anyhow!("stub grpc error"));
        }
        let tenant = Tenant {
            id: format!(
                "tenant_{}",
                uuid::Uuid::new_v4().to_string().replace('-', "")
            ),
            name: name.to_string(),
            status: TenantStatus::Active,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };
        self.tenants.write().await.push(tenant.clone());
        Ok(tenant)
    }

    async fn update_tenant(
        &self,
        id: &str,
        name: Option<&str>,
        status: Option<&str>,
    ) -> anyhow::Result<Tenant> {
        if self.should_fail {
            return Err(anyhow::anyhow!("stub grpc error"));
        }
        let mut tenants = self.tenants.write().await;
        let tenant = tenants
            .iter_mut()
            .find(|t| t.id == id)
            .ok_or_else(|| anyhow::anyhow!("tenant not found: {}", id))?;
        if let Some(new_name) = name {
            tenant.name = new_name.to_string();
        }
        if let Some(new_status) = status {
            tenant.status = TenantStatus::from(new_status.to_ascii_uppercase());
        }
        tenant.updated_at = chrono::Utc::now().to_rfc3339();
        Ok(tenant.clone())
    }
}

// ---------------------------------------------------------------------------
// Stub: In-memory ConfigClient
// ---------------------------------------------------------------------------

struct StubConfigClient {
    configs: RwLock<HashMap<String, ConfigEntry>>,
    should_fail: bool,
}

impl StubConfigClient {
    fn new() -> Self {
        Self {
            configs: RwLock::new(HashMap::new()),
            should_fail: false,
        }
    }

    fn failing() -> Self {
        Self {
            configs: RwLock::new(HashMap::new()),
            should_fail: true,
        }
    }

    async fn seed(&self, namespace: &str, key: &str, value: &str) {
        let full_key = format!("{}/{}", namespace, key);
        self.configs.write().await.insert(
            full_key.clone(),
            ConfigEntry {
                key: full_key,
                value: value.to_string(),
                updated_at: chrono::Utc::now().to_rfc3339(),
            },
        );
    }
}

#[async_trait]
impl ConfigClient for StubConfigClient {
    async fn get_config(&self, namespace: &str, key: &str) -> anyhow::Result<Option<ConfigEntry>> {
        if self.should_fail {
            return Err(anyhow::anyhow!("stub grpc error"));
        }
        let full_key = format!("{}/{}", namespace, key);
        let configs = self.configs.read().await;
        Ok(configs.get(&full_key).cloned())
    }
}

// ---------------------------------------------------------------------------
// Stub: In-memory FeatureFlagClient
// ---------------------------------------------------------------------------

struct StubFeatureFlagClient {
    flags: RwLock<Vec<FeatureFlag>>,
    should_fail: bool,
}

impl StubFeatureFlagClient {
    fn new() -> Self {
        Self {
            flags: RwLock::new(Vec::new()),
            should_fail: false,
        }
    }

    fn failing() -> Self {
        Self {
            flags: RwLock::new(Vec::new()),
            should_fail: true,
        }
    }

    async fn seed(&self, flag: FeatureFlag) {
        self.flags.write().await.push(flag);
    }
}

#[async_trait]
impl FeatureFlagClient for StubFeatureFlagClient {
    async fn get_flag(&self, key: &str) -> anyhow::Result<Option<FeatureFlag>> {
        if self.should_fail {
            return Err(anyhow::anyhow!("stub grpc error"));
        }
        let flags = self.flags.read().await;
        Ok(flags.iter().find(|f| f.key == key).cloned())
    }

    async fn list_flags(&self, environment: Option<&str>) -> anyhow::Result<Vec<FeatureFlag>> {
        if self.should_fail {
            return Err(anyhow::anyhow!("stub grpc error"));
        }
        let flags = self.flags.read().await;
        let mut result: Vec<FeatureFlag> = flags.iter().cloned().collect();
        if let Some(env) = environment {
            result.retain(|f| {
                f.target_environments.is_empty() || f.target_environments.iter().any(|e| e == env)
            });
        }
        Ok(result)
    }
}

// ---------------------------------------------------------------------------
// Stub: In-memory NavigationClient
// ---------------------------------------------------------------------------

struct StubNavigationClient {
    navigation: RwLock<Navigation>,
    should_fail: bool,
}

impl StubNavigationClient {
    fn new() -> Self {
        Self {
            navigation: RwLock::new(Navigation {
                routes: vec![],
                guards: vec![],
            }),
            should_fail: false,
        }
    }

    fn failing() -> Self {
        Self {
            navigation: RwLock::new(Navigation {
                routes: vec![],
                guards: vec![],
            }),
            should_fail: true,
        }
    }

    async fn set_navigation(&self, nav: Navigation) {
        *self.navigation.write().await = nav;
    }
}

#[async_trait]
impl NavigationClient for StubNavigationClient {
    async fn get_navigation(&self, _bearer_token: &str) -> anyhow::Result<Navigation> {
        if self.should_fail {
            return Err(anyhow::anyhow!("stub grpc error"));
        }
        Ok(self.navigation.read().await.clone())
    }
}

// ---------------------------------------------------------------------------
// Stub: In-memory ServiceCatalogClient
// ---------------------------------------------------------------------------

struct StubServiceCatalogClient {
    services: RwLock<Vec<CatalogService>>,
    health_statuses: RwLock<Vec<CatalogServiceHealth>>,
    should_fail: bool,
}

impl StubServiceCatalogClient {
    fn new() -> Self {
        Self {
            services: RwLock::new(Vec::new()),
            health_statuses: RwLock::new(Vec::new()),
            should_fail: false,
        }
    }

    fn failing() -> Self {
        Self {
            services: RwLock::new(Vec::new()),
            health_statuses: RwLock::new(Vec::new()),
            should_fail: true,
        }
    }

    async fn seed_service(&self, svc: CatalogService) {
        self.services.write().await.push(svc);
    }

    async fn seed_health(&self, health: CatalogServiceHealth) {
        self.health_statuses.write().await.push(health);
    }
}

#[async_trait]
impl ServiceCatalogClient for StubServiceCatalogClient {
    async fn get_service(&self, service_id: &str) -> anyhow::Result<Option<CatalogService>> {
        if self.should_fail {
            return Err(anyhow::anyhow!("stub grpc error"));
        }
        let services = self.services.read().await;
        Ok(services.iter().find(|s| s.id == service_id).cloned())
    }

    async fn list_services(
        &self,
        page: i32,
        page_size: i32,
        tier: Option<&str>,
        status: Option<&str>,
        _search: Option<&str>,
    ) -> anyhow::Result<CatalogServiceConnection> {
        if self.should_fail {
            return Err(anyhow::anyhow!("stub grpc error"));
        }
        let services = self.services.read().await;
        let mut filtered: Vec<CatalogService> = services.iter().cloned().collect();
        if let Some(t) = tier {
            filtered.retain(|s| s.tier == t);
        }
        if let Some(st) = status {
            filtered.retain(|s| s.status == st);
        }
        let total = filtered.len() as i32;
        let start = ((page - 1) * page_size) as usize;
        let page_services: Vec<CatalogService> = filtered
            .into_iter()
            .skip(start)
            .take(page_size as usize)
            .collect();
        let has_next = (page * page_size) < total;
        Ok(CatalogServiceConnection {
            services: page_services,
            total_count: total,
            has_next,
        })
    }

    async fn register_service(
        &self,
        name: &str,
        display_name: &str,
        description: &str,
        tier: &str,
        version: &str,
        base_url: &str,
        grpc_endpoint: Option<&str>,
        health_url: &str,
        metadata: HashMap<String, String>,
    ) -> anyhow::Result<CatalogService> {
        if self.should_fail {
            return Err(anyhow::anyhow!("stub grpc error"));
        }
        let svc = CatalogService {
            id: format!("svc_{}", uuid::Uuid::new_v4().to_string().replace('-', "")),
            name: name.to_string(),
            display_name: display_name.to_string(),
            description: description.to_string(),
            tier: tier.to_string(),
            version: version.to_string(),
            base_url: base_url.to_string(),
            grpc_endpoint: grpc_endpoint.map(|s| s.to_string()),
            health_url: health_url.to_string(),
            status: "ACTIVE".to_string(),
            metadata: metadata
                .into_iter()
                .map(|(k, v)| MetadataEntry { key: k, value: v })
                .collect(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };
        self.services.write().await.push(svc.clone());
        Ok(svc)
    }

    async fn update_service(
        &self,
        service_id: &str,
        display_name: Option<&str>,
        description: Option<&str>,
        version: Option<&str>,
        base_url: Option<&str>,
        grpc_endpoint: Option<&str>,
        health_url: Option<&str>,
        _metadata: HashMap<String, String>,
    ) -> anyhow::Result<CatalogService> {
        if self.should_fail {
            return Err(anyhow::anyhow!("stub grpc error"));
        }
        let mut services = self.services.write().await;
        let svc = services
            .iter_mut()
            .find(|s| s.id == service_id)
            .ok_or_else(|| anyhow::anyhow!("service not found: {}", service_id))?;
        if let Some(dn) = display_name {
            svc.display_name = dn.to_string();
        }
        if let Some(desc) = description {
            svc.description = desc.to_string();
        }
        if let Some(v) = version {
            svc.version = v.to_string();
        }
        if let Some(url) = base_url {
            svc.base_url = url.to_string();
        }
        if let Some(ep) = grpc_endpoint {
            svc.grpc_endpoint = Some(ep.to_string());
        }
        if let Some(url) = health_url {
            svc.health_url = url.to_string();
        }
        svc.updated_at = chrono::Utc::now().to_rfc3339();
        Ok(svc.clone())
    }

    async fn delete_service(&self, service_id: &str) -> anyhow::Result<bool> {
        if self.should_fail {
            return Err(anyhow::anyhow!("stub grpc error"));
        }
        let mut services = self.services.write().await;
        let len_before = services.len();
        services.retain(|s| s.id != service_id);
        Ok(services.len() < len_before)
    }

    async fn health_check(
        &self,
        service_id: Option<&str>,
    ) -> anyhow::Result<Vec<CatalogServiceHealth>> {
        if self.should_fail {
            return Err(anyhow::anyhow!("stub grpc error"));
        }
        let statuses = self.health_statuses.read().await;
        let result = if let Some(id) = service_id {
            statuses
                .iter()
                .filter(|h| h.service_id == id)
                .cloned()
                .collect()
        } else {
            statuses.iter().cloned().collect()
        };
        Ok(result)
    }
}

// ---------------------------------------------------------------------------
// Resolver logic wrappers (mirror actual resolver patterns using trait objects)
// ---------------------------------------------------------------------------

/// Mirrors ConfigQueryResolver.get_config logic
async fn resolve_get_config(
    client: &dyn ConfigClient,
    key: &str,
) -> anyhow::Result<Option<ConfigEntry>> {
    let parts: Vec<&str> = key.splitn(2, '/').collect();
    if parts.len() != 2 {
        return Ok(None);
    }
    client.get_config(parts[0], parts[1]).await
}

/// Mirrors TenantQueryResolver.get_tenant logic
async fn resolve_get_tenant(client: &dyn TenantClient, id: &str) -> anyhow::Result<Option<Tenant>> {
    client.get_tenant(id).await
}

/// Mirrors TenantQueryResolver.list_tenants with Relay cursor pagination
async fn resolve_list_tenants(
    client: &dyn TenantClient,
    first: Option<i32>,
    after: Option<String>,
) -> anyhow::Result<TenantConnection> {
    let page_size = first.unwrap_or(20);
    let offset = after
        .as_deref()
        .and_then(decode_cursor)
        .map(|o| o + 1)
        .unwrap_or(0);
    let page = if page_size > 0 {
        (offset as i32 / page_size) + 1
    } else {
        1
    };

    let raw = client.list_tenants(page, page_size).await?;

    let edges: Vec<TenantEdge> = raw
        .nodes
        .into_iter()
        .enumerate()
        .map(|(i, node)| TenantEdge {
            cursor: encode_cursor(offset + i),
            node,
        })
        .collect();

    let start_cursor = edges.first().map(|e| e.cursor.clone());
    let end_cursor = edges.last().map(|e| e.cursor.clone());

    Ok(TenantConnection {
        total_count: raw.total_count,
        page_info: PageInfo {
            has_next_page: raw.has_next,
            has_previous_page: offset > 0,
            start_cursor,
            end_cursor,
        },
        edges,
    })
}

/// Mirrors TenantMutationResolver.create_tenant logic
async fn resolve_create_tenant(
    client: &dyn TenantClient,
    name: &str,
    owner_user_id: &str,
) -> CreateTenantPayload {
    match client.create_tenant(name, owner_user_id).await {
        Ok(tenant) => CreateTenantPayload {
            tenant: Some(tenant),
            errors: vec![],
        },
        Err(e) => CreateTenantPayload {
            tenant: None,
            errors: vec![UserError {
                field: None,
                message: e.to_string(),
            }],
        },
    }
}

/// Mirrors TenantMutationResolver.update_tenant logic
async fn resolve_update_tenant(
    client: &dyn TenantClient,
    id: &str,
    name: Option<&str>,
    status: Option<&str>,
) -> UpdateTenantPayload {
    match client.update_tenant(id, name, status).await {
        Ok(tenant) => UpdateTenantPayload {
            tenant: Some(tenant),
            errors: vec![],
        },
        Err(e) => UpdateTenantPayload {
            tenant: None,
            errors: vec![UserError {
                field: None,
                message: e.to_string(),
            }],
        },
    }
}

/// Mirrors FeatureFlagQueryResolver.get_feature_flag logic
async fn resolve_get_feature_flag(
    client: &dyn FeatureFlagClient,
    key: &str,
) -> anyhow::Result<Option<FeatureFlag>> {
    client.get_flag(key).await
}

/// Mirrors FeatureFlagQueryResolver.list_feature_flags logic
async fn resolve_list_feature_flags(
    client: &dyn FeatureFlagClient,
    environment: Option<&str>,
) -> anyhow::Result<Vec<FeatureFlag>> {
    client.list_flags(environment).await
}

/// Mirrors NavigationQueryResolver.get_navigation logic
async fn resolve_get_navigation(
    client: &dyn NavigationClient,
    bearer_token: &str,
) -> anyhow::Result<Navigation> {
    client.get_navigation(bearer_token).await
}

/// Mirrors ServiceCatalogQueryResolver.get_service logic
async fn resolve_get_service(
    client: &dyn ServiceCatalogClient,
    service_id: &str,
) -> anyhow::Result<Option<CatalogService>> {
    client.get_service(service_id).await
}

/// Mirrors ServiceCatalogQueryResolver.list_services logic
async fn resolve_list_services(
    client: &dyn ServiceCatalogClient,
    first: Option<i32>,
    tier: Option<&str>,
    status: Option<&str>,
    search: Option<&str>,
) -> anyhow::Result<CatalogServiceConnection> {
    let page_size = first.unwrap_or(20);
    client
        .list_services(1, page_size, tier, status, search)
        .await
}

/// Mirrors ServiceCatalogMutationResolver.register_service logic
#[allow(clippy::too_many_arguments)]
async fn resolve_register_service(
    client: &dyn ServiceCatalogClient,
    name: &str,
    display_name: &str,
    description: &str,
    tier: &str,
    version: &str,
    base_url: &str,
    grpc_endpoint: Option<&str>,
    health_url: &str,
    metadata: HashMap<String, String>,
) -> RegisterServicePayload {
    match client
        .register_service(
            name,
            display_name,
            description,
            tier,
            version,
            base_url,
            grpc_endpoint,
            health_url,
            metadata,
        )
        .await
    {
        Ok(service) => RegisterServicePayload {
            service: Some(service),
            errors: vec![],
        },
        Err(e) => RegisterServicePayload {
            service: None,
            errors: vec![UserError {
                field: None,
                message: e.to_string(),
            }],
        },
    }
}

/// Mirrors ServiceCatalogMutationResolver.update_service logic
#[allow(clippy::too_many_arguments)]
async fn resolve_update_service(
    client: &dyn ServiceCatalogClient,
    service_id: &str,
    display_name: Option<&str>,
    description: Option<&str>,
    version: Option<&str>,
    base_url: Option<&str>,
    grpc_endpoint: Option<&str>,
    health_url: Option<&str>,
    metadata: HashMap<String, String>,
) -> UpdateServicePayload {
    match client
        .update_service(
            service_id,
            display_name,
            description,
            version,
            base_url,
            grpc_endpoint,
            health_url,
            metadata,
        )
        .await
    {
        Ok(service) => UpdateServicePayload {
            service: Some(service),
            errors: vec![],
        },
        Err(e) => UpdateServicePayload {
            service: None,
            errors: vec![UserError {
                field: None,
                message: e.to_string(),
            }],
        },
    }
}

/// Mirrors ServiceCatalogMutationResolver.delete_service logic
async fn resolve_delete_service(
    client: &dyn ServiceCatalogClient,
    service_id: &str,
) -> DeleteServicePayload {
    match client.delete_service(service_id).await {
        Ok(success) => DeleteServicePayload {
            success,
            errors: vec![],
        },
        Err(e) => DeleteServicePayload {
            success: false,
            errors: vec![UserError {
                field: None,
                message: e.to_string(),
            }],
        },
    }
}

/// Mirrors ServiceCatalogQueryResolver.health_check logic
async fn resolve_health_check(
    client: &dyn ServiceCatalogClient,
    service_id: Option<&str>,
) -> anyhow::Result<Vec<CatalogServiceHealth>> {
    client.health_check(service_id).await
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn sample_tenant(id: &str, name: &str) -> Tenant {
    Tenant {
        id: id.to_string(),
        name: name.to_string(),
        status: TenantStatus::Active,
        created_at: "2025-01-01T00:00:00Z".to_string(),
        updated_at: "2025-01-01T00:00:00Z".to_string(),
    }
}

fn sample_flag(key: &str, enabled: bool) -> FeatureFlag {
    FeatureFlag {
        key: key.to_string(),
        name: format!("Flag {}", key),
        enabled,
        rollout_percentage: if enabled { 100 } else { 0 },
        target_environments: vec!["production".to_string()],
    }
}

fn sample_catalog_service(id: &str, name: &str) -> CatalogService {
    CatalogService {
        id: id.to_string(),
        name: name.to_string(),
        display_name: format!("{} Service", name),
        description: format!("Description for {}", name),
        tier: "system".to_string(),
        version: "1.0.0".to_string(),
        base_url: format!("http://{}.example.com", name),
        grpc_endpoint: Some(format!("http://{}.example.com:50051", name)),
        health_url: format!("http://{}.example.com/healthz", name),
        status: "ACTIVE".to_string(),
        metadata: vec![],
        created_at: "2025-01-01T00:00:00Z".to_string(),
        updated_at: "2025-01-01T00:00:00Z".to_string(),
    }
}

fn sample_health(service_id: &str, service_name: &str, status: &str) -> CatalogServiceHealth {
    CatalogServiceHealth {
        service_id: service_id.to_string(),
        service_name: service_name.to_string(),
        status: status.to_string(),
        response_time_ms: Some(42),
        error_message: None,
        checked_at: "2025-01-01T00:00:00Z".to_string(),
    }
}

// ===========================================================================
// Domain Model: cursor encoding/decoding
// ===========================================================================

mod cursor_tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrip() {
        let cursor = encode_cursor(42);
        let decoded = decode_cursor(&cursor);
        assert_eq!(decoded, Some(42));
    }

    #[test]
    fn encode_decode_zero_offset() {
        let cursor = encode_cursor(0);
        let decoded = decode_cursor(&cursor);
        assert_eq!(decoded, Some(0));
    }

    #[test]
    fn encode_decode_large_offset() {
        let cursor = encode_cursor(999999);
        let decoded = decode_cursor(&cursor);
        assert_eq!(decoded, Some(999999));
    }

    #[test]
    fn decode_invalid_cursor_returns_none() {
        assert_eq!(decode_cursor("not-valid-base64!!!"), None);
    }

    #[test]
    fn decode_valid_base64_but_wrong_format_returns_none() {
        use base64::Engine;
        let encoded = base64::engine::general_purpose::STANDARD.encode("not_a_cursor:42");
        assert_eq!(decode_cursor(&encoded), None);
    }

    #[test]
    fn decode_empty_string_returns_none() {
        assert_eq!(decode_cursor(""), None);
    }
}

// ===========================================================================
// Domain Model: TenantStatus conversion
// ===========================================================================

mod tenant_status_tests {
    use super::*;

    #[test]
    fn from_active() {
        assert_eq!(
            TenantStatus::from("ACTIVE".to_string()),
            TenantStatus::Active
        );
    }

    #[test]
    fn from_suspended() {
        assert_eq!(
            TenantStatus::from("SUSPENDED".to_string()),
            TenantStatus::Suspended
        );
    }

    #[test]
    fn from_deleted() {
        assert_eq!(
            TenantStatus::from("DELETED".to_string()),
            TenantStatus::Deleted
        );
    }

    #[test]
    fn from_unknown_defaults_to_active() {
        assert_eq!(
            TenantStatus::from("UNKNOWN".to_string()),
            TenantStatus::Active
        );
    }

    #[test]
    fn from_empty_defaults_to_active() {
        assert_eq!(TenantStatus::from(String::new()), TenantStatus::Active);
    }
}

// ===========================================================================
// ConfigQuery Resolver
// ===========================================================================

mod config_query {
    use super::*;

    #[tokio::test]
    async fn success_returns_config_entry() {
        let client = StubConfigClient::new();
        client.seed("app", "max_retries", "3").await;

        let result = resolve_get_config(&client, "app/max_retries").await;
        assert!(result.is_ok());

        let entry = result.unwrap();
        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.key, "app/max_retries");
        assert_eq!(entry.value, "3");
    }

    #[tokio::test]
    async fn returns_none_for_missing_config() {
        let client = StubConfigClient::new();

        let result = resolve_get_config(&client, "app/missing_key").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn returns_none_for_invalid_key_format() {
        let client = StubConfigClient::new();

        // No slash separator -> parts.len() != 2
        let result = resolve_get_config(&client, "invalid-key-no-slash").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn handles_key_with_multiple_slashes() {
        let client = StubConfigClient::new();
        client.seed("app", "nested/deep/key", "value").await;

        // splitn(2, '/') should keep everything after first slash
        let result = resolve_get_config(&client, "app/nested/deep/key").await;
        assert!(result.is_ok());
        let entry = result.unwrap();
        assert!(entry.is_some());
    }

    #[tokio::test]
    async fn error_propagation() {
        let client = StubConfigClient::failing();

        let result = resolve_get_config(&client, "app/key").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("stub grpc error"));
    }
}

// ===========================================================================
// TenantQuery Resolver
// ===========================================================================

mod tenant_query {
    use super::*;

    #[tokio::test]
    async fn get_tenant_success() {
        let client = StubTenantClient::new();
        client.seed(sample_tenant("t-001", "Acme Corp")).await;

        let result = resolve_get_tenant(&client, "t-001").await;
        assert!(result.is_ok());

        let tenant = result.unwrap();
        assert!(tenant.is_some());
        let tenant = tenant.unwrap();
        assert_eq!(tenant.id, "t-001");
        assert_eq!(tenant.name, "Acme Corp");
    }

    #[tokio::test]
    async fn get_tenant_not_found() {
        let client = StubTenantClient::new();

        let result = resolve_get_tenant(&client, "nonexistent").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn get_tenant_error_propagation() {
        let client = StubTenantClient::failing();

        let result = resolve_get_tenant(&client, "t-001").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn list_tenants_first_page() {
        let client = StubTenantClient::new();
        for i in 0..5 {
            client
                .seed(sample_tenant(
                    &format!("t-{:03}", i),
                    &format!("Tenant {}", i),
                ))
                .await;
        }

        let result = resolve_list_tenants(&client, Some(3), None).await;
        assert!(result.is_ok());

        let conn = result.unwrap();
        assert_eq!(conn.edges.len(), 3);
        assert_eq!(conn.total_count, 5);
        assert!(conn.page_info.has_next_page);
        assert!(!conn.page_info.has_previous_page);
        assert!(conn.page_info.start_cursor.is_some());
        assert!(conn.page_info.end_cursor.is_some());
    }

    #[tokio::test]
    async fn list_tenants_with_cursor_pagination() {
        let client = StubTenantClient::new();
        for i in 0..5 {
            client
                .seed(sample_tenant(
                    &format!("t-{:03}", i),
                    &format!("Tenant {}", i),
                ))
                .await;
        }

        // Get first page
        let first_page = resolve_list_tenants(&client, Some(2), None).await.unwrap();
        assert_eq!(first_page.edges.len(), 2);
        let after_cursor = first_page.page_info.end_cursor.clone();

        // Get second page using cursor
        let second_page = resolve_list_tenants(&client, Some(2), after_cursor)
            .await
            .unwrap();
        assert_eq!(second_page.edges.len(), 2);
        assert!(second_page.page_info.has_previous_page);
    }

    #[tokio::test]
    async fn list_tenants_default_page_size() {
        let client = StubTenantClient::new();
        client.seed(sample_tenant("t-001", "Test")).await;

        // None for first -> defaults to 20
        let result = resolve_list_tenants(&client, None, None).await;
        assert!(result.is_ok());
        let conn = result.unwrap();
        assert_eq!(conn.edges.len(), 1);
    }

    #[tokio::test]
    async fn list_tenants_empty() {
        let client = StubTenantClient::new();

        let result = resolve_list_tenants(&client, Some(20), None).await;
        assert!(result.is_ok());

        let conn = result.unwrap();
        assert!(conn.edges.is_empty());
        assert_eq!(conn.total_count, 0);
        assert!(!conn.page_info.has_next_page);
        assert!(conn.page_info.start_cursor.is_none());
        assert!(conn.page_info.end_cursor.is_none());
    }

    #[tokio::test]
    async fn list_tenants_error_propagation() {
        let client = StubTenantClient::failing();

        let result = resolve_list_tenants(&client, Some(20), None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn cursor_encodes_correct_offset() {
        let client = StubTenantClient::new();
        for i in 0..3 {
            client
                .seed(sample_tenant(&format!("t-{:03}", i), &format!("T{}", i)))
                .await;
        }

        let conn = resolve_list_tenants(&client, Some(3), None).await.unwrap();
        // First edge should encode offset 0, second offset 1, etc.
        assert_eq!(decode_cursor(&conn.edges[0].cursor), Some(0));
        assert_eq!(decode_cursor(&conn.edges[1].cursor), Some(1));
        assert_eq!(decode_cursor(&conn.edges[2].cursor), Some(2));
    }
}

// ===========================================================================
// TenantMutation Resolver
// ===========================================================================

mod tenant_mutation {
    use super::*;

    #[tokio::test]
    async fn create_tenant_success() {
        let client = StubTenantClient::new();

        let payload = resolve_create_tenant(&client, "New Corp", "user-001").await;
        assert!(payload.tenant.is_some());
        assert!(payload.errors.is_empty());

        let tenant = payload.tenant.unwrap();
        assert_eq!(tenant.name, "New Corp");
        assert_eq!(tenant.status, TenantStatus::Active);
    }

    #[tokio::test]
    async fn create_tenant_error_returns_user_error() {
        let client = StubTenantClient::failing();

        let payload = resolve_create_tenant(&client, "Bad Corp", "user-001").await;
        assert!(payload.tenant.is_none());
        assert_eq!(payload.errors.len(), 1);
        assert!(payload.errors[0].message.contains("stub grpc error"));
    }

    #[tokio::test]
    async fn update_tenant_name_success() {
        let client = StubTenantClient::new();
        client.seed(sample_tenant("t-001", "Old Name")).await;

        let payload = resolve_update_tenant(&client, "t-001", Some("New Name"), None).await;
        assert!(payload.tenant.is_some());
        assert!(payload.errors.is_empty());

        let tenant = payload.tenant.unwrap();
        assert_eq!(tenant.name, "New Name");
    }

    #[tokio::test]
    async fn update_tenant_status_success() {
        let client = StubTenantClient::new();
        client.seed(sample_tenant("t-001", "Corp")).await;

        let payload = resolve_update_tenant(&client, "t-001", None, Some("SUSPENDED")).await;
        assert!(payload.tenant.is_some());
        assert!(payload.errors.is_empty());

        let tenant = payload.tenant.unwrap();
        assert_eq!(tenant.status, TenantStatus::Suspended);
    }

    #[tokio::test]
    async fn update_tenant_not_found_returns_user_error() {
        let client = StubTenantClient::new();

        let payload = resolve_update_tenant(&client, "nonexistent", Some("New Name"), None).await;
        assert!(payload.tenant.is_none());
        assert_eq!(payload.errors.len(), 1);
        assert!(payload.errors[0].message.contains("tenant not found"));
    }

    #[tokio::test]
    async fn update_tenant_error_returns_user_error() {
        let client = StubTenantClient::failing();

        let payload = resolve_update_tenant(&client, "t-001", Some("New Name"), None).await;
        assert!(payload.tenant.is_none());
        assert_eq!(payload.errors.len(), 1);
        assert!(payload.errors[0].message.contains("stub grpc error"));
    }

    #[tokio::test]
    async fn update_tenant_both_name_and_status() {
        let client = StubTenantClient::new();
        client.seed(sample_tenant("t-001", "Old")).await;

        let payload =
            resolve_update_tenant(&client, "t-001", Some("Updated"), Some("SUSPENDED")).await;
        assert!(payload.tenant.is_some());
        let tenant = payload.tenant.unwrap();
        assert_eq!(tenant.name, "Updated");
        assert_eq!(tenant.status, TenantStatus::Suspended);
    }
}

// ===========================================================================
// FeatureFlagQuery Resolver
// ===========================================================================

mod feature_flag_query {
    use super::*;

    #[tokio::test]
    async fn get_flag_success() {
        let client = StubFeatureFlagClient::new();
        client.seed(sample_flag("dark-mode", true)).await;

        let result = resolve_get_feature_flag(&client, "dark-mode").await;
        assert!(result.is_ok());

        let flag = result.unwrap();
        assert!(flag.is_some());
        let flag = flag.unwrap();
        assert_eq!(flag.key, "dark-mode");
        assert!(flag.enabled);
        assert_eq!(flag.rollout_percentage, 100);
    }

    #[tokio::test]
    async fn get_flag_not_found() {
        let client = StubFeatureFlagClient::new();

        let result = resolve_get_feature_flag(&client, "nonexistent").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn get_flag_error_propagation() {
        let client = StubFeatureFlagClient::failing();

        let result = resolve_get_feature_flag(&client, "dark-mode").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn list_flags_all() {
        let client = StubFeatureFlagClient::new();
        client.seed(sample_flag("flag-a", true)).await;
        client.seed(sample_flag("flag-b", false)).await;

        let result = resolve_list_feature_flags(&client, None).await;
        assert!(result.is_ok());

        let flags = result.unwrap();
        assert_eq!(flags.len(), 2);
    }

    #[tokio::test]
    async fn list_flags_filter_by_environment() {
        let client = StubFeatureFlagClient::new();
        client.seed(sample_flag("prod-flag", true)).await;

        let staging_flag = FeatureFlag {
            key: "staging-flag".to_string(),
            name: "Staging Only".to_string(),
            enabled: true,
            rollout_percentage: 50,
            target_environments: vec!["staging".to_string()],
        };
        client.seed(staging_flag).await;

        // Filter for production
        let result = resolve_list_feature_flags(&client, Some("production")).await;
        assert!(result.is_ok());

        let flags = result.unwrap();
        assert_eq!(flags.len(), 1);
        assert_eq!(flags[0].key, "prod-flag");
    }

    #[tokio::test]
    async fn list_flags_empty_targets_matches_all_environments() {
        let client = StubFeatureFlagClient::new();
        let global_flag = FeatureFlag {
            key: "global-flag".to_string(),
            name: "Global".to_string(),
            enabled: true,
            rollout_percentage: 100,
            target_environments: vec![], // empty targets = matches all
        };
        client.seed(global_flag).await;

        let result = resolve_list_feature_flags(&client, Some("any-env")).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn list_flags_empty_result() {
        let client = StubFeatureFlagClient::new();

        let result = resolve_list_feature_flags(&client, None).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn list_flags_error_propagation() {
        let client = StubFeatureFlagClient::failing();

        let result = resolve_list_feature_flags(&client, None).await;
        assert!(result.is_err());
    }
}

// ===========================================================================
// Subscription (stream construction logic, no actual streaming)
// ===========================================================================

mod subscription {
    use super::*;

    // Subscription resolvers simply delegate to gRPC client watch methods.
    // We verify the resolver construction pattern works with our stubs.
    // Actual stream testing would require a running gRPC server, so we
    // test the model types used in stream events.

    #[test]
    fn config_entry_can_be_constructed() {
        let entry = ConfigEntry {
            key: "app/timeout".to_string(),
            value: "30".to_string(),
            updated_at: "2025-01-01T00:00:00Z".to_string(),
        };
        assert_eq!(entry.key, "app/timeout");
        assert_eq!(entry.value, "30");
    }

    #[test]
    fn tenant_can_be_constructed_for_stream_events() {
        let tenant = Tenant {
            id: "t-001".to_string(),
            name: "Stream Tenant".to_string(),
            status: TenantStatus::Active,
            created_at: "2025-01-01T00:00:00Z".to_string(),
            updated_at: "2025-06-01T00:00:00Z".to_string(),
        };
        assert_eq!(tenant.id, "t-001");
        assert_eq!(tenant.status, TenantStatus::Active);
    }

    #[test]
    fn feature_flag_can_be_constructed_for_stream_events() {
        let flag = FeatureFlag {
            key: "new-feature".to_string(),
            name: "New Feature".to_string(),
            enabled: true,
            rollout_percentage: 50,
            target_environments: vec!["staging".to_string(), "production".to_string()],
        };
        assert_eq!(flag.key, "new-feature");
        assert!(flag.enabled);
        assert_eq!(flag.rollout_percentage, 50);
        assert_eq!(flag.target_environments.len(), 2);
    }

    #[test]
    fn disabled_flag_has_zero_rollout() {
        let flag = FeatureFlag {
            key: "disabled".to_string(),
            name: "Disabled".to_string(),
            enabled: false,
            rollout_percentage: 0,
            target_environments: vec![],
        };
        assert!(!flag.enabled);
        assert_eq!(flag.rollout_percentage, 0);
    }
}

// ===========================================================================
// NavigationQuery Resolver
// ===========================================================================

mod navigation_query {
    use super::*;

    #[tokio::test]
    async fn get_navigation_returns_routes() {
        let client = StubNavigationClient::new();
        client
            .set_navigation(Navigation {
                routes: vec![NavigationRoute {
                    id: "route-1".to_string(),
                    path: "/dashboard".to_string(),
                    component_id: Some("DashboardPage".to_string()),
                    guard_ids: vec![],
                    children: vec![],
                    transition: None,
                    params: vec![],
                    redirect_to: None,
                }],
                guards: vec![],
            })
            .await;

        let result = resolve_get_navigation(&client, "token-123").await;
        assert!(result.is_ok());
        let nav = result.unwrap();
        assert_eq!(nav.routes.len(), 1);
        assert_eq!(nav.routes[0].id, "route-1");
        assert_eq!(nav.routes[0].path, "/dashboard");
        assert_eq!(
            nav.routes[0].component_id,
            Some("DashboardPage".to_string())
        );
    }

    #[tokio::test]
    async fn get_navigation_with_guards() {
        let client = StubNavigationClient::new();
        client
            .set_navigation(Navigation {
                routes: vec![NavigationRoute {
                    id: "route-1".to_string(),
                    path: "/admin".to_string(),
                    component_id: Some("AdminPage".to_string()),
                    guard_ids: vec!["guard-1".to_string()],
                    children: vec![],
                    transition: None,
                    params: vec![],
                    redirect_to: None,
                }],
                guards: vec![NavigationGuard {
                    id: "guard-1".to_string(),
                    guard_type: GuardType::AuthRequired,
                    redirect_to: Some("/login".to_string()),
                    roles: vec![],
                }],
            })
            .await;

        let result = resolve_get_navigation(&client, "").await;
        assert!(result.is_ok());
        let nav = result.unwrap();
        assert_eq!(nav.guards.len(), 1);
        assert_eq!(nav.guards[0].id, "guard-1");
        assert_eq!(nav.guards[0].guard_type, GuardType::AuthRequired);
        assert_eq!(nav.guards[0].redirect_to, Some("/login".to_string()));
        assert_eq!(nav.routes[0].guard_ids, vec!["guard-1".to_string()]);
    }

    #[tokio::test]
    async fn get_navigation_empty() {
        let client = StubNavigationClient::new();

        let result = resolve_get_navigation(&client, "").await;
        assert!(result.is_ok());
        let nav = result.unwrap();
        assert!(nav.routes.is_empty());
        assert!(nav.guards.is_empty());
    }

    #[tokio::test]
    async fn get_navigation_with_nested_routes() {
        let client = StubNavigationClient::new();
        client
            .set_navigation(Navigation {
                routes: vec![NavigationRoute {
                    id: "parent".to_string(),
                    path: "/settings".to_string(),
                    component_id: Some("SettingsLayout".to_string()),
                    guard_ids: vec![],
                    children: vec![
                        NavigationRoute {
                            id: "child-1".to_string(),
                            path: "/settings/profile".to_string(),
                            component_id: Some("ProfilePage".to_string()),
                            guard_ids: vec![],
                            children: vec![],
                            transition: None,
                            params: vec![],
                            redirect_to: None,
                        },
                        NavigationRoute {
                            id: "child-2".to_string(),
                            path: "/settings/security".to_string(),
                            component_id: Some("SecurityPage".to_string()),
                            guard_ids: vec![],
                            children: vec![],
                            transition: None,
                            params: vec![],
                            redirect_to: None,
                        },
                    ],
                    transition: None,
                    params: vec![],
                    redirect_to: None,
                }],
                guards: vec![],
            })
            .await;

        let result = resolve_get_navigation(&client, "").await;
        assert!(result.is_ok());
        let nav = result.unwrap();
        assert_eq!(nav.routes.len(), 1);
        assert_eq!(nav.routes[0].children.len(), 2);
        assert_eq!(nav.routes[0].children[0].id, "child-1");
        assert_eq!(nav.routes[0].children[1].id, "child-2");
    }

    #[tokio::test]
    async fn get_navigation_with_transitions() {
        let client = StubNavigationClient::new();
        client
            .set_navigation(Navigation {
                routes: vec![NavigationRoute {
                    id: "route-1".to_string(),
                    path: "/page".to_string(),
                    component_id: None,
                    guard_ids: vec![],
                    children: vec![],
                    transition: Some(NavTransitionConfig {
                        transition_type: TransitionType::Fade,
                        duration_ms: 300,
                    }),
                    params: vec![],
                    redirect_to: None,
                }],
                guards: vec![],
            })
            .await;

        let result = resolve_get_navigation(&client, "").await;
        assert!(result.is_ok());
        let nav = result.unwrap();
        let transition = nav.routes[0].transition.as_ref().unwrap();
        assert_eq!(transition.transition_type, TransitionType::Fade);
        assert_eq!(transition.duration_ms, 300);
    }

    #[tokio::test]
    async fn get_navigation_with_params() {
        let client = StubNavigationClient::new();
        client
            .set_navigation(Navigation {
                routes: vec![NavigationRoute {
                    id: "route-1".to_string(),
                    path: "/users/:id".to_string(),
                    component_id: Some("UserPage".to_string()),
                    guard_ids: vec![],
                    children: vec![],
                    transition: None,
                    params: vec![RouteParam {
                        name: "id".to_string(),
                        param_type: ParamType::UuidType,
                    }],
                    redirect_to: None,
                }],
                guards: vec![],
            })
            .await;

        let result = resolve_get_navigation(&client, "").await;
        assert!(result.is_ok());
        let nav = result.unwrap();
        assert_eq!(nav.routes[0].params.len(), 1);
        assert_eq!(nav.routes[0].params[0].name, "id");
        assert_eq!(nav.routes[0].params[0].param_type, ParamType::UuidType);
    }

    #[tokio::test]
    async fn get_navigation_error_propagation() {
        let client = StubNavigationClient::failing();

        let result = resolve_get_navigation(&client, "token").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("stub grpc error"));
    }

    #[test]
    fn guard_type_enum_values() {
        assert_eq!(GuardType::Unspecified, GuardType::Unspecified);
        assert_eq!(GuardType::AuthRequired, GuardType::AuthRequired);
        assert_eq!(GuardType::RoleRequired, GuardType::RoleRequired);
        assert_eq!(
            GuardType::RedirectIfAuthenticated,
            GuardType::RedirectIfAuthenticated
        );
        assert_ne!(GuardType::AuthRequired, GuardType::RoleRequired);
    }
}

// ===========================================================================
// ServiceCatalogQuery Resolver
// ===========================================================================

mod service_catalog_query {
    use super::*;

    #[tokio::test]
    async fn get_service_success() {
        let client = StubServiceCatalogClient::new();
        client
            .seed_service(sample_catalog_service("svc-001", "auth"))
            .await;

        let result = resolve_get_service(&client, "svc-001").await;
        assert!(result.is_ok());
        let svc = result.unwrap();
        assert!(svc.is_some());
        let svc = svc.unwrap();
        assert_eq!(svc.id, "svc-001");
        assert_eq!(svc.name, "auth");
        assert_eq!(svc.display_name, "auth Service");
    }

    #[tokio::test]
    async fn get_service_not_found() {
        let client = StubServiceCatalogClient::new();

        let result = resolve_get_service(&client, "nonexistent").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn get_service_error_propagation() {
        let client = StubServiceCatalogClient::failing();

        let result = resolve_get_service(&client, "svc-001").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("stub grpc error"));
    }

    #[tokio::test]
    async fn list_services_paginated() {
        let client = StubServiceCatalogClient::new();
        for i in 0..5 {
            client
                .seed_service(sample_catalog_service(
                    &format!("svc-{:03}", i),
                    &format!("service-{}", i),
                ))
                .await;
        }

        let result = resolve_list_services(&client, Some(3), None, None, None).await;
        assert!(result.is_ok());
        let conn = result.unwrap();
        assert_eq!(conn.services.len(), 3);
        assert_eq!(conn.total_count, 5);
        assert!(conn.has_next);
    }

    #[tokio::test]
    async fn list_services_filtered_by_tier() {
        let client = StubServiceCatalogClient::new();
        client
            .seed_service(sample_catalog_service("svc-001", "auth"))
            .await;
        let mut business_svc = sample_catalog_service("svc-002", "orders");
        business_svc.tier = "business".to_string();
        client.seed_service(business_svc).await;

        let result = resolve_list_services(&client, Some(20), Some("system"), None, None).await;
        assert!(result.is_ok());
        let conn = result.unwrap();
        assert_eq!(conn.services.len(), 1);
        assert_eq!(conn.services[0].name, "auth");
    }

    #[tokio::test]
    async fn list_services_empty() {
        let client = StubServiceCatalogClient::new();

        let result = resolve_list_services(&client, Some(20), None, None, None).await;
        assert!(result.is_ok());
        let conn = result.unwrap();
        assert!(conn.services.is_empty());
        assert_eq!(conn.total_count, 0);
    }

    #[tokio::test]
    async fn list_services_error_propagation() {
        let client = StubServiceCatalogClient::failing();

        let result = resolve_list_services(&client, Some(20), None, None, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn health_check_returns_status() {
        let client = StubServiceCatalogClient::new();
        client
            .seed_health(sample_health("svc-001", "auth", "HEALTHY"))
            .await;
        client
            .seed_health(sample_health("svc-002", "config", "UNHEALTHY"))
            .await;

        let result = resolve_health_check(&client, None).await;
        assert!(result.is_ok());
        let statuses = result.unwrap();
        assert_eq!(statuses.len(), 2);
        assert_eq!(statuses[0].service_id, "svc-001");
        assert_eq!(statuses[0].status, "HEALTHY");
        assert_eq!(statuses[1].status, "UNHEALTHY");
    }
}

// ===========================================================================
// ServiceCatalogMutation Resolver
// ===========================================================================

mod service_catalog_mutation {
    use super::*;

    #[tokio::test]
    async fn register_service_success() {
        let client = StubServiceCatalogClient::new();

        let payload = resolve_register_service(
            &client,
            "new-service",
            "New Service",
            "A new service",
            "system",
            "1.0.0",
            "http://new.example.com",
            Some("http://new.example.com:50051"),
            "http://new.example.com/healthz",
            HashMap::new(),
        )
        .await;
        assert!(payload.service.is_some());
        assert!(payload.errors.is_empty());
        let svc = payload.service.unwrap();
        assert_eq!(svc.name, "new-service");
        assert_eq!(svc.display_name, "New Service");
        assert_eq!(svc.status, "ACTIVE");
    }

    #[tokio::test]
    async fn register_service_error_returns_user_error() {
        let client = StubServiceCatalogClient::failing();

        let payload = resolve_register_service(
            &client,
            "fail-service",
            "Fail",
            "desc",
            "system",
            "1.0.0",
            "http://fail.example.com",
            None,
            "http://fail.example.com/healthz",
            HashMap::new(),
        )
        .await;
        assert!(payload.service.is_none());
        assert_eq!(payload.errors.len(), 1);
        assert!(payload.errors[0].message.contains("stub grpc error"));
    }

    #[tokio::test]
    async fn update_service_success() {
        let client = StubServiceCatalogClient::new();
        client
            .seed_service(sample_catalog_service("svc-001", "auth"))
            .await;

        let payload = resolve_update_service(
            &client,
            "svc-001",
            Some("Updated Auth"),
            None,
            Some("2.0.0"),
            None,
            None,
            None,
            HashMap::new(),
        )
        .await;
        assert!(payload.service.is_some());
        assert!(payload.errors.is_empty());
        let svc = payload.service.unwrap();
        assert_eq!(svc.display_name, "Updated Auth");
        assert_eq!(svc.version, "2.0.0");
    }

    #[tokio::test]
    async fn update_service_not_found_returns_user_error() {
        let client = StubServiceCatalogClient::new();

        let payload = resolve_update_service(
            &client,
            "nonexistent",
            Some("Updated"),
            None,
            None,
            None,
            None,
            None,
            HashMap::new(),
        )
        .await;
        assert!(payload.service.is_none());
        assert_eq!(payload.errors.len(), 1);
        assert!(payload.errors[0].message.contains("service not found"));
    }

    #[tokio::test]
    async fn update_service_error_returns_user_error() {
        let client = StubServiceCatalogClient::failing();

        let payload = resolve_update_service(
            &client,
            "svc-001",
            Some("Updated"),
            None,
            None,
            None,
            None,
            None,
            HashMap::new(),
        )
        .await;
        assert!(payload.service.is_none());
        assert_eq!(payload.errors.len(), 1);
        assert!(payload.errors[0].message.contains("stub grpc error"));
    }

    #[tokio::test]
    async fn delete_service_success() {
        let client = StubServiceCatalogClient::new();
        client
            .seed_service(sample_catalog_service("svc-001", "auth"))
            .await;

        let payload = resolve_delete_service(&client, "svc-001").await;
        assert!(payload.success);
        assert!(payload.errors.is_empty());
    }

    #[tokio::test]
    async fn delete_service_error_returns_user_error() {
        let client = StubServiceCatalogClient::failing();

        let payload = resolve_delete_service(&client, "svc-001").await;
        assert!(!payload.success);
        assert_eq!(payload.errors.len(), 1);
        assert!(payload.errors[0].message.contains("stub grpc error"));
    }
}

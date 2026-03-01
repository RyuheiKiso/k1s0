use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use moka::future::Cache;
use serde::{Deserialize, Serialize};

use crate::config::TenantClientConfig;
use crate::error::TenantError;
use crate::tenant::{
    CreateTenantRequest, ProvisioningStatus, Tenant, TenantFilter, TenantMember, TenantSettings,
    TenantStatus,
};

#[async_trait]
#[cfg_attr(feature = "mock", mockall::automock)]
pub trait TenantClient: Send + Sync {
    async fn get_tenant(&self, tenant_id: &str) -> Result<Tenant, TenantError>;
    async fn list_tenants(&self, filter: TenantFilter) -> Result<Vec<Tenant>, TenantError>;
    async fn is_active(&self, tenant_id: &str) -> Result<bool, TenantError>;
    async fn get_settings(&self, tenant_id: &str) -> Result<TenantSettings, TenantError>;
    async fn create_tenant(&self, req: CreateTenantRequest) -> Result<Tenant, TenantError>;
    async fn add_member(
        &self,
        tenant_id: &str,
        user_id: &str,
        role: &str,
    ) -> Result<TenantMember, TenantError>;
    async fn remove_member(&self, tenant_id: &str, user_id: &str) -> Result<(), TenantError>;
    async fn list_members(&self, tenant_id: &str) -> Result<Vec<TenantMember>, TenantError>;
    async fn get_provisioning_status(
        &self,
        tenant_id: &str,
    ) -> Result<ProvisioningStatus, TenantError>;
}

pub struct InMemoryTenantClient {
    tenants: Arc<std::sync::Mutex<HashMap<String, Tenant>>>,
    members: Arc<tokio::sync::Mutex<HashMap<String, Vec<TenantMember>>>>,
    provisioning: Arc<tokio::sync::Mutex<HashMap<String, ProvisioningStatus>>>,
}

impl InMemoryTenantClient {
    pub fn new() -> Self {
        Self {
            tenants: Arc::new(std::sync::Mutex::new(HashMap::new())),
            members: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            provisioning: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        }
    }

    pub fn with_tenants(tenants: Vec<Tenant>) -> Self {
        let map: HashMap<String, Tenant> = tenants.into_iter().map(|t| (t.id.clone(), t)).collect();
        Self {
            tenants: Arc::new(std::sync::Mutex::new(map)),
            members: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            provisioning: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        }
    }

    pub fn add_tenant(&self, tenant: Tenant) {
        self.tenants
            .lock()
            .unwrap()
            .insert(tenant.id.clone(), tenant);
    }
}

impl Default for InMemoryTenantClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TenantClient for InMemoryTenantClient {
    async fn get_tenant(&self, tenant_id: &str) -> Result<Tenant, TenantError> {
        let tenants = self.tenants.lock().unwrap();
        tenants
            .get(tenant_id)
            .cloned()
            .ok_or_else(|| TenantError::NotFound(tenant_id.to_string()))
    }

    async fn list_tenants(&self, filter: TenantFilter) -> Result<Vec<Tenant>, TenantError> {
        let tenants = self.tenants.lock().unwrap();
        let result: Vec<Tenant> = tenants
            .values()
            .filter(|t| {
                if let Some(status) = &filter.status {
                    if t.status != *status {
                        return false;
                    }
                }
                if let Some(plan) = &filter.plan {
                    if t.plan != *plan {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();
        Ok(result)
    }

    async fn is_active(&self, tenant_id: &str) -> Result<bool, TenantError> {
        let tenant = self.get_tenant(tenant_id).await?;
        Ok(tenant.status == TenantStatus::Active)
    }

    async fn get_settings(&self, tenant_id: &str) -> Result<TenantSettings, TenantError> {
        let tenant = self.get_tenant(tenant_id).await?;
        Ok(TenantSettings::new(tenant.settings))
    }

    async fn create_tenant(&self, req: CreateTenantRequest) -> Result<Tenant, TenantError> {
        let tenant = Tenant {
            id: uuid::Uuid::new_v4().to_string(),
            name: req.name,
            status: TenantStatus::Active,
            plan: req.plan,
            settings: HashMap::new(),
            created_at: chrono::Utc::now(),
        };
        {
            let mut tenants = self.tenants.lock().unwrap();
            tenants.insert(tenant.id.clone(), tenant.clone());
        }
        // Set provisioning status to Pending
        let mut prov = self.provisioning.lock().await;
        prov.insert(tenant.id.clone(), ProvisioningStatus::Pending);
        Ok(tenant)
    }

    async fn add_member(
        &self,
        tenant_id: &str,
        user_id: &str,
        role: &str,
    ) -> Result<TenantMember, TenantError> {
        // Verify tenant exists
        {
            let tenants = self.tenants.lock().unwrap();
            if !tenants.contains_key(tenant_id) {
                return Err(TenantError::NotFound(tenant_id.to_string()));
            }
        }
        let member = TenantMember {
            user_id: user_id.to_string(),
            role: role.to_string(),
            joined_at: chrono::Utc::now(),
        };
        let mut members = self.members.lock().await;
        members
            .entry(tenant_id.to_string())
            .or_default()
            .push(member.clone());
        Ok(member)
    }

    async fn remove_member(&self, tenant_id: &str, user_id: &str) -> Result<(), TenantError> {
        let mut members = self.members.lock().await;
        if let Some(list) = members.get_mut(tenant_id) {
            list.retain(|m| m.user_id != user_id);
        }
        Ok(())
    }

    async fn list_members(&self, tenant_id: &str) -> Result<Vec<TenantMember>, TenantError> {
        let members = self.members.lock().await;
        Ok(members.get(tenant_id).cloned().unwrap_or_default())
    }

    async fn get_provisioning_status(
        &self,
        tenant_id: &str,
    ) -> Result<ProvisioningStatus, TenantError> {
        let prov = self.provisioning.lock().await;
        prov.get(tenant_id)
            .cloned()
            .ok_or_else(|| TenantError::NotFound(tenant_id.to_string()))
    }
}

// ---------------------------------------------------------------------------
// HttpTenantClient — tenant-server 経由の HTTP 実装
// ---------------------------------------------------------------------------

/// tenant-server から返却される JSON レスポンスの内部 DTO。
#[derive(Debug, Deserialize)]
struct TenantResponse {
    id: String,
    name: String,
    status: TenantStatus,
    plan: String,
    settings: std::collections::HashMap<String, String>,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<TenantResponse> for Tenant {
    fn from(r: TenantResponse) -> Self {
        Tenant {
            id: r.id,
            name: r.name,
            status: r.status,
            plan: r.plan,
            settings: r.settings,
            created_at: r.created_at,
        }
    }
}

/// `TenantSettings` 取得専用レスポンス DTO。
#[derive(Debug, Deserialize)]
struct TenantSettingsResponse {
    values: std::collections::HashMap<String, String>,
}

/// POST /api/v1/tenants のリクエストボディ。
#[derive(Debug, Serialize)]
struct CreateTenantBody<'a> {
    name: &'a str,
    plan: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    admin_user_id: Option<&'a str>,
}

/// POST /api/v1/tenants のレスポンス DTO。
#[derive(Debug, Deserialize)]
struct CreateTenantResponse {
    tenant: TenantResponse,
}

/// メンバー情報レスポンス DTO。
#[derive(Debug, Deserialize)]
struct MemberResponse {
    user_id: String,
    role: String,
    joined_at: chrono::DateTime<chrono::Utc>,
}

impl From<MemberResponse> for TenantMember {
    fn from(r: MemberResponse) -> Self {
        TenantMember {
            user_id: r.user_id,
            role: r.role,
            joined_at: r.joined_at,
        }
    }
}

/// POST /api/v1/tenants/{tenant_id}/members のレスポンス DTO。
#[derive(Debug, Deserialize)]
struct MemberWrapperResponse {
    member: MemberResponse,
}

/// GET /api/v1/tenants/{tenant_id}/members のレスポンス DTO。
#[derive(Debug, Deserialize)]
struct MembersResponse {
    members: Vec<MemberResponse>,
}

/// GET /api/v1/tenants/{tenant_id}/provisioning-status のレスポンス DTO。
#[derive(Debug, Deserialize)]
struct ProvisioningStatusResponse {
    status: String,
}

/// "pending" / "in_progress" / "completed" / "failed" 文字列から `ProvisioningStatus` へ変換。
fn parse_provisioning_status(s: &str) -> ProvisioningStatus {
    match s {
        "pending" => ProvisioningStatus::Pending,
        "in_progress" => ProvisioningStatus::InProgress,
        "completed" => ProvisioningStatus::Completed,
        other => ProvisioningStatus::Failed(other.to_string()),
    }
}

/// tenant-server へ HTTP で委譲する `TenantClient` 実装。TTL 付きキャッシュを内蔵する。
///
/// 名称は将来的な gRPC 移行を見越した `HttpTenantClient` だが、
/// 現時点では REST/HTTP で実装されている。
pub struct HttpTenantClient {
    http: reqwest::Client,
    base_url: String,
    tenant_cache: Cache<String, Tenant>,
    settings_cache: Cache<String, TenantSettings>,
}

impl HttpTenantClient {
    /// 新しい `HttpTenantClient` を生成する。
    pub fn new(config: TenantClientConfig) -> Result<Self, TenantError> {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| TenantError::ServerError(e.to_string()))?;

        let tenant_cache = Cache::builder()
            .max_capacity(config.cache_max_capacity)
            .time_to_live(config.cache_ttl)
            .build();

        let settings_cache = Cache::builder()
            .max_capacity(config.cache_max_capacity)
            .time_to_live(config.cache_ttl)
            .build();

        Ok(Self {
            http,
            base_url: config.server_url,
            tenant_cache,
            settings_cache,
        })
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    /// HTTP レスポンスのステータスを確認し、エラーを `TenantError` に変換する。
    async fn check_response(
        resp: reqwest::Response,
        tenant_id: &str,
    ) -> Result<reqwest::Response, TenantError> {
        let status = resp.status();
        if status.is_success() {
            return Ok(resp);
        }
        let body = resp.text().await.unwrap_or_default();
        match status.as_u16() {
            404 => Err(TenantError::NotFound(tenant_id.to_string())),
            _ => Err(TenantError::ServerError(format!(
                "HTTP {}: {}",
                status, body
            ))),
        }
    }

    /// `reqwest::Error` を `TenantError` へ変換するヘルパー。
    fn map_request_error(e: reqwest::Error) -> TenantError {
        if e.is_timeout() {
            TenantError::Timeout(e.to_string())
        } else {
            TenantError::ServerError(e.to_string())
        }
    }
}

#[async_trait]
impl TenantClient for HttpTenantClient {
    async fn get_tenant(&self, tenant_id: &str) -> Result<Tenant, TenantError> {
        if let Some(cached) = self.tenant_cache.get(tenant_id).await {
            return Ok(cached);
        }

        let resp = self
            .http
            .get(self.url(&format!("/api/v1/tenants/{}", tenant_id)))
            .send()
            .await
            .map_err(Self::map_request_error)?;
        let resp = Self::check_response(resp, tenant_id).await?;
        let data: TenantResponse = resp
            .json()
            .await
            .map_err(|e| TenantError::ServerError(e.to_string()))?;
        let tenant: Tenant = data.into();
        self.tenant_cache
            .insert(tenant_id.to_string(), tenant.clone())
            .await;
        Ok(tenant)
    }

    async fn list_tenants(&self, filter: TenantFilter) -> Result<Vec<Tenant>, TenantError> {
        let mut query: Vec<(&str, String)> = Vec::new();
        if let Some(ref status) = filter.status {
            let s = serde_json::to_value(status)
                .ok()
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_default();
            query.push(("status", s));
        }
        if let Some(ref plan) = filter.plan {
            query.push(("plan", plan.clone()));
        }

        let resp = self
            .http
            .get(self.url("/api/v1/tenants"))
            .query(&query)
            .send()
            .await
            .map_err(Self::map_request_error)?;
        let resp = Self::check_response(resp, "").await?;
        let data: Vec<TenantResponse> = resp
            .json()
            .await
            .map_err(|e| TenantError::ServerError(e.to_string()))?;
        Ok(data.into_iter().map(Into::into).collect())
    }

    async fn is_active(&self, tenant_id: &str) -> Result<bool, TenantError> {
        let tenant = self.get_tenant(tenant_id).await?;
        Ok(tenant.status == TenantStatus::Active)
    }

    async fn get_settings(&self, tenant_id: &str) -> Result<TenantSettings, TenantError> {
        if let Some(cached) = self.settings_cache.get(tenant_id).await {
            return Ok(cached);
        }

        let resp = self
            .http
            .get(self.url(&format!("/api/v1/tenants/{}/settings", tenant_id)))
            .send()
            .await
            .map_err(Self::map_request_error)?;
        let resp = Self::check_response(resp, tenant_id).await?;
        let data: TenantSettingsResponse = resp
            .json()
            .await
            .map_err(|e| TenantError::ServerError(e.to_string()))?;
        let settings = TenantSettings::new(data.values);
        self.settings_cache
            .insert(tenant_id.to_string(), settings.clone())
            .await;
        Ok(settings)
    }

    async fn create_tenant(&self, req: CreateTenantRequest) -> Result<Tenant, TenantError> {
        let body = CreateTenantBody {
            name: &req.name,
            plan: &req.plan,
            admin_user_id: req.admin_user_id.as_deref(),
        };
        let resp = self
            .http
            .post(self.url("/api/v1/tenants"))
            .json(&body)
            .send()
            .await
            .map_err(Self::map_request_error)?;
        let resp = Self::check_response(resp, "").await?;
        let data: CreateTenantResponse = resp
            .json()
            .await
            .map_err(|e| TenantError::ServerError(e.to_string()))?;
        Ok(data.tenant.into())
    }

    async fn add_member(
        &self,
        tenant_id: &str,
        user_id: &str,
        role: &str,
    ) -> Result<TenantMember, TenantError> {
        let body = serde_json::json!({ "user_id": user_id, "role": role });
        let resp = self
            .http
            .post(self.url(&format!("/api/v1/tenants/{}/members", tenant_id)))
            .json(&body)
            .send()
            .await
            .map_err(Self::map_request_error)?;
        let resp = Self::check_response(resp, tenant_id).await?;
        let data: MemberWrapperResponse = resp
            .json()
            .await
            .map_err(|e| TenantError::ServerError(e.to_string()))?;
        Ok(data.member.into())
    }

    async fn remove_member(&self, tenant_id: &str, user_id: &str) -> Result<(), TenantError> {
        let resp = self
            .http
            .delete(self.url(&format!(
                "/api/v1/tenants/{}/members/{}",
                tenant_id, user_id
            )))
            .send()
            .await
            .map_err(Self::map_request_error)?;
        Self::check_response(resp, tenant_id).await?;
        Ok(())
    }

    async fn list_members(&self, tenant_id: &str) -> Result<Vec<TenantMember>, TenantError> {
        let resp = self
            .http
            .get(self.url(&format!("/api/v1/tenants/{}/members", tenant_id)))
            .send()
            .await
            .map_err(Self::map_request_error)?;
        let resp = Self::check_response(resp, tenant_id).await?;
        let data: MembersResponse = resp
            .json()
            .await
            .map_err(|e| TenantError::ServerError(e.to_string()))?;
        Ok(data.members.into_iter().map(Into::into).collect())
    }

    async fn get_provisioning_status(
        &self,
        tenant_id: &str,
    ) -> Result<ProvisioningStatus, TenantError> {
        let resp = self
            .http
            .get(self.url(&format!(
                "/api/v1/tenants/{}/provisioning-status",
                tenant_id
            )))
            .send()
            .await
            .map_err(Self::map_request_error)?;
        if resp.status().as_u16() == 404 {
            return Ok(ProvisioningStatus::Completed);
        }
        let resp = Self::check_response(resp, tenant_id).await?;
        let data: ProvisioningStatusResponse = resp
            .json()
            .await
            .map_err(|e| TenantError::ServerError(e.to_string()))?;
        Ok(parse_provisioning_status(&data.status))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::collections::HashMap;

    fn make_tenant(id: &str, status: TenantStatus, plan: &str) -> Tenant {
        let mut settings = HashMap::new();
        settings.insert("max_users".to_string(), "100".to_string());
        Tenant {
            id: id.to_string(),
            name: format!("Tenant {}", id),
            status,
            plan: plan.to_string(),
            settings,
            created_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_get_tenant() {
        let client = InMemoryTenantClient::new();
        client.add_tenant(make_tenant("T-001", TenantStatus::Active, "enterprise"));
        let tenant = client.get_tenant("T-001").await.unwrap();
        assert_eq!(tenant.id, "T-001");
        assert_eq!(tenant.status, TenantStatus::Active);
    }

    #[tokio::test]
    async fn test_get_tenant_not_found() {
        let client = InMemoryTenantClient::new();
        let result = client.get_tenant("T-999").await;
        assert!(matches!(result, Err(TenantError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_list_tenants_by_status() {
        let client = InMemoryTenantClient::with_tenants(vec![
            make_tenant("T-001", TenantStatus::Active, "enterprise"),
            make_tenant("T-002", TenantStatus::Suspended, "basic"),
            make_tenant("T-003", TenantStatus::Active, "basic"),
        ]);
        let filter = TenantFilter::new().status(TenantStatus::Active);
        let tenants = client.list_tenants(filter).await.unwrap();
        assert_eq!(tenants.len(), 2);
    }

    #[tokio::test]
    async fn test_list_tenants_by_plan() {
        let client = InMemoryTenantClient::with_tenants(vec![
            make_tenant("T-001", TenantStatus::Active, "enterprise"),
            make_tenant("T-002", TenantStatus::Active, "basic"),
        ]);
        let filter = TenantFilter::new().plan("enterprise");
        let tenants = client.list_tenants(filter).await.unwrap();
        assert_eq!(tenants.len(), 1);
        assert_eq!(tenants[0].id, "T-001");
    }

    #[tokio::test]
    async fn test_is_active_true() {
        let client = InMemoryTenantClient::new();
        client.add_tenant(make_tenant("T-001", TenantStatus::Active, "basic"));
        assert!(client.is_active("T-001").await.unwrap());
    }

    #[tokio::test]
    async fn test_is_active_false() {
        let client = InMemoryTenantClient::new();
        client.add_tenant(make_tenant("T-001", TenantStatus::Suspended, "basic"));
        assert!(!client.is_active("T-001").await.unwrap());
    }

    #[tokio::test]
    async fn test_get_settings() {
        let client = InMemoryTenantClient::new();
        client.add_tenant(make_tenant("T-001", TenantStatus::Active, "basic"));
        let settings = client.get_settings("T-001").await.unwrap();
        assert_eq!(settings.get("max_users"), Some("100"));
        assert_eq!(settings.get("nonexistent"), None);
    }

    #[test]
    fn test_tenant_status_variants() {
        assert!(matches!(TenantStatus::Active, TenantStatus::Active));
        assert!(matches!(TenantStatus::Suspended, TenantStatus::Suspended));
        assert!(matches!(TenantStatus::Deleted, TenantStatus::Deleted));
    }

    #[test]
    fn test_tenant_filter_builder() {
        let filter = TenantFilter::new()
            .status(TenantStatus::Active)
            .plan("enterprise");
        assert_eq!(filter.status, Some(TenantStatus::Active));
        assert_eq!(filter.plan, Some("enterprise".to_string()));
    }

    #[test]
    fn test_tenant_settings_get() {
        let mut values = HashMap::new();
        values.insert("key1".to_string(), "value1".to_string());
        let settings = TenantSettings::new(values);
        assert_eq!(settings.get("key1"), Some("value1"));
        assert_eq!(settings.get("key2"), None);
    }

    #[test]
    fn test_config_builder() {
        use std::time::Duration;
        let config = TenantClientConfig::new("http://localhost:8080")
            .cache_ttl(Duration::from_secs(60))
            .cache_max_capacity(500);
        assert_eq!(config.server_url, "http://localhost:8080");
        assert_eq!(config.cache_ttl, Duration::from_secs(60));
        assert_eq!(config.cache_max_capacity, 500);
    }

    #[tokio::test]
    async fn test_create_tenant() {
        let client = InMemoryTenantClient::new();
        let tenant = client
            .create_tenant(CreateTenantRequest {
                name: "New Tenant".to_string(),
                plan: "starter".to_string(),
                admin_user_id: None,
            })
            .await
            .unwrap();
        assert_eq!(tenant.name, "New Tenant");
        assert_eq!(tenant.plan, "starter");
        assert!(matches!(tenant.status, TenantStatus::Active));
    }

    #[tokio::test]
    async fn test_member_management() {
        let client = InMemoryTenantClient::new();
        let tenant = client
            .create_tenant(CreateTenantRequest {
                name: "T1".to_string(),
                plan: "pro".to_string(),
                admin_user_id: None,
            })
            .await
            .unwrap();

        client
            .add_member(&tenant.id, "user-1", "admin")
            .await
            .unwrap();
        client
            .add_member(&tenant.id, "user-2", "member")
            .await
            .unwrap();

        let members = client.list_members(&tenant.id).await.unwrap();
        assert_eq!(members.len(), 2);

        client.remove_member(&tenant.id, "user-1").await.unwrap();
        let members = client.list_members(&tenant.id).await.unwrap();
        assert_eq!(members.len(), 1);
        assert_eq!(members[0].user_id, "user-2");
    }

    #[tokio::test]
    async fn test_provisioning_status() {
        let client = InMemoryTenantClient::new();
        let tenant = client
            .create_tenant(CreateTenantRequest {
                name: "T2".to_string(),
                plan: "enterprise".to_string(),
                admin_user_id: None,
            })
            .await
            .unwrap();

        let status = client.get_provisioning_status(&tenant.id).await.unwrap();
        assert!(matches!(status, ProvisioningStatus::Pending));
    }
}

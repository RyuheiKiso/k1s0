use async_graphql::SimpleObject;

#[derive(Debug, Clone, SimpleObject)]
pub struct CatalogService {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub tier: String,
    pub version: String,
    pub base_url: String,
    pub grpc_endpoint: Option<String>,
    pub health_url: String,
    pub status: String,
    pub metadata: Vec<MetadataEntry>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct MetadataEntry {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct CatalogServiceConnection {
    pub services: Vec<CatalogService>,
    pub total_count: i64,
    pub has_next: bool,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct ServiceHealth {
    pub service_id: String,
    pub service_name: String,
    pub status: String,
    pub response_time_ms: Option<i64>,
    pub error_message: Option<String>,
    pub checked_at: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, SimpleObject)]
pub struct DeleteServiceResult {
    pub success: bool,
}

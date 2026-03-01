use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub data: T,
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub records: Vec<T>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<TableMetadata>,
}

#[derive(Debug, Serialize)]
pub struct TableMetadata {
    pub table_name: String,
    pub display_name: String,
    pub allow_create: bool,
    pub allow_update: bool,
    pub allow_delete: bool,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

#[derive(Debug, Serialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    pub request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

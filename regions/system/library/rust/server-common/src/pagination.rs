use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct PaginationResponse {
    pub total_count: u64,
    pub page: u32,
    pub page_size: u32,
    pub has_next: bool,
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub pagination: PaginationResponse,
}

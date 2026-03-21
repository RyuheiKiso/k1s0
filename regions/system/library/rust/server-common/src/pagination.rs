use serde::Serialize;

use crate::error::{ErrorCode, ErrorDetail, ServiceError};

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

/// ページネーションパラメータを検証するヘルパー。
///
/// 全サーバーで繰り返される `page >= 1` / `page_size in 1..=100` の検証ロジックを一元化する。
/// 不正な値の場合は `ServiceError::BadRequest` を返す。
///
/// # 使用例
///
/// ```rust,ignore
/// use k1s0_server_common::pagination::validate_pagination;
///
/// let (page, page_size) = validate_pagination(query.page, query.page_size)?;
/// ```
pub fn validate_pagination(
    page: Option<u32>,
    page_size: Option<u32>,
) -> Result<(u32, u32), ServiceError> {
    let page = page.unwrap_or(1);
    let page_size = page_size.unwrap_or(20);

    if page < 1 {
        return Err(ServiceError::BadRequest {
            code: ErrorCode::new("INVALID_PAGINATION"),
            message: "page must be >= 1".to_string(),
            details: vec![ErrorDetail::new("page", "invalid_range", "must be >= 1")],
        });
    }

    // ページサイズ上限: 4言語共通で 100 に統一（validation/rules.rs H-18 と同一）
    if !(1..=100).contains(&page_size) {
        return Err(ServiceError::BadRequest {
            code: ErrorCode::new("INVALID_PAGINATION"),
            message: "page_size must be 1-100".to_string(),
            details: vec![ErrorDetail::new(
                "page_size",
                "invalid_range",
                "must be between 1 and 100",
            )],
        });
    }

    Ok((page, page_size))
}

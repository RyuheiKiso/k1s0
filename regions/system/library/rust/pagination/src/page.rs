use serde::{Deserialize, Serialize};

use crate::error::PerPageValidationError;

const MIN_PER_PAGE: u32 = 1;
const MAX_PER_PAGE: u32 = 100;

/// Validate that `per_page` is between 1 and 100.
pub fn validate_per_page(per_page: u32) -> Result<u32, PerPageValidationError> {
    if !(MIN_PER_PAGE..=MAX_PER_PAGE).contains(&per_page) {
        return Err(PerPageValidationError::InvalidPerPage {
            value: per_page,
            min: MIN_PER_PAGE,
            max: MAX_PER_PAGE,
        });
    }
    Ok(per_page)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageRequest {
    pub page: u32,
    pub per_page: u32,
}

impl Default for PageRequest {
    fn default() -> Self {
        Self {
            page: 1,
            per_page: 20,
        }
    }
}

impl PageRequest {
    #[must_use]
    pub fn offset(&self) -> u64 {
        (u64::from(self.page) - 1) * u64::from(self.per_page)
    }

    #[must_use]
    pub fn has_next(&self, total: u64) -> bool {
        u64::from(self.page) * u64::from(self.per_page) < total
    }
}

/// Returns a default page request (`page = 1`, `per_page = 20`).
#[must_use]
pub fn default_page_request() -> PageRequest {
    PageRequest::default()
}

/// Offset pagination metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationMeta {
    pub total: u64,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageResponse<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
}

impl<T> PageResponse<T> {
    #[must_use]
    pub fn new(items: Vec<T>, total: u64, request: &PageRequest) -> Self {
        // LOW-008: 整数演算による切り上げ除算でキャストを回避する
        let total_pages = if request.per_page == 0 {
            0
        } else {
            let per_page_u64 = u64::from(request.per_page);
            // div_ceil で切り上げ除算を実装する（std の div_ceil は u64 で利用可能）
            u32::try_from(total.div_ceil(per_page_u64)).unwrap_or(u32::MAX)
        };
        Self {
            items,
            total,
            page: request.page,
            per_page: request.per_page,
            total_pages,
        }
    }

    /// Return the pagination metadata for this response.
    #[must_use]
    pub fn meta(&self) -> PaginationMeta {
        PaginationMeta {
            total: self.total,
            page: self.page,
            per_page: self.per_page,
            total_pages: self.total_pages,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // PageResponseが正しく生成され、総ページ数が計算されることを確認する。
    #[test]
    fn test_page_response_new() {
        let request = PageRequest {
            page: 0,
            per_page: 10,
        };
        let items: Vec<i32> = vec![1, 2, 3];
        let response = PageResponse::new(items, 25, &request);

        assert_eq!(response.total, 25);
        assert_eq!(response.page, 0);
        assert_eq!(response.per_page, 10);
        assert_eq!(response.total_pages, 3);
        assert_eq!(response.items.len(), 3);
    }

    // 総件数がper_pageで割り切れるときのtotal_pagesが正確であることを確認する。
    #[test]
    fn test_page_response_exact_pages() {
        let request = PageRequest {
            page: 0,
            per_page: 5,
        };
        let items: Vec<i32> = vec![];
        let response = PageResponse::new(items, 10, &request);
        assert_eq!(response.total_pages, 2);
    }

    // PageResponseからメタデータが正しく取得できることを確認する。
    #[test]
    fn test_page_response_meta() {
        let request = PageRequest {
            page: 2,
            per_page: 10,
        };
        let response = PageResponse::new(vec![1, 2, 3], 25, &request);
        let meta = response.meta();
        assert_eq!(meta.total, 25);
        assert_eq!(meta.page, 2);
        assert_eq!(meta.per_page, 10);
        assert_eq!(meta.total_pages, 3);
    }

    // 有効なper_page値（1、50、100）がバリデーションを通過することを確認する。
    #[test]
    fn test_validate_per_page_valid() {
        assert!(validate_per_page(1).is_ok());
        assert!(validate_per_page(50).is_ok());
        assert!(validate_per_page(100).is_ok());
    }

    // per_pageが0の場合にバリデーションエラーが返されることを確認する。
    #[test]
    fn test_validate_per_page_zero() {
        assert!(validate_per_page(0).is_err());
    }

    // per_pageが最大値（100）を超えた場合にバリデーションエラーが返されることを確認する。
    #[test]
    fn test_validate_per_page_over_max() {
        assert!(validate_per_page(101).is_err());
    }

    // PaginationMetaの各フィールドが正しく保持されることを確認する。
    #[test]
    fn test_pagination_meta_fields() {
        let meta = PaginationMeta {
            total: 100,
            page: 1,
            per_page: 10,
            total_pages: 10,
        };
        assert_eq!(meta.total, 100);
        assert_eq!(meta.total_pages, 10);
    }

    // PageRequestのデフォルト値がpage=1、per_page=20であることを確認する。
    #[test]
    fn test_page_request_default() {
        let req = PageRequest::default();
        assert_eq!(req.page, 1);
        assert_eq!(req.per_page, 20);
    }

    // default_page_request関数がデフォルトのPageRequestを返すことを確認する。
    #[test]
    fn test_default_page_request_function() {
        let req = default_page_request();
        assert_eq!(req.page, 1);
        assert_eq!(req.per_page, 20);
    }

    // 1ページ目のオフセットが0であることを確認する。
    #[test]
    fn test_page_request_offset_first_page() {
        let req = PageRequest {
            page: 1,
            per_page: 20,
        };
        assert_eq!(req.offset(), 0);
    }

    // 2ページ目のオフセットがper_pageと等しいことを確認する。
    #[test]
    fn test_page_request_offset_second_page() {
        let req = PageRequest {
            page: 2,
            per_page: 20,
        };
        assert_eq!(req.offset(), 20);
    }

    // 3ページ目のオフセットが正しく計算されることを確認する。
    #[test]
    fn test_page_request_offset_third_page() {
        let req = PageRequest {
            page: 3,
            per_page: 10,
        };
        assert_eq!(req.offset(), 20);
    }

    // 次のページが存在するときhas_nextがtrueを返すことを確認する。
    #[test]
    fn test_has_next_true() {
        let req = PageRequest {
            page: 1,
            per_page: 10,
        };
        assert!(req.has_next(25));
    }

    // ページ数と総件数がちょうど一致する場合has_nextがfalseを返すことを確認する。
    #[test]
    fn test_has_next_false_exact() {
        let req = PageRequest {
            page: 2,
            per_page: 10,
        };
        // page 2 * per_page 10 = 20, total = 20 → no next
        assert!(!req.has_next(20));
    }

    // 最終ページを超えた場合has_nextがfalseを返すことを確認する。
    #[test]
    fn test_has_next_false_last_page() {
        let req = PageRequest {
            page: 3,
            per_page: 10,
        };
        assert!(!req.has_next(25));
    }

    // まだアイテムが残っている場合has_nextがtrueを返すことを確認する。
    #[test]
    fn test_has_next_more_items_remaining() {
        let req = PageRequest {
            page: 2,
            per_page: 10,
        };
        assert!(req.has_next(25));
    }
}

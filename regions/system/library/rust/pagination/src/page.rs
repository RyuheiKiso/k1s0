use serde::{Deserialize, Serialize};

use crate::error::PaginationError;

const MIN_PER_PAGE: u32 = 1;
const MAX_PER_PAGE: u32 = 100;

/// Validate that per_page is between 1 and 100.
pub fn validate_per_page(per_page: u32) -> Result<u32, PaginationError> {
    if !(MIN_PER_PAGE..=MAX_PER_PAGE).contains(&per_page) {
        return Err(PaginationError::InvalidPerPage {
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
    pub fn offset(&self) -> u64 {
        (self.page as u64 - 1) * self.per_page as u64
    }

    pub fn has_next(&self, total: u64) -> bool {
        (self.page as u64) * (self.per_page as u64) < total
    }
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
    pub fn new(items: Vec<T>, total: u64, request: &PageRequest) -> Self {
        let total_pages = if request.per_page == 0 {
            0
        } else {
            ((total as f64) / (request.per_page as f64)).ceil() as u32
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

    #[test]
    fn test_validate_per_page_valid() {
        assert!(validate_per_page(1).is_ok());
        assert!(validate_per_page(50).is_ok());
        assert!(validate_per_page(100).is_ok());
    }

    #[test]
    fn test_validate_per_page_zero() {
        assert!(validate_per_page(0).is_err());
    }

    #[test]
    fn test_validate_per_page_over_max() {
        assert!(validate_per_page(101).is_err());
    }

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

    #[test]
    fn test_page_request_default() {
        let req = PageRequest::default();
        assert_eq!(req.page, 1);
        assert_eq!(req.per_page, 20);
    }

    #[test]
    fn test_page_request_offset_first_page() {
        let req = PageRequest { page: 1, per_page: 20 };
        assert_eq!(req.offset(), 0);
    }

    #[test]
    fn test_page_request_offset_second_page() {
        let req = PageRequest { page: 2, per_page: 20 };
        assert_eq!(req.offset(), 20);
    }

    #[test]
    fn test_page_request_offset_third_page() {
        let req = PageRequest { page: 3, per_page: 10 };
        assert_eq!(req.offset(), 20);
    }

    #[test]
    fn test_has_next_true() {
        let req = PageRequest { page: 1, per_page: 10 };
        assert!(req.has_next(25));
    }

    #[test]
    fn test_has_next_false_exact() {
        let req = PageRequest { page: 2, per_page: 10 };
        // page 2 * per_page 10 = 20, total = 20 → no next
        assert!(!req.has_next(20));
    }

    #[test]
    fn test_has_next_false_last_page() {
        let req = PageRequest { page: 3, per_page: 10 };
        assert!(!req.has_next(25));
    }

    #[test]
    fn test_has_next_more_items_remaining() {
        let req = PageRequest { page: 2, per_page: 10 };
        assert!(req.has_next(25));
    }
}

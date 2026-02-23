use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageRequest {
    pub page: u32,
    pub per_page: u32,
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
}

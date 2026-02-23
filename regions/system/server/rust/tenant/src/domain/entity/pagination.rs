#[derive(Debug, Clone)]
pub struct Pagination {
    pub total_count: i64,
    pub page: i32,
    pub page_size: i32,
    pub has_next: bool,
}

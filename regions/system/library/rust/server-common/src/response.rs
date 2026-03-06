use serde::Serialize;

/// 標準 API レスポンスラッパー。
#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub data: T,
}

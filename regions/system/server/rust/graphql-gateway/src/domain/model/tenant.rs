use async_graphql::{Enum, SimpleObject};

#[derive(Debug, Clone, SimpleObject)]
pub struct Tenant {
    pub id: String,
    pub name: String,
    pub status: TenantStatus,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum TenantStatus {
    Active,
    Suspended,
    Deleted,
}

impl From<String> for TenantStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "ACTIVE" => TenantStatus::Active,
            "SUSPENDED" => TenantStatus::Suspended,
            "DELETED" => TenantStatus::Deleted,
            _ => TenantStatus::Active,
        }
    }
}

#[derive(Debug, Clone, SimpleObject)]
pub struct TenantConnection {
    pub edges: Vec<TenantEdge>,
    pub page_info: PageInfo,
    pub total_count: i32,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct TenantEdge {
    pub node: Tenant,
    pub cursor: String,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct PageInfo {
    pub has_next_page: bool,
    pub has_previous_page: bool,
    pub start_cursor: Option<String>,
    pub end_cursor: Option<String>,
}

/// カーソルエンコード: オフセットを base64 エンコードする
pub fn encode_cursor(offset: usize) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(format!("cursor:{offset}"))
}

/// カーソルデコード: base64 カーソルからオフセットを復元する
pub fn decode_cursor(cursor: &str) -> Option<usize> {
    use base64::Engine;
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(cursor)
        .ok()?;
    let s = String::from_utf8(decoded).ok()?;
    s.strip_prefix("cursor:")?.parse().ok()
}

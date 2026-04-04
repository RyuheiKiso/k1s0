use async_graphql::{Enum, SimpleObject};

/// テナントドメインモデル。
/// C-002 監査対応: GraphQL スキーマ（11フィールド）と proto の Tenant メッセージに合わせてフィールドを追加する。
#[derive(Debug, Clone, SimpleObject)]
pub struct Tenant {
    pub id: String,
    pub name: String,
    /// テナント表示名（display_name）。proto フィールド 3 に対応する。
    pub display_name: String,
    pub status: TenantStatus,
    /// テナントプラン名（free/standard/enterprise 等）。proto フィールド 5 に対応する。
    pub plan: String,
    /// オーナーユーザー UUID。proto フィールド 7 に対応する。
    pub owner_id: String,
    /// テナント設定（JSON 文字列）。空文字の場合は None として扱う。proto フィールド 8 に対応する。
    pub settings: Option<String>,
    /// テナントごとの DB スキーマ名（マルチテナント DB 分離用）。空文字の場合は None。proto フィールド 9 に対応する。
    pub db_schema: Option<String>,
    /// Keycloak realm 名（認証プロバイダー連携用）。空文字の場合は None。proto フィールド 11 に対応する。
    pub keycloak_realm: Option<String>,
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
    pub total_count: i64,
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

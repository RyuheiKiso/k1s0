use async_graphql::{Enum, SimpleObject};

/// セッション状態
#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum SessionStatus {
    Active,
    Revoked,
}

impl From<&str> for SessionStatus {
    fn from(s: &str) -> Self {
        // "active" アームと wildcard アームが同一の返り値のため統合する
        match s.to_ascii_lowercase().as_str() {
            "revoked" => SessionStatus::Revoked,
            _ => SessionStatus::Active,
        }
    }
}

/// セッション情報（token はlist/getでは省略）
// session_id は Proto/API との整合性のため構造体名と同じプレフィックスを使用する
#[allow(clippy::struct_field_names)]
#[derive(Debug, Clone, SimpleObject)]
pub struct Session {
    pub session_id: String,
    pub user_id: String,
    pub device_id: String,
    pub device_name: Option<String>,
    pub device_type: Option<String>,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub status: SessionStatus,
    pub expires_at: String,
    pub created_at: String,
    pub last_accessed_at: Option<String>,
}

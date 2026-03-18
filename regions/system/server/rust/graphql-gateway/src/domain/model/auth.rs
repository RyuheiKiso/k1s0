use async_graphql::SimpleObject;

/// ユーザー情報
#[derive(Debug, Clone, SimpleObject)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub enabled: bool,
    pub email_verified: bool,
    pub created_at: String,
}

/// ロール情報
#[derive(Debug, Clone, SimpleObject)]
pub struct Role {
    pub id: String,
    pub name: String,
    pub description: String,
}

/// パーミッション確認結果
#[derive(Debug, Clone, SimpleObject)]
pub struct PermissionCheck {
    pub allowed: bool,
    pub reason: String,
}

/// 監査ログエントリ
#[derive(Debug, Clone, SimpleObject)]
pub struct AuditLog {
    pub id: String,
    pub event_type: String,
    pub user_id: String,
    pub ip_address: String,
    pub user_agent: String,
    pub resource: String,
    pub action: String,
    pub result: String,
    pub resource_id: String,
    pub trace_id: String,
    pub created_at: String,
}

/// 監査ログ接続（ページネーション）
#[derive(Debug, Clone, SimpleObject)]
pub struct AuditLogConnection {
    pub logs: Vec<AuditLog>,
    pub total_count: i64,
    pub has_next: bool,
}

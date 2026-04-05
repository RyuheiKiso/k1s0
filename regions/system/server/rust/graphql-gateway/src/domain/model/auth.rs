use async_graphql::{Enum, SimpleObject};

/// 監査ログのイベント種別（C-9 監査対応: 文字列フィールドから型安全な enum へ移行）。
/// C-004 監査対応: proto との双方向整合のため TokenRefresh, PermissionCheck を追加する。
/// スキーマの AuditEventType enum に対応する。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum AuditEventType {
    Login,
    Logout,
    /// C-004 監査対応: proto AUDIT_EVENT_TYPE_TOKEN_REFRESH に対応
    TokenRefresh,
    /// C-004 監査対応: proto AUDIT_EVENT_TYPE_PERMISSION_CHECK に対応
    PermissionCheck,
    Create,
    Update,
    Delete,
    Read,
    PermissionDenied,
    ApiKeyCreated,
    ApiKeyRevoked,
    SecretAccessed,
    SecretRotated,
}

/// 監査ログの実行結果（C-9 監査対応: 文字列フィールドから型安全な enum へ移行）。
/// スキーマの AuditResult enum に対応する。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum AuditResult {
    Success,
    Failure,
    Partial,
}

/// AuditEventType enum → 後方互換文字列表現（GraphQL deprecated string フィールド向け）
pub fn audit_event_type_to_str(e: AuditEventType) -> &'static str {
    match e {
        AuditEventType::Login => "LOGIN",
        AuditEventType::Logout => "LOGOUT",
        AuditEventType::TokenRefresh => "TOKEN_REFRESH",
        AuditEventType::PermissionCheck => "PERMISSION_CHECK",
        AuditEventType::Create => "CREATE",
        AuditEventType::Update => "UPDATE",
        AuditEventType::Delete => "DELETE",
        AuditEventType::Read => "READ",
        AuditEventType::PermissionDenied => "PERMISSION_DENIED",
        AuditEventType::ApiKeyCreated => "API_KEY_CREATED",
        AuditEventType::ApiKeyRevoked => "API_KEY_REVOKED",
        AuditEventType::SecretAccessed => "SECRET_ACCESSED",
        AuditEventType::SecretRotated => "SECRET_ROTATED",
    }
}

/// AuditResult enum → 後方互換文字列表現（GraphQL deprecated string フィールド向け）
pub fn audit_result_to_str(e: AuditResult) -> &'static str {
    match e {
        AuditResult::Success => "SUCCESS",
        AuditResult::Failure => "FAILURE",
        AuditResult::Partial => "PARTIAL",
    }
}

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
    /// 後方互換性のため文字列フィールドを維持する（非推奨: `event_type_enum` フィールドを使用すること）
    // async-graphql の deprecation 属性で GraphQL スキーマに廃止予定の旨を通知する
    #[graphql(deprecation = "event_type_enum フィールドを使用してください")]
    pub event_type: String,
    pub user_id: String,
    pub ip_address: String,
    pub user_agent: String,
    pub resource: String,
    pub action: String,
    /// 後方互換性のため文字列フィールドを維持する（非推奨: `result_enum` フィールドを使用すること）
    // async-graphql の deprecation 属性で GraphQL スキーマに廃止予定の旨を通知する
    #[graphql(deprecation = "result_enum フィールドを使用してください")]
    pub result: String,
    pub resource_id: String,
    pub trace_id: String,
    pub created_at: String,
    /// C-9 監査対応: `event_type` の型安全な enum バージョン。新規クライアントはこのフィールドを使用すること
    pub event_type_enum: Option<AuditEventType>,
    /// C-9 監査対応: `result` の型安全な enum バージョン。新規クライアントはこのフィールドを使用すること
    pub result_enum: Option<AuditResult>,
}

/// 監査ログ接続（ページネーション）
#[derive(Debug, Clone, SimpleObject)]
pub struct AuditLogConnection {
    pub logs: Vec<AuditLog>,
    pub total_count: i64,
    pub has_next: bool,
}

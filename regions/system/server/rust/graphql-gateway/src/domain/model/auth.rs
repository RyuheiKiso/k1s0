use async_graphql::{Enum, SimpleObject};

/// 監査ログのイベント種別（C-9 監査対応: 文字列フィールドから型安全な enum へ移行）。
/// スキーマの AuditEventType enum に対応する。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum AuditEventType {
    Login,
    Logout,
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

/// AuditLog.event_type（文字列）から AuditEventType enum へ変換するヘルパー関数。
/// 未知の文字列は None を返す（クライアントは eventTypeEnum が null の場合は eventType 文字列を参照すること）。
pub fn parse_audit_event_type(s: &str) -> Option<AuditEventType> {
    match s.to_ascii_uppercase().as_str() {
        "LOGIN" => Some(AuditEventType::Login),
        "LOGOUT" => Some(AuditEventType::Logout),
        "CREATE" => Some(AuditEventType::Create),
        "UPDATE" => Some(AuditEventType::Update),
        "DELETE" => Some(AuditEventType::Delete),
        "READ" => Some(AuditEventType::Read),
        "PERMISSION_DENIED" => Some(AuditEventType::PermissionDenied),
        "API_KEY_CREATED" => Some(AuditEventType::ApiKeyCreated),
        "API_KEY_REVOKED" => Some(AuditEventType::ApiKeyRevoked),
        "SECRET_ACCESSED" => Some(AuditEventType::SecretAccessed),
        "SECRET_ROTATED" => Some(AuditEventType::SecretRotated),
        _ => None,
    }
}

/// AuditLog.result（文字列）から AuditResult enum へ変換するヘルパー関数。
pub fn parse_audit_result(s: &str) -> Option<AuditResult> {
    match s.to_ascii_uppercase().as_str() {
        "SUCCESS" => Some(AuditResult::Success),
        "FAILURE" => Some(AuditResult::Failure),
        "PARTIAL" => Some(AuditResult::Partial),
        _ => None,
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

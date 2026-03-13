use async_graphql::SimpleObject;

/// シークレットメタデータ（秘密値はGraphQL非公開）
#[derive(Debug, Clone, SimpleObject)]
pub struct SecretMetadata {
    pub path: String,
    pub current_version: i64,
    pub version_count: i32,
    pub created_at: String,
    pub updated_at: String,
}

/// Vault 監査ログエントリ
#[derive(Debug, Clone, SimpleObject)]
pub struct VaultAuditLogEntry {
    pub id: String,
    pub key_path: String,
    pub action: String,
    pub actor_id: String,
    pub ip_address: String,
    pub success: bool,
    pub error_msg: Option<String>,
    pub created_at: String,
}

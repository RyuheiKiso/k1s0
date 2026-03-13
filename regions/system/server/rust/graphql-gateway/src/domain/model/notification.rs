use async_graphql::SimpleObject;

/// 通知ログ
#[derive(Debug, Clone, SimpleObject)]
pub struct NotificationLog {
    pub id: String,
    pub channel_id: String,
    pub channel_type: String,
    pub template_id: Option<String>,
    pub recipient: String,
    pub subject: Option<String>,
    pub body: String,
    pub status: String,
    pub retry_count: i32,
    pub error_message: Option<String>,
    pub sent_at: Option<String>,
    pub created_at: String,
}

/// 通知チャネル
#[derive(Debug, Clone, SimpleObject)]
pub struct NotificationChannel {
    pub id: String,
    pub name: String,
    pub channel_type: String,
    pub config_json: String,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// 通知テンプレート
#[derive(Debug, Clone, SimpleObject)]
pub struct NotificationTemplate {
    pub id: String,
    pub name: String,
    pub channel_type: String,
    pub subject_template: Option<String>,
    pub body_template: String,
    pub created_at: String,
    pub updated_at: String,
}

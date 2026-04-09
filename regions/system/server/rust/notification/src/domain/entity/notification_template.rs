use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 通知テンプレートエンティティ。テナント分離のための tenant_id を保持する。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationTemplate {
    pub id: String,
    /// RLS によるテナント分離に使用するテナント識別子
    pub tenant_id: String,
    pub name: String,
    pub channel_type: String,
    pub subject_template: Option<String>,
    pub body_template: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl NotificationTemplate {
    /// テナント ID を指定して新しいテンプレートを生成する
    #[must_use]
    pub fn new(
        tenant_id: String,
        name: String,
        channel_type: String,
        subject_template: Option<String>,
        body_template: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: format!("tpl_{}", uuid::Uuid::new_v4().simple()),
            tenant_id,
            name,
            channel_type,
            subject_template,
            body_template,
            created_at: now,
            updated_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// NotificationTemplate::new が tpl_ プレフィックス付きの ID を生成する
    #[test]
    fn new_template_id_has_prefix() {
        let tpl = NotificationTemplate::new(
            "tenant_a".to_string(),
            "welcome-email".to_string(),
            "email".to_string(),
            Some("Welcome {{name}}".to_string()),
            "Hello {{name}}, welcome!".to_string(),
        );
        assert!(tpl.id.starts_with("tpl_"));
        assert_eq!(tpl.tenant_id, "tenant_a");
        assert_eq!(tpl.name, "welcome-email");
        assert_eq!(tpl.subject_template.as_deref(), Some("Welcome {{name}}"));
    }

    /// subject_template が None の場合も正常に生成される
    #[test]
    fn new_without_subject() {
        let tpl = NotificationTemplate::new(
            "tenant_b".to_string(),
            "slack-alert".to_string(),
            "slack".to_string(),
            None,
            "Alert: {{message}}".to_string(),
        );
        assert!(tpl.subject_template.is_none());
        assert_eq!(tpl.body_template, "Alert: {{message}}");
    }
}

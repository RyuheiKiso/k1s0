use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use k1s0_notification_server::domain::entity::notification_channel::NotificationChannel;
use k1s0_notification_server::domain::entity::notification_log::NotificationLog;
use k1s0_notification_server::domain::entity::notification_template::NotificationTemplate;
use k1s0_notification_server::domain::repository::{
    NotificationChannelRepository, NotificationLogRepository, NotificationTemplateRepository,
};
use k1s0_notification_server::domain::service::{DeliveryClient, DeliveryError};
use k1s0_notification_server::usecase::*;

// ---------------------------------------------------------------------------
// Stub: NotificationChannelRepository
// ---------------------------------------------------------------------------

struct StubChannelRepo {
    channels: RwLock<Vec<NotificationChannel>>,
    fail_on_write: bool,
}

impl StubChannelRepo {
    fn new() -> Self {
        Self {
            channels: RwLock::new(Vec::new()),
            fail_on_write: false,
        }
    }

    fn with_channels(channels: Vec<NotificationChannel>) -> Self {
        Self {
            channels: RwLock::new(channels),
            fail_on_write: false,
        }
    }

    fn failing() -> Self {
        Self {
            channels: RwLock::new(Vec::new()),
            fail_on_write: true,
        }
    }
}

#[async_trait]
impl NotificationChannelRepository for StubChannelRepo {
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<NotificationChannel>> {
        let channels = self.channels.read().await;
        Ok(channels.iter().find(|c| c.id == id).cloned())
    }

    async fn find_all(&self) -> anyhow::Result<Vec<NotificationChannel>> {
        let channels = self.channels.read().await;
        Ok(channels.clone())
    }

    async fn find_all_paginated(
        &self,
        page: u32,
        page_size: u32,
        channel_type: Option<String>,
        enabled_only: bool,
    ) -> anyhow::Result<(Vec<NotificationChannel>, u64)> {
        let channels = self.channels.read().await;
        let filtered: Vec<_> = channels
            .iter()
            .filter(|c| {
                if enabled_only && !c.enabled {
                    return false;
                }
                if let Some(ref ct) = channel_type {
                    if c.channel_type != *ct {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();
        let total = filtered.len() as u64;
        let start = ((page - 1) * page_size) as usize;
        let page_items: Vec<_> = filtered
            .into_iter()
            .skip(start)
            .take(page_size as usize)
            .collect();
        Ok((page_items, total))
    }

    async fn create(&self, channel: &NotificationChannel) -> anyhow::Result<()> {
        if self.fail_on_write {
            return Err(anyhow::anyhow!("db write error"));
        }
        self.channels.write().await.push(channel.clone());
        Ok(())
    }

    async fn update(&self, channel: &NotificationChannel) -> anyhow::Result<()> {
        if self.fail_on_write {
            return Err(anyhow::anyhow!("db write error"));
        }
        let mut channels = self.channels.write().await;
        if let Some(pos) = channels.iter().position(|c| c.id == channel.id) {
            channels[pos] = channel.clone();
            Ok(())
        } else {
            Err(anyhow::anyhow!("not found"))
        }
    }

    async fn delete(&self, id: &str) -> anyhow::Result<bool> {
        if self.fail_on_write {
            return Err(anyhow::anyhow!("db write error"));
        }
        let mut channels = self.channels.write().await;
        let len_before = channels.len();
        channels.retain(|c| c.id != id);
        Ok(channels.len() < len_before)
    }
}

// ---------------------------------------------------------------------------
// Stub: NotificationTemplateRepository
// ---------------------------------------------------------------------------

struct StubTemplateRepo {
    templates: RwLock<Vec<NotificationTemplate>>,
    fail_on_write: bool,
}

impl StubTemplateRepo {
    fn new() -> Self {
        Self {
            templates: RwLock::new(Vec::new()),
            fail_on_write: false,
        }
    }

    fn with_templates(templates: Vec<NotificationTemplate>) -> Self {
        Self {
            templates: RwLock::new(templates),
            fail_on_write: false,
        }
    }

    fn failing() -> Self {
        Self {
            templates: RwLock::new(Vec::new()),
            fail_on_write: true,
        }
    }
}

#[async_trait]
impl NotificationTemplateRepository for StubTemplateRepo {
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<NotificationTemplate>> {
        let templates = self.templates.read().await;
        Ok(templates.iter().find(|t| t.id == id).cloned())
    }

    async fn find_all(&self) -> anyhow::Result<Vec<NotificationTemplate>> {
        let templates = self.templates.read().await;
        Ok(templates.clone())
    }

    async fn find_all_paginated(
        &self,
        page: u32,
        page_size: u32,
        channel_type: Option<String>,
    ) -> anyhow::Result<(Vec<NotificationTemplate>, u64)> {
        let templates = self.templates.read().await;
        let filtered: Vec<_> = templates
            .iter()
            .filter(|t| {
                if let Some(ref ct) = channel_type {
                    if t.channel_type != *ct {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();
        let total = filtered.len() as u64;
        let start = ((page - 1) * page_size) as usize;
        let page_items: Vec<_> = filtered
            .into_iter()
            .skip(start)
            .take(page_size as usize)
            .collect();
        Ok((page_items, total))
    }

    async fn create(&self, template: &NotificationTemplate) -> anyhow::Result<()> {
        if self.fail_on_write {
            return Err(anyhow::anyhow!("db write error"));
        }
        self.templates.write().await.push(template.clone());
        Ok(())
    }

    async fn update(&self, template: &NotificationTemplate) -> anyhow::Result<()> {
        if self.fail_on_write {
            return Err(anyhow::anyhow!("db write error"));
        }
        let mut templates = self.templates.write().await;
        if let Some(pos) = templates.iter().position(|t| t.id == template.id) {
            templates[pos] = template.clone();
            Ok(())
        } else {
            Err(anyhow::anyhow!("not found"))
        }
    }

    async fn delete(&self, id: &str) -> anyhow::Result<bool> {
        if self.fail_on_write {
            return Err(anyhow::anyhow!("db write error"));
        }
        let mut templates = self.templates.write().await;
        let len_before = templates.len();
        templates.retain(|t| t.id != id);
        Ok(templates.len() < len_before)
    }
}

// ---------------------------------------------------------------------------
// Stub: NotificationLogRepository
// ---------------------------------------------------------------------------

struct StubLogRepo {
    logs: RwLock<Vec<NotificationLog>>,
    fail_on_write: bool,
}

impl StubLogRepo {
    fn new() -> Self {
        Self {
            logs: RwLock::new(Vec::new()),
            fail_on_write: false,
        }
    }

    fn with_logs(logs: Vec<NotificationLog>) -> Self {
        Self {
            logs: RwLock::new(logs),
            fail_on_write: false,
        }
    }

    fn failing() -> Self {
        Self {
            logs: RwLock::new(Vec::new()),
            fail_on_write: true,
        }
    }
}

#[async_trait]
impl NotificationLogRepository for StubLogRepo {
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<NotificationLog>> {
        let logs = self.logs.read().await;
        Ok(logs.iter().find(|l| l.id == id).cloned())
    }

    async fn find_by_channel_id(&self, channel_id: &str) -> anyhow::Result<Vec<NotificationLog>> {
        let logs = self.logs.read().await;
        Ok(logs
            .iter()
            .filter(|l| l.channel_id == channel_id)
            .cloned()
            .collect())
    }

    async fn find_all_paginated(
        &self,
        page: u32,
        page_size: u32,
        channel_id: Option<String>,
        status: Option<String>,
    ) -> anyhow::Result<(Vec<NotificationLog>, u64)> {
        let logs = self.logs.read().await;
        let filtered: Vec<_> = logs
            .iter()
            .filter(|l| {
                if let Some(ref cid) = channel_id {
                    if l.channel_id != *cid {
                        return false;
                    }
                }
                if let Some(ref s) = status {
                    if l.status != *s {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();
        let total = filtered.len() as u64;
        let start = ((page - 1) * page_size) as usize;
        let page_items: Vec<_> = filtered
            .into_iter()
            .skip(start)
            .take(page_size as usize)
            .collect();
        Ok((page_items, total))
    }

    async fn create(&self, log: &NotificationLog) -> anyhow::Result<()> {
        if self.fail_on_write {
            return Err(anyhow::anyhow!("db write error"));
        }
        self.logs.write().await.push(log.clone());
        Ok(())
    }

    async fn update(&self, log: &NotificationLog) -> anyhow::Result<()> {
        if self.fail_on_write {
            return Err(anyhow::anyhow!("db write error"));
        }
        let mut logs = self.logs.write().await;
        if let Some(pos) = logs.iter().position(|l| l.id == log.id) {
            logs[pos] = log.clone();
            Ok(())
        } else {
            Err(anyhow::anyhow!("not found"))
        }
    }
}

// ---------------------------------------------------------------------------
// Stub: DeliveryClient
// ---------------------------------------------------------------------------

struct StubDeliveryClient {
    should_fail: bool,
}

impl StubDeliveryClient {
    fn success() -> Self {
        Self { should_fail: false }
    }

    fn failing() -> Self {
        Self { should_fail: true }
    }
}

#[async_trait]
impl DeliveryClient for StubDeliveryClient {
    async fn send(
        &self,
        _recipient: &str,
        _subject: &str,
        _body: &str,
    ) -> Result<(), DeliveryError> {
        if self.should_fail {
            Err(DeliveryError::ConnectionFailed(
                "connection timeout".to_string(),
            ))
        } else {
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------
// Helper: create test entities
// ---------------------------------------------------------------------------

fn make_channel(name: &str, channel_type: &str, enabled: bool) -> NotificationChannel {
    NotificationChannel::new(
        name.to_string(),
        channel_type.to_string(),
        serde_json::json!({"host": "localhost"}),
        enabled,
    )
}

fn make_template(name: &str, channel_type: &str) -> NotificationTemplate {
    NotificationTemplate::new(
        name.to_string(),
        channel_type.to_string(),
        Some("Subject: {{title}}".to_string()),
        "Hello {{name}}, {{message}}".to_string(),
    )
}

fn make_failed_log(channel_id: &str) -> NotificationLog {
    let mut log = NotificationLog::new(
        channel_id.to_string(),
        "user@example.com".to_string(),
        Some("Test Subject".to_string()),
        "Test body".to_string(),
    );
    log.status = "failed".to_string();
    log.error_message = Some("previous failure".to_string());
    log
}

// ===========================================================================
// CreateChannel tests
// ===========================================================================

mod create_channel {
    use super::*;
    use k1s0_notification_server::usecase::create_channel::{
        CreateChannelError, CreateChannelInput,
    };

    #[tokio::test]
    async fn success_creates_email_channel() {
        let repo = Arc::new(StubChannelRepo::new());
        let uc = CreateChannelUseCase::new(repo.clone());
        let input = CreateChannelInput {
            name: "email-prod".to_string(),
            channel_type: "email".to_string(),
            config: serde_json::json!({"smtp_host": "smtp.example.com"}),
            enabled: true,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let channel = result.unwrap();
        assert_eq!(channel.name, "email-prod");
        assert_eq!(channel.channel_type, "email");
        assert!(channel.enabled);
        assert!(channel.id.starts_with("ch_"));

        // Verify persisted in repo
        let stored = repo.channels.read().await;
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].id, channel.id);
    }

    #[tokio::test]
    async fn success_creates_slack_channel() {
        let repo = Arc::new(StubChannelRepo::new());
        let uc = CreateChannelUseCase::new(repo.clone());
        let input = CreateChannelInput {
            name: "slack-alerts".to_string(),
            channel_type: "slack".to_string(),
            config: serde_json::json!({"webhook_url": "https://hooks.slack.com/xxx"}),
            enabled: false,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        let channel = result.unwrap();
        assert_eq!(channel.channel_type, "slack");
        assert!(!channel.enabled);
    }

    #[tokio::test]
    async fn success_creates_all_supported_types() {
        for channel_type in &["email", "slack", "webhook", "sms", "push"] {
            let repo = Arc::new(StubChannelRepo::new());
            let uc = CreateChannelUseCase::new(repo.clone());
            let input = CreateChannelInput {
                name: format!("{}-channel", channel_type),
                channel_type: channel_type.to_string(),
                config: serde_json::json!({}),
                enabled: true,
            };
            let result = uc.execute(&input).await;
            assert!(
                result.is_ok(),
                "should succeed for channel_type={}",
                channel_type
            );
        }
    }

    #[tokio::test]
    async fn validation_error_invalid_channel_type() {
        let repo = Arc::new(StubChannelRepo::new());
        let uc = CreateChannelUseCase::new(repo);
        let input = CreateChannelInput {
            name: "bad-channel".to_string(),
            channel_type: "telegram".to_string(),
            config: serde_json::json!({}),
            enabled: true,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            CreateChannelError::Validation(msg) => {
                assert!(msg.contains("telegram"));
            }
            e => panic!("expected Validation error, got: {:?}", e),
        }
    }

    #[tokio::test]
    async fn internal_error_on_repo_failure() {
        let repo = Arc::new(StubChannelRepo::failing());
        let uc = CreateChannelUseCase::new(repo);
        let input = CreateChannelInput {
            name: "fail-channel".to_string(),
            channel_type: "email".to_string(),
            config: serde_json::json!({}),
            enabled: true,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            CreateChannelError::Internal(msg) => {
                assert!(msg.contains("db write error"));
            }
            e => panic!("expected Internal error, got: {:?}", e),
        }
    }
}

// ===========================================================================
// GetChannel tests
// ===========================================================================

mod get_channel {
    use super::*;
    use k1s0_notification_server::usecase::get_channel::GetChannelError;

    #[tokio::test]
    async fn success_returns_existing_channel() {
        let channel = make_channel("email-prod", "email", true);
        let channel_id = channel.id.clone();
        let repo = Arc::new(StubChannelRepo::with_channels(vec![channel.clone()]));
        let uc = GetChannelUseCase::new(repo);

        let result = uc.execute(&channel_id).await;
        assert!(result.is_ok());
        let found = result.unwrap();
        assert_eq!(found.id, channel_id);
        assert_eq!(found.name, "email-prod");
    }

    #[tokio::test]
    async fn not_found_for_missing_channel() {
        let repo = Arc::new(StubChannelRepo::new());
        let uc = GetChannelUseCase::new(repo);

        let result = uc.execute("ch_nonexistent").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            GetChannelError::NotFound(id) => assert_eq!(id, "ch_nonexistent"),
            e => panic!("expected NotFound error, got: {:?}", e),
        }
    }
}

// ===========================================================================
// ListChannels tests
// ===========================================================================

mod list_channels {
    use super::*;
    #[tokio::test]
    async fn success_returns_empty_list() {
        let repo = Arc::new(StubChannelRepo::new());
        let uc = ListChannelsUseCase::new(repo);

        let result = uc.execute().await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn success_returns_all_channels() {
        let channels = vec![
            make_channel("ch1", "email", true),
            make_channel("ch2", "slack", true),
            make_channel("ch3", "sms", false),
        ];
        let repo = Arc::new(StubChannelRepo::with_channels(channels));
        let uc = ListChannelsUseCase::new(repo);

        let result = uc.execute().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 3);
    }

    #[tokio::test]
    async fn paginated_filters_by_channel_type() {
        let channels = vec![
            make_channel("ch1", "email", true),
            make_channel("ch2", "slack", true),
            make_channel("ch3", "email", false),
        ];
        let repo = Arc::new(StubChannelRepo::with_channels(channels));
        let uc = ListChannelsUseCase::new(repo);

        let result = uc
            .execute_paginated(1, 10, Some("email".to_string()), false)
            .await;
        assert!(result.is_ok());
        let (items, total) = result.unwrap();
        assert_eq!(total, 2);
        assert_eq!(items.len(), 2);
        assert!(items.iter().all(|c| c.channel_type == "email"));
    }

    #[tokio::test]
    async fn paginated_filters_enabled_only() {
        let channels = vec![
            make_channel("ch1", "email", true),
            make_channel("ch2", "slack", false),
            make_channel("ch3", "sms", true),
        ];
        let repo = Arc::new(StubChannelRepo::with_channels(channels));
        let uc = ListChannelsUseCase::new(repo);

        let result = uc.execute_paginated(1, 10, None, true).await;
        assert!(result.is_ok());
        let (items, total) = result.unwrap();
        assert_eq!(total, 2);
        assert!(items.iter().all(|c| c.enabled));
    }

    #[tokio::test]
    async fn paginated_respects_page_size() {
        let channels = vec![
            make_channel("ch1", "email", true),
            make_channel("ch2", "slack", true),
            make_channel("ch3", "sms", true),
        ];
        let repo = Arc::new(StubChannelRepo::with_channels(channels));
        let uc = ListChannelsUseCase::new(repo);

        // Page 1, size 2
        let (items, total) = uc.execute_paginated(1, 2, None, false).await.unwrap();
        assert_eq!(total, 3);
        assert_eq!(items.len(), 2);

        // Page 2, size 2
        let (items, _) = uc.execute_paginated(2, 2, None, false).await.unwrap();
        assert_eq!(items.len(), 1);
    }
}

// ===========================================================================
// UpdateChannel tests
// ===========================================================================

mod update_channel {
    use super::*;
    use k1s0_notification_server::usecase::update_channel::{
        UpdateChannelError, UpdateChannelInput,
    };

    #[tokio::test]
    async fn success_updates_name_and_enabled() {
        let channel = make_channel("old-name", "email", true);
        let channel_id = channel.id.clone();
        let repo = Arc::new(StubChannelRepo::with_channels(vec![channel]));
        let uc = UpdateChannelUseCase::new(repo.clone());

        let input = UpdateChannelInput {
            id: channel_id.clone(),
            name: Some("new-name".to_string()),
            enabled: Some(false),
            config: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.name, "new-name");
        assert!(!updated.enabled);

        // Verify persisted
        let stored = repo.channels.read().await;
        assert_eq!(stored[0].name, "new-name");
        assert!(!stored[0].enabled);
    }

    #[tokio::test]
    async fn success_updates_config_only() {
        let channel = make_channel("email-ch", "email", true);
        let channel_id = channel.id.clone();
        let repo = Arc::new(StubChannelRepo::with_channels(vec![channel]));
        let uc = UpdateChannelUseCase::new(repo);

        let new_config = serde_json::json!({"smtp_host": "new-host", "port": 587});
        let input = UpdateChannelInput {
            id: channel_id.clone(),
            name: None,
            enabled: None,
            config: Some(new_config.clone()),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.config, new_config);
        // Name should remain unchanged
        assert_eq!(updated.name, "email-ch");
    }

    #[tokio::test]
    async fn success_updated_at_changes() {
        let channel = make_channel("email-ch", "email", true);
        let channel_id = channel.id.clone();
        let original_updated_at = channel.updated_at;
        let repo = Arc::new(StubChannelRepo::with_channels(vec![channel]));
        let uc = UpdateChannelUseCase::new(repo);

        let input = UpdateChannelInput {
            id: channel_id,
            name: Some("renamed".to_string()),
            enabled: None,
            config: None,
        };

        let result = uc.execute(&input).await.unwrap();
        assert!(result.updated_at >= original_updated_at);
    }

    #[tokio::test]
    async fn not_found_for_missing_channel() {
        let repo = Arc::new(StubChannelRepo::new());
        let uc = UpdateChannelUseCase::new(repo);

        let input = UpdateChannelInput {
            id: "ch_nonexistent".to_string(),
            name: Some("x".to_string()),
            enabled: None,
            config: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            UpdateChannelError::NotFound(id) => assert_eq!(id, "ch_nonexistent"),
            e => panic!("expected NotFound, got: {:?}", e),
        }
    }
}

// ===========================================================================
// DeleteChannel tests
// ===========================================================================

mod delete_channel {
    use super::*;
    use k1s0_notification_server::usecase::delete_channel::DeleteChannelError;

    #[tokio::test]
    async fn success_deletes_existing_channel() {
        let channel = make_channel("to-delete", "email", true);
        let channel_id = channel.id.clone();
        let repo = Arc::new(StubChannelRepo::with_channels(vec![channel]));
        let uc = DeleteChannelUseCase::new(repo.clone());

        let result = uc.execute(&channel_id).await;
        assert!(result.is_ok());

        // Verify removed from repo
        let stored = repo.channels.read().await;
        assert!(stored.is_empty());
    }

    #[tokio::test]
    async fn not_found_for_missing_channel() {
        let repo = Arc::new(StubChannelRepo::new());
        let uc = DeleteChannelUseCase::new(repo);

        let result = uc.execute("ch_nonexistent").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            DeleteChannelError::NotFound(id) => assert_eq!(id, "ch_nonexistent"),
            e => panic!("expected NotFound, got: {:?}", e),
        }
    }

    #[tokio::test]
    async fn internal_error_on_repo_failure() {
        let repo = Arc::new(StubChannelRepo::failing());
        let uc = DeleteChannelUseCase::new(repo);

        let result = uc.execute("ch_any").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            DeleteChannelError::Internal(msg) => assert!(msg.contains("db write error")),
            e => panic!("expected Internal, got: {:?}", e),
        }
    }
}

// ===========================================================================
// CreateTemplate tests
// ===========================================================================

mod create_template {
    use super::*;
    use k1s0_notification_server::usecase::create_template::{
        CreateTemplateError, CreateTemplateInput,
    };

    #[tokio::test]
    async fn success_creates_template_with_subject() {
        let repo = Arc::new(StubTemplateRepo::new());
        let uc = CreateTemplateUseCase::new(repo.clone());
        let input = CreateTemplateInput {
            name: "welcome-email".to_string(),
            channel_type: "email".to_string(),
            subject_template: Some("Welcome {{name}}".to_string()),
            body_template: "Hello {{name}}, welcome to our platform!".to_string(),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        let template = result.unwrap();
        assert_eq!(template.name, "welcome-email");
        assert_eq!(template.channel_type, "email");
        assert_eq!(
            template.subject_template.as_deref(),
            Some("Welcome {{name}}")
        );
        assert!(template.id.starts_with("tpl_"));

        let stored = repo.templates.read().await;
        assert_eq!(stored.len(), 1);
    }

    #[tokio::test]
    async fn success_creates_template_without_subject() {
        let repo = Arc::new(StubTemplateRepo::new());
        let uc = CreateTemplateUseCase::new(repo.clone());
        let input = CreateTemplateInput {
            name: "slack-alert".to_string(),
            channel_type: "slack".to_string(),
            subject_template: None,
            body_template: "Alert: {{message}}".to_string(),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        assert!(result.unwrap().subject_template.is_none());
    }

    #[tokio::test]
    async fn validation_error_invalid_body_template_syntax() {
        let repo = Arc::new(StubTemplateRepo::new());
        let uc = CreateTemplateUseCase::new(repo);
        let input = CreateTemplateInput {
            name: "bad-template".to_string(),
            channel_type: "email".to_string(),
            subject_template: None,
            body_template: "Hello {{#if}}".to_string(), // invalid handlebars
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            CreateTemplateError::Validation(msg) => {
                assert!(msg.contains("template"));
            }
            e => panic!("expected Validation, got: {:?}", e),
        }
    }

    #[tokio::test]
    async fn validation_error_invalid_subject_template_syntax() {
        let repo = Arc::new(StubTemplateRepo::new());
        let uc = CreateTemplateUseCase::new(repo);
        let input = CreateTemplateInput {
            name: "bad-subject".to_string(),
            channel_type: "email".to_string(),
            subject_template: Some("Subject {{#if}}".to_string()),
            body_template: "Valid body".to_string(),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            CreateTemplateError::Validation(_) => {}
            e => panic!("expected Validation, got: {:?}", e),
        }
    }

    #[tokio::test]
    async fn internal_error_on_repo_failure() {
        let repo = Arc::new(StubTemplateRepo::failing());
        let uc = CreateTemplateUseCase::new(repo);
        let input = CreateTemplateInput {
            name: "fail".to_string(),
            channel_type: "sms".to_string(),
            subject_template: None,
            body_template: "test body".to_string(),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            CreateTemplateError::Internal(msg) => assert!(msg.contains("db write error")),
            e => panic!("expected Internal, got: {:?}", e),
        }
    }
}

// ===========================================================================
// GetTemplate tests
// ===========================================================================

mod get_template {
    use super::*;
    use k1s0_notification_server::usecase::get_template::GetTemplateError;

    #[tokio::test]
    async fn success_returns_existing_template() {
        let template = make_template("welcome", "email");
        let template_id = template.id.clone();
        let repo = Arc::new(StubTemplateRepo::with_templates(vec![template]));
        let uc = GetTemplateUseCase::new(repo);

        let result = uc.execute(&template_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, template_id);
    }

    #[tokio::test]
    async fn not_found_for_missing_template() {
        let repo = Arc::new(StubTemplateRepo::new());
        let uc = GetTemplateUseCase::new(repo);

        let result = uc.execute("tpl_nonexistent").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            GetTemplateError::NotFound(id) => assert_eq!(id, "tpl_nonexistent"),
            e => panic!("expected NotFound, got: {:?}", e),
        }
    }
}

// ===========================================================================
// ListTemplates tests
// ===========================================================================

mod list_templates {
    use super::*;
    #[tokio::test]
    async fn success_returns_empty_list() {
        let repo = Arc::new(StubTemplateRepo::new());
        let uc = ListTemplatesUseCase::new(repo);

        let result = uc.execute().await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn success_returns_all_templates() {
        let templates = vec![make_template("t1", "email"), make_template("t2", "slack")];
        let repo = Arc::new(StubTemplateRepo::with_templates(templates));
        let uc = ListTemplatesUseCase::new(repo);

        let result = uc.execute().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn paginated_filters_by_channel_type() {
        let templates = vec![
            make_template("t1", "email"),
            make_template("t2", "slack"),
            make_template("t3", "email"),
        ];
        let repo = Arc::new(StubTemplateRepo::with_templates(templates));
        let uc = ListTemplatesUseCase::new(repo);

        let (items, total) = uc
            .execute_paginated(1, 10, Some("email".to_string()))
            .await
            .unwrap();
        assert_eq!(total, 2);
        assert!(items.iter().all(|t| t.channel_type == "email"));
    }

    #[tokio::test]
    async fn paginated_respects_page_size() {
        let templates = vec![
            make_template("t1", "email"),
            make_template("t2", "slack"),
            make_template("t3", "sms"),
        ];
        let repo = Arc::new(StubTemplateRepo::with_templates(templates));
        let uc = ListTemplatesUseCase::new(repo);

        let (items, total) = uc.execute_paginated(1, 2, None).await.unwrap();
        assert_eq!(total, 3);
        assert_eq!(items.len(), 2);

        let (items, _) = uc.execute_paginated(2, 2, None).await.unwrap();
        assert_eq!(items.len(), 1);
    }
}

// ===========================================================================
// UpdateTemplate tests
// ===========================================================================

mod update_template {
    use super::*;
    use k1s0_notification_server::usecase::update_template::{
        UpdateTemplateError, UpdateTemplateInput,
    };

    #[tokio::test]
    async fn success_updates_name_and_body() {
        let template = make_template("old-name", "email");
        let template_id = template.id.clone();
        let repo = Arc::new(StubTemplateRepo::with_templates(vec![template]));
        let uc = UpdateTemplateUseCase::new(repo.clone());

        let input = UpdateTemplateInput {
            id: template_id.clone(),
            name: Some("new-name".to_string()),
            subject_template: None,
            body_template: Some("Updated body: {{content}}".to_string()),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.name, "new-name");
        assert_eq!(updated.body_template, "Updated body: {{content}}");

        // Verify persisted
        let stored = repo.templates.read().await;
        assert_eq!(stored[0].name, "new-name");
    }

    #[tokio::test]
    async fn success_updates_subject_only() {
        let template = make_template("tpl", "email");
        let template_id = template.id.clone();
        let repo = Arc::new(StubTemplateRepo::with_templates(vec![template]));
        let uc = UpdateTemplateUseCase::new(repo);

        let input = UpdateTemplateInput {
            id: template_id,
            name: None,
            subject_template: Some("New Subject".to_string()),
            body_template: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.subject_template.as_deref(), Some("New Subject"));
        // Name remains unchanged
        assert_eq!(updated.name, "tpl");
    }

    #[tokio::test]
    async fn success_updated_at_changes() {
        let template = make_template("tpl", "email");
        let template_id = template.id.clone();
        let original_updated_at = template.updated_at;
        let repo = Arc::new(StubTemplateRepo::with_templates(vec![template]));
        let uc = UpdateTemplateUseCase::new(repo);

        let input = UpdateTemplateInput {
            id: template_id,
            name: Some("renamed".to_string()),
            subject_template: None,
            body_template: None,
        };

        let result = uc.execute(&input).await.unwrap();
        assert!(result.updated_at >= original_updated_at);
    }

    #[tokio::test]
    async fn not_found_for_missing_template() {
        let repo = Arc::new(StubTemplateRepo::new());
        let uc = UpdateTemplateUseCase::new(repo);

        let input = UpdateTemplateInput {
            id: "tpl_nonexistent".to_string(),
            name: Some("x".to_string()),
            subject_template: None,
            body_template: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            UpdateTemplateError::NotFound(id) => assert_eq!(id, "tpl_nonexistent"),
            e => panic!("expected NotFound, got: {:?}", e),
        }
    }
}

// ===========================================================================
// DeleteTemplate tests
// ===========================================================================

mod delete_template {
    use super::*;
    use k1s0_notification_server::usecase::delete_template::DeleteTemplateError;

    #[tokio::test]
    async fn success_deletes_existing_template() {
        let template = make_template("to-delete", "email");
        let template_id = template.id.clone();
        let repo = Arc::new(StubTemplateRepo::with_templates(vec![template]));
        let uc = DeleteTemplateUseCase::new(repo.clone());

        let result = uc.execute(&template_id).await;
        assert!(result.is_ok());

        let stored = repo.templates.read().await;
        assert!(stored.is_empty());
    }

    #[tokio::test]
    async fn not_found_for_missing_template() {
        let repo = Arc::new(StubTemplateRepo::new());
        let uc = DeleteTemplateUseCase::new(repo);

        let result = uc.execute("tpl_nonexistent").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            DeleteTemplateError::NotFound(id) => assert_eq!(id, "tpl_nonexistent"),
            e => panic!("expected NotFound, got: {:?}", e),
        }
    }

    #[tokio::test]
    async fn internal_error_on_repo_failure() {
        let repo = Arc::new(StubTemplateRepo::failing());
        let uc = DeleteTemplateUseCase::new(repo);

        let result = uc.execute("tpl_any").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            DeleteTemplateError::Internal(msg) => assert!(msg.contains("db write error")),
            e => panic!("expected Internal, got: {:?}", e),
        }
    }
}

// ===========================================================================
// SendNotification tests
// ===========================================================================

mod send_notification {
    use super::*;
    use k1s0_notification_server::usecase::send_notification::{
        SendNotificationError, SendNotificationInput,
    };

    #[tokio::test]
    async fn success_sends_without_template() {
        let channel = make_channel("email-ch", "email", true);
        let channel_id = channel.id.clone();
        let channel_repo = Arc::new(StubChannelRepo::with_channels(vec![channel]));
        let log_repo = Arc::new(StubLogRepo::new());

        let uc = SendNotificationUseCase::new(channel_repo, log_repo.clone());
        let input = SendNotificationInput {
            channel_id: channel_id.clone(),
            template_id: None,
            recipient: "user@example.com".to_string(),
            subject: Some("Test Subject".to_string()),
            body: "Test body content".to_string(),
            template_variables: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.status, "sent");
        assert!(output.log_id.starts_with("notif_"));

        // Verify log was persisted
        let logs = log_repo.logs.read().await;
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].status, "sent");
        assert_eq!(logs[0].recipient, "user@example.com");
    }

    #[tokio::test]
    async fn success_sends_with_template() {
        let channel = make_channel("email-ch", "email", true);
        let channel_id = channel.id.clone();
        let template = make_template("welcome", "email");
        let template_id = template.id.clone();

        let channel_repo = Arc::new(StubChannelRepo::with_channels(vec![channel]));
        let log_repo = Arc::new(StubLogRepo::new());
        let template_repo = Arc::new(StubTemplateRepo::with_templates(vec![template]));

        let uc = SendNotificationUseCase::with_template_repo(
            channel_repo,
            log_repo.clone(),
            template_repo,
        );
        let input = SendNotificationInput {
            channel_id,
            template_id: Some(template_id),
            recipient: "user@example.com".to_string(),
            subject: None,
            body: String::new(),
            template_variables: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, "sent");

        // Verify template_id was recorded on the log
        let logs = log_repo.logs.read().await;
        assert!(logs[0].template_id.is_some());
    }

    #[tokio::test]
    async fn success_with_template_variable_substitution() {
        let channel = make_channel("email-ch", "email", true);
        let channel_id = channel.id.clone();
        let channel_repo = Arc::new(StubChannelRepo::with_channels(vec![channel]));
        let log_repo = Arc::new(StubLogRepo::new());

        let uc = SendNotificationUseCase::new(channel_repo, log_repo.clone());

        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "Alice".to_string());
        vars.insert("order_id".to_string(), "42".to_string());

        let input = SendNotificationInput {
            channel_id,
            template_id: None,
            recipient: "alice@example.com".to_string(),
            subject: Some("Order {{order_id}} Confirmation".to_string()),
            body: "Hello {{name}}, your order {{order_id}} is ready.".to_string(),
            template_variables: Some(vars),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let logs = log_repo.logs.read().await;
        assert_eq!(logs[0].body, "Hello Alice, your order 42 is ready.");
        assert_eq!(logs[0].subject.as_deref(), Some("Order 42 Confirmation"));
    }

    #[tokio::test]
    async fn error_channel_not_found() {
        let channel_repo = Arc::new(StubChannelRepo::new());
        let log_repo = Arc::new(StubLogRepo::new());

        let uc = SendNotificationUseCase::new(channel_repo, log_repo);
        let input = SendNotificationInput {
            channel_id: "ch_missing".to_string(),
            template_id: None,
            recipient: "user@example.com".to_string(),
            subject: None,
            body: "Test".to_string(),
            template_variables: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            SendNotificationError::ChannelNotFound(id) => assert_eq!(id, "ch_missing"),
            e => panic!("expected ChannelNotFound, got: {:?}", e),
        }
    }

    #[tokio::test]
    async fn error_channel_disabled() {
        let channel = make_channel("disabled-ch", "email", false);
        let channel_id = channel.id.clone();
        let channel_repo = Arc::new(StubChannelRepo::with_channels(vec![channel]));
        let log_repo = Arc::new(StubLogRepo::new());

        let uc = SendNotificationUseCase::new(channel_repo, log_repo);
        let input = SendNotificationInput {
            channel_id: channel_id.clone(),
            template_id: None,
            recipient: "user@example.com".to_string(),
            subject: None,
            body: "Test".to_string(),
            template_variables: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            SendNotificationError::ChannelDisabled(id) => assert_eq!(id, channel_id),
            e => panic!("expected ChannelDisabled, got: {:?}", e),
        }
    }

    #[tokio::test]
    async fn error_template_not_found() {
        let channel = make_channel("email-ch", "email", true);
        let channel_id = channel.id.clone();
        let channel_repo = Arc::new(StubChannelRepo::with_channels(vec![channel]));
        let log_repo = Arc::new(StubLogRepo::new());
        let template_repo = Arc::new(StubTemplateRepo::new());

        let uc = SendNotificationUseCase::with_template_repo(channel_repo, log_repo, template_repo);
        let input = SendNotificationInput {
            channel_id,
            template_id: Some("tpl_missing".to_string()),
            recipient: "user@example.com".to_string(),
            subject: None,
            body: String::new(),
            template_variables: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            SendNotificationError::TemplateNotFound(id) => assert_eq!(id, "tpl_missing"),
            e => panic!("expected TemplateNotFound, got: {:?}", e),
        }
    }

    #[tokio::test]
    async fn delivery_client_success_marks_sent() {
        let channel = make_channel("email-ch", "email", true);
        let channel_id = channel.id.clone();
        let channel_repo = Arc::new(StubChannelRepo::with_channels(vec![channel]));
        let log_repo = Arc::new(StubLogRepo::new());

        let mut clients: HashMap<String, Arc<dyn DeliveryClient>> = HashMap::new();
        clients.insert("email".to_string(), Arc::new(StubDeliveryClient::success()));

        let uc =
            SendNotificationUseCase::with_delivery_clients(channel_repo, log_repo.clone(), clients);
        let input = SendNotificationInput {
            channel_id,
            template_id: None,
            recipient: "user@example.com".to_string(),
            subject: Some("Hello".to_string()),
            body: "Test".to_string(),
            template_variables: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, "sent");

        let logs = log_repo.logs.read().await;
        assert!(logs[0].sent_at.is_some());
        assert!(logs[0].error_message.is_none());
    }

    #[tokio::test]
    async fn delivery_client_failure_records_error() {
        let channel = make_channel("slack-ch", "slack", true);
        let channel_id = channel.id.clone();
        let channel_repo = Arc::new(StubChannelRepo::with_channels(vec![channel]));
        let log_repo = Arc::new(StubLogRepo::new());

        let mut clients: HashMap<String, Arc<dyn DeliveryClient>> = HashMap::new();
        clients.insert("slack".to_string(), Arc::new(StubDeliveryClient::failing()));

        let uc =
            SendNotificationUseCase::with_delivery_clients(channel_repo, log_repo.clone(), clients);
        let input = SendNotificationInput {
            channel_id,
            template_id: None,
            recipient: "#general".to_string(),
            subject: None,
            body: "Alert!".to_string(),
            template_variables: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.status, "failed");

        let logs = log_repo.logs.read().await;
        assert_eq!(logs[0].status, "failed");
        assert!(logs[0]
            .error_message
            .as_ref()
            .unwrap()
            .contains("connection timeout"));
    }

    #[tokio::test]
    async fn no_delivery_client_for_channel_type_marks_failed() {
        let channel = make_channel("sms-ch", "sms", true);
        let channel_id = channel.id.clone();
        let channel_repo = Arc::new(StubChannelRepo::with_channels(vec![channel]));
        let log_repo = Arc::new(StubLogRepo::new());

        // Has email client but channel is sms
        let mut clients: HashMap<String, Arc<dyn DeliveryClient>> = HashMap::new();
        clients.insert("email".to_string(), Arc::new(StubDeliveryClient::success()));

        let uc =
            SendNotificationUseCase::with_delivery_clients(channel_repo, log_repo.clone(), clients);
        let input = SendNotificationInput {
            channel_id,
            template_id: None,
            recipient: "+1234567890".to_string(),
            subject: None,
            body: "Test SMS".to_string(),
            template_variables: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, "failed");

        let logs = log_repo.logs.read().await;
        assert!(logs[0]
            .error_message
            .as_ref()
            .unwrap()
            .contains("no delivery client"));
    }

    #[tokio::test]
    async fn log_repo_failure_returns_internal_error() {
        let channel = make_channel("email-ch", "email", true);
        let channel_id = channel.id.clone();
        let channel_repo = Arc::new(StubChannelRepo::with_channels(vec![channel]));
        let log_repo = Arc::new(StubLogRepo::failing());

        let uc = SendNotificationUseCase::new(channel_repo, log_repo);
        let input = SendNotificationInput {
            channel_id,
            template_id: None,
            recipient: "user@example.com".to_string(),
            subject: None,
            body: "Test".to_string(),
            template_variables: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            SendNotificationError::Internal(msg) => assert!(msg.contains("db write error")),
            e => panic!("expected Internal, got: {:?}", e),
        }
    }

    #[tokio::test]
    async fn template_id_not_set_without_template_repo_returns_internal_error() {
        let channel = make_channel("email-ch", "email", true);
        let channel_id = channel.id.clone();
        let channel_repo = Arc::new(StubChannelRepo::with_channels(vec![channel]));
        let log_repo = Arc::new(StubLogRepo::new());

        // Use new() which does NOT set template_repo
        let uc = SendNotificationUseCase::new(channel_repo, log_repo);
        let input = SendNotificationInput {
            channel_id,
            template_id: Some("tpl_123".to_string()),
            recipient: "user@example.com".to_string(),
            subject: None,
            body: String::new(),
            template_variables: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            SendNotificationError::Internal(msg) => {
                assert!(msg.contains("template repository is not configured"));
            }
            e => panic!("expected Internal, got: {:?}", e),
        }
    }

    #[tokio::test]
    async fn with_delivery_clients_and_template_repo() {
        let channel = make_channel("email-ch", "email", true);
        let channel_id = channel.id.clone();
        let template = make_template("welcome", "email");
        let template_id = template.id.clone();

        let channel_repo = Arc::new(StubChannelRepo::with_channels(vec![channel]));
        let log_repo = Arc::new(StubLogRepo::new());
        let template_repo = Arc::new(StubTemplateRepo::with_templates(vec![template]));

        let mut clients: HashMap<String, Arc<dyn DeliveryClient>> = HashMap::new();
        clients.insert("email".to_string(), Arc::new(StubDeliveryClient::success()));

        let uc = SendNotificationUseCase::with_delivery_clients_and_template_repo(
            channel_repo,
            log_repo.clone(),
            template_repo,
            clients,
        );

        let mut vars = HashMap::new();
        vars.insert("title".to_string(), "Welcome!".to_string());
        vars.insert("name".to_string(), "Bob".to_string());
        vars.insert("message".to_string(), "enjoy your stay".to_string());

        let input = SendNotificationInput {
            channel_id,
            template_id: Some(template_id.clone()),
            recipient: "bob@example.com".to_string(),
            subject: None,
            body: String::new(),
            template_variables: Some(vars),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, "sent");

        let logs = log_repo.logs.read().await;
        assert_eq!(logs[0].template_id.as_deref(), Some(template_id.as_str()));
        assert_eq!(logs[0].subject.as_deref(), Some("Subject: Welcome!"));
        assert_eq!(logs[0].body, "Hello Bob, enjoy your stay");
    }
}

// ===========================================================================
// RetryNotification tests
// ===========================================================================

mod retry_notification {
    use super::*;
    use k1s0_notification_server::usecase::retry_notification::{
        RetryNotificationError, RetryNotificationInput,
    };

    #[tokio::test]
    async fn success_retries_failed_notification() {
        let channel = make_channel("email-ch", "email", true);
        let channel_id = channel.id.clone();
        let log = make_failed_log(&channel_id);
        let log_id = log.id.clone();

        let log_repo = Arc::new(StubLogRepo::with_logs(vec![log]));
        let channel_repo = Arc::new(StubChannelRepo::with_channels(vec![channel]));

        let uc = RetryNotificationUseCase::new(log_repo.clone(), channel_repo);
        let input = RetryNotificationInput {
            notification_id: log_id.clone(),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        let retried = result.unwrap();
        assert_eq!(retried.status, "sent");
        assert_eq!(retried.retry_count, 1);
        assert!(retried.error_message.is_none());
        assert!(retried.sent_at.is_some());

        // Verify persisted
        let stored = log_repo.logs.read().await;
        assert_eq!(stored[0].status, "sent");
        assert_eq!(stored[0].retry_count, 1);
    }

    #[tokio::test]
    async fn error_notification_not_found() {
        let log_repo = Arc::new(StubLogRepo::new());
        let channel_repo = Arc::new(StubChannelRepo::new());

        let uc = RetryNotificationUseCase::new(log_repo, channel_repo);
        let input = RetryNotificationInput {
            notification_id: "notif_nonexistent".to_string(),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            RetryNotificationError::NotFound(id) => assert_eq!(id, "notif_nonexistent"),
            e => panic!("expected NotFound, got: {:?}", e),
        }
    }

    #[tokio::test]
    async fn error_already_sent() {
        let channel = make_channel("email-ch", "email", true);
        let channel_id = channel.id.clone();
        let mut log = make_failed_log(&channel_id);
        log.status = "sent".to_string();
        let log_id = log.id.clone();

        let log_repo = Arc::new(StubLogRepo::with_logs(vec![log]));
        let channel_repo = Arc::new(StubChannelRepo::with_channels(vec![channel]));

        let uc = RetryNotificationUseCase::new(log_repo, channel_repo);
        let input = RetryNotificationInput {
            notification_id: log_id.clone(),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            RetryNotificationError::AlreadySent(id) => assert_eq!(id, log_id),
            e => panic!("expected AlreadySent, got: {:?}", e),
        }
    }

    #[tokio::test]
    async fn error_channel_not_found_during_retry() {
        // Log references a channel that no longer exists
        let log = make_failed_log("ch_deleted");
        let log_id = log.id.clone();

        let log_repo = Arc::new(StubLogRepo::with_logs(vec![log]));
        let channel_repo = Arc::new(StubChannelRepo::new()); // empty - no channels

        let uc = RetryNotificationUseCase::new(log_repo, channel_repo);
        let input = RetryNotificationInput {
            notification_id: log_id,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            RetryNotificationError::ChannelNotFound(id) => assert_eq!(id, "ch_deleted"),
            e => panic!("expected ChannelNotFound, got: {:?}", e),
        }
    }

    #[tokio::test]
    async fn retry_increments_count_on_multiple_retries() {
        let channel = make_channel("email-ch", "email", true);
        let channel_id = channel.id.clone();
        let mut log = make_failed_log(&channel_id);
        log.retry_count = 3; // already retried 3 times
        let log_id = log.id.clone();

        let log_repo = Arc::new(StubLogRepo::with_logs(vec![log]));
        let channel_repo = Arc::new(StubChannelRepo::with_channels(vec![channel]));

        let uc = RetryNotificationUseCase::new(log_repo, channel_repo);
        let input = RetryNotificationInput {
            notification_id: log_id,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().retry_count, 4);
    }
}

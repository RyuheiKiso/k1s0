pub struct NotificationDomainService;

impl NotificationDomainService {
    pub fn is_supported_channel_type(channel_type: &str) -> bool {
        matches!(channel_type, "email" | "slack" | "webhook" | "sms" | "push")
    }

    pub fn validate_channel_type(channel_type: &str) -> Result<(), String> {
        if Self::is_supported_channel_type(channel_type) {
            Ok(())
        } else {
            Err(format!(
                "invalid channel_type: {} (allowed: email, slack, webhook, sms, push)",
                channel_type
            ))
        }
    }

    pub fn validate_template_body(template: &str) -> Result<(), String> {
        let mut handlebars = handlebars::Handlebars::new();
        handlebars
            .register_template_string("template", template)
            .map_err(|e| format!("invalid template syntax: {}", e))
    }

    pub fn is_retryable_status(status: &str) -> bool {
        status != "sent"
    }
}

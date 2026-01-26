//! 監査ログモジュール
//!
//! 認証・認可・操作の監査ログを標準形式で出力

use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use uuid::Uuid;

use crate::jwt::Claims;
use crate::policy::{PolicyDecision, PolicyResult};

/// 監査イベントタイプ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    /// 認証成功
    AuthenticationSuccess,
    /// 認証失敗
    AuthenticationFailure,
    /// 認可成功
    AuthorizationSuccess,
    /// 認可失敗
    AuthorizationFailure,
    /// トークンリフレッシュ
    TokenRefresh,
    /// トークン無効化
    TokenRevocation,
    /// ログアウト
    Logout,
    /// リソース作成
    ResourceCreate,
    /// リソース読み取り
    ResourceRead,
    /// リソース更新
    ResourceUpdate,
    /// リソース削除
    ResourceDelete,
    /// 設定変更
    ConfigurationChange,
    /// 権限変更
    PermissionChange,
    /// パスワード変更
    PasswordChange,
    /// アカウントロック
    AccountLock,
    /// アカウントアンロック
    AccountUnlock,
    /// セッション開始
    SessionStart,
    /// セッション終了
    SessionEnd,
    /// カスタムイベント
    Custom,
}

impl AuditEventType {
    /// セキュリティ関連イベントかどうか
    pub fn is_security_event(&self) -> bool {
        matches!(
            self,
            Self::AuthenticationSuccess
                | Self::AuthenticationFailure
                | Self::AuthorizationFailure
                | Self::TokenRevocation
                | Self::PermissionChange
                | Self::PasswordChange
                | Self::AccountLock
                | Self::AccountUnlock
        )
    }
}

/// 監査イベント結果
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditResult {
    /// 成功
    Success,
    /// 失敗
    Failure,
    /// 拒否
    Denied,
    /// 不明
    Unknown,
}

impl From<PolicyDecision> for AuditResult {
    fn from(decision: PolicyDecision) -> Self {
        match decision {
            PolicyDecision::Allow => Self::Success,
            PolicyDecision::Deny => Self::Denied,
            PolicyDecision::NotApplicable => Self::Unknown,
        }
    }
}

/// 監査対象（誰が）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditActor {
    /// ユーザーID
    pub user_id: String,
    /// ユーザー名（オプション）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// メールアドレス（オプション）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// IPアドレス
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,
    /// ユーザーエージェント
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    /// セッションID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// テナントID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
    /// ロール
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub roles: Vec<String>,
}

impl AuditActor {
    /// 新しいアクターを作成
    pub fn new(user_id: impl Into<String>) -> Self {
        Self {
            user_id: user_id.into(),
            username: None,
            email: None,
            ip_address: None,
            user_agent: None,
            session_id: None,
            tenant_id: None,
            roles: Vec::new(),
        }
    }

    /// JWTクレームから作成
    pub fn from_claims(claims: &Claims) -> Self {
        Self {
            user_id: claims.sub.clone(),
            username: claims.name.clone(),
            email: claims.email.clone(),
            ip_address: None,
            user_agent: None,
            session_id: None,
            tenant_id: claims.tenant_id.clone(),
            roles: claims.roles.clone(),
        }
    }

    /// システムアクターを作成
    pub fn system() -> Self {
        Self {
            user_id: "system".to_string(),
            username: Some("System".to_string()),
            email: None,
            ip_address: None,
            user_agent: None,
            session_id: None,
            tenant_id: None,
            roles: vec!["system".to_string()],
        }
    }

    /// 匿名アクターを作成
    pub fn anonymous() -> Self {
        Self {
            user_id: "anonymous".to_string(),
            username: None,
            email: None,
            ip_address: None,
            user_agent: None,
            session_id: None,
            tenant_id: None,
            roles: Vec::new(),
        }
    }

    /// IPアドレスを設定
    pub fn with_ip_address(mut self, ip: impl Into<String>) -> Self {
        self.ip_address = Some(ip.into());
        self
    }

    /// ユーザーエージェントを設定
    pub fn with_user_agent(mut self, ua: impl Into<String>) -> Self {
        self.user_agent = Some(ua.into());
        self
    }

    /// セッションIDを設定
    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }
}

/// 監査対象リソース（何を）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditResource {
    /// リソースタイプ
    pub resource_type: String,
    /// リソースID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<String>,
    /// リソース名
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_name: Option<String>,
    /// 追加属性
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub attributes: HashMap<String, String>,
}

impl AuditResource {
    /// 新しいリソースを作成
    pub fn new(resource_type: impl Into<String>) -> Self {
        Self {
            resource_type: resource_type.into(),
            resource_id: None,
            resource_name: None,
            attributes: HashMap::new(),
        }
    }

    /// リソースIDを設定
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.resource_id = Some(id.into());
        self
    }

    /// リソース名を設定
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.resource_name = Some(name.into());
        self
    }

    /// 属性を追加
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }
}

/// 監査イベント
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// イベントID
    pub event_id: String,
    /// イベントタイプ
    pub event_type: AuditEventType,
    /// カスタムイベント名（event_type が Custom の場合）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_event_name: Option<String>,
    /// タイムスタンプ
    pub timestamp: DateTime<Utc>,
    /// 結果
    pub result: AuditResult,
    /// アクター（誰が）
    pub actor: AuditActor,
    /// 操作（何を）
    pub action: String,
    /// リソース
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource: Option<AuditResource>,
    /// 理由/詳細
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// サービス名
    pub service: String,
    /// トレースID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    /// スパンID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span_id: Option<String>,
    /// 追加メタデータ
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl AuditEvent {
    /// 新しいイベントを作成
    pub fn new(
        event_type: AuditEventType,
        actor: AuditActor,
        action: impl Into<String>,
        service: impl Into<String>,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4().to_string(),
            event_type,
            custom_event_name: None,
            timestamp: Utc::now(),
            result: AuditResult::Unknown,
            actor,
            action: action.into(),
            resource: None,
            reason: None,
            service: service.into(),
            trace_id: None,
            span_id: None,
            metadata: HashMap::new(),
        }
    }

    /// カスタムイベントを作成
    pub fn custom(
        event_name: impl Into<String>,
        actor: AuditActor,
        action: impl Into<String>,
        service: impl Into<String>,
    ) -> Self {
        let mut event = Self::new(AuditEventType::Custom, actor, action, service);
        event.custom_event_name = Some(event_name.into());
        event
    }

    /// 結果を設定
    pub fn with_result(mut self, result: AuditResult) -> Self {
        self.result = result;
        self
    }

    /// リソースを設定
    pub fn with_resource(mut self, resource: AuditResource) -> Self {
        self.resource = Some(resource);
        self
    }

    /// 理由を設定
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    /// トレースIDを設定
    pub fn with_trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = Some(trace_id.into());
        self
    }

    /// スパンIDを設定
    pub fn with_span_id(mut self, span_id: impl Into<String>) -> Self {
        self.span_id = Some(span_id.into());
        self
    }

    /// メタデータを追加
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// PolicyResultから監査イベントを作成
    pub fn from_policy_result(
        policy_result: &PolicyResult,
        actor: AuditActor,
        action: impl Into<String>,
        service: impl Into<String>,
    ) -> Self {
        let event_type = match policy_result.decision {
            PolicyDecision::Allow => AuditEventType::AuthorizationSuccess,
            PolicyDecision::Deny | PolicyDecision::NotApplicable => {
                AuditEventType::AuthorizationFailure
            }
        };

        let mut event = Self::new(event_type, actor, action, service)
            .with_result(policy_result.decision.into());

        if let Some(ref policy) = policy_result.matched_policy {
            event = event.with_metadata("matched_policy", serde_json::json!(policy));
        }

        if let Some(ref reason) = policy_result.reason {
            event = event.with_reason(reason);
        }

        event
    }
}

/// 監査ログ出力先トレイト
pub trait AuditSink: Send + Sync + 'static {
    /// イベントを出力
    fn emit(&self, event: &AuditEvent);
}

/// 標準出力シンク（tracing経由）
pub struct TracingAuditSink {
    /// サービス名
    service_name: String,
}

impl TracingAuditSink {
    /// 新しいシンクを作成
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
        }
    }
}

impl AuditSink for TracingAuditSink {
    fn emit(&self, event: &AuditEvent) {
        let json = serde_json::to_string(event).unwrap_or_else(|_| "{}".to_string());

        if event.event_type.is_security_event() || matches!(event.result, AuditResult::Failure | AuditResult::Denied) {
            warn!(
                audit.event_id = %event.event_id,
                audit.event_type = ?event.event_type,
                audit.actor.user_id = %event.actor.user_id,
                audit.action = %event.action,
                audit.result = ?event.result,
                audit.service = %event.service,
                audit.json = %json,
                "Security audit event"
            );
        } else {
            info!(
                audit.event_id = %event.event_id,
                audit.event_type = ?event.event_type,
                audit.actor.user_id = %event.actor.user_id,
                audit.action = %event.action,
                audit.result = ?event.result,
                audit.service = %event.service,
                audit.json = %json,
                "Audit event"
            );
        }
    }
}

/// JSON Lines出力シンク
pub struct JsonLinesAuditSink<W: std::io::Write + Send + Sync + 'static> {
    writer: std::sync::Mutex<W>,
}

impl<W: std::io::Write + Send + Sync + 'static> JsonLinesAuditSink<W> {
    /// 新しいシンクを作成
    pub fn new(writer: W) -> Self {
        Self {
            writer: std::sync::Mutex::new(writer),
        }
    }
}

impl<W: std::io::Write + Send + Sync + 'static> AuditSink for JsonLinesAuditSink<W> {
    fn emit(&self, event: &AuditEvent) {
        if let Ok(json) = serde_json::to_string(event) {
            if let Ok(mut writer) = self.writer.lock() {
                let _ = writeln!(writer, "{}", json);
            }
        }
    }
}

/// 監査ロガー
pub struct AuditLogger {
    sinks: Vec<Arc<dyn AuditSink>>,
    service_name: String,
}

impl AuditLogger {
    /// 新しいロガーを作成
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            sinks: Vec::new(),
            service_name: service_name.into(),
        }
    }

    /// デフォルトのロガーを作成（tracing出力）
    pub fn with_default_sink(service_name: impl Into<String>) -> Self {
        let name = service_name.into();
        let mut logger = Self::new(&name);
        logger.add_sink(Arc::new(TracingAuditSink::new(&name)));
        logger
    }

    /// シンクを追加
    pub fn add_sink(&mut self, sink: Arc<dyn AuditSink>) {
        self.sinks.push(sink);
    }

    /// イベントを記録
    pub fn log(&self, event: AuditEvent) {
        for sink in &self.sinks {
            sink.emit(&event);
        }
    }

    /// 認証成功を記録
    pub fn log_authentication_success(&self, actor: AuditActor) {
        let event = AuditEvent::new(
            AuditEventType::AuthenticationSuccess,
            actor,
            "authenticate",
            &self.service_name,
        )
        .with_result(AuditResult::Success);
        self.log(event);
    }

    /// 認証失敗を記録
    pub fn log_authentication_failure(&self, actor: AuditActor, reason: impl Into<String>) {
        let event = AuditEvent::new(
            AuditEventType::AuthenticationFailure,
            actor,
            "authenticate",
            &self.service_name,
        )
        .with_result(AuditResult::Failure)
        .with_reason(reason);
        self.log(event);
    }

    /// 認可イベントを記録
    pub fn log_authorization(&self, actor: AuditActor, action: impl Into<String>, result: &PolicyResult) {
        let event = AuditEvent::from_policy_result(result, actor, action, &self.service_name);
        self.log(event);
    }

    /// リソース操作を記録
    pub fn log_resource_operation(
        &self,
        event_type: AuditEventType,
        actor: AuditActor,
        resource: AuditResource,
        result: AuditResult,
    ) {
        let action = match event_type {
            AuditEventType::ResourceCreate => "create",
            AuditEventType::ResourceRead => "read",
            AuditEventType::ResourceUpdate => "update",
            AuditEventType::ResourceDelete => "delete",
            _ => "operation",
        };
        let event = AuditEvent::new(event_type, actor, action, &self.service_name)
            .with_resource(resource)
            .with_result(result);
        self.log(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::sync::Arc;

    #[test]
    fn test_audit_event_serialization() {
        let actor = AuditActor::new("user123")
            .with_ip_address("192.168.1.1");
        let resource = AuditResource::new("order")
            .with_id("order-456");
        let event = AuditEvent::new(
            AuditEventType::ResourceUpdate,
            actor,
            "update_status",
            "order-service",
        )
        .with_resource(resource)
        .with_result(AuditResult::Success);

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"event_type\":\"resource_update\""));
        assert!(json.contains("\"user_id\":\"user123\""));
        assert!(json.contains("\"resource_type\":\"order\""));
    }

    #[test]
    fn test_audit_actor_from_claims() {
        let claims = Claims {
            sub: "user789".to_string(),
            iss: "test".to_string(),
            aud: None,
            exp: 0,
            iat: 0,
            nbf: None,
            jti: None,
            roles: vec!["admin".to_string()],
            permissions: vec![],
            tenant_id: Some("tenant1".to_string()),
            email: Some("user@example.com".to_string()),
            email_verified: Some(true),
            name: Some("Test User".to_string()),
        };

        let actor = AuditActor::from_claims(&claims);
        assert_eq!(actor.user_id, "user789");
        assert_eq!(actor.email, Some("user@example.com".to_string()));
        assert_eq!(actor.tenant_id, Some("tenant1".to_string()));
    }

    #[test]
    fn test_audit_result_from_policy_decision() {
        assert_eq!(AuditResult::from(PolicyDecision::Allow), AuditResult::Success);
        assert_eq!(AuditResult::from(PolicyDecision::Deny), AuditResult::Denied);
        assert_eq!(AuditResult::from(PolicyDecision::NotApplicable), AuditResult::Unknown);
    }

    #[test]
    fn test_security_event_detection() {
        assert!(AuditEventType::AuthenticationFailure.is_security_event());
        assert!(AuditEventType::AuthorizationFailure.is_security_event());
        assert!(AuditEventType::PermissionChange.is_security_event());
        assert!(!AuditEventType::ResourceRead.is_security_event());
        assert!(!AuditEventType::ResourceCreate.is_security_event());
    }

    #[test]
    fn test_audit_logger() {
        let logger = AuditLogger::new("test-service");
        let actor = AuditActor::new("test-user");

        // Should not panic
        logger.log_authentication_success(actor.clone());
        logger.log_authentication_failure(actor, "invalid password");
    }
}

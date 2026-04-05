use crate::domain::model::auth::{
    audit_event_type_to_str, audit_result_to_str, AuditEventType, AuditResult,
};
use crate::domain::model::{AuditLog, RecordAuditLogPayload, UserError};
use crate::infrastructure::grpc::AuthGrpcClient;
use std::sync::Arc;
use tracing::instrument;

pub struct AuthMutationResolver {
    client: Arc<AuthGrpcClient>,
}

impl AuthMutationResolver {
    pub fn new(client: Arc<AuthGrpcClient>) -> Self {
        Self { client }
    }

    #[allow(clippy::too_many_arguments)]
    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn record_audit_log(
        &self,
        event_type: AuditEventType,
        user_id: &str,
        ip_address: &str,
        user_agent: &str,
        resource: &str,
        action: &str,
        result: AuditResult,
        resource_id: Option<&str>,
        trace_id: Option<&str>,
    ) -> RecordAuditLogPayload {
        match self
            .client
            .record_audit_log(
                event_type,
                user_id,
                ip_address,
                user_agent,
                resource,
                action,
                result,
                resource_id.unwrap_or(""),
                trace_id.unwrap_or(""),
            )
            .await
        {
            Ok((id, created_at)) => RecordAuditLogPayload {
                audit_log: Some(AuditLog {
                    id,
                    // HIGH-014 監査対応: GraphQL deprecated string フィールドは enum から逆変換して提供する
                    event_type: audit_event_type_to_str(event_type).to_string(),
                    user_id: user_id.to_string(),
                    ip_address: ip_address.to_string(),
                    user_agent: user_agent.to_string(),
                    resource: resource.to_string(),
                    action: action.to_string(),
                    result: audit_result_to_str(result).to_string(),
                    resource_id: resource_id.unwrap_or("").to_string(),
                    trace_id: trace_id.unwrap_or("").to_string(),
                    created_at,
                    event_type_enum: Some(event_type),
                    result_enum: Some(result),
                }),
                errors: vec![],
            },
            Err(e) => RecordAuditLogPayload {
                audit_log: None,
                errors: vec![UserError {
                    field: None,
                    message: e.to_string(),
                }],
            },
        }
    }
}

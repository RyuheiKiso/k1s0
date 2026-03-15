use std::sync::Arc;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::entity::rule::{Rule, RuleSet};
use crate::usecase::create_rule::{CreateRuleError, CreateRuleInput, CreateRuleUseCase};
use crate::usecase::create_rule_set::{
    CreateRuleSetError, CreateRuleSetInput, CreateRuleSetUseCase,
};
use crate::usecase::delete_rule::{DeleteRuleError, DeleteRuleUseCase};
use crate::usecase::delete_rule_set::{DeleteRuleSetError, DeleteRuleSetUseCase};
use crate::usecase::evaluate::{EvaluateError, EvaluateInput, EvaluateUseCase};
use crate::usecase::get_rule::{GetRuleError, GetRuleUseCase};
use crate::usecase::get_rule_set::{GetRuleSetError, GetRuleSetUseCase};
use crate::usecase::list_rule_sets::{ListRuleSetsError, ListRuleSetsInput, ListRuleSetsUseCase};
use crate::usecase::list_rules::{ListRulesError, ListRulesInput, ListRulesUseCase};
use crate::usecase::publish_rule_set::{PublishRuleSetError, PublishRuleSetUseCase};
use crate::usecase::rollback_rule_set::{RollbackRuleSetError, RollbackRuleSetUseCase};
use crate::usecase::update_rule::{UpdateRuleError, UpdateRuleInput, UpdateRuleUseCase};
use crate::usecase::update_rule_set::{
    UpdateRuleSetError, UpdateRuleSetInput, UpdateRuleSetUseCase,
};

#[derive(Debug, Clone)]
pub struct RuleData {
    pub id: String,
    pub name: String,
    pub description: String,
    pub priority: i32,
    pub when_json: Vec<u8>,
    pub then_json: Vec<u8>,
    pub enabled: bool,
    pub version: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct RuleSetData {
    pub id: String,
    pub name: String,
    pub description: String,
    pub domain: String,
    pub evaluation_mode: String,
    pub default_result_json: Vec<u8>,
    pub rule_ids: Vec<String>,
    pub current_version: u32,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum GrpcError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("invalid argument: {0}")]
    InvalidArgument(String),
    #[error("already exists: {0}")]
    AlreadyExists(String),
    #[error("failed precondition: {0}")]
    FailedPrecondition(String),
    #[error("internal: {0}")]
    Internal(String),
}

pub struct RuleEngineGrpcService {
    create_rule_uc: Arc<CreateRuleUseCase>,
    get_rule_uc: Arc<GetRuleUseCase>,
    update_rule_uc: Arc<UpdateRuleUseCase>,
    delete_rule_uc: Arc<DeleteRuleUseCase>,
    list_rules_uc: Arc<ListRulesUseCase>,
    create_rule_set_uc: Arc<CreateRuleSetUseCase>,
    get_rule_set_uc: Arc<GetRuleSetUseCase>,
    update_rule_set_uc: Arc<UpdateRuleSetUseCase>,
    delete_rule_set_uc: Arc<DeleteRuleSetUseCase>,
    list_rule_sets_uc: Arc<ListRuleSetsUseCase>,
    publish_rule_set_uc: Arc<PublishRuleSetUseCase>,
    rollback_rule_set_uc: Arc<RollbackRuleSetUseCase>,
    evaluate_uc: Arc<EvaluateUseCase>,
}

impl RuleEngineGrpcService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        create_rule_uc: Arc<CreateRuleUseCase>,
        get_rule_uc: Arc<GetRuleUseCase>,
        update_rule_uc: Arc<UpdateRuleUseCase>,
        delete_rule_uc: Arc<DeleteRuleUseCase>,
        list_rules_uc: Arc<ListRulesUseCase>,
        create_rule_set_uc: Arc<CreateRuleSetUseCase>,
        get_rule_set_uc: Arc<GetRuleSetUseCase>,
        update_rule_set_uc: Arc<UpdateRuleSetUseCase>,
        delete_rule_set_uc: Arc<DeleteRuleSetUseCase>,
        list_rule_sets_uc: Arc<ListRuleSetsUseCase>,
        publish_rule_set_uc: Arc<PublishRuleSetUseCase>,
        rollback_rule_set_uc: Arc<RollbackRuleSetUseCase>,
        evaluate_uc: Arc<EvaluateUseCase>,
    ) -> Self {
        Self {
            create_rule_uc,
            get_rule_uc,
            update_rule_uc,
            delete_rule_uc,
            list_rules_uc,
            create_rule_set_uc,
            get_rule_set_uc,
            update_rule_set_uc,
            delete_rule_set_uc,
            list_rule_sets_uc,
            publish_rule_set_uc,
            rollback_rule_set_uc,
            evaluate_uc,
        }
    }

    pub async fn get_rule(&self, id: String) -> Result<RuleData, GrpcError> {
        let uid = Uuid::parse_str(&id)
            .map_err(|_| GrpcError::InvalidArgument(format!("invalid rule id: {}", id)))?;
        let rule = self
            .get_rule_uc
            .execute(&uid)
            .await
            .map_err(|e| match e {
                GetRuleError::Internal(msg) => GrpcError::Internal(msg),
            })?
            .ok_or_else(|| GrpcError::NotFound(format!("rule not found: {}", id)))?;
        Ok(to_rule_data(rule))
    }

    pub async fn list_rules(
        &self,
        page: i32,
        page_size: i32,
        rule_set_id: Option<String>,
        domain: Option<String>,
    ) -> Result<(Vec<RuleData>, u64, i32, i32, bool), GrpcError> {
        let page = if page <= 0 { 1 } else { page as u32 };
        let page_size = if page_size <= 0 { 20 } else { page_size as u32 };

        let rule_set_uuid =
            match rule_set_id.as_deref() {
                Some(id) => Some(Uuid::parse_str(id).map_err(|_| {
                    GrpcError::InvalidArgument(format!("invalid rule_set_id: {}", id))
                })?),
                None => None,
            };

        let output = self
            .list_rules_uc
            .execute(&ListRulesInput {
                page,
                page_size,
                rule_set_id: rule_set_uuid,
                domain,
            })
            .await
            .map_err(|e| match e {
                ListRulesError::Internal(msg) => GrpcError::Internal(msg),
            })?;

        Ok((
            output.rules.into_iter().map(to_rule_data).collect(),
            output.total_count,
            output.page as i32,
            output.page_size as i32,
            output.has_next,
        ))
    }

    pub async fn create_rule(
        &self,
        name: String,
        description: String,
        priority: i32,
        when_json: Vec<u8>,
        then_json: Vec<u8>,
    ) -> Result<RuleData, GrpcError> {
        let when_condition: serde_json::Value = if when_json.is_empty() {
            serde_json::json!({})
        } else {
            serde_json::from_slice(&when_json)
                .map_err(|e| GrpcError::InvalidArgument(format!("invalid when_json: {}", e)))?
        };
        let then_result: serde_json::Value = if then_json.is_empty() {
            serde_json::json!({})
        } else {
            serde_json::from_slice(&then_json)
                .map_err(|e| GrpcError::InvalidArgument(format!("invalid then_json: {}", e)))?
        };

        let rule = self
            .create_rule_uc
            .execute(&CreateRuleInput {
                name,
                description,
                priority,
                when_condition,
                then_result,
            })
            .await
            .map_err(|e| match e {
                CreateRuleError::AlreadyExists(name) => {
                    GrpcError::AlreadyExists(format!("rule already exists: {}", name))
                }
                CreateRuleError::Validation(msg) => GrpcError::InvalidArgument(msg),
                CreateRuleError::InvalidCondition(msg) => GrpcError::InvalidArgument(msg),
                CreateRuleError::Internal(msg) => GrpcError::Internal(msg),
            })?;

        Ok(to_rule_data(rule))
    }

    pub async fn update_rule(
        &self,
        id: String,
        description: Option<String>,
        priority: Option<i32>,
        when_json: Option<Vec<u8>>,
        then_json: Option<Vec<u8>>,
        enabled: Option<bool>,
    ) -> Result<RuleData, GrpcError> {
        let uid = Uuid::parse_str(&id)
            .map_err(|_| GrpcError::InvalidArgument(format!("invalid rule id: {}", id)))?;

        let when_condition = match when_json {
            Some(bytes) if !bytes.is_empty() => Some(
                serde_json::from_slice(&bytes)
                    .map_err(|e| GrpcError::InvalidArgument(format!("invalid when_json: {}", e)))?,
            ),
            _ => None,
        };
        let then_result = match then_json {
            Some(bytes) if !bytes.is_empty() => Some(
                serde_json::from_slice(&bytes)
                    .map_err(|e| GrpcError::InvalidArgument(format!("invalid then_json: {}", e)))?,
            ),
            _ => None,
        };

        let rule = self
            .update_rule_uc
            .execute(&UpdateRuleInput {
                id: uid,
                description,
                priority,
                when_condition,
                then_result,
                enabled,
            })
            .await
            .map_err(|e| match e {
                UpdateRuleError::NotFound(id) => {
                    GrpcError::NotFound(format!("rule not found: {}", id))
                }
                UpdateRuleError::Validation(msg) => GrpcError::InvalidArgument(msg),
                UpdateRuleError::InvalidCondition(msg) => GrpcError::InvalidArgument(msg),
                UpdateRuleError::Internal(msg) => GrpcError::Internal(msg),
            })?;

        Ok(to_rule_data(rule))
    }

    pub async fn delete_rule(&self, id: String) -> Result<(bool, String), GrpcError> {
        let uid = Uuid::parse_str(&id)
            .map_err(|_| GrpcError::InvalidArgument(format!("invalid rule id: {}", id)))?;

        self.delete_rule_uc
            .execute(&uid)
            .await
            .map_err(|e| match e {
                DeleteRuleError::NotFound(id) => {
                    GrpcError::NotFound(format!("rule not found: {}", id))
                }
                DeleteRuleError::Internal(msg) => GrpcError::Internal(msg),
            })?;

        Ok((true, format!("rule {} deleted", id)))
    }

    pub async fn get_rule_set(&self, id: String) -> Result<RuleSetData, GrpcError> {
        let uid = Uuid::parse_str(&id)
            .map_err(|_| GrpcError::InvalidArgument(format!("invalid rule set id: {}", id)))?;
        let rs = self
            .get_rule_set_uc
            .execute(&uid)
            .await
            .map_err(|e| match e {
                GetRuleSetError::Internal(msg) => GrpcError::Internal(msg),
            })?
            .ok_or_else(|| GrpcError::NotFound(format!("rule set not found: {}", id)))?;
        Ok(to_rule_set_data(rs))
    }

    pub async fn list_rule_sets(
        &self,
        page: i32,
        page_size: i32,
        domain: Option<String>,
    ) -> Result<(Vec<RuleSetData>, u64, i32, i32, bool), GrpcError> {
        let page = if page <= 0 { 1 } else { page as u32 };
        let page_size = if page_size <= 0 { 20 } else { page_size as u32 };

        let output = self
            .list_rule_sets_uc
            .execute(&ListRuleSetsInput {
                page,
                page_size,
                domain,
            })
            .await
            .map_err(|e| match e {
                ListRuleSetsError::Internal(msg) => GrpcError::Internal(msg),
            })?;

        Ok((
            output.rule_sets.into_iter().map(to_rule_set_data).collect(),
            output.total_count,
            output.page as i32,
            output.page_size as i32,
            output.has_next,
        ))
    }

    pub async fn create_rule_set(
        &self,
        name: String,
        description: String,
        domain: String,
        evaluation_mode: String,
        default_result_json: Vec<u8>,
        rule_ids: Vec<String>,
    ) -> Result<RuleSetData, GrpcError> {
        let default_result: serde_json::Value = if default_result_json.is_empty() {
            serde_json::json!({})
        } else {
            serde_json::from_slice(&default_result_json).map_err(|e| {
                GrpcError::InvalidArgument(format!("invalid default_result_json: {}", e))
            })?
        };

        let rule_uuids: Result<Vec<Uuid>, _> =
            rule_ids.iter().map(|id| Uuid::parse_str(id)).collect();
        let rule_uuids = rule_uuids
            .map_err(|_| GrpcError::InvalidArgument("invalid rule_ids format".to_string()))?;

        let rs = self
            .create_rule_set_uc
            .execute(&CreateRuleSetInput {
                name,
                description,
                domain,
                evaluation_mode,
                default_result,
                rule_ids: rule_uuids,
            })
            .await
            .map_err(|e| match e {
                CreateRuleSetError::AlreadyExists(name) => {
                    GrpcError::AlreadyExists(format!("rule set already exists: {}", name))
                }
                CreateRuleSetError::Validation(msg) => GrpcError::InvalidArgument(msg),
                CreateRuleSetError::Internal(msg) => GrpcError::Internal(msg),
            })?;

        Ok(to_rule_set_data(rs))
    }

    pub async fn update_rule_set(
        &self,
        id: String,
        description: Option<String>,
        evaluation_mode: Option<String>,
        default_result_json: Option<Vec<u8>>,
        rule_ids: Vec<String>,
        enabled: Option<bool>,
    ) -> Result<RuleSetData, GrpcError> {
        let uid = Uuid::parse_str(&id)
            .map_err(|_| GrpcError::InvalidArgument(format!("invalid rule set id: {}", id)))?;

        let default_result = match default_result_json {
            Some(bytes) if !bytes.is_empty() => {
                Some(serde_json::from_slice(&bytes).map_err(|e| {
                    GrpcError::InvalidArgument(format!("invalid default_result_json: {}", e))
                })?)
            }
            _ => None,
        };

        let rule_id_uuids =
            if rule_ids.is_empty() {
                None
            } else {
                let uuids: Result<Vec<Uuid>, _> =
                    rule_ids.iter().map(|id| Uuid::parse_str(id)).collect();
                Some(uuids.map_err(|_| {
                    GrpcError::InvalidArgument("invalid rule_ids format".to_string())
                })?)
            };

        let rs = self
            .update_rule_set_uc
            .execute(&UpdateRuleSetInput {
                id: uid,
                description,
                evaluation_mode,
                default_result,
                rule_ids: rule_id_uuids,
                enabled,
            })
            .await
            .map_err(|e| match e {
                UpdateRuleSetError::NotFound(id) => {
                    GrpcError::NotFound(format!("rule set not found: {}", id))
                }
                UpdateRuleSetError::Validation(msg) => GrpcError::InvalidArgument(msg),
                UpdateRuleSetError::Internal(msg) => GrpcError::Internal(msg),
            })?;

        Ok(to_rule_set_data(rs))
    }

    pub async fn delete_rule_set(&self, id: String) -> Result<(bool, String), GrpcError> {
        let uid = Uuid::parse_str(&id)
            .map_err(|_| GrpcError::InvalidArgument(format!("invalid rule set id: {}", id)))?;

        self.delete_rule_set_uc
            .execute(&uid)
            .await
            .map_err(|e| match e {
                DeleteRuleSetError::NotFound(id) => {
                    GrpcError::NotFound(format!("rule set not found: {}", id))
                }
                DeleteRuleSetError::Internal(msg) => GrpcError::Internal(msg),
            })?;

        Ok((true, format!("rule set {} deleted", id)))
    }

    pub async fn publish_rule_set(
        &self,
        id: String,
    ) -> Result<(String, u32, u32, DateTime<Utc>), GrpcError> {
        let uid = Uuid::parse_str(&id)
            .map_err(|_| GrpcError::InvalidArgument(format!("invalid rule set id: {}", id)))?;

        let output = self
            .publish_rule_set_uc
            .execute(&uid)
            .await
            .map_err(|e| match e {
                PublishRuleSetError::NotFound(id) => {
                    GrpcError::NotFound(format!("rule set not found: {}", id))
                }
                PublishRuleSetError::Internal(msg) => GrpcError::Internal(msg),
            })?;

        Ok((
            output.id.to_string(),
            output.published_version,
            output.previous_version,
            output.published_at,
        ))
    }

    pub async fn rollback_rule_set(
        &self,
        id: String,
    ) -> Result<(String, u32, u32, DateTime<Utc>), GrpcError> {
        let uid = Uuid::parse_str(&id)
            .map_err(|_| GrpcError::InvalidArgument(format!("invalid rule set id: {}", id)))?;

        let output = self
            .rollback_rule_set_uc
            .execute(&uid)
            .await
            .map_err(|e| match e {
                RollbackRuleSetError::NotFound(id) => {
                    GrpcError::NotFound(format!("rule set not found: {}", id))
                }
                RollbackRuleSetError::NoPreviousVersion(v) => GrpcError::FailedPrecondition(
                    format!("no previous version to rollback: current version is {}", v),
                ),
                RollbackRuleSetError::Internal(msg) => GrpcError::Internal(msg),
            })?;

        Ok((
            output.id.to_string(),
            output.rolled_back_to_version,
            output.previous_version,
            output.rolled_back_at,
        ))
    }

    pub async fn evaluate(
        &self,
        rule_set: String,
        input_json: Vec<u8>,
        context_json: Vec<u8>,
        dry_run: bool,
    ) -> Result<crate::usecase::evaluate::EvaluateOutput, GrpcError> {
        let input: serde_json::Value = if input_json.is_empty() {
            serde_json::json!({})
        } else {
            serde_json::from_slice(&input_json)
                .map_err(|e| GrpcError::InvalidArgument(format!("invalid input_json: {}", e)))?
        };
        let context: serde_json::Value = if context_json.is_empty() {
            serde_json::json!({})
        } else {
            serde_json::from_slice(&context_json)
                .map_err(|e| GrpcError::InvalidArgument(format!("invalid context_json: {}", e)))?
        };

        self.evaluate_uc
            .execute(&EvaluateInput {
                rule_set,
                input,
                context,
                dry_run,
            })
            .await
            .map_err(|e| match e {
                EvaluateError::RuleSetNotFound(name) => {
                    GrpcError::NotFound(format!("rule set not found: {}", name))
                }
                EvaluateError::EvaluationError(msg) => GrpcError::Internal(msg),
                EvaluateError::Internal(msg) => GrpcError::Internal(msg),
            })
    }
}

fn to_rule_data(rule: Rule) -> RuleData {
    RuleData {
        id: rule.id.to_string(),
        name: rule.name,
        description: rule.description,
        priority: rule.priority,
        when_json: serde_json::to_vec(&rule.when_condition).unwrap_or_default(),
        then_json: serde_json::to_vec(&rule.then_result).unwrap_or_default(),
        enabled: rule.enabled,
        version: rule.version,
        created_at: rule.created_at,
        updated_at: rule.updated_at,
    }
}

fn to_rule_set_data(rs: RuleSet) -> RuleSetData {
    RuleSetData {
        id: rs.id.to_string(),
        name: rs.name,
        description: rs.description,
        domain: rs.domain,
        evaluation_mode: rs.evaluation_mode.as_str().to_string(),
        default_result_json: serde_json::to_vec(&rs.default_result).unwrap_or_default(),
        rule_ids: rs.rule_ids.iter().map(|id| id.to_string()).collect(),
        current_version: rs.current_version,
        enabled: rs.enabled,
        created_at: rs.created_at,
        updated_at: rs.updated_at,
    }
}

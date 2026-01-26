//! ポリシー評価モジュール
//!
//! 権限制御のための汎用ポリシー評価エンジン。
//!
//! # 機能
//!
//! - ロールベースアクセス制御（RBAC）
//! - 属性ベースアクセス制御（ABAC）
//! - ポリシーリポジトリによる永続化
//!
//! # 使用例
//!
//! ```ignore
//! use k1s0_auth::policy::{PolicyRepository, PolicyEvaluator, InMemoryPolicyRepository};
//!
//! // インメモリリポジトリを使用
//! let repository = InMemoryPolicyRepository::new();
//!
//! // 外部ストレージを使用する場合は PolicyRepository trait を実装
//! let evaluator = PolicyEvaluator::with_repository(repository);
//! ```

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, warn};

use crate::error::AuthError;
use crate::jwt::Claims;

/// ポリシー決定
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyDecision {
    /// 許可
    Allow,
    /// 拒否
    Deny,
    /// 該当なし（他のポリシーに委譲）
    NotApplicable,
}

/// アクション
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Action {
    /// リソースタイプ（例: "user", "order"）
    pub resource: String,
    /// 操作（例: "read", "write", "delete"）
    pub operation: String,
}

impl Action {
    /// 新しいアクションを作成
    pub fn new(resource: impl Into<String>, operation: impl Into<String>) -> Self {
        Self {
            resource: resource.into(),
            operation: operation.into(),
        }
    }

    /// パーミッション文字列から作成（例: "user:read"）
    pub fn from_permission(permission: &str) -> Option<Self> {
        let parts: Vec<&str> = permission.split(':').collect();
        if parts.len() == 2 {
            Some(Self::new(parts[0], parts[1]))
        } else {
            None
        }
    }

    /// パーミッション文字列に変換
    pub fn to_permission(&self) -> String {
        format!("{}:{}", self.resource, self.operation)
    }
}

/// リソースコンテキスト
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceContext {
    /// リソースID
    pub id: Option<String>,
    /// オーナーID
    pub owner_id: Option<String>,
    /// テナントID
    pub tenant_id: Option<String>,
    /// 追加属性
    pub attributes: HashMap<String, String>,
}

impl ResourceContext {
    /// 新しいコンテキストを作成
    pub fn new() -> Self {
        Self::default()
    }

    /// リソースIDを設定
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// オーナーIDを設定
    pub fn with_owner_id(mut self, owner_id: impl Into<String>) -> Self {
        self.owner_id = Some(owner_id.into());
        self
    }

    /// テナントIDを設定
    pub fn with_tenant_id(mut self, tenant_id: impl Into<String>) -> Self {
        self.tenant_id = Some(tenant_id.into());
        self
    }

    /// 属性を追加
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }
}

/// ポリシー評価リクエスト
#[derive(Debug, Clone)]
pub struct PolicyRequest {
    /// 主体（ユーザー）
    pub subject: PolicySubject,
    /// アクション
    pub action: Action,
    /// リソースコンテキスト
    pub resource: ResourceContext,
}

/// 主体（ユーザー）情報
#[derive(Debug, Clone)]
pub struct PolicySubject {
    /// ユーザーID
    pub user_id: String,
    /// ロール
    pub roles: HashSet<String>,
    /// パーミッション
    pub permissions: HashSet<String>,
    /// テナントID
    pub tenant_id: Option<String>,
    /// 追加属性
    pub attributes: HashMap<String, String>,
}

impl PolicySubject {
    /// JWTクレームから作成
    pub fn from_claims(claims: &Claims) -> Self {
        Self {
            user_id: claims.sub.clone(),
            roles: claims.roles.iter().cloned().collect(),
            permissions: claims.permissions.iter().cloned().collect(),
            tenant_id: claims.tenant_id.clone(),
            attributes: HashMap::new(),
        }
    }

    /// 新しい主体を作成
    pub fn new(user_id: impl Into<String>) -> Self {
        Self {
            user_id: user_id.into(),
            roles: HashSet::new(),
            permissions: HashSet::new(),
            tenant_id: None,
            attributes: HashMap::new(),
        }
    }

    /// ロールを追加
    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        self.roles.insert(role.into());
        self
    }

    /// パーミッションを追加
    pub fn with_permission(mut self, permission: impl Into<String>) -> Self {
        self.permissions.insert(permission.into());
        self
    }

    /// テナントIDを設定
    pub fn with_tenant_id(mut self, tenant_id: impl Into<String>) -> Self {
        self.tenant_id = Some(tenant_id.into());
        self
    }
}

/// ポリシー評価結果
#[derive(Debug, Clone)]
pub struct PolicyResult {
    /// 決定
    pub decision: PolicyDecision,
    /// 適用されたポリシー名
    pub matched_policy: Option<String>,
    /// 理由
    pub reason: Option<String>,
}

impl PolicyResult {
    /// 許可結果を作成
    pub fn allow(policy: impl Into<String>) -> Self {
        Self {
            decision: PolicyDecision::Allow,
            matched_policy: Some(policy.into()),
            reason: None,
        }
    }

    /// 拒否結果を作成
    pub fn deny(policy: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            decision: PolicyDecision::Deny,
            matched_policy: Some(policy.into()),
            reason: Some(reason.into()),
        }
    }

    /// 該当なし結果を作成
    pub fn not_applicable() -> Self {
        Self {
            decision: PolicyDecision::NotApplicable,
            matched_policy: None,
            reason: None,
        }
    }

    /// 許可されているかどうか
    pub fn is_allowed(&self) -> bool {
        matches!(self.decision, PolicyDecision::Allow)
    }
}

/// ポリシールール
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    /// ルール名
    pub name: String,
    /// 対象リソース（ワイルドカード可: "*"）
    pub resources: Vec<String>,
    /// 対象操作（ワイルドカード可: "*"）
    pub operations: Vec<String>,
    /// 許可するロール
    pub allowed_roles: Vec<String>,
    /// 許可するパーミッション
    pub allowed_permissions: Vec<String>,
    /// 条件（オプション）
    pub conditions: Vec<PolicyCondition>,
    /// 効果
    pub effect: PolicyEffect,
    /// 優先度（小さいほど高優先度）
    pub priority: i32,
}

/// ポリシー効果
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyEffect {
    /// 許可
    Allow,
    /// 拒否
    Deny,
}

/// ポリシー条件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PolicyCondition {
    /// オーナーのみ
    #[serde(rename = "owner_only")]
    OwnerOnly,
    /// 同一テナントのみ
    #[serde(rename = "same_tenant")]
    SameTenant,
    /// 属性一致
    #[serde(rename = "attribute_match")]
    AttributeMatch {
        subject_key: String,
        resource_key: String,
    },
}

impl PolicyRule {
    /// ルールがアクションに適用されるかチェック
    pub fn matches_action(&self, action: &Action) -> bool {
        let resource_match = self.resources.iter().any(|r| {
            r == "*" || r == &action.resource
        });
        let operation_match = self.operations.iter().any(|o| {
            o == "*" || o == &action.operation
        });
        resource_match && operation_match
    }

    /// 主体がルールに合致するかチェック
    pub fn matches_subject(&self, subject: &PolicySubject) -> bool {
        // ロールチェック
        let role_match = self.allowed_roles.is_empty()
            || self
                .allowed_roles
                .iter()
                .any(|r| r == "*" || subject.roles.contains(r));

        // パーミッションチェック
        let permission_match = self.allowed_permissions.is_empty()
            || self
                .allowed_permissions
                .iter()
                .any(|p| p == "*" || subject.permissions.contains(p));

        role_match || permission_match
    }

    /// 条件を評価
    pub fn evaluate_conditions(
        &self,
        subject: &PolicySubject,
        resource: &ResourceContext,
    ) -> bool {
        self.conditions.iter().all(|condition| match condition {
            PolicyCondition::OwnerOnly => {
                resource.owner_id.as_ref() == Some(&subject.user_id)
            }
            PolicyCondition::SameTenant => {
                match (&subject.tenant_id, &resource.tenant_id) {
                    (Some(s), Some(r)) => s == r,
                    _ => false,
                }
            }
            PolicyCondition::AttributeMatch {
                subject_key,
                resource_key,
            } => {
                match (
                    subject.attributes.get(subject_key),
                    resource.attributes.get(resource_key),
                ) {
                    (Some(s), Some(r)) => s == r,
                    _ => false,
                }
            }
        })
    }
}

/// ポリシーエバリュエーター
pub struct PolicyEvaluator {
    /// ポリシールール
    rules: Arc<RwLock<Vec<PolicyRule>>>,
    /// デフォルト決定（ルールが該当しない場合）
    default_decision: PolicyDecision,
}

impl PolicyEvaluator {
    /// 新しいエバリュエーターを作成
    pub fn new() -> Self {
        Self {
            rules: Arc::new(RwLock::new(Vec::new())),
            default_decision: PolicyDecision::Deny, // 安全側デフォルト
        }
    }

    /// デフォルト決定を設定
    pub fn with_default_decision(mut self, decision: PolicyDecision) -> Self {
        self.default_decision = decision;
        self
    }

    /// ルールを追加
    pub async fn add_rule(&self, rule: PolicyRule) {
        let mut rules = self.rules.write().await;
        rules.push(rule);
        // 優先度でソート
        rules.sort_by_key(|r| r.priority);
    }

    /// 複数のルールを追加
    pub async fn add_rules(&self, new_rules: Vec<PolicyRule>) {
        let mut rules = self.rules.write().await;
        rules.extend(new_rules);
        rules.sort_by_key(|r| r.priority);
    }

    /// ルールをクリア
    pub async fn clear_rules(&self) {
        let mut rules = self.rules.write().await;
        rules.clear();
    }

    /// ポリシーを評価
    pub async fn evaluate(&self, request: &PolicyRequest) -> PolicyResult {
        let rules = self.rules.read().await;

        for rule in rules.iter() {
            // アクションにマッチするかチェック
            if !rule.matches_action(&request.action) {
                continue;
            }

            // 主体にマッチするかチェック
            if !rule.matches_subject(&request.subject) {
                continue;
            }

            // 条件を評価
            if !rule.evaluate_conditions(&request.subject, &request.resource) {
                continue;
            }

            // ルールにマッチした
            debug!(
                rule = %rule.name,
                action = %request.action.to_permission(),
                user = %request.subject.user_id,
                "Policy rule matched"
            );

            return match rule.effect {
                PolicyEffect::Allow => PolicyResult::allow(&rule.name),
                PolicyEffect::Deny => PolicyResult::deny(&rule.name, "Explicitly denied by policy"),
            };
        }

        // どのルールにもマッチしなかった
        debug!(
            action = %request.action.to_permission(),
            user = %request.subject.user_id,
            default = ?self.default_decision,
            "No policy matched, using default"
        );

        match self.default_decision {
            PolicyDecision::Allow => PolicyResult {
                decision: PolicyDecision::Allow,
                matched_policy: None,
                reason: Some("Default allow".to_string()),
            },
            PolicyDecision::Deny => PolicyResult {
                decision: PolicyDecision::Deny,
                matched_policy: None,
                reason: Some("No matching policy, default deny".to_string()),
            },
            PolicyDecision::NotApplicable => PolicyResult::not_applicable(),
        }
    }

    /// 簡易パーミッションチェック
    pub async fn check_permission(
        &self,
        subject: &PolicySubject,
        permission: &str,
    ) -> Result<bool, AuthError> {
        // 直接パーミッションを持っているかチェック
        if subject.permissions.contains(permission) {
            return Ok(true);
        }

        // ワイルドカードパーミッションをチェック
        if subject.permissions.contains("*") {
            return Ok(true);
        }

        // アクションとしてポリシー評価
        if let Some(action) = Action::from_permission(permission) {
            let request = PolicyRequest {
                subject: subject.clone(),
                action,
                resource: ResourceContext::default(),
            };
            let result = self.evaluate(&request).await;
            return Ok(result.is_allowed());
        }

        Ok(false)
    }
}

impl Default for PolicyEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

/// 標準ポリシールールのビルダー
pub struct PolicyBuilder {
    rules: Vec<PolicyRule>,
}

impl PolicyBuilder {
    /// 新しいビルダーを作成
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// 管理者用ルールを追加（すべて許可）
    pub fn admin_rule(mut self, admin_role: &str) -> Self {
        self.rules.push(PolicyRule {
            name: "admin_full_access".to_string(),
            resources: vec!["*".to_string()],
            operations: vec!["*".to_string()],
            allowed_roles: vec![admin_role.to_string()],
            allowed_permissions: vec![],
            conditions: vec![],
            effect: PolicyEffect::Allow,
            priority: 0, // 最高優先度
        });
        self
    }

    /// リソース読み取りルールを追加
    pub fn read_rule(
        mut self,
        name: &str,
        resource: &str,
        roles: Vec<&str>,
        priority: i32,
    ) -> Self {
        self.rules.push(PolicyRule {
            name: name.to_string(),
            resources: vec![resource.to_string()],
            operations: vec!["read".to_string(), "list".to_string()],
            allowed_roles: roles.into_iter().map(String::from).collect(),
            allowed_permissions: vec![],
            conditions: vec![],
            effect: PolicyEffect::Allow,
            priority,
        });
        self
    }

    /// オーナーのみ許可ルールを追加
    pub fn owner_only_rule(
        mut self,
        name: &str,
        resource: &str,
        operations: Vec<&str>,
        priority: i32,
    ) -> Self {
        self.rules.push(PolicyRule {
            name: name.to_string(),
            resources: vec![resource.to_string()],
            operations: operations.into_iter().map(String::from).collect(),
            allowed_roles: vec![],
            allowed_permissions: vec![],
            conditions: vec![PolicyCondition::OwnerOnly],
            effect: PolicyEffect::Allow,
            priority,
        });
        self
    }

    /// 同一テナントルールを追加
    pub fn same_tenant_rule(
        mut self,
        name: &str,
        resources: Vec<&str>,
        operations: Vec<&str>,
        priority: i32,
    ) -> Self {
        self.rules.push(PolicyRule {
            name: name.to_string(),
            resources: resources.into_iter().map(String::from).collect(),
            operations: operations.into_iter().map(String::from).collect(),
            allowed_roles: vec![],
            allowed_permissions: vec![],
            conditions: vec![PolicyCondition::SameTenant],
            effect: PolicyEffect::Allow,
            priority,
        });
        self
    }

    /// ルールをビルド
    pub fn build(self) -> Vec<PolicyRule> {
        self.rules
    }
}

impl Default for PolicyBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// ポリシーリポジトリ
// ============================================================================

/// ポリシーリポジトリ
///
/// ポリシールールの永続化と取得を抽象化する。
#[async_trait]
pub trait PolicyRepository: Send + Sync {
    /// すべてのルールを取得
    async fn get_all_rules(&self) -> Result<Vec<PolicyRule>, AuthError>;

    /// ルールを追加
    async fn add_rule(&self, rule: PolicyRule) -> Result<(), AuthError>;

    /// 複数のルールを追加
    async fn add_rules(&self, rules: Vec<PolicyRule>) -> Result<(), AuthError>;

    /// ルールを削除
    async fn remove_rule(&self, name: &str) -> Result<bool, AuthError>;

    /// すべてのルールをクリア
    async fn clear_rules(&self) -> Result<(), AuthError>;

    /// ルール数を取得
    async fn count_rules(&self) -> Result<usize, AuthError>;

    /// 名前でルールを取得
    async fn get_rule(&self, name: &str) -> Result<Option<PolicyRule>, AuthError>;

    /// ルールを更新
    async fn update_rule(&self, rule: PolicyRule) -> Result<bool, AuthError>;
}

/// インメモリポリシーリポジトリ
///
/// ポリシールールをメモリ上で管理する。
/// 開発・テスト用、または設定ファイルからの読み込み用。
#[derive(Default)]
pub struct InMemoryPolicyRepository {
    rules: RwLock<Vec<PolicyRule>>,
}

impl InMemoryPolicyRepository {
    /// 新しいリポジトリを作成
    pub fn new() -> Self {
        Self::default()
    }

    /// 初期ルール付きで作成
    pub fn with_rules(rules: Vec<PolicyRule>) -> Self {
        Self {
            rules: RwLock::new(rules),
        }
    }
}

#[async_trait]
impl PolicyRepository for InMemoryPolicyRepository {
    async fn get_all_rules(&self) -> Result<Vec<PolicyRule>, AuthError> {
        let rules = self.rules.read().await;
        Ok(rules.clone())
    }

    async fn add_rule(&self, rule: PolicyRule) -> Result<(), AuthError> {
        let mut rules = self.rules.write().await;
        rules.push(rule);
        rules.sort_by_key(|r| r.priority);
        Ok(())
    }

    async fn add_rules(&self, new_rules: Vec<PolicyRule>) -> Result<(), AuthError> {
        let mut rules = self.rules.write().await;
        rules.extend(new_rules);
        rules.sort_by_key(|r| r.priority);
        Ok(())
    }

    async fn remove_rule(&self, name: &str) -> Result<bool, AuthError> {
        let mut rules = self.rules.write().await;
        let len_before = rules.len();
        rules.retain(|r| r.name != name);
        Ok(rules.len() < len_before)
    }

    async fn clear_rules(&self) -> Result<(), AuthError> {
        let mut rules = self.rules.write().await;
        rules.clear();
        Ok(())
    }

    async fn count_rules(&self) -> Result<usize, AuthError> {
        let rules = self.rules.read().await;
        Ok(rules.len())
    }

    async fn get_rule(&self, name: &str) -> Result<Option<PolicyRule>, AuthError> {
        let rules = self.rules.read().await;
        Ok(rules.iter().find(|r| r.name == name).cloned())
    }

    async fn update_rule(&self, rule: PolicyRule) -> Result<bool, AuthError> {
        let mut rules = self.rules.write().await;
        if let Some(existing) = rules.iter_mut().find(|r| r.name == rule.name) {
            *existing = rule;
            rules.sort_by_key(|r| r.priority);
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

/// キャッシュ付きポリシーリポジトリ
///
/// 外部ストレージからのルール取得をキャッシュする。
pub struct CachedPolicyRepository<R: PolicyRepository> {
    inner: R,
    cache: RwLock<Option<CacheEntry>>,
    ttl_secs: u64,
}

struct CacheEntry {
    rules: Vec<PolicyRule>,
    created_at: std::time::Instant,
}

impl<R: PolicyRepository> CachedPolicyRepository<R> {
    /// 新しいキャッシュ付きリポジトリを作成
    pub fn new(inner: R, ttl_secs: u64) -> Self {
        Self {
            inner,
            cache: RwLock::new(None),
            ttl_secs,
        }
    }

    /// キャッシュを無効化
    pub async fn invalidate_cache(&self) {
        let mut cache = self.cache.write().await;
        *cache = None;
    }

    /// キャッシュが有効かどうか
    async fn is_cache_valid(&self) -> bool {
        let cache = self.cache.read().await;
        if let Some(ref entry) = *cache {
            entry.created_at.elapsed().as_secs() < self.ttl_secs
        } else {
            false
        }
    }
}

#[async_trait]
impl<R: PolicyRepository> PolicyRepository for CachedPolicyRepository<R> {
    async fn get_all_rules(&self) -> Result<Vec<PolicyRule>, AuthError> {
        // キャッシュが有効ならそれを返す
        if self.is_cache_valid().await {
            let cache = self.cache.read().await;
            if let Some(ref entry) = *cache {
                return Ok(entry.rules.clone());
            }
        }

        // キャッシュが無効なら更新
        let rules = self.inner.get_all_rules().await?;
        let mut cache = self.cache.write().await;
        *cache = Some(CacheEntry {
            rules: rules.clone(),
            created_at: std::time::Instant::now(),
        });
        Ok(rules)
    }

    async fn add_rule(&self, rule: PolicyRule) -> Result<(), AuthError> {
        self.inner.add_rule(rule).await?;
        self.invalidate_cache().await;
        Ok(())
    }

    async fn add_rules(&self, rules: Vec<PolicyRule>) -> Result<(), AuthError> {
        self.inner.add_rules(rules).await?;
        self.invalidate_cache().await;
        Ok(())
    }

    async fn remove_rule(&self, name: &str) -> Result<bool, AuthError> {
        let result = self.inner.remove_rule(name).await?;
        self.invalidate_cache().await;
        Ok(result)
    }

    async fn clear_rules(&self) -> Result<(), AuthError> {
        self.inner.clear_rules().await?;
        self.invalidate_cache().await;
        Ok(())
    }

    async fn count_rules(&self) -> Result<usize, AuthError> {
        // キャッシュが有効ならそれを使う
        if self.is_cache_valid().await {
            let cache = self.cache.read().await;
            if let Some(ref entry) = *cache {
                return Ok(entry.rules.len());
            }
        }
        self.inner.count_rules().await
    }

    async fn get_rule(&self, name: &str) -> Result<Option<PolicyRule>, AuthError> {
        // キャッシュが有効ならそれを使う
        if self.is_cache_valid().await {
            let cache = self.cache.read().await;
            if let Some(ref entry) = *cache {
                return Ok(entry.rules.iter().find(|r| r.name == name).cloned());
            }
        }
        self.inner.get_rule(name).await
    }

    async fn update_rule(&self, rule: PolicyRule) -> Result<bool, AuthError> {
        let result = self.inner.update_rule(rule).await?;
        self.invalidate_cache().await;
        Ok(result)
    }
}

/// リポジトリ付きポリシーエバリュエーター
///
/// PolicyRepository を使用してルールを管理する。
pub struct RepositoryPolicyEvaluator<R: PolicyRepository> {
    repository: Arc<R>,
    default_decision: PolicyDecision,
}

impl<R: PolicyRepository> RepositoryPolicyEvaluator<R> {
    /// リポジトリからエバリュエーターを作成
    pub fn with_repository(repository: R) -> Self {
        Self {
            repository: Arc::new(repository),
            default_decision: PolicyDecision::Deny,
        }
    }

    /// デフォルト決定を設定
    pub fn with_default_decision(mut self, decision: PolicyDecision) -> Self {
        self.default_decision = decision;
        self
    }

    /// リポジトリへの参照を取得
    pub fn repository(&self) -> &R {
        &self.repository
    }

    /// ポリシーを評価
    pub async fn evaluate(&self, request: &PolicyRequest) -> PolicyResult {
        let rules = match self.repository.get_all_rules().await {
            Ok(rules) => rules,
            Err(e) => {
                warn!(error = %e, "Failed to fetch policy rules");
                return PolicyResult {
                    decision: self.default_decision,
                    matched_policy: None,
                    reason: Some(format!("Failed to fetch rules: {}", e)),
                };
            }
        };

        for rule in rules.iter() {
            if !rule.matches_action(&request.action) {
                continue;
            }

            if !rule.matches_subject(&request.subject) {
                continue;
            }

            if !rule.evaluate_conditions(&request.subject, &request.resource) {
                continue;
            }

            debug!(
                rule = %rule.name,
                action = %request.action.to_permission(),
                user = %request.subject.user_id,
                "Policy rule matched"
            );

            return match rule.effect {
                PolicyEffect::Allow => PolicyResult::allow(&rule.name),
                PolicyEffect::Deny => PolicyResult::deny(&rule.name, "Explicitly denied by policy"),
            };
        }

        debug!(
            action = %request.action.to_permission(),
            user = %request.subject.user_id,
            default = ?self.default_decision,
            "No policy matched, using default"
        );

        match self.default_decision {
            PolicyDecision::Allow => PolicyResult {
                decision: PolicyDecision::Allow,
                matched_policy: None,
                reason: Some("Default allow".to_string()),
            },
            PolicyDecision::Deny => PolicyResult {
                decision: PolicyDecision::Deny,
                matched_policy: None,
                reason: Some("No matching policy, default deny".to_string()),
            },
            PolicyDecision::NotApplicable => PolicyResult::not_applicable(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_from_permission() {
        let action = Action::from_permission("user:read").unwrap();
        assert_eq!(action.resource, "user");
        assert_eq!(action.operation, "read");
        assert_eq!(action.to_permission(), "user:read");
    }

    #[test]
    fn test_policy_subject_from_claims() {
        let claims = Claims {
            sub: "user123".to_string(),
            iss: "test".to_string(),
            aud: None,
            exp: 0,
            iat: 0,
            nbf: None,
            jti: None,
            roles: vec!["admin".to_string(), "user".to_string()],
            permissions: vec!["user:read".to_string()],
            tenant_id: Some("tenant1".to_string()),
            email: None,
            email_verified: None,
            name: None,
        };

        let subject = PolicySubject::from_claims(&claims);
        assert_eq!(subject.user_id, "user123");
        assert!(subject.roles.contains("admin"));
        assert!(subject.permissions.contains("user:read"));
        assert_eq!(subject.tenant_id, Some("tenant1".to_string()));
    }

    #[tokio::test]
    async fn test_policy_evaluator_admin() {
        let evaluator = PolicyEvaluator::new();
        let rules = PolicyBuilder::new()
            .admin_rule("admin")
            .build();
        evaluator.add_rules(rules).await;

        let subject = PolicySubject::new("admin-user")
            .with_role("admin");
        let action = Action::new("order", "delete");
        let request = PolicyRequest {
            subject,
            action,
            resource: ResourceContext::new(),
        };

        let result = evaluator.evaluate(&request).await;
        assert!(result.is_allowed());
        assert_eq!(result.matched_policy, Some("admin_full_access".to_string()));
    }

    #[tokio::test]
    async fn test_policy_evaluator_owner_only() {
        let evaluator = PolicyEvaluator::new();
        let rules = PolicyBuilder::new()
            .owner_only_rule("owner_edit", "profile", vec!["update", "delete"], 10)
            .build();
        evaluator.add_rules(rules).await;

        // オーナーの場合
        let subject = PolicySubject::new("user123");
        let action = Action::new("profile", "update");
        let resource = ResourceContext::new().with_owner_id("user123");
        let request = PolicyRequest {
            subject,
            action,
            resource,
        };

        let result = evaluator.evaluate(&request).await;
        assert!(result.is_allowed());

        // オーナーでない場合
        let subject = PolicySubject::new("user456");
        let action = Action::new("profile", "update");
        let resource = ResourceContext::new().with_owner_id("user123");
        let request = PolicyRequest {
            subject,
            action,
            resource,
        };

        let result = evaluator.evaluate(&request).await;
        assert!(!result.is_allowed());
    }

    #[tokio::test]
    async fn test_policy_evaluator_same_tenant() {
        let evaluator = PolicyEvaluator::new();
        let rules = PolicyBuilder::new()
            .same_tenant_rule("tenant_access", vec!["order"], vec!["read", "list"], 10)
            .build();
        evaluator.add_rules(rules).await;

        // 同一テナントの場合
        let subject = PolicySubject::new("user1")
            .with_tenant_id("tenant-a");
        let action = Action::new("order", "read");
        let resource = ResourceContext::new().with_tenant_id("tenant-a");
        let request = PolicyRequest {
            subject,
            action,
            resource,
        };

        let result = evaluator.evaluate(&request).await;
        assert!(result.is_allowed());

        // 異なるテナントの場合
        let subject = PolicySubject::new("user1")
            .with_tenant_id("tenant-b");
        let action = Action::new("order", "read");
        let resource = ResourceContext::new().with_tenant_id("tenant-a");
        let request = PolicyRequest {
            subject,
            action,
            resource,
        };

        let result = evaluator.evaluate(&request).await;
        assert!(!result.is_allowed());
    }

    #[tokio::test]
    async fn test_check_permission_direct() {
        let evaluator = PolicyEvaluator::new();

        let subject = PolicySubject::new("user1")
            .with_permission("user:read")
            .with_permission("order:list");

        assert!(evaluator.check_permission(&subject, "user:read").await.unwrap());
        assert!(evaluator.check_permission(&subject, "order:list").await.unwrap());
        assert!(!evaluator.check_permission(&subject, "user:write").await.unwrap());
    }
}

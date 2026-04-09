use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct EvaluationContext {
    pub user_id: Option<String>,
    pub tenant_id: Option<String>,
    pub attributes: HashMap<String, String>,
}

impl EvaluationContext {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// ユーザー ID を設定する（ビルダーパターン）。
    #[must_use]
    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// テナント ID を設定する（ビルダーパターン）。
    #[must_use]
    pub fn with_tenant_id(mut self, tenant_id: impl Into<String>) -> Self {
        self.tenant_id = Some(tenant_id.into());
        self
    }

    /// 任意の属性を設定する（ビルダーパターン）。
    #[must_use]
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }
}

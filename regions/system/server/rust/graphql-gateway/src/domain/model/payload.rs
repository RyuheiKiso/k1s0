use async_graphql::SimpleObject;

use super::{FeatureFlag, Tenant};

/// Mutation 戻り値の Payload パターン: tenant + errors
#[derive(Debug, Clone, SimpleObject)]
pub struct CreateTenantPayload {
    pub tenant: Option<Tenant>,
    pub errors: Vec<UserError>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct UpdateTenantPayload {
    pub tenant: Option<Tenant>,
    pub errors: Vec<UserError>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct SetFeatureFlagPayload {
    pub feature_flag: Option<FeatureFlag>,
    pub errors: Vec<UserError>,
}

/// GraphQL UserError: フィールドレベルエラーの構造化表現
#[derive(Debug, Clone, SimpleObject)]
pub struct UserError {
    pub field: Option<Vec<String>>,
    pub message: String,
}

use async_graphql::dataloader::Loader;
use std::collections::HashMap;
use std::sync::Arc;

use crate::domain::model::graphql_context::{FeatureFlagLoader, TenantLoader};
use crate::domain::model::{FeatureFlag, Tenant};

/// TenantLoader は ID リストを受け取り、TenantService を呼び出してバッチ取得する。
impl Loader<String> for TenantLoader {
    type Value = Tenant;
    type Error = Arc<anyhow::Error>;

    async fn load(
        &self,
        keys: &[String],
    ) -> Result<HashMap<String, Self::Value>, Self::Error> {
        // TenantService.ListTenants を呼び出して ID フィルタリング
        let tenants = self
            .client
            .list_tenants_by_ids(keys)
            .await
            .map_err(Arc::new)?;
        Ok(tenants.into_iter().map(|t| (t.id.clone(), t)).collect())
    }
}

/// FeatureFlagLoader はフラグキーリストを受け取り、FeatureFlagService を呼び出してバッチ取得する。
impl Loader<String> for FeatureFlagLoader {
    type Value = FeatureFlag;
    type Error = Arc<anyhow::Error>;

    async fn load(
        &self,
        keys: &[String],
    ) -> Result<HashMap<String, Self::Value>, Self::Error> {
        let flags = self
            .client
            .list_flags_by_keys(keys)
            .await
            .map_err(Arc::new)?;
        Ok(flags.into_iter().map(|f| (f.key.clone(), f)).collect())
    }
}

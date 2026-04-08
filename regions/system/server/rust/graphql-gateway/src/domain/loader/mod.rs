use async_graphql::dataloader::Loader;
use std::collections::HashMap;
use std::sync::Arc;

use crate::domain::model::graphql_context::{ConfigLoader, FeatureFlagLoader, TenantLoader};
use crate::domain::model::{ConfigEntry, FeatureFlag, Tenant};

/// `TenantLoader` は ID リストを受け取り、TenantService を呼び出してバッチ取得する。
impl Loader<String> for TenantLoader {
    type Value = Tenant;
    type Error = Arc<anyhow::Error>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        // TenantService.ListTenants を呼び出して ID フィルタリング
        let tenants = self
            .client
            .list_tenants_by_ids(keys)
            .await
            .map_err(Arc::new)?;
        Ok(tenants.into_iter().map(|t| (t.id.clone(), t)).collect())
    }
}

/// `FeatureFlagLoader` はフラグキーリストを受け取り、FeatureFlagService を呼び出してバッチ取得する。
impl Loader<String> for FeatureFlagLoader {
    type Value = FeatureFlag;
    type Error = Arc<anyhow::Error>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        let flags = self
            .client
            .list_flags_by_keys(keys)
            .await
            .map_err(Arc::new)?;
        // CRIT-007 対応: FeatureFlag のキーフィールドが flag_key に変更されたため更新する
        Ok(flags.into_iter().map(|f| (f.flag_key.clone(), f)).collect())
    }
}

/// `ConfigLoader` は config キー（"namespace/key" 形式）リストを受け取り、
/// `ConfigService` の `ListConfigs` をバッチ呼び出しして一括取得する。
/// namespace ごとにグルーピングし、N+1 問題を回避する。
impl Loader<String> for ConfigLoader {
    type Value = ConfigEntry;
    type Error = Arc<anyhow::Error>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        let entries = self
            .client
            .list_configs_by_keys(keys)
            .await
            .map_err(Arc::new)?;
        Ok(entries.into_iter().map(|e| (e.key.clone(), e)).collect())
    }
}

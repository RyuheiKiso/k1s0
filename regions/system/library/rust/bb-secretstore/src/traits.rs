use std::collections::HashMap;

use async_trait::async_trait;

use crate::SecretStoreError;

/// `SecretValue` はシークレットストアのエントリを表す。
#[derive(Debug, Clone)]
pub struct SecretValue {
    pub key: String,
    pub value: String,
    pub metadata: HashMap<String, String>,
}

/// `SecretStore` はシークレット管理の抽象インターフェース。
/// Component トレイトを拡張する。
#[async_trait]
pub trait SecretStore: k1s0_bb_core::Component {
    /// キーに対応するシークレットを取得する。
    async fn get_secret(&self, key: &str) -> Result<SecretValue, SecretStoreError>;

    /// 複数キーのシークレットを一括取得する。
    async fn bulk_get(
        &self,
        keys: &[&str],
    ) -> Result<HashMap<String, SecretValue>, SecretStoreError>;
}

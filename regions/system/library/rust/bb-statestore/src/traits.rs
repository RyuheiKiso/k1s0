use async_trait::async_trait;

use crate::StateStoreError;

/// `StateEntry` はステートストアのエントリを表す。
#[derive(Debug, Clone)]
pub struct StateEntry {
    pub key: String,
    pub value: Vec<u8>,
    pub etag: String,
}

/// `StateStore` はキーバリューストアの抽象インターフェース。
/// Component トレイトを拡張する。
#[async_trait]
pub trait StateStore: k1s0_bb_core::Component {
    /// キーに対応する値を取得する。
    async fn get(&self, key: &str) -> Result<Option<StateEntry>, StateStoreError>;

    /// キーに値を設定し、新しい `ETag` を返す。
    /// etag が指定された場合、楽観的ロックを行う。
    async fn set(
        &self,
        key: &str,
        value: &[u8],
        etag: Option<&str>,
    ) -> Result<String, StateStoreError>;

    /// キーを削除する。
    /// etag が指定された場合、楽観的ロックを行う。
    async fn delete(&self, key: &str, etag: Option<&str>) -> Result<(), StateStoreError>;

    /// 複数キーを一括取得する。
    async fn bulk_get(&self, keys: &[&str]) -> Result<Vec<StateEntry>, StateStoreError>;

    /// 複数キーを一括設定し、新しい `ETag` のリストを返す。
    async fn bulk_set(&self, entries: &[(&str, &[u8])]) -> Result<Vec<String>, StateStoreError>;
}

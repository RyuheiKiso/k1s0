// ドメイン層がインフラストラクチャ層に直接依存しないよう、
// 各 gRPC クライアントが実装すべきポートトレイトを定義する。
// クリーンアーキテクチャにおける依存性逆転の原則（DIP）を適用し、
// domain 層は具象型ではなく抽象（trait）に依存する。

use crate::domain::model::{ConfigEntry, FeatureFlag, Tenant};

/// テナントサービスへのアクセスを抽象化するポートトレイト。
/// DataLoader から呼び出されるバッチ取得メソッドを定義する。
#[async_trait::async_trait]
pub trait TenantPort: Send + Sync {
    /// 複数のテナント ID をまとめて取得する（DataLoader バッチ用）。
    async fn list_tenants_by_ids(&self, ids: &[String]) -> anyhow::Result<Vec<Tenant>>;
}

/// フィーチャーフラグサービスへのアクセスを抽象化するポートトレイト。
/// DataLoader から呼び出されるバッチ取得メソッドを定義する。
#[async_trait::async_trait]
pub trait FeatureFlagPort: Send + Sync {
    /// 複数のフラグキーをまとめて取得する（DataLoader バッチ用）。
    async fn list_flags_by_keys(&self, keys: &[String]) -> anyhow::Result<Vec<FeatureFlag>>;
}

/// コンフィグサービスへのアクセスを抽象化するポートトレイト。
/// DataLoader から呼び出されるバッチ取得メソッドを定義する。
#[async_trait::async_trait]
pub trait ConfigPort: Send + Sync {
    /// 複数の "namespace/key" 形式キーをまとめて取得する（DataLoader バッチ用）。
    async fn list_configs_by_keys(&self, keys: &[String]) -> anyhow::Result<Vec<ConfigEntry>>;
}

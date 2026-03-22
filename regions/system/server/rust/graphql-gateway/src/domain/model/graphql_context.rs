// GraphQL リクエストコンテキストの定義。
// インフラストラクチャ層への直接依存を排除し、ドメイン層のポートトレイトに依存する。
// クリーンアーキテクチャの依存性逆転の原則（DIP）に従った設計。
use std::sync::Arc;

use async_graphql::dataloader::DataLoader;

use crate::domain::port::{ConfigPort, FeatureFlagPort, TenantPort};

/// GraphQL リクエストコンテキスト。JWT から抽出した認証情報と DataLoader を保持する。
#[allow(dead_code)]
pub struct GraphqlContext {
    /// JWT sub クレームから取得したユーザー ID
    pub user_id: String,
    /// JWT realm_access.roles から取得したロールリスト
    pub roles: Vec<String>,
    /// リクエスト追跡 ID（X-Request-Id ヘッダーまたは UUID 自動生成）
    pub request_id: String,
    /// テナントバッチローダー
    pub tenant_loader: Arc<DataLoader<TenantLoader>>,
    /// フィーチャーフラグバッチローダー
    pub flag_loader: Arc<DataLoader<FeatureFlagLoader>>,
    /// Config バッチローダー
    pub config_loader: Arc<DataLoader<ConfigLoader>>,
}

/// テナントバッチロード用の DataLoader 実装体。
/// client フィールドは TenantPort トレイトオブジェクトを保持し、具象型に依存しない。
pub struct TenantLoader {
    pub client: Arc<dyn TenantPort>,
}

/// フィーチャーフラグバッチロード用の DataLoader 実装体。
/// client フィールドは FeatureFlagPort トレイトオブジェクトを保持し、具象型に依存しない。
pub struct FeatureFlagLoader {
    pub client: Arc<dyn FeatureFlagPort>,
}

/// コンフィグバッチロード用の DataLoader 実装体。
/// client フィールドは ConfigPort トレイトオブジェクトを保持し、具象型に依存しない。
pub struct ConfigLoader {
    pub client: Arc<dyn ConfigPort>,
}

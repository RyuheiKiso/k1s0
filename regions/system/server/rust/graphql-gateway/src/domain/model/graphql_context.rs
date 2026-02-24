use std::sync::Arc;

use async_graphql::dataloader::DataLoader;

use crate::infra::grpc::{ConfigGrpcClient, FeatureFlagGrpcClient, TenantGrpcClient};

/// GraphQL リクエストコンテキスト。JWT から抽出した認証情報と DataLoader を保持する。
pub struct GraphqlContext {
    /// JWT sub クレームから取得したユーザー ID
    pub user_id: String,
    /// JWT realm_access.roles から取得したロールリスト
    pub roles: Vec<String>,
    /// リクエスト追跡 ID（X-Request-Id ヘッダーまたは UUID 自動生成）
    pub request_id: String,
    /// テナントバッチローダー
    pub tenant_loader: DataLoader<TenantLoader>,
    /// フィーチャーフラグバッチローダー
    pub flag_loader: DataLoader<FeatureFlagLoader>,
}

pub struct TenantLoader {
    pub client: Arc<TenantGrpcClient>,
}

pub struct FeatureFlagLoader {
    pub client: Arc<FeatureFlagGrpcClient>,
}

pub struct ConfigLoader {
    pub client: Arc<ConfigGrpcClient>,
}

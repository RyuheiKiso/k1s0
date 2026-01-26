//! リポジトリトレイト

use std::future::Future;

use super::entity::{Endpoint, EndpointList, EndpointQuery, ResolvedAddress};
use super::error::EndpointError;

/// エンドポイントリポジトリトレイト
pub trait EndpointRepository: Send + Sync {
    /// エンドポイントを取得
    fn get(
        &self,
        service_name: &str,
        method: Option<&str>,
        path: Option<&str>,
    ) -> impl Future<Output = Result<Option<Endpoint>, EndpointError>> + Send;

    /// エンドポイント一覧を取得
    fn list(
        &self,
        query: &EndpointQuery,
    ) -> impl Future<Output = Result<EndpointList, EndpointError>> + Send;

    /// サービス名からアドレスを解決
    fn resolve(
        &self,
        service_name: &str,
        protocol: &str,
    ) -> impl Future<Output = Result<ResolvedAddress, EndpointError>> + Send;

    /// エンドポイントを保存
    fn save(&self, endpoint: &Endpoint) -> impl Future<Output = Result<(), EndpointError>> + Send;
}

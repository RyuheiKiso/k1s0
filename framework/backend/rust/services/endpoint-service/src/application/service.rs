//! エンドポイントサービス

use std::sync::Arc;

use crate::domain::{Endpoint, EndpointError, EndpointList, EndpointQuery, EndpointRepository, ResolvedAddress};

/// エンドポイントサービス
pub struct EndpointService<R>
where
    R: EndpointRepository,
{
    repository: Arc<R>,
    namespace: String,
    cluster_domain: String,
}

impl<R> EndpointService<R>
where
    R: EndpointRepository,
{
    /// 新しいサービスを作成
    pub fn new(
        repository: Arc<R>,
        namespace: impl Into<String>,
        cluster_domain: impl Into<String>,
    ) -> Self {
        Self {
            repository,
            namespace: namespace.into(),
            cluster_domain: cluster_domain.into(),
        }
    }

    /// エンドポイントを取得
    pub async fn get_endpoint(
        &self,
        service_name: &str,
        method: Option<&str>,
        path: Option<&str>,
    ) -> Result<Endpoint, EndpointError> {
        self.repository
            .get(service_name, method, path)
            .await?
            .ok_or_else(|| EndpointError::not_found(service_name))
    }

    /// エンドポイント一覧を取得
    pub async fn list_endpoints(&self, query: &EndpointQuery) -> Result<EndpointList, EndpointError> {
        self.repository.list(query).await
    }

    /// サービス名からアドレスを解決
    ///
    /// Kubernetes DNS の規約に基づいてアドレスを生成する。
    /// 例: "auth-service" -> "auth-service.{namespace}.svc.{cluster_domain}:{port}"
    pub async fn resolve_endpoint(
        &self,
        service_name: &str,
        protocol: &str,
    ) -> Result<ResolvedAddress, EndpointError> {
        // リポジトリから解決を試みる
        if let Ok(addr) = self.repository.resolve(service_name, protocol).await {
            return Ok(addr);
        }

        // デフォルトの解決ロジック（Kubernetes DNS）
        let port = match protocol.to_lowercase().as_str() {
            "grpc" | "grpcs" => 50051,
            "http" | "https" => 8080,
            _ => return Err(EndpointError::unresolvable(service_name, "unknown protocol")),
        };

        let address = format!(
            "{}.{}.svc.{}:{}",
            service_name, self.namespace, self.cluster_domain, port
        );

        let use_tls = protocol.ends_with('s');

        Ok(ResolvedAddress::new(address, use_tls))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::InMemoryRepository;

    #[tokio::test]
    async fn test_get_endpoint() {
        let repo = Arc::new(InMemoryRepository::new());
        repo.save(&Endpoint::new(1, "auth-service", "/v1/login", "POST"))
            .await
            .unwrap();

        let service = EndpointService::new(repo, "default", "cluster.local");
        let result = service.get_endpoint("auth-service", None, None).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().service_name, "auth-service");
    }

    #[tokio::test]
    async fn test_get_endpoint_not_found() {
        let repo = Arc::new(InMemoryRepository::new());
        let service = EndpointService::new(repo, "default", "cluster.local");

        let result = service.get_endpoint("unknown", None, None).await;
        assert!(matches!(result, Err(EndpointError::NotFound { .. })));
    }

    #[tokio::test]
    async fn test_resolve_endpoint_grpc() {
        let repo = Arc::new(InMemoryRepository::new());
        let service = EndpointService::new(repo, "k1s0-prod", "cluster.local");

        let result = service.resolve_endpoint("auth-service", "grpc").await.unwrap();
        assert_eq!(result.address, "auth-service.k1s0-prod.svc.cluster.local:50051");
        assert!(!result.use_tls);
    }

    #[tokio::test]
    async fn test_resolve_endpoint_http() {
        let repo = Arc::new(InMemoryRepository::new());
        let service = EndpointService::new(repo, "k1s0-prod", "cluster.local");

        let result = service.resolve_endpoint("api-gateway", "http").await.unwrap();
        assert_eq!(result.address, "api-gateway.k1s0-prod.svc.cluster.local:8080");
        assert!(!result.use_tls);
    }

    #[tokio::test]
    async fn test_resolve_endpoint_https() {
        let repo = Arc::new(InMemoryRepository::new());
        let service = EndpointService::new(repo, "k1s0-prod", "cluster.local");

        let result = service.resolve_endpoint("api-gateway", "https").await.unwrap();
        assert!(result.use_tls);
    }
}

//! リポジトリ実装

use std::collections::HashMap;
use std::sync::RwLock;

use crate::domain::{Endpoint, EndpointError, EndpointList, EndpointQuery, EndpointRepository, ResolvedAddress};

/// インメモリリポジトリ
pub struct InMemoryRepository {
    endpoints: RwLock<HashMap<String, Endpoint>>,
    address_overrides: RwLock<HashMap<String, ResolvedAddress>>,
}

impl InMemoryRepository {
    /// 新しいリポジトリを作成
    pub fn new() -> Self {
        Self {
            endpoints: RwLock::new(HashMap::new()),
            address_overrides: RwLock::new(HashMap::new()),
        }
    }

    /// キーを生成
    fn make_key(service_name: &str, method: Option<&str>, path: Option<&str>) -> String {
        format!(
            "{}:{}:{}",
            service_name,
            method.unwrap_or("*"),
            path.unwrap_or("*")
        )
    }

    /// アドレスオーバーライドを追加
    pub fn add_address_override(&self, service_name: &str, protocol: &str, address: ResolvedAddress) {
        let key = format!("{}:{}", service_name, protocol);
        self.address_overrides.write().unwrap().insert(key, address);
    }
}

impl Default for InMemoryRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl EndpointRepository for InMemoryRepository {
    async fn get(
        &self,
        service_name: &str,
        method: Option<&str>,
        path: Option<&str>,
    ) -> Result<Option<Endpoint>, EndpointError> {
        let endpoints = self.endpoints.read().unwrap();

        // 完全マッチ
        let key = Self::make_key(service_name, method, path);
        if let Some(endpoint) = endpoints.get(&key) {
            return Ok(Some(endpoint.clone()));
        }

        // サービス名のみでマッチ
        let key = Self::make_key(service_name, None, None);
        if let Some(endpoint) = endpoints.get(&key) {
            return Ok(Some(endpoint.clone()));
        }

        // サービス名で部分マッチ
        for endpoint in endpoints.values() {
            if endpoint.service_name == service_name {
                return Ok(Some(endpoint.clone()));
            }
        }

        Ok(None)
    }

    async fn list(&self, query: &EndpointQuery) -> Result<EndpointList, EndpointError> {
        let endpoints = self.endpoints.read().unwrap();

        let mut results: Vec<Endpoint> = endpoints
            .values()
            .filter(|e| {
                if let Some(ref service_name) = query.service_name {
                    if &e.service_name != service_name {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();

        results.sort_by(|a, b| a.service_name.cmp(&b.service_name));

        let page_size = query.page_size.unwrap_or(100).min(1000) as usize;
        let start_index = query
            .page_token
            .as_ref()
            .and_then(|t| t.strip_prefix("offset:"))
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);

        let total = results.len();
        let end_index = (start_index + page_size).min(total);
        let page_results = results[start_index..end_index].to_vec();

        let next_page_token = if end_index < total {
            Some(format!("offset:{}", end_index))
        } else {
            None
        };

        let mut list = EndpointList::new(page_results);
        if let Some(token) = next_page_token {
            list = list.with_next_page_token(token);
        }

        Ok(list)
    }

    async fn resolve(
        &self,
        service_name: &str,
        protocol: &str,
    ) -> Result<ResolvedAddress, EndpointError> {
        let key = format!("{}:{}", service_name, protocol);
        let overrides = self.address_overrides.read().unwrap();

        overrides
            .get(&key)
            .cloned()
            .ok_or_else(|| EndpointError::not_found(service_name))
    }

    async fn save(&self, endpoint: &Endpoint) -> Result<(), EndpointError> {
        let key = Self::make_key(&endpoint.service_name, Some(&endpoint.method), Some(&endpoint.path));
        let mut endpoints = self.endpoints.write().unwrap();
        endpoints.insert(key, endpoint.clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_save_and_get() {
        let repo = InMemoryRepository::new();
        let endpoint = Endpoint::new(1, "auth-service", "/v1/login", "POST");
        repo.save(&endpoint).await.unwrap();

        let result = repo.get("auth-service", None, None).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().service_name, "auth-service");
    }

    #[tokio::test]
    async fn test_get_not_found() {
        let repo = InMemoryRepository::new();
        let result = repo.get("unknown", None, None).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_list() {
        let repo = InMemoryRepository::new();
        repo.save(&Endpoint::new(1, "svc-a", "/path", "GET")).await.unwrap();
        repo.save(&Endpoint::new(2, "svc-b", "/path", "GET")).await.unwrap();

        let query = EndpointQuery::new();
        let result = repo.list(&query).await.unwrap();
        assert_eq!(result.endpoints.len(), 2);
    }

    #[tokio::test]
    async fn test_list_filter() {
        let repo = InMemoryRepository::new();
        repo.save(&Endpoint::new(1, "svc-a", "/path", "GET")).await.unwrap();
        repo.save(&Endpoint::new(2, "svc-b", "/path", "GET")).await.unwrap();

        let query = EndpointQuery::new().with_service_name("svc-a");
        let result = repo.list(&query).await.unwrap();
        assert_eq!(result.endpoints.len(), 1);
    }

    #[tokio::test]
    async fn test_resolve_override() {
        let repo = InMemoryRepository::new();
        repo.add_address_override(
            "auth-service",
            "grpc",
            ResolvedAddress::new("auth-service.custom.svc:50051", true),
        );

        let result = repo.resolve("auth-service", "grpc").await.unwrap();
        assert_eq!(result.address, "auth-service.custom.svc:50051");
        assert!(result.use_tls);
    }
}

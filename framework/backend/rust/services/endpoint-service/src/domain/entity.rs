//! エンドポイントエンティティ

use std::time::SystemTime;

/// HTTPメソッド
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Options,
    Head,
}

impl HttpMethod {
    /// 文字列表現を取得
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Patch => "PATCH",
            Self::Delete => "DELETE",
            Self::Options => "OPTIONS",
            Self::Head => "HEAD",
        }
    }

    /// 文字列から変換
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "GET" => Some(Self::Get),
            "POST" => Some(Self::Post),
            "PUT" => Some(Self::Put),
            "PATCH" => Some(Self::Patch),
            "DELETE" => Some(Self::Delete),
            "OPTIONS" => Some(Self::Options),
            "HEAD" => Some(Self::Head),
            _ => None,
        }
    }
}

/// プロトコル
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    Http,
    Grpc,
}

impl Protocol {
    /// 文字列表現を取得
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Http => "http",
            Self::Grpc => "grpc",
        }
    }

    /// 文字列から変換
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "http" | "https" => Some(Self::Http),
            "grpc" | "grpcs" => Some(Self::Grpc),
            _ => None,
        }
    }
}

/// エンドポイントエンティティ
#[derive(Debug, Clone)]
pub struct Endpoint {
    /// エンドポイントID
    pub id: i32,
    /// サービス名
    pub service_name: String,
    /// パス
    pub path: String,
    /// HTTPメソッド
    pub method: String,
    /// 作成日時
    pub created_at: SystemTime,
    /// 更新日時
    pub updated_at: SystemTime,
}

impl Endpoint {
    /// 新しいエンドポイントを作成
    pub fn new(
        id: i32,
        service_name: impl Into<String>,
        path: impl Into<String>,
        method: impl Into<String>,
    ) -> Self {
        let now = SystemTime::now();
        Self {
            id,
            service_name: service_name.into(),
            path: path.into(),
            method: method.into(),
            created_at: now,
            updated_at: now,
        }
    }
}

/// 解決済みアドレス
#[derive(Debug, Clone)]
pub struct ResolvedAddress {
    /// アドレス
    pub address: String,
    /// TLSを使用するか
    pub use_tls: bool,
}

impl ResolvedAddress {
    /// 新しい解決済みアドレスを作成
    pub fn new(address: impl Into<String>, use_tls: bool) -> Self {
        Self {
            address: address.into(),
            use_tls,
        }
    }
}

/// エンドポイント検索条件
#[derive(Debug, Clone, Default)]
pub struct EndpointQuery {
    /// サービス名
    pub service_name: Option<String>,
    /// ページサイズ
    pub page_size: Option<u32>,
    /// ページトークン
    pub page_token: Option<String>,
}

impl EndpointQuery {
    /// 新しいクエリを作成
    pub fn new() -> Self {
        Self::default()
    }

    /// サービス名を設定
    pub fn with_service_name(mut self, name: impl Into<String>) -> Self {
        self.service_name = Some(name.into());
        self
    }

    /// ページサイズを設定
    pub fn with_page_size(mut self, size: u32) -> Self {
        self.page_size = Some(size);
        self
    }

    /// ページトークンを設定
    pub fn with_page_token(mut self, token: impl Into<String>) -> Self {
        self.page_token = Some(token.into());
        self
    }
}

/// エンドポイント一覧の結果
#[derive(Debug, Clone)]
pub struct EndpointList {
    /// エンドポイント一覧
    pub endpoints: Vec<Endpoint>,
    /// 次ページのトークン
    pub next_page_token: Option<String>,
}

impl EndpointList {
    /// 新しい結果を作成
    pub fn new(endpoints: Vec<Endpoint>) -> Self {
        Self {
            endpoints,
            next_page_token: None,
        }
    }

    /// 次ページトークンを設定
    pub fn with_next_page_token(mut self, token: impl Into<String>) -> Self {
        self.next_page_token = Some(token.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================
    // HttpMethod Tests
    // ========================================

    #[test]
    fn test_http_method_as_str() {
        assert_eq!(HttpMethod::Get.as_str(), "GET");
        assert_eq!(HttpMethod::Post.as_str(), "POST");
        assert_eq!(HttpMethod::Put.as_str(), "PUT");
        assert_eq!(HttpMethod::Patch.as_str(), "PATCH");
        assert_eq!(HttpMethod::Delete.as_str(), "DELETE");
        assert_eq!(HttpMethod::Options.as_str(), "OPTIONS");
        assert_eq!(HttpMethod::Head.as_str(), "HEAD");
    }

    #[test]
    fn test_http_method_from_str_lowercase() {
        assert_eq!(HttpMethod::from_str("get"), Some(HttpMethod::Get));
        assert_eq!(HttpMethod::from_str("post"), Some(HttpMethod::Post));
        assert_eq!(HttpMethod::from_str("put"), Some(HttpMethod::Put));
        assert_eq!(HttpMethod::from_str("patch"), Some(HttpMethod::Patch));
        assert_eq!(HttpMethod::from_str("delete"), Some(HttpMethod::Delete));
        assert_eq!(HttpMethod::from_str("options"), Some(HttpMethod::Options));
        assert_eq!(HttpMethod::from_str("head"), Some(HttpMethod::Head));
    }

    #[test]
    fn test_http_method_from_str_uppercase() {
        assert_eq!(HttpMethod::from_str("GET"), Some(HttpMethod::Get));
        assert_eq!(HttpMethod::from_str("POST"), Some(HttpMethod::Post));
        assert_eq!(HttpMethod::from_str("DELETE"), Some(HttpMethod::Delete));
    }

    #[test]
    fn test_http_method_from_str_mixed_case() {
        assert_eq!(HttpMethod::from_str("GeT"), Some(HttpMethod::Get));
        assert_eq!(HttpMethod::from_str("pOsT"), Some(HttpMethod::Post));
    }

    #[test]
    fn test_http_method_from_str_invalid() {
        assert_eq!(HttpMethod::from_str("unknown"), None);
        assert_eq!(HttpMethod::from_str(""), None);
        assert_eq!(HttpMethod::from_str("CONNECT"), None);
        assert_eq!(HttpMethod::from_str("TRACE"), None);
    }

    #[test]
    fn test_http_method_eq() {
        assert_eq!(HttpMethod::Get, HttpMethod::Get);
        assert_ne!(HttpMethod::Get, HttpMethod::Post);
    }

    #[test]
    fn test_http_method_clone() {
        let method = HttpMethod::Post;
        let cloned = method.clone();
        assert_eq!(method, cloned);
    }

    #[test]
    fn test_http_method_copy() {
        let method = HttpMethod::Delete;
        let copied = method;
        assert_eq!(method, copied);
    }

    // ========================================
    // Protocol Tests
    // ========================================

    #[test]
    fn test_protocol_as_str() {
        assert_eq!(Protocol::Http.as_str(), "http");
        assert_eq!(Protocol::Grpc.as_str(), "grpc");
    }

    #[test]
    fn test_protocol_from_str_valid() {
        assert_eq!(Protocol::from_str("http"), Some(Protocol::Http));
        assert_eq!(Protocol::from_str("https"), Some(Protocol::Http));
        assert_eq!(Protocol::from_str("grpc"), Some(Protocol::Grpc));
        assert_eq!(Protocol::from_str("grpcs"), Some(Protocol::Grpc));
    }

    #[test]
    fn test_protocol_from_str_case_insensitive() {
        assert_eq!(Protocol::from_str("HTTP"), Some(Protocol::Http));
        assert_eq!(Protocol::from_str("HTTPS"), Some(Protocol::Http));
        assert_eq!(Protocol::from_str("GRPC"), Some(Protocol::Grpc));
        assert_eq!(Protocol::from_str("GrPc"), Some(Protocol::Grpc));
    }

    #[test]
    fn test_protocol_from_str_invalid() {
        assert_eq!(Protocol::from_str("unknown"), None);
        assert_eq!(Protocol::from_str(""), None);
        assert_eq!(Protocol::from_str("tcp"), None);
        assert_eq!(Protocol::from_str("ws"), None);
    }

    #[test]
    fn test_protocol_eq() {
        assert_eq!(Protocol::Http, Protocol::Http);
        assert_ne!(Protocol::Http, Protocol::Grpc);
    }

    #[test]
    fn test_protocol_clone() {
        let protocol = Protocol::Grpc;
        let cloned = protocol.clone();
        assert_eq!(protocol, cloned);
    }

    // ========================================
    // Endpoint Tests
    // ========================================

    #[test]
    fn test_endpoint_new() {
        let endpoint = Endpoint::new(1, "auth-service", "/v1/login", "POST");
        assert_eq!(endpoint.id, 1);
        assert_eq!(endpoint.service_name, "auth-service");
        assert_eq!(endpoint.path, "/v1/login");
        assert_eq!(endpoint.method, "POST");
    }

    #[test]
    fn test_endpoint_new_with_string_types() {
        let endpoint = Endpoint::new(
            1,
            String::from("service"),
            String::from("/path"),
            String::from("GET"),
        );
        assert_eq!(endpoint.service_name, "service");
        assert_eq!(endpoint.path, "/path");
        assert_eq!(endpoint.method, "GET");
    }

    #[test]
    fn test_endpoint_timestamps_are_set() {
        let before = SystemTime::now();
        let endpoint = Endpoint::new(1, "service", "/path", "GET");
        let after = SystemTime::now();

        assert!(endpoint.created_at >= before);
        assert!(endpoint.created_at <= after);
        assert!(endpoint.updated_at >= before);
        assert!(endpoint.updated_at <= after);
    }

    #[test]
    fn test_endpoint_clone() {
        let endpoint = Endpoint::new(1, "service", "/path", "GET");
        let cloned = endpoint.clone();

        assert_eq!(endpoint.id, cloned.id);
        assert_eq!(endpoint.service_name, cloned.service_name);
        assert_eq!(endpoint.path, cloned.path);
        assert_eq!(endpoint.method, cloned.method);
    }

    #[test]
    fn test_endpoint_with_empty_strings() {
        let endpoint = Endpoint::new(0, "", "", "");
        assert_eq!(endpoint.service_name, "");
        assert_eq!(endpoint.path, "");
        assert_eq!(endpoint.method, "");
    }

    #[test]
    fn test_endpoint_with_negative_id() {
        let endpoint = Endpoint::new(-1, "service", "/path", "GET");
        assert_eq!(endpoint.id, -1);
    }

    // ========================================
    // ResolvedAddress Tests
    // ========================================

    #[test]
    fn test_resolved_address_new() {
        let addr = ResolvedAddress::new("auth-service:50051", true);
        assert_eq!(addr.address, "auth-service:50051");
        assert!(addr.use_tls);
    }

    #[test]
    fn test_resolved_address_no_tls() {
        let addr = ResolvedAddress::new("service:8080", false);
        assert_eq!(addr.address, "service:8080");
        assert!(!addr.use_tls);
    }

    #[test]
    fn test_resolved_address_with_string() {
        let addr = ResolvedAddress::new(String::from("localhost:9090"), true);
        assert_eq!(addr.address, "localhost:9090");
    }

    #[test]
    fn test_resolved_address_clone() {
        let addr = ResolvedAddress::new("service:8080", true);
        let cloned = addr.clone();

        assert_eq!(addr.address, cloned.address);
        assert_eq!(addr.use_tls, cloned.use_tls);
    }

    #[test]
    fn test_resolved_address_with_ip() {
        let addr = ResolvedAddress::new("192.168.1.100:443", true);
        assert_eq!(addr.address, "192.168.1.100:443");
    }

    #[test]
    fn test_resolved_address_with_fqdn() {
        let addr = ResolvedAddress::new("my-service.namespace.svc.cluster.local:50051", false);
        assert_eq!(addr.address, "my-service.namespace.svc.cluster.local:50051");
    }

    // ========================================
    // EndpointQuery Tests
    // ========================================

    #[test]
    fn test_endpoint_query_new() {
        let query = EndpointQuery::new();
        assert!(query.service_name.is_none());
        assert!(query.page_size.is_none());
        assert!(query.page_token.is_none());
    }

    #[test]
    fn test_endpoint_query_default() {
        let query = EndpointQuery::default();
        assert!(query.service_name.is_none());
        assert!(query.page_size.is_none());
        assert!(query.page_token.is_none());
    }

    #[test]
    fn test_endpoint_query_with_service_name() {
        let query = EndpointQuery::new().with_service_name("my-service");
        assert_eq!(query.service_name, Some("my-service".to_string()));
    }

    #[test]
    fn test_endpoint_query_with_page_size() {
        let query = EndpointQuery::new().with_page_size(50);
        assert_eq!(query.page_size, Some(50));
    }

    #[test]
    fn test_endpoint_query_with_page_token() {
        let query = EndpointQuery::new().with_page_token("offset:100");
        assert_eq!(query.page_token, Some("offset:100".to_string()));
    }

    #[test]
    fn test_endpoint_query_builder_chain() {
        let query = EndpointQuery::new()
            .with_service_name("service")
            .with_page_size(25)
            .with_page_token("next");

        assert_eq!(query.service_name, Some("service".to_string()));
        assert_eq!(query.page_size, Some(25));
        assert_eq!(query.page_token, Some("next".to_string()));
    }

    #[test]
    fn test_endpoint_query_clone() {
        let query = EndpointQuery::new()
            .with_service_name("service")
            .with_page_size(10);
        let cloned = query.clone();

        assert_eq!(query.service_name, cloned.service_name);
        assert_eq!(query.page_size, cloned.page_size);
    }

    // ========================================
    // EndpointList Tests
    // ========================================

    #[test]
    fn test_endpoint_list_new_empty() {
        let list = EndpointList::new(vec![]);
        assert!(list.endpoints.is_empty());
        assert!(list.next_page_token.is_none());
    }

    #[test]
    fn test_endpoint_list_new_with_endpoints() {
        let endpoints = vec![
            Endpoint::new(1, "svc-a", "/path1", "GET"),
            Endpoint::new(2, "svc-b", "/path2", "POST"),
        ];
        let list = EndpointList::new(endpoints);

        assert_eq!(list.endpoints.len(), 2);
        assert!(list.next_page_token.is_none());
    }

    #[test]
    fn test_endpoint_list_with_next_page_token() {
        let list = EndpointList::new(vec![])
            .with_next_page_token("offset:50");
        assert_eq!(list.next_page_token, Some("offset:50".to_string()));
    }

    #[test]
    fn test_endpoint_list_clone() {
        let endpoints = vec![Endpoint::new(1, "svc", "/path", "GET")];
        let list = EndpointList::new(endpoints).with_next_page_token("token");
        let cloned = list.clone();

        assert_eq!(list.endpoints.len(), cloned.endpoints.len());
        assert_eq!(list.next_page_token, cloned.next_page_token);
    }

    // ========================================
    // Debug Trait Tests
    // ========================================

    #[test]
    fn test_http_method_debug() {
        let debug = format!("{:?}", HttpMethod::Get);
        assert!(debug.contains("Get"));
    }

    #[test]
    fn test_protocol_debug() {
        let debug = format!("{:?}", Protocol::Grpc);
        assert!(debug.contains("Grpc"));
    }

    #[test]
    fn test_endpoint_debug() {
        let endpoint = Endpoint::new(1, "service", "/path", "GET");
        let debug = format!("{:?}", endpoint);
        assert!(debug.contains("service"));
        assert!(debug.contains("/path"));
    }

    #[test]
    fn test_resolved_address_debug() {
        let addr = ResolvedAddress::new("host:port", true);
        let debug = format!("{:?}", addr);
        assert!(debug.contains("host:port"));
    }
}

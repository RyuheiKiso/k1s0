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

    #[test]
    fn test_http_method() {
        assert_eq!(HttpMethod::Get.as_str(), "GET");
        assert_eq!(HttpMethod::Post.as_str(), "POST");
        assert_eq!(HttpMethod::from_str("get"), Some(HttpMethod::Get));
        assert_eq!(HttpMethod::from_str("unknown"), None);
    }

    #[test]
    fn test_protocol() {
        assert_eq!(Protocol::Http.as_str(), "http");
        assert_eq!(Protocol::Grpc.as_str(), "grpc");
        assert_eq!(Protocol::from_str("http"), Some(Protocol::Http));
        assert_eq!(Protocol::from_str("grpc"), Some(Protocol::Grpc));
    }

    #[test]
    fn test_endpoint_new() {
        let endpoint = Endpoint::new(1, "auth-service", "/v1/login", "POST");
        assert_eq!(endpoint.id, 1);
        assert_eq!(endpoint.service_name, "auth-service");
        assert_eq!(endpoint.path, "/v1/login");
        assert_eq!(endpoint.method, "POST");
    }

    #[test]
    fn test_resolved_address() {
        let addr = ResolvedAddress::new("auth-service:50051", true);
        assert_eq!(addr.address, "auth-service:50051");
        assert!(addr.use_tls);
    }
}

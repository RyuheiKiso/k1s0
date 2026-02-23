use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// モックルート定義。
#[derive(Debug, Clone)]
pub struct MockRoute {
    pub method: String,
    pub path: String,
    pub status: u16,
    pub body: String,
}

/// モックサーバー (インメモリ)。
///
/// 実際の HTTP サーバーは起動せず、リクエスト/レスポンスのマッピングを保持する。
pub struct MockServer {
    routes: Vec<MockRoute>,
    requests: Arc<Mutex<Vec<(String, String)>>>,
}

impl MockServer {
    fn new(routes: Vec<MockRoute>) -> Self {
        Self {
            routes,
            requests: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// 登録済みルートからレスポンスを取得する。
    pub fn handle(&self, method: &str, path: &str) -> Option<(u16, String)> {
        self.requests
            .lock()
            .unwrap()
            .push((method.to_string(), path.to_string()));
        self.routes
            .iter()
            .find(|r| r.method == method && r.path == path)
            .map(|r| (r.status, r.body.clone()))
    }

    /// 記録されたリクエスト数を返す。
    pub fn request_count(&self) -> usize {
        self.requests.lock().unwrap().len()
    }

    /// 記録されたリクエストを返す。
    pub fn recorded_requests(&self) -> Vec<(String, String)> {
        self.requests.lock().unwrap().clone()
    }

    /// ベース URL を返す（テスト用にプレースホルダー）。
    pub fn base_url(&self) -> String {
        "http://localhost:0".to_string()
    }
}

/// モックサーバービルダー。
pub struct MockServerBuilder {
    server_type: String,
    routes: Vec<MockRoute>,
}

impl MockServerBuilder {
    /// Notification サーバーモックを構築する。
    pub fn notification_server() -> Self {
        Self {
            server_type: "notification".to_string(),
            routes: Vec::new(),
        }
    }

    /// Ratelimit サーバーモックを構築する。
    pub fn ratelimit_server() -> Self {
        Self {
            server_type: "ratelimit".to_string(),
            routes: Vec::new(),
        }
    }

    /// Tenant サーバーモックを構築する。
    pub fn tenant_server() -> Self {
        Self {
            server_type: "tenant".to_string(),
            routes: Vec::new(),
        }
    }

    /// ヘルスチェック用の成功レスポンスを追加する。
    pub fn with_health_ok(mut self) -> Self {
        self.routes.push(MockRoute {
            method: "GET".to_string(),
            path: "/health".to_string(),
            status: 200,
            body: r#"{"status":"ok"}"#.to_string(),
        });
        self
    }

    /// 成功レスポンスルートを追加する。
    pub fn with_success_response(mut self, path: &str, body: &str) -> Self {
        self.routes.push(MockRoute {
            method: "POST".to_string(),
            path: path.to_string(),
            status: 200,
            body: body.to_string(),
        });
        self
    }

    /// エラーレスポンスルートを追加する。
    pub fn with_error_response(mut self, path: &str, status: u16) -> Self {
        self.routes.push(MockRoute {
            method: "POST".to_string(),
            path: path.to_string(),
            status,
            body: r#"{"error":"mock error"}"#.to_string(),
        });
        self
    }

    /// サーバータイプを返す。
    pub fn server_type(&self) -> &str {
        &self.server_type
    }

    /// モックサーバーを構築する。
    pub fn build(self) -> MockServer {
        MockServer::new(self.routes)
    }
}

/// サーバータイプ別のデフォルトルート付きモックを生成するヘルパー。
pub fn default_mock_routes(server_type: &str) -> HashMap<String, MockRoute> {
    let mut routes = HashMap::new();
    routes.insert(
        "health".to_string(),
        MockRoute {
            method: "GET".to_string(),
            path: "/health".to_string(),
            status: 200,
            body: format!(r#"{{"service":"{}","status":"ok"}}"#, server_type),
        },
    );
    routes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_server_builder_notification() {
        let server = MockServerBuilder::notification_server()
            .with_health_ok()
            .with_success_response("/send", r#"{"id":"1","status":"sent"}"#)
            .build();

        let (status, body) = server.handle("GET", "/health").unwrap();
        assert_eq!(status, 200);
        assert!(body.contains("ok"));

        let (status, _) = server.handle("POST", "/send").unwrap();
        assert_eq!(status, 200);

        assert_eq!(server.request_count(), 2);
    }

    #[test]
    fn test_mock_server_builder_ratelimit() {
        let server = MockServerBuilder::ratelimit_server()
            .with_health_ok()
            .build();

        assert_eq!(server.handle("GET", "/health").unwrap().0, 200);
        assert!(server.handle("GET", "/nonexistent").is_none());
    }

    #[test]
    fn test_mock_server_error_response() {
        let server = MockServerBuilder::tenant_server()
            .with_error_response("/create", 500)
            .build();

        let (status, body) = server.handle("POST", "/create").unwrap();
        assert_eq!(status, 500);
        assert!(body.contains("error"));
    }

    #[test]
    fn test_recorded_requests() {
        let server = MockServerBuilder::notification_server()
            .with_health_ok()
            .build();
        server.handle("GET", "/health");
        server.handle("GET", "/unknown");
        let reqs = server.recorded_requests();
        assert_eq!(reqs.len(), 2);
        assert_eq!(reqs[0], ("GET".to_string(), "/health".to_string()));
    }

    #[test]
    fn test_default_mock_routes() {
        let routes = default_mock_routes("notification");
        assert!(routes.contains_key("health"));
        assert_eq!(routes["health"].status, 200);
    }
}

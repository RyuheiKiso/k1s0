//! Docker ビルド時のプロキシ自動転送
//!
//! 環境変数からプロキシ設定を検出し、Docker build-args に変換する。

/// プロキシ設定
#[derive(Debug, Clone, Default)]
pub struct ProxyConfig {
    pub http_proxy: Option<String>,
    pub https_proxy: Option<String>,
    pub no_proxy: Option<String>,
}

impl ProxyConfig {
    /// 環境変数からプロキシ設定を検出する
    pub fn detect() -> Self {
        Self {
            http_proxy: std::env::var("HTTP_PROXY")
                .or_else(|_| std::env::var("http_proxy"))
                .ok(),
            https_proxy: std::env::var("HTTPS_PROXY")
                .or_else(|_| std::env::var("https_proxy"))
                .ok(),
            no_proxy: std::env::var("NO_PROXY")
                .or_else(|_| std::env::var("no_proxy"))
                .ok(),
        }
    }

    /// プロキシが設定されているか
    pub fn is_configured(&self) -> bool {
        self.http_proxy.is_some() || self.https_proxy.is_some()
    }

    /// Docker build-args 形式に変換する
    pub fn to_build_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        if let Some(ref v) = self.http_proxy {
            args.push(format!("--build-arg=HTTP_PROXY={v}"));
        }
        if let Some(ref v) = self.https_proxy {
            args.push(format!("--build-arg=HTTPS_PROXY={v}"));
        }
        if let Some(ref v) = self.no_proxy {
            args.push(format!("--build-arg=NO_PROXY={v}"));
        }
        args
    }

    /// 診断情報を返す
    pub fn diagnostics(&self) -> Vec<String> {
        let mut info = Vec::new();
        if let Some(ref v) = self.http_proxy {
            info.push(format!("HTTP_PROXY: {v}"));
        }
        if let Some(ref v) = self.https_proxy {
            info.push(format!("HTTPS_PROXY: {v}"));
        }
        if let Some(ref v) = self.no_proxy {
            info.push(format!("NO_PROXY: {v}"));
        }
        if info.is_empty() {
            info.push("プロキシ設定: なし".to_string());
        }
        info
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_no_proxy() {
        let config = ProxyConfig::default();
        assert!(!config.is_configured());
        assert!(config.to_build_args().is_empty());
    }

    #[test]
    fn test_with_proxy() {
        let config = ProxyConfig {
            http_proxy: Some("http://proxy:8080".to_string()),
            https_proxy: Some("http://proxy:8443".to_string()),
            no_proxy: Some("localhost,127.0.0.1".to_string()),
        };
        assert!(config.is_configured());
        let args = config.to_build_args();
        assert_eq!(args.len(), 3);
        assert!(args[0].contains("HTTP_PROXY=http://proxy:8080"));
        assert!(args[1].contains("HTTPS_PROXY=http://proxy:8443"));
        assert!(args[2].contains("NO_PROXY=localhost,127.0.0.1"));
    }

    #[test]
    fn test_partial_proxy() {
        let config = ProxyConfig {
            http_proxy: Some("http://proxy:8080".to_string()),
            https_proxy: None,
            no_proxy: None,
        };
        assert!(config.is_configured());
        assert_eq!(config.to_build_args().len(), 1);
    }

    #[test]
    fn test_diagnostics_no_proxy() {
        let config = ProxyConfig::default();
        let diag = config.diagnostics();
        assert_eq!(diag.len(), 1);
        assert!(diag[0].contains("なし"));
    }

    #[test]
    fn test_diagnostics_with_proxy() {
        let config = ProxyConfig {
            http_proxy: Some("http://proxy:8080".to_string()),
            https_proxy: None,
            no_proxy: None,
        };
        let diag = config.diagnostics();
        assert_eq!(diag.len(), 1);
        assert!(diag[0].contains("HTTP_PROXY"));
    }
}

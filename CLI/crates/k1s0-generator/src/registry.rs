//! テンプレートレジストリ
//!
//! リモートレジストリからテンプレートを取得・公開する機能を提供する。

use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

/// レジストリエラー
#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    /// ネットワークエラー
    #[error("Network error: {0}")]
    Network(String),

    /// 認証エラー
    #[error("Authentication failed: {0}")]
    Auth(String),

    /// テンプレートが見つからない
    #[error("Template not found: {0}")]
    NotFound(String),

    /// 無効なテンプレート
    #[error("Invalid template: {0}")]
    InvalidTemplate(String),

    /// キャッシュエラー
    #[error("Cache error: {0}")]
    Cache(String),

    /// IOエラー
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSONエラー
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// バージョン競合
    #[error("Version conflict: {0}")]
    VersionConflict(String),
}

/// レジストリ結果型
pub type RegistryResult<T> = Result<T, RegistryError>;

/// レジストリ設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    /// レジストリURL
    pub url: String,
    /// キャッシュディレクトリ
    pub cache_dir: PathBuf,
    /// 認証トークン（オプション）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_token: Option<String>,
    /// タイムアウト（秒）
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    /// キャッシュ有効期限（秒）
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl_secs: u64,
}

fn default_timeout() -> u64 {
    30
}

fn default_cache_ttl() -> u64 {
    3600 // 1時間
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            url: "https://registry.k1s0.dev".to_string(),
            cache_dir: dirs::cache_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("k1s0/templates"),
            auth_token: None,
            timeout_secs: default_timeout(),
            cache_ttl_secs: default_cache_ttl(),
        }
    }
}

/// テンプレートメタデータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMetadata {
    /// テンプレート名
    pub name: String,
    /// バージョン
    pub version: String,
    /// 説明
    pub description: String,
    /// 作者
    pub author: String,
    /// タグ
    #[serde(default)]
    pub tags: Vec<String>,
    /// 言語
    pub language: String,
    /// サービスタイプ
    pub service_type: String,
    /// 作成日時
    pub created_at: String,
    /// 更新日時
    pub updated_at: String,
    /// ダウンロード数
    #[serde(default)]
    pub downloads: u64,
    /// 依存関係
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
    /// 変数定義
    #[serde(default)]
    pub variables: Vec<TemplateVariable>,
}

/// テンプレート変数定義
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    /// 変数名
    pub name: String,
    /// 説明
    pub description: String,
    /// デフォルト値
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    /// 必須かどうか
    #[serde(default)]
    pub required: bool,
    /// 型（string, boolean, number, array）
    #[serde(default = "default_var_type")]
    pub var_type: String,
}

fn default_var_type() -> String {
    "string".to_string()
}

/// テンプレート一覧レスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateListResponse {
    /// テンプレート一覧
    pub templates: Vec<TemplateMetadata>,
    /// 総数
    pub total: usize,
    /// ページ番号
    pub page: usize,
    /// ページサイズ
    pub page_size: usize,
}

/// テンプレートレジストリクライアント
pub struct RegistryClient {
    config: RegistryConfig,
}

impl RegistryClient {
    /// 新しいクライアントを作成
    pub fn new(config: RegistryConfig) -> Self {
        Self { config }
    }

    /// デフォルト設定で作成
    pub fn with_defaults() -> Self {
        Self::new(RegistryConfig::default())
    }

    /// 認証トークンを設定
    pub fn with_auth_token(mut self, token: impl Into<String>) -> Self {
        self.config.auth_token = Some(token.into());
        self
    }

    /// テンプレート一覧を取得
    pub fn list_templates(
        &self,
        filter: Option<&TemplateFilter>,
    ) -> RegistryResult<TemplateListResponse> {
        // キャッシュを確認
        let cache_key = format!("list_{}", filter.map(|f| f.cache_key()).unwrap_or_default());
        if let Some(cached) = self.get_from_cache::<TemplateListResponse>(&cache_key)? {
            return Ok(cached);
        }

        // 実際のAPI呼び出し（将来実装）
        // 現在はモックデータを返す
        let response = self.mock_list_templates(filter);

        // キャッシュに保存
        self.save_to_cache(&cache_key, &response)?;

        Ok(response)
    }

    /// テンプレートを取得
    pub fn fetch_template(&self, name: &str, version: Option<&str>) -> RegistryResult<PathBuf> {
        let version = version.unwrap_or("latest");

        // キャッシュを確認
        let cache_path = self
            .config
            .cache_dir
            .join(name)
            .join(format!("{}.tar.gz", version));

        if cache_path.exists() && !self.is_cache_expired(&cache_path)? {
            return Ok(cache_path);
        }

        // 実際のダウンロード（将来実装）
        // 現在はエラーを返す
        Err(RegistryError::Network(format!(
            "Remote registry not implemented. Template: {}@{}",
            name, version
        )))
    }

    /// テンプレートメタデータを取得
    pub fn get_template_info(&self, name: &str) -> RegistryResult<TemplateMetadata> {
        // キャッシュを確認
        let cache_key = format!("info_{}", name);
        if let Some(cached) = self.get_from_cache::<TemplateMetadata>(&cache_key)? {
            return Ok(cached);
        }

        // 実際のAPI呼び出し（将来実装）
        Err(RegistryError::NotFound(format!(
            "Template '{}' not found in registry",
            name
        )))
    }

    /// テンプレートを公開
    pub fn publish_template(&self, template_path: &Path) -> RegistryResult<TemplateMetadata> {
        // 認証確認
        if self.config.auth_token.is_none() {
            return Err(RegistryError::Auth(
                "Authentication token required for publishing".to_string(),
            ));
        }

        // テンプレートの検証
        self.validate_template(template_path)?;

        // 実際のアップロード（将来実装）
        Err(RegistryError::Network(
            "Remote registry not implemented".to_string(),
        ))
    }

    /// テンプレートを検証
    fn validate_template(&self, path: &Path) -> RegistryResult<()> {
        // manifest.json の存在確認
        let manifest_path = path.join("manifest.json");
        if !manifest_path.exists() {
            return Err(RegistryError::InvalidTemplate(
                "manifest.json not found".to_string(),
            ));
        }

        // manifest.json の読み込み
        let content = fs::read_to_string(&manifest_path)?;
        let _manifest: serde_json::Value = serde_json::from_str(&content)?;

        // 必須フィールドの確認（将来拡張）

        Ok(())
    }

    /// キャッシュから取得
    fn get_from_cache<T: for<'de> Deserialize<'de>>(
        &self,
        key: &str,
    ) -> RegistryResult<Option<T>> {
        let cache_file = self.config.cache_dir.join(format!("{}.json", key));

        if !cache_file.exists() {
            return Ok(None);
        }

        // 有効期限確認
        if self.is_cache_expired(&cache_file)? {
            return Ok(None);
        }

        let content = fs::read_to_string(&cache_file)?;
        let value = serde_json::from_str(&content)?;

        Ok(Some(value))
    }

    /// キャッシュに保存
    fn save_to_cache<T: Serialize>(&self, key: &str, value: &T) -> RegistryResult<()> {
        fs::create_dir_all(&self.config.cache_dir)?;

        let cache_file = self.config.cache_dir.join(format!("{}.json", key));
        let content = serde_json::to_string_pretty(value)?;

        let mut file = fs::File::create(&cache_file)?;
        file.write_all(content.as_bytes())?;

        Ok(())
    }

    /// キャッシュが期限切れかどうか
    fn is_cache_expired(&self, path: &Path) -> RegistryResult<bool> {
        let metadata = fs::metadata(path)?;
        let modified = metadata.modified()?;

        let now = SystemTime::now();
        let age = now.duration_since(modified).unwrap_or(Duration::ZERO);

        Ok(age.as_secs() > self.config.cache_ttl_secs)
    }

    /// キャッシュをクリア
    pub fn clear_cache(&self) -> RegistryResult<()> {
        if self.config.cache_dir.exists() {
            fs::remove_dir_all(&self.config.cache_dir)?;
        }
        Ok(())
    }

    /// モック: テンプレート一覧
    fn mock_list_templates(&self, filter: Option<&TemplateFilter>) -> TemplateListResponse {
        let mut templates = vec![
            TemplateMetadata {
                name: "backend-rust".to_string(),
                version: "1.0.0".to_string(),
                description: "Rust バックエンドサービステンプレート".to_string(),
                author: "k1s0 Team".to_string(),
                tags: vec!["rust".to_string(), "backend".to_string(), "grpc".to_string()],
                language: "rust".to_string(),
                service_type: "backend".to_string(),
                created_at: "2024-01-01T00:00:00Z".to_string(),
                updated_at: "2024-06-01T00:00:00Z".to_string(),
                downloads: 1250,
                dependencies: HashMap::new(),
                variables: vec![
                    TemplateVariable {
                        name: "feature_name".to_string(),
                        description: "サービス名".to_string(),
                        default: None,
                        required: true,
                        var_type: "string".to_string(),
                    },
                    TemplateVariable {
                        name: "with_db".to_string(),
                        description: "データベースを使用するか".to_string(),
                        default: Some("true".to_string()),
                        required: false,
                        var_type: "boolean".to_string(),
                    },
                ],
            },
            TemplateMetadata {
                name: "backend-go".to_string(),
                version: "1.0.0".to_string(),
                description: "Go バックエンドサービステンプレート".to_string(),
                author: "k1s0 Team".to_string(),
                tags: vec!["go".to_string(), "backend".to_string(), "grpc".to_string()],
                language: "go".to_string(),
                service_type: "backend".to_string(),
                created_at: "2024-01-01T00:00:00Z".to_string(),
                updated_at: "2024-05-15T00:00:00Z".to_string(),
                downloads: 980,
                dependencies: HashMap::new(),
                variables: vec![],
            },
            TemplateMetadata {
                name: "frontend-react".to_string(),
                version: "1.0.0".to_string(),
                description: "React フロントエンドテンプレート".to_string(),
                author: "k1s0 Team".to_string(),
                tags: vec!["react".to_string(), "frontend".to_string(), "typescript".to_string()],
                language: "typescript".to_string(),
                service_type: "frontend".to_string(),
                created_at: "2024-01-01T00:00:00Z".to_string(),
                updated_at: "2024-06-15T00:00:00Z".to_string(),
                downloads: 2100,
                dependencies: HashMap::new(),
                variables: vec![],
            },
            TemplateMetadata {
                name: "frontend-flutter".to_string(),
                version: "1.0.0".to_string(),
                description: "Flutter モバイルアプリテンプレート".to_string(),
                author: "k1s0 Team".to_string(),
                tags: vec!["flutter".to_string(), "dart".to_string(), "mobile".to_string()],
                language: "dart".to_string(),
                service_type: "frontend".to_string(),
                created_at: "2024-02-01T00:00:00Z".to_string(),
                updated_at: "2024-06-01T00:00:00Z".to_string(),
                downloads: 750,
                dependencies: HashMap::new(),
                variables: vec![],
            },
        ];

        // フィルタ適用
        if let Some(filter) = filter {
            if let Some(lang) = &filter.language {
                templates.retain(|t| t.language == *lang);
            }
            if let Some(stype) = &filter.service_type {
                templates.retain(|t| t.service_type == *stype);
            }
            if let Some(query) = &filter.search {
                let query_lower = query.to_lowercase();
                templates.retain(|t| {
                    t.name.to_lowercase().contains(&query_lower)
                        || t.description.to_lowercase().contains(&query_lower)
                        || t.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower))
                });
            }
        }

        let total = templates.len();

        TemplateListResponse {
            templates,
            total,
            page: 1,
            page_size: 20,
        }
    }
}

/// テンプレートフィルター
#[derive(Debug, Clone, Default)]
pub struct TemplateFilter {
    /// 言語フィルタ
    pub language: Option<String>,
    /// サービスタイプフィルタ
    pub service_type: Option<String>,
    /// 検索クエリ
    pub search: Option<String>,
    /// タグフィルタ
    pub tags: Vec<String>,
}

impl TemplateFilter {
    /// 新しいフィルターを作成
    pub fn new() -> Self {
        Self::default()
    }

    /// 言語でフィルタ
    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    /// サービスタイプでフィルタ
    pub fn with_service_type(mut self, service_type: impl Into<String>) -> Self {
        self.service_type = Some(service_type.into());
        self
    }

    /// 検索クエリを設定
    pub fn with_search(mut self, query: impl Into<String>) -> Self {
        self.search = Some(query.into());
        self
    }

    /// キャッシュキーを生成
    fn cache_key(&self) -> String {
        let mut parts = Vec::new();

        if let Some(lang) = &self.language {
            parts.push(format!("lang_{}", lang));
        }
        if let Some(stype) = &self.service_type {
            parts.push(format!("type_{}", stype));
        }
        if let Some(query) = &self.search {
            parts.push(format!("q_{}", query));
        }

        if parts.is_empty() {
            "all".to_string()
        } else {
            parts.join("_")
        }
    }
}

/// ローカルテンプレートマネージャー
pub struct LocalTemplateManager {
    /// テンプレートディレクトリ
    templates_dir: PathBuf,
}

impl LocalTemplateManager {
    /// 新しいマネージャーを作成
    pub fn new(templates_dir: impl AsRef<Path>) -> Self {
        Self {
            templates_dir: templates_dir.as_ref().to_path_buf(),
        }
    }

    /// ローカルテンプレート一覧を取得
    pub fn list_local_templates(&self) -> RegistryResult<Vec<String>> {
        if !self.templates_dir.exists() {
            return Ok(Vec::new());
        }

        let mut templates = Vec::new();

        for entry in fs::read_dir(&self.templates_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    // manifest.json があるディレクトリをテンプレートとして認識
                    if path.join("manifest.json").exists() {
                        templates.push(name.to_string());
                    }
                }
            }
        }

        templates.sort();
        Ok(templates)
    }

    /// ローカルテンプレートのパスを取得
    pub fn get_template_path(&self, name: &str) -> Option<PathBuf> {
        let path = self.templates_dir.join(name);
        if path.exists() && path.join("manifest.json").exists() {
            Some(path)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_registry_config_default() {
        let config = RegistryConfig::default();

        assert_eq!(config.url, "https://registry.k1s0.dev");
        assert_eq!(config.timeout_secs, 30);
        assert_eq!(config.cache_ttl_secs, 3600);
    }

    #[test]
    fn test_template_filter_cache_key() {
        let filter = TemplateFilter::new()
            .with_language("rust")
            .with_service_type("backend");

        assert!(filter.cache_key().contains("lang_rust"));
        assert!(filter.cache_key().contains("type_backend"));
    }

    #[test]
    fn test_list_templates_mock() {
        let temp = tempdir().unwrap();
        let config = RegistryConfig {
            cache_dir: temp.path().to_path_buf(),
            ..Default::default()
        };

        let client = RegistryClient::new(config);
        let response = client.list_templates(None).unwrap();

        assert!(!response.templates.is_empty());
        assert!(response.templates.iter().any(|t| t.name == "backend-rust"));
    }

    #[test]
    fn test_list_templates_with_filter() {
        let temp = tempdir().unwrap();
        let config = RegistryConfig {
            cache_dir: temp.path().to_path_buf(),
            ..Default::default()
        };

        let client = RegistryClient::new(config);
        let filter = TemplateFilter::new().with_language("rust");
        let response = client.list_templates(Some(&filter)).unwrap();

        assert!(response.templates.iter().all(|t| t.language == "rust"));
    }

    #[test]
    fn test_local_template_manager() {
        let temp = tempdir().unwrap();

        // テンプレートディレクトリを作成
        let template_dir = temp.path().join("backend-rust");
        fs::create_dir_all(&template_dir).unwrap();
        fs::write(template_dir.join("manifest.json"), "{}").unwrap();

        let manager = LocalTemplateManager::new(temp.path());
        let templates = manager.list_local_templates().unwrap();

        assert_eq!(templates, vec!["backend-rust"]);
    }

    #[test]
    fn test_cache_operations() {
        let temp = tempdir().unwrap();
        let config = RegistryConfig {
            cache_dir: temp.path().to_path_buf(),
            ..Default::default()
        };

        let client = RegistryClient::new(config.clone());

        // 一覧取得（キャッシュに保存される）
        let _ = client.list_templates(None).unwrap();

        // キャッシュディレクトリにファイルが存在することを確認
        let entries: Vec<_> = fs::read_dir(temp.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert!(!entries.is_empty(), "Cache directory should have files");

        // キャッシュクリア
        client.clear_cache().unwrap();
        assert!(!config.cache_dir.exists());
    }
}

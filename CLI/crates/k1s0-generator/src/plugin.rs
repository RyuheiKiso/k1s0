//! プラグインシステム
//!
//! カスタム lint ルールやテンプレート処理をプラグインとして追加する機能を提供する。
//!
//! # プラグインの種類
//!
//! - `LintPlugin`: カスタム lint ルールを追加
//! - `TemplatePlugin`: テンプレート処理をカスタマイズ
//! - `CommandPlugin`: カスタムコマンドを追加
//!
//! # 将来的な拡張
//!
//! - WASM プラグインのサポート
//! - 動的ライブラリ（.so/.dll）のサポート
//! - npm/cargo パッケージからのプラグイン読み込み

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::lint::{LintResult, Violation};

/// プラグインエラー
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    /// プラグインが見つからない
    #[error("Plugin not found: {0}")]
    NotFound(String),

    /// プラグインの読み込みに失敗
    #[error("Failed to load plugin: {0}")]
    LoadError(String),

    /// プラグインの実行に失敗
    #[error("Plugin execution failed: {0}")]
    ExecutionError(String),

    /// 無効なプラグイン
    #[error("Invalid plugin: {0}")]
    InvalidPlugin(String),

    /// IOエラー
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// プラグイン結果型
pub type PluginResult<T> = Result<T, PluginError>;

/// プラグインの種類
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PluginKind {
    /// Lint プラグイン
    Lint,
    /// テンプレートプラグイン
    Template,
    /// コマンドプラグイン
    Command,
}

/// プラグインメタデータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// プラグイン名
    pub name: String,
    /// バージョン
    pub version: String,
    /// 説明
    pub description: String,
    /// 種類
    pub kind: PluginKind,
    /// 作者
    #[serde(default)]
    pub author: String,
    /// ホームページ
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    /// ライセンス
    #[serde(default)]
    pub license: String,
}

/// Lint プラグイントレイト
pub trait LintPlugin: Send + Sync {
    /// プラグインメタデータを取得
    fn metadata(&self) -> &PluginMetadata;

    /// lint を実行
    fn lint(&self, path: &Path, config: &HashMap<String, serde_yaml::Value>) -> Vec<Violation>;

    /// 自動修正を実行（サポートしている場合）
    fn fix(&self, _path: &Path, _violation: &Violation) -> Option<String> {
        None
    }
}

/// テンプレートプラグイントレイト
pub trait TemplatePlugin: Send + Sync {
    /// プラグインメタデータを取得
    fn metadata(&self) -> &PluginMetadata;

    /// テンプレート変数を追加
    fn variables(&self) -> HashMap<String, String> {
        HashMap::new()
    }

    /// テンプレートフィルターを追加
    fn filters(&self) -> Vec<String> {
        Vec::new()
    }

    /// 後処理を実行
    fn post_process(&self, _output_dir: &Path) -> PluginResult<()> {
        Ok(())
    }
}

/// プラグインローダー
pub struct PluginLoader {
    /// プラグインディレクトリ
    plugin_dirs: Vec<PathBuf>,
    /// 読み込まれたプラグイン
    loaded_plugins: HashMap<String, Box<dyn LintPlugin>>,
}

impl PluginLoader {
    /// 新しいプラグインローダーを作成
    pub fn new() -> Self {
        Self {
            plugin_dirs: vec![],
            loaded_plugins: HashMap::new(),
        }
    }

    /// プラグインディレクトリを追加
    pub fn add_plugin_dir(&mut self, path: impl AsRef<Path>) {
        self.plugin_dirs.push(path.as_ref().to_path_buf());
    }

    /// 利用可能なプラグインを検索
    pub fn discover_plugins(&self) -> PluginResult<Vec<PluginMetadata>> {
        let mut plugins = Vec::new();

        for dir in &self.plugin_dirs {
            if !dir.exists() {
                continue;
            }

            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                // plugin.json を探す
                let metadata_path = path.join("plugin.json");
                if metadata_path.exists() {
                    if let Ok(content) = std::fs::read_to_string(&metadata_path) {
                        if let Ok(metadata) = serde_json::from_str::<PluginMetadata>(&content) {
                            plugins.push(metadata);
                        }
                    }
                }
            }
        }

        Ok(plugins)
    }

    /// プラグインを読み込む（将来的に WASM や dylib をサポート）
    pub fn load_plugin(&mut self, name: &str) -> PluginResult<()> {
        // 現在はスタブ実装
        // 将来的に:
        // - WASM: wasmtime/wasmer を使用
        // - dylib: libloading を使用
        // - JavaScript: deno_core を使用

        Err(PluginError::LoadError(format!(
            "Plugin loading not implemented yet: {}",
            name
        )))
    }

    /// lint プラグインを取得
    pub fn get_lint_plugin(&self, name: &str) -> Option<&dyn LintPlugin> {
        self.loaded_plugins.get(name).map(|p| p.as_ref())
    }

    /// すべての lint プラグインで lint を実行
    pub fn run_lint_plugins(
        &self,
        path: &Path,
        config: &HashMap<String, HashMap<String, serde_yaml::Value>>,
    ) -> LintResult {
        let mut result = LintResult::new(path.to_path_buf());

        for (name, plugin) in &self.loaded_plugins {
            let plugin_config = config.get(name).cloned().unwrap_or_default();
            let violations = plugin.lint(path, &plugin_config);

            for v in violations {
                result.add_violation(v);
            }
        }

        result
    }
}

impl Default for PluginLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// 組み込みプラグイン: サンプル
pub struct SampleLintPlugin {
    metadata: PluginMetadata,
}

impl SampleLintPlugin {
    /// 新しいサンプルプラグインを作成
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                name: "sample-lint".to_string(),
                version: "1.0.0".to_string(),
                description: "Sample lint plugin for demonstration".to_string(),
                kind: PluginKind::Lint,
                author: "k1s0 Team".to_string(),
                homepage: None,
                license: "MIT".to_string(),
            },
        }
    }
}

impl Default for SampleLintPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl LintPlugin for SampleLintPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn lint(&self, _path: &Path, _config: &HashMap<String, serde_yaml::Value>) -> Vec<Violation> {
        // サンプル: 何も検出しない
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_loader_new() {
        let loader = PluginLoader::new();
        assert!(loader.plugin_dirs.is_empty());
        assert!(loader.loaded_plugins.is_empty());
    }

    #[test]
    fn test_sample_lint_plugin() {
        let plugin = SampleLintPlugin::new();
        let metadata = plugin.metadata();

        assert_eq!(metadata.name, "sample-lint");
        assert_eq!(metadata.kind, PluginKind::Lint);

        let violations = plugin.lint(Path::new("."), &HashMap::new());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_plugin_metadata_serialization() {
        let metadata = PluginMetadata {
            name: "test-plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "Test plugin".to_string(),
            kind: PluginKind::Lint,
            author: "Test".to_string(),
            homepage: None,
            license: "MIT".to_string(),
        };

        let json = serde_json::to_string(&metadata).unwrap();
        let parsed: PluginMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.name, metadata.name);
        assert_eq!(parsed.kind, PluginKind::Lint);
    }
}

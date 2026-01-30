//! テンプレートのレンダリング
//!
//! Tera を使用したテンプレート展開を提供する。

use std::path::{Path, PathBuf};

use tera::{Context, Tera};
use walkdir::WalkDir;

use crate::fs::{write_file, WriteResult};
use crate::progress::{NoopProgress, ProgressCallback};
use crate::Result;

/// テンプレートファイルの拡張子
const TEMPLATE_EXTENSION: &str = ".tera";

/// テンプレートレンダラー
pub struct TemplateRenderer {
    /// Tera テンプレートエンジン
    tera: Tera,
    /// テンプレートディレクトリ
    template_dir: PathBuf,
}

/// 展開結果
#[derive(Debug, Clone)]
pub struct RenderResult {
    /// 生成されたファイル
    pub created_files: Vec<String>,
    /// スキップされたファイル（既に同一内容）
    pub skipped_files: Vec<String>,
    /// 上書きされたファイル
    pub overwritten_files: Vec<String>,
}

/// プレビュー結果（実際のファイル書き込みなし）
#[derive(Debug, Clone)]
pub struct PreviewResult {
    /// 生成されるファイル一覧
    pub files: Vec<String>,
    /// 生成されるディレクトリ数
    pub directory_count: usize,
}

impl TemplateRenderer {
    /// 新しいレンダラーを作成する
    pub fn new<P: AsRef<Path>>(template_dir: P) -> Result<Self> {
        let template_dir = template_dir.as_ref().to_path_buf();
        let pattern = format!("{}/**/*.tera", template_dir.display());
        let tera = Tera::new(&pattern)?;

        Ok(Self { tera, template_dir })
    }

    /// テンプレートをレンダリングする
    pub fn render(&self, template_name: &str, context: &Context) -> Result<String> {
        let result = self.tera.render(template_name, context)?;
        Ok(result)
    }

    /// テンプレートディレクトリを展開する
    ///
    /// - `.tera` 拡張子のファイルは Tera でレンダリング後、拡張子を除去して出力
    /// - その他のファイルはそのままコピー
    /// - ディレクトリ構造を維持
    pub fn render_directory<P: AsRef<Path>>(
        &self,
        output_dir: P,
        context: &Context,
    ) -> Result<RenderResult> {
        self.render_directory_with_progress(output_dir, context, &NoopProgress)
    }

    /// テンプレートディレクトリのプレビュー（書き込みなし）
    ///
    /// 生成されるファイル一覧とディレクトリ数を返す。
    pub fn preview_directory(&self) -> Result<PreviewResult> {
        let mut files = Vec::new();
        let mut dirs = std::collections::HashSet::new();

        for entry in WalkDir::new(&self.template_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let src_path = entry.path();
            let relative_path = src_path
                .strip_prefix(&self.template_dir)
                .unwrap_or(src_path);

            // .tera 拡張子を除去
            let display_path = if src_path.to_string_lossy().ends_with(TEMPLATE_EXTENSION) {
                let path_str = relative_path.to_string_lossy();
                path_str.trim_end_matches(TEMPLATE_EXTENSION).to_string()
            } else {
                relative_path.to_string_lossy().replace('\\', "/")
            };

            // ディレクトリをカウント
            if let Some(parent) = std::path::Path::new(&display_path).parent() {
                let parent_str = parent.to_string_lossy().replace('\\', "/");
                if !parent_str.is_empty() {
                    dirs.insert(parent_str);
                }
            }

            files.push(display_path);
        }

        Ok(PreviewResult {
            files,
            directory_count: dirs.len(),
        })
    }

    /// 進捗コールバック付きでテンプレートディレクトリを展開する
    pub fn render_directory_with_progress<P: AsRef<Path>>(
        &self,
        output_dir: P,
        context: &Context,
        progress: &dyn ProgressCallback,
    ) -> Result<RenderResult> {
        let output_dir = output_dir.as_ref();
        let mut result = RenderResult {
            created_files: Vec::new(),
            skipped_files: Vec::new(),
            overwritten_files: Vec::new(),
        };

        let entries: Vec<_> = WalkDir::new(&self.template_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .collect();

        progress.on_total(entries.len());

        for entry in entries {
            let src_path = entry.path();
            let relative_path = src_path
                .strip_prefix(&self.template_dir)
                .unwrap_or(src_path);

            let output_path = if src_path.to_string_lossy().ends_with(TEMPLATE_EXTENSION) {
                let path_str = relative_path.to_string_lossy();
                let without_tera = path_str.trim_end_matches(TEMPLATE_EXTENSION);
                output_dir.join(without_tera)
            } else {
                output_dir.join(relative_path)
            };

            let write_result = if src_path.to_string_lossy().ends_with(TEMPLATE_EXTENSION) {
                let template_name = relative_path.to_string_lossy().replace('\\', "/");
                let content = self.render(&template_name, context)?;
                write_file(&output_path, &content)?
            } else {
                let content = std::fs::read_to_string(src_path)?;
                write_file(&output_path, &content)?
            };

            let relative_output = output_path
                .strip_prefix(output_dir)
                .unwrap_or(&output_path)
                .to_string_lossy()
                .replace('\\', "/");

            progress.on_file_done(&relative_output);

            match write_result {
                WriteResult::Created => result.created_files.push(relative_output),
                WriteResult::Skipped => result.skipped_files.push(relative_output),
                WriteResult::Overwritten => result.overwritten_files.push(relative_output),
            }
        }

        Ok(result)
    }

    /// 利用可能なテンプレート一覧を取得
    pub fn list_templates(&self) -> Vec<String> {
        self.tera
            .get_template_names()
            .map(|s| s.to_string())
            .collect()
    }
}

/// テンプレート用のコンテキストを作成する
pub fn create_context(
    service_name: &str,
    language: &str,
    service_type: &str,
    k1s0_version: &str,
) -> Context {
    let mut context = Context::new();
    context.insert("feature_name", service_name);
    context.insert("service_name", service_name);
    context.insert("language", language);
    context.insert("service_type", service_type);
    context.insert("k1s0_version", k1s0_version);

    // 命名規則の変換
    context.insert("feature_name_snake", &service_name.replace('-', "_"));
    context.insert(
        "feature_name_pascal",
        &to_pascal_case(service_name),
    );

    context
}

/// kebab-case を PascalCase に変換する
fn to_pascal_case(s: &str) -> String {
    s.split('-')
        .map(|word| {
            let mut chars: Vec<char> = word.chars().collect();
            if let Some(first) = chars.first_mut() {
                *first = first.to_ascii_uppercase();
            }
            chars.into_iter().collect::<String>()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("user-management"), "UserManagement");
        assert_eq!(to_pascal_case("order"), "Order");
        assert_eq!(to_pascal_case("auth-service"), "AuthService");
    }

    #[test]
    fn test_preview_directory() {
        // tempdir にテンプレートファイルを作成してプレビュー
        let temp_dir = tempfile::tempdir().unwrap();
        let template_dir = temp_dir.path();

        // テンプレートファイルを作成
        std::fs::create_dir_all(template_dir.join("src")).unwrap();
        std::fs::write(template_dir.join("Cargo.toml.tera"), "name = \"{{ name }}\"").unwrap();
        std::fs::write(template_dir.join("src/main.rs.tera"), "fn main() {}").unwrap();
        std::fs::write(template_dir.join("README.md"), "# README").unwrap();

        let renderer = TemplateRenderer::new(template_dir).unwrap();
        let preview = renderer.preview_directory().unwrap();

        assert_eq!(preview.files.len(), 3);
        assert!(preview.directory_count >= 1); // src/
    }
}

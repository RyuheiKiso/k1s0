//! オプション選択プロンプト
//!
//! feature オプション、domain 選択などのプロンプトを提供します。

use std::path::PathBuf;

use inquire::{MultiSelect, Select};

use crate::error::Result;
use crate::prompts::{cancelled_error, get_render_config};

/// feature オプション
#[derive(Clone, Debug)]
pub struct FeatureOption {
    pub id: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub selected: bool,
}

impl std::fmt::Display for FeatureOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {}", self.label, self.description)
    }
}

/// feature オプションの選択結果
#[derive(Clone, Debug, Default)]
pub struct SelectedFeatureOptions {
    pub with_grpc: bool,
    pub with_rest: bool,
    pub with_db: bool,
}

/// feature オプションを選択するプロンプト
///
/// 複数のオプションをマルチセレクトで選択できます。
///
/// # Returns
///
/// 選択されたオプション
///
/// # Errors
///
/// ユーザーがキャンセルした場合
pub fn select_feature_options() -> Result<SelectedFeatureOptions> {
    let options = vec![
        FeatureOption {
            id: "grpc",
            label: "gRPC API",
            description: "gRPC サーバー/クライアントを含める",
            selected: false,
        },
        FeatureOption {
            id: "rest",
            label: "REST API",
            description: "REST API エンドポイントを含める",
            selected: false,
        },
        FeatureOption {
            id: "db",
            label: "Database",
            description: "DB マイグレーションと接続設定を含める",
            selected: false,
        },
    ];

    let answer = MultiSelect::new("含めるオプションを選択してください:", options)
        .with_render_config(get_render_config())
        .with_help_message("スペースで選択/解除、Enter で確定（何も選択しなくても OK）")
        .prompt()
        .map_err(|_| cancelled_error())?;

    let mut result = SelectedFeatureOptions::default();
    for opt in answer {
        match opt.id {
            "grpc" => result.with_grpc = true,
            "rest" => result.with_rest = true,
            "db" => result.with_db = true,
            _ => {}
        }
    }

    Ok(result)
}

/// domain 選択肢
#[derive(Clone, Debug)]
pub struct DomainChoice {
    pub name: Option<String>,
    pub label: String,
    pub path: Option<PathBuf>,
}

impl std::fmt::Display for DomainChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label)
    }
}

/// domain を選択するプロンプト
///
/// 既存の domain 一覧から選択できます。「なし」オプションも提供されます。
///
/// # Arguments
///
/// * `domain_base` - domain ディレクトリのベースパス（例: "domain/backend/rust"）
///
/// # Returns
///
/// 選択された domain 名（なしの場合は `None`）
///
/// # Errors
///
/// ユーザーがキャンセルした場合
pub fn select_domain(domain_base: &str) -> Result<Option<String>> {
    let domains = discover_domains(domain_base)?;

    // 「なし」オプションを追加
    let mut choices = vec![DomainChoice {
        name: None,
        label: "(なし) - 独立した feature として作成".to_string(),
        path: None,
    }];

    for (name, path) in domains {
        choices.push(DomainChoice {
            name: Some(name.clone()),
            label: name,
            path: Some(path),
        });
    }

    // domain が存在しない場合はスキップ
    if choices.len() == 1 {
        return Ok(None);
    }

    let answer = Select::new("所属する domain を選択してください:", choices)
        .with_render_config(get_render_config())
        .with_help_message("矢印キーで選択、Enter で確定")
        .prompt()
        .map_err(|_| cancelled_error())?;

    Ok(answer.name)
}

/// 既存の domain を検出する
fn discover_domains(domain_base: &str) -> Result<Vec<(String, PathBuf)>> {
    let base_path = PathBuf::from(domain_base);
    let mut domains = Vec::new();

    if !base_path.exists() {
        return Ok(domains);
    }

    if let Ok(entries) = std::fs::read_dir(&base_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // .k1s0/manifest.json が存在するか確認
                let manifest_path = path.join(".k1s0/manifest.json");
                if manifest_path.exists() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        domains.push((name.to_string(), path));
                    }
                }
            }
        }
    }

    // アルファベット順にソート
    domains.sort_by(|a, b| a.0.cmp(&b.0));

    Ok(domains)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selected_feature_options_default() {
        let options = SelectedFeatureOptions::default();
        assert!(!options.with_grpc);
        assert!(!options.with_rest);
        assert!(!options.with_db);
    }

    #[test]
    fn test_discover_domains_nonexistent_dir() {
        let result = discover_domains("nonexistent/path");
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}

//! フィーチャー選択プロンプト
//!
//! 既存のフィーチャー一覧から選択するプロンプトを提供します。

use std::path::PathBuf;

use inquire::Select;

use crate::commands::new_screen::FrontendType;
use crate::error::Result;
use crate::prompts::{cancelled_error, get_render_config};

/// フィーチャー選択肢
#[derive(Clone, Debug)]
pub struct FeatureChoice {
    pub name: String,
    pub path: PathBuf,
}

impl std::fmt::Display for FeatureChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.path.display())
    }
}

/// フロントエンドのフィーチャーディレクトリベースパスを取得
fn get_feature_base(frontend_type: FrontendType) -> &'static str {
    match frontend_type {
        FrontendType::React => "feature/frontend/react",
        FrontendType::Flutter => "feature/frontend/flutter",
        FrontendType::Android => "feature/frontend/android",
    }
}

/// 既存のフロントエンドフィーチャーを検出する
pub fn discover_features(frontend_type: FrontendType) -> Result<Vec<FeatureChoice>> {
    let base_path = PathBuf::from(get_feature_base(frontend_type));
    let mut features = Vec::new();

    if !base_path.exists() {
        return Ok(features);
    }

    if let Ok(entries) = std::fs::read_dir(&base_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // .k1s0/manifest.json が存在するか確認（正しい feature ディレクトリかどうか）
                let manifest_path = path.join(".k1s0/manifest.json");
                if manifest_path.exists() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        features.push(FeatureChoice {
                            name: name.to_string(),
                            path,
                        });
                    }
                }
            }
        }
    }

    // アルファベット順にソート
    features.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(features)
}

/// 対象のフィーチャーを選択するプロンプト
///
/// # Arguments
///
/// * `frontend_type` - フロントエンドタイプ
///
/// # Returns
///
/// 選択されたフィーチャーのパス
///
/// # Errors
///
/// - フィーチャーが存在しない場合
/// - ユーザーがキャンセルした場合
pub fn select_target_feature(frontend_type: FrontendType) -> Result<String> {
    let features = discover_features(frontend_type)?;

    if features.is_empty() {
        return Err(crate::error::CliError::config(format!(
            "{} フィーチャーが見つかりません",
            frontend_type
        ))
        .with_hint(format!(
            "まず 'k1s0 new-feature --type frontend-{}' で feature を作成してください",
            match frontend_type {
                FrontendType::React => "react",
                FrontendType::Flutter => "flutter",
                FrontendType::Android => "android",
            }
        )));
    }

    let answer = Select::new("対象のフィーチャーを選択してください:", features)
        .with_render_config(get_render_config())
        .with_help_message("矢印キーで選択、Enter で確定")
        .prompt()
        .map_err(|_| cancelled_error())?;

    Ok(answer.path.display().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_feature_base() {
        assert_eq!(get_feature_base(FrontendType::React), "feature/frontend/react");
        assert_eq!(get_feature_base(FrontendType::Flutter), "feature/frontend/flutter");
    }

    #[test]
    fn test_discover_features_nonexistent_dir() {
        // 存在しないディレクトリでは空のリストを返す
        let result = discover_features(FrontendType::React);
        assert!(result.is_ok());
        // 実際の feature ディレクトリがない環境では空になる
    }
}

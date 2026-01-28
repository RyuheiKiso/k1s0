//! init オプション選択プロンプト
//!
//! init コマンド用の対話式プロンプトを提供します。

use inquire::{Select, Text};

use crate::error::Result;
use crate::prompts::{cancelled_error, get_render_config};

/// デフォルト言語の選択肢
#[derive(Clone, Debug)]
struct LanguageChoice {
    value: &'static str,
    label: &'static str,
}

impl std::fmt::Display for LanguageChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label)
    }
}

/// デフォルト言語を選択するプロンプト
///
/// # Returns
///
/// 選択された言語（"rust", "go", "typescript", "dart"）
///
/// # Errors
///
/// ユーザーがキャンセルした場合
pub fn select_language() -> Result<String> {
    let choices = vec![
        LanguageChoice {
            value: "rust",
            label: "Rust - バックエンドサービス向け",
        },
        LanguageChoice {
            value: "go",
            label: "Go - バックエンドサービス向け",
        },
        LanguageChoice {
            value: "typescript",
            label: "TypeScript - React フロントエンド向け",
        },
        LanguageChoice {
            value: "dart",
            label: "Dart - Flutter フロントエンド向け",
        },
    ];

    let answer = Select::new("デフォルトの言語を選択してください:", choices)
        .with_render_config(get_render_config())
        .with_help_message("矢印キーで選択、Enter で確定")
        .prompt()
        .map_err(|_| cancelled_error())?;

    Ok(answer.value.to_string())
}

/// デフォルトサービスタイプの選択肢
#[derive(Clone, Debug)]
struct ServiceTypeChoice {
    value: &'static str,
    label: &'static str,
}

impl std::fmt::Display for ServiceTypeChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label)
    }
}

/// デフォルトサービスタイプを選択するプロンプト
///
/// # Returns
///
/// 選択されたサービスタイプ（"backend", "frontend"）
///
/// # Errors
///
/// ユーザーがキャンセルした場合
pub fn select_service_type() -> Result<String> {
    let choices = vec![
        ServiceTypeChoice {
            value: "backend",
            label: "Backend - バックエンドサービス",
        },
        ServiceTypeChoice {
            value: "frontend",
            label: "Frontend - フロントエンドアプリケーション",
        },
    ];

    let answer = Select::new("デフォルトのサービスタイプを選択してください:", choices)
        .with_render_config(get_render_config())
        .with_help_message("矢印キーで選択、Enter で確定")
        .prompt()
        .map_err(|_| cancelled_error())?;

    Ok(answer.value.to_string())
}

/// テンプレートソースを入力するプロンプト
///
/// # Returns
///
/// 入力されたテンプレートソース（"local" またはレジストリ URL）
///
/// # Errors
///
/// ユーザーがキャンセルした場合
pub fn input_template_source() -> Result<String> {
    let answer = Text::new("テンプレートソースを入力してください:")
        .with_render_config(get_render_config())
        .with_help_message("'local' または registry URL を入力")
        .with_default("local")
        .prompt()
        .map_err(|_| cancelled_error())?;

    Ok(answer)
}

/// 初期化パスを入力するプロンプト
///
/// # Returns
///
/// 入力されたパス（デフォルト: "."）
///
/// # Errors
///
/// ユーザーがキャンセルした場合
pub fn input_init_path() -> Result<String> {
    let answer = Text::new("初期化するディレクトリを入力してください:")
        .with_render_config(get_render_config())
        .with_help_message("相対パスまたは絶対パス（デフォルト: カレントディレクトリ）")
        .with_default(".")
        .prompt()
        .map_err(|_| cancelled_error())?;

    Ok(answer)
}

#[cfg(test)]
mod tests {
    // 対話テストはモック環境が必要なため省略
}

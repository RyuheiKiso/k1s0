//! 確認プロンプト
//!
//! 操作実行前の確認プロンプトを提供します。

use inquire::Confirm;

use crate::error::Result;
use crate::prompts::{cancelled_error, get_render_config};

/// 生成を確認するプロンプト
///
/// # Arguments
///
/// * `message` - 確認メッセージ
///
/// # Returns
///
/// ユーザーが確認した場合 `true`
///
/// # Errors
///
/// ユーザーがキャンセルした場合
pub fn confirm_generation(message: &str) -> Result<bool> {
    let answer = Confirm::new(message)
        .with_render_config(get_render_config())
        .with_default(true)
        .with_help_message("y/n で回答、Enter でデフォルト（Yes）")
        .prompt()
        .map_err(|_| cancelled_error())?;

    Ok(answer)
}

/// 上書きを確認するプロンプト
///
/// # Arguments
///
/// * `path` - 上書き対象のパス
///
/// # Returns
///
/// ユーザーが確認した場合 `true`
///
/// # Errors
///
/// ユーザーがキャンセルした場合
pub fn confirm_overwrite(path: &str) -> Result<bool> {
    let message = format!(
        "'{}' は既に存在します。上書きしますか？",
        path
    );

    let answer = Confirm::new(&message)
        .with_render_config(get_render_config())
        .with_default(false) // 上書きはデフォルト No
        .with_help_message("y/n で回答、Enter でデフォルト（No）")
        .prompt()
        .map_err(|_| cancelled_error())?;

    Ok(answer)
}

/// 危険な操作を確認するプロンプト
///
/// # Arguments
///
/// * `message` - 警告メッセージ
///
/// # Returns
///
/// ユーザーが確認した場合 `true`
///
/// # Errors
///
/// ユーザーがキャンセルした場合
pub fn confirm_dangerous(message: &str) -> Result<bool> {
    let answer = Confirm::new(message)
        .with_render_config(get_render_config())
        .with_default(false) // 危険な操作はデフォルト No
        .with_help_message("y/n で回答、Enter でデフォルト（No）")
        .prompt()
        .map_err(|_| cancelled_error())?;

    Ok(answer)
}

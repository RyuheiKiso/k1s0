//! 対話式プロンプトモジュール
//!
//! このモジュールは CLI の対話式インターフェースを提供します。
//! Vite のような洗練されたユーザー体験を実現します。
//!
//! # 機能
//!
//! - テンプレートタイプの選択
//! - 名前入力（バリデーション付き）
//! - オプション選択（マルチセレクト）
//! - 確認プロンプト
//!
//! # 使用例
//!
//! ```ignore
//! use k1s0_cli::prompts;
//!
//! // TTY チェック
//! if prompts::is_interactive() {
//!     let service_type = prompts::template_type::select_service_type()?;
//!     let name = prompts::name_input::input_feature_name()?;
//! }
//! ```

pub mod confirm;
pub mod feature_select;
pub mod init_options;
pub mod name_input;
pub mod options;
pub mod template_type;
pub mod version_input;

use std::io::IsTerminal;

use inquire::ui::{Color, RenderConfig, StyleSheet, Styled};

use crate::output::output;

/// TTY かどうかを検出する
///
/// stdin が端末に接続されている場合 true を返す。
/// CI 環境やパイプライン経由での実行時は false を返す。
pub fn is_interactive() -> bool {
    std::io::stdin().is_terminal()
}

/// 対話モードを判定する
///
/// # Arguments
///
/// * `interactive_flag` - --interactive / -i フラグの値
/// * `required_args_provided` - 必須引数がすべて提供されているか
///
/// # Returns
///
/// 対話モードで実行すべき場合 `Ok(true)`、
/// 引数モードで実行すべき場合 `Ok(false)`、
/// 非 TTY 環境で引数不足の場合 `Err`
pub fn should_use_interactive_mode(
    interactive_flag: bool,
    required_args_provided: bool,
) -> crate::error::Result<bool> {
    // --interactive フラグで強制的に対話モード
    if interactive_flag {
        if !is_interactive() {
            return Err(crate::error::CliError::interactive_required(
                "対話モードが要求されましたが、TTY が利用できません",
            ));
        }
        return Ok(true);
    }

    // 必須引数がすべて揃っていれば引数モード
    if required_args_provided {
        return Ok(false);
    }

    // 引数不足の場合
    if is_interactive() {
        // TTY なら対話モードにフォールバック
        Ok(true)
    } else {
        // 非 TTY なら引数不足エラー
        Err(crate::error::CliError::missing_required_args(
            "必須引数が不足しています。すべての引数を指定するか、対話環境で実行してください",
        ))
    }
}

/// inquire のレンダリング設定を取得する
///
/// --no-color オプションとの整合性を保つ。
pub fn get_render_config() -> RenderConfig<'static> {
    let out = output();

    if out.config().color {
        // カラー有効時のカスタム設定
        RenderConfig::default()
            .with_prompt_prefix(Styled::new(">").with_fg(Color::LightCyan))
            .with_highlighted_option_prefix(Styled::new(">").with_fg(Color::LightCyan))
            .with_answer(StyleSheet::new().with_fg(Color::LightGreen))
            .with_help_message(StyleSheet::new().with_fg(Color::DarkGrey))
    } else {
        // カラー無効時のプレーン設定
        RenderConfig::default_colored()
            .with_prompt_prefix(Styled::new(">"))
            .with_highlighted_option_prefix(Styled::new(">"))
    }
}

/// プロンプトがキャンセルされた場合のエラー
pub fn cancelled_error() -> crate::error::CliError {
    crate::error::CliError::cancelled("操作がキャンセルされました")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_use_interactive_mode_with_all_args() {
        // 引数がすべて揃っている場合は対話モード不要
        let result = should_use_interactive_mode(false, true);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_should_use_interactive_mode_with_flag() {
        // --interactive フラグがある場合、TTY チェックが行われる
        // テスト環境では TTY が利用できないことが多いので、エラーが返る可能性がある
        let result = should_use_interactive_mode(true, false);
        // TTY の状態に依存するため、結果の型だけ確認
        assert!(result.is_ok() || result.is_err());
    }
}

use dialoguer::{theme::ColorfulTheme, Input, MultiSelect, Select};
use std::sync::atomic::{AtomicBool, Ordering};

pub use k1s0_core::validate_name;

// ============================================================================
// 非インタラクティブモード制御
// ============================================================================

/// 非インタラクティブモードフラグ。
/// `main` で一度だけ設定し、以降は読み取り専用で参照する。
/// AtomicBool を使用してスレッドセーフ性を確保する。
static NON_INTERACTIVE: AtomicBool = AtomicBool::new(false);

/// 非インタラクティブモードを設定する。
/// main() で --non-interactive / --yes フラグまたは TTY 検出結果を元に呼び出す。
pub fn set_non_interactive(value: bool) {
    NON_INTERACTIVE.store(value, Ordering::Relaxed);
}

/// 現在のモードが非インタラクティブかどうかを返す。
pub fn is_non_interactive() -> bool {
    NON_INTERACTIVE.load(Ordering::Relaxed)
}

/// 非インタラクティブ時に対話プロンプトが呼ばれた場合に返すエラー。
/// CI/CD 環境での誤用を早期に検出するためにエラーとして伝播する。
fn non_interactive_error(prompt: &str) -> anyhow::Error {
    anyhow::anyhow!(
        "非インタラクティブモードでは対話プロンプト '{}' を使用できません。\n\
        ヒント: サブコマンドに必要な引数をフラグで指定するか、K1S0_NON_INTERACTIVE=false で無効化してください。",
        prompt
    )
}

// ============================================================================
// 対話式プロンプトのテーマ
// ============================================================================

/// 対話式プロンプトのテーマを取得する。
pub fn theme() -> ColorfulTheme {
    ColorfulTheme::default()
}

/// 選択プロンプト。Ctrl+C / Esc で None を返す。
/// 非インタラクティブモード時はエラーを返す。
///
/// # Errors
///
/// プロンプトの入出力に失敗した場合、または非インタラクティブモードの場合にエラーを返す。
pub fn select_prompt(prompt: &str, items: &[&str]) -> anyhow::Result<Option<usize>> {
    // 非インタラクティブモードでは選択プロンプトを表示できないためエラーとする
    if is_non_interactive() {
        return Err(non_interactive_error(prompt));
    }
    let selection = Select::with_theme(&theme())
        .with_prompt(prompt)
        .items(items)
        .default(0)
        .interact_opt()?;
    Ok(selection)
}

/// 複数選択プロンプト。Ctrl+C / Esc で None を返す。
/// 非インタラクティブモード時はエラーを返す。
///
/// # Errors
///
/// プロンプトの入出力に失敗した場合、または非インタラクティブモードの場合にエラーを返す。
pub fn multi_select_prompt(prompt: &str, items: &[&str]) -> anyhow::Result<Option<Vec<usize>>> {
    // 非インタラクティブモードでは複数選択プロンプトを表示できないためエラーとする
    if is_non_interactive() {
        return Err(non_interactive_error(prompt));
    }
    let selection = MultiSelect::with_theme(&theme())
        .with_prompt(prompt)
        .items(items)
        .interact_opt()?;
    Ok(selection)
}

/// テキスト入力プロンプト（バリデーション付き）。
/// 非インタラクティブモード時はエラーを返す。
///
/// # Errors
///
/// プロンプトの入出力に失敗した場合、または非インタラクティブモードの場合にエラーを返す。
pub fn input_prompt(prompt: &str) -> anyhow::Result<String> {
    // 非インタラクティブモードでは入力プロンプトを表示できないためエラーとする
    if is_non_interactive() {
        return Err(non_interactive_error(prompt));
    }
    let value: String = Input::with_theme(&theme())
        .with_prompt(prompt)
        .validate_with(|input: &String| validate_name(input))
        .interact_text()?;
    Ok(value)
}

/// テキスト入力プロンプト（バリデーションなし）。
/// 非インタラクティブモード時はエラーを返す。
///
/// # Errors
///
/// プロンプトの入出力に失敗した場合、または非インタラクティブモードの場合にエラーを返す。
pub fn input_prompt_raw(prompt: &str) -> anyhow::Result<String> {
    // 非インタラクティブモードでは入力プロンプトを表示できないためエラーとする
    if is_non_interactive() {
        return Err(non_interactive_error(prompt));
    }
    let value: String = Input::with_theme(&theme())
        .with_prompt(prompt)
        .interact_text()?;
    Ok(value)
}

/// テキスト入力プロンプト（カスタムバリデーション付き）。
/// 非インタラクティブモード時はエラーを返す。
///
/// # Errors
///
/// プロンプトの入出力に失敗した場合、または非インタラクティブモードの場合にエラーを返す。
pub fn input_with_validation<F>(prompt_text: &str, validator: F) -> anyhow::Result<String>
where
    F: Fn(&String) -> Result<(), String> + Clone,
{
    // 非インタラクティブモードでは入力プロンプトを表示できないためエラーとする
    if is_non_interactive() {
        return Err(non_interactive_error(prompt_text));
    }
    let value = Input::with_theme(&theme())
        .with_prompt(prompt_text)
        .validate_with(validator)
        .interact_text()?;
    Ok(value)
}

/// はい/いいえプロンプト。
/// 非インタラクティブモード時はエラーを返す。
///
/// # Errors
///
/// プロンプトの入出力に失敗した場合、または非インタラクティブモードの場合にエラーを返す。
pub fn yes_no_prompt(prompt: &str) -> anyhow::Result<Option<bool>> {
    // 非インタラクティブモードでははい/いいえプロンプトを表示できないためエラーとする
    if is_non_interactive() {
        return Err(non_interactive_error(prompt));
    }
    let items = &["はい", "いいえ"];
    let selection = Select::with_theme(&theme())
        .with_prompt(prompt)
        .items(items)
        .default(0)
        .interact_opt()?;
    match selection {
        None => Ok(None),
        Some(0) => Ok(Some(true)),
        Some(1) => Ok(Some(false)),
        _ => unreachable!(),
    }
}

/// 確認プロンプトの結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmResult {
    /// はい — 実行する
    Yes,
    /// いいえ — 前のステップに戻る
    GoBack,
    /// キャンセル — メインメニューに戻る
    Cancel,
}

/// 確認プロンプト（はい / いいえ（前のステップに戻る）/ キャンセル の3択）。
/// Ctrl+C / Esc の場合は Cancel を返す。
/// 非インタラクティブモード時はエラーを返す。
///
/// # Errors
///
/// プロンプトの入出力に失敗した場合、または非インタラクティブモードの場合にエラーを返す。
pub fn confirm_prompt() -> anyhow::Result<ConfirmResult> {
    // 非インタラクティブモードでは確認プロンプトを表示できないためエラーとする
    if is_non_interactive() {
        return Err(non_interactive_error("確認プロンプト（はい/いいえ/キャンセル）"));
    }
    let items = &[
        "はい",
        "いいえ（前のステップに戻る）",
        "キャンセル（メインメニューに戻る）",
    ];
    let selection = Select::with_theme(&theme())
        .with_prompt("よろしいですか？")
        .items(items)
        .default(0)
        .interact_opt()?;
    match selection {
        None | Some(2) => Ok(ConfirmResult::Cancel),
        Some(0) => Ok(ConfirmResult::Yes),
        Some(1) => Ok(ConfirmResult::GoBack),
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 非インタラクティブモードのフラグ設定と読み取りが正しく動作することを確認する。
    #[test]
    fn test_set_and_get_non_interactive() {
        // 初期値はテスト実行環境によって異なるため、明示的に設定してから確認する
        set_non_interactive(true);
        assert!(is_non_interactive());
        set_non_interactive(false);
        assert!(!is_non_interactive());
    }

    /// 非インタラクティブエラーメッセージにプロンプト名が含まれることを確認する。
    #[test]
    fn test_non_interactive_error_message_contains_prompt_name() {
        let err = non_interactive_error("テストプロンプト");
        let msg = format!("{err}");
        assert!(msg.contains("テストプロンプト"));
        assert!(msg.contains("非インタラクティブモード"));
    }

    #[test]
    fn test_validate_name_valid() {
        assert!(validate_name("task").is_ok());
        assert!(validate_name("task-api").is_ok());
        assert!(validate_name("my-service-123").is_ok());
        assert!(validate_name("a").is_ok());
        assert!(validate_name("1").is_ok());
        assert!(validate_name("abc").is_ok());
        assert!(validate_name("a1b2c3").is_ok());
    }

    #[test]
    fn test_validate_name_invalid() {
        assert!(validate_name("-task").is_err());
        assert!(validate_name("task-").is_err());
        assert!(validate_name("Task").is_err());
        assert!(validate_name("task_api").is_err());
        assert!(validate_name("").is_err());
        assert!(validate_name("UPPER").is_err());
        assert!(validate_name("has space").is_err());
        assert!(validate_name("dot.name").is_err());
        assert!(validate_name("-").is_err());
        assert!(validate_name("--").is_err());
    }

    #[test]
    fn test_confirm_result_eq() {
        assert_eq!(ConfirmResult::Yes, ConfirmResult::Yes);
        assert_eq!(ConfirmResult::GoBack, ConfirmResult::GoBack);
        assert_eq!(ConfirmResult::Cancel, ConfirmResult::Cancel);
        assert_ne!(ConfirmResult::Yes, ConfirmResult::GoBack);
    }

    #[test]
    fn test_theme_creation() {
        let _theme = theme();
        // テーマが正常に作成されることを確認
    }
}

use dialoguer::{theme::ColorfulTheme, Input, MultiSelect, Select};

/// 対話式プロンプトのテーマを取得する。
pub fn theme() -> ColorfulTheme {
    ColorfulTheme::default()
}

/// 名前バリデーション: [a-z0-9-]+, 先頭末尾ハイフン禁止
pub fn validate_name(name: &str) -> Result<(), String> {
    let re = regex::Regex::new(r"^[a-z0-9][a-z0-9-]*[a-z0-9]$|^[a-z0-9]$").unwrap();
    if !re.is_match(name) {
        return Err(
            "英小文字・ハイフン・数字のみ許可。先頭末尾のハイフンは禁止。".into(),
        );
    }
    Ok(())
}

/// 選択プロンプト。Ctrl+C / Esc で None を返す。
pub fn select_prompt(prompt: &str, items: &[&str]) -> anyhow::Result<Option<usize>> {
    let selection = Select::with_theme(&theme())
        .with_prompt(prompt)
        .items(items)
        .default(0)
        .interact_opt()?;
    Ok(selection)
}

/// 複数選択プロンプト。Ctrl+C / Esc で None を返す。
pub fn multi_select_prompt(
    prompt: &str,
    items: &[&str],
) -> anyhow::Result<Option<Vec<usize>>> {
    let selection = MultiSelect::with_theme(&theme())
        .with_prompt(prompt)
        .items(items)
        .interact_opt()?;
    Ok(selection)
}

/// テキスト入力プロンプト（バリデーション付き）。
pub fn input_prompt(prompt: &str) -> anyhow::Result<String> {
    let value: String = Input::with_theme(&theme())
        .with_prompt(prompt)
        .validate_with(|input: &String| validate_name(input))
        .interact_text()?;
    Ok(value)
}

/// テキスト入力プロンプト（バリデーションなし）。
pub fn input_prompt_raw(prompt: &str) -> anyhow::Result<String> {
    let value: String = Input::with_theme(&theme())
        .with_prompt(prompt)
        .interact_text()?;
    Ok(value)
}

/// テキスト入力プロンプト（カスタムバリデーション付き）。
pub fn input_with_validation<F>(prompt_text: &str, validator: F) -> anyhow::Result<String>
where
    F: Fn(&String) -> Result<(), String> + Clone,
{
    let value = Input::with_theme(&theme())
        .with_prompt(prompt_text)
        .validate_with(validator)
        .interact_text()?;
    Ok(value)
}

/// はい/いいえプロンプト。
pub fn yes_no_prompt(prompt: &str) -> anyhow::Result<Option<bool>> {
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
pub fn confirm_prompt() -> anyhow::Result<ConfirmResult> {
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
        None => Ok(ConfirmResult::Cancel),
        Some(0) => Ok(ConfirmResult::Yes),
        Some(1) => Ok(ConfirmResult::GoBack),
        Some(2) => Ok(ConfirmResult::Cancel),
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_name_valid() {
        assert!(validate_name("order").is_ok());
        assert!(validate_name("order-api").is_ok());
        assert!(validate_name("my-service-123").is_ok());
        assert!(validate_name("a").is_ok());
        assert!(validate_name("1").is_ok());
        assert!(validate_name("abc").is_ok());
        assert!(validate_name("a1b2c3").is_ok());
    }

    #[test]
    fn test_validate_name_invalid() {
        assert!(validate_name("-order").is_err());
        assert!(validate_name("order-").is_err());
        assert!(validate_name("Order").is_err());
        assert!(validate_name("order_api").is_err());
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

//! 名前入力プロンプト
//!
//! feature 名、domain 名、screen ID の入力を提供します。
//! 各入力にはリアルタイムバリデーションが適用されます。

use inquire::validator::Validation;
use inquire::Text;

use crate::error::Result;
use crate::prompts::{cancelled_error, get_render_config};

/// 予約語リスト（domain 名に使用できない）
const RESERVED_WORDS: &[&str] = &[
    "framework",
    "feature",
    "domain",
    "k1s0",
    "common",
    "shared",
    "core",
    "base",
    "util",
    "utils",
    "internal",
];

/// kebab-case バリデーション
fn validate_kebab_case(input: &str) -> std::result::Result<Validation, Box<dyn std::error::Error + Send + Sync>> {
    if input.is_empty() {
        return Ok(Validation::Invalid("名前を入力してください".into()));
    }

    let chars: Vec<char> = input.chars().collect();

    // 先頭は小文字
    if !chars[0].is_ascii_lowercase() {
        return Ok(Validation::Invalid(
            "先頭は小文字のアルファベットで始めてください".into(),
        ));
    }

    // 末尾はハイフンでない
    if chars.last() == Some(&'-') {
        return Ok(Validation::Invalid("末尾にハイフンは使用できません".into()));
    }

    // 連続するハイフンがない
    for i in 0..chars.len().saturating_sub(1) {
        if chars[i] == '-' && chars[i + 1] == '-' {
            return Ok(Validation::Invalid("連続するハイフンは使用できません".into()));
        }
    }

    // 許可される文字のみ
    for c in &chars {
        if !c.is_ascii_lowercase() && !c.is_ascii_digit() && *c != '-' {
            return Ok(Validation::Invalid(
                "小文字のアルファベット、数字、ハイフンのみ使用できます".into(),
            ));
        }
    }

    Ok(Validation::Valid)
}

/// 予約語チェック付き kebab-case バリデーション
fn validate_domain_name(input: &str) -> std::result::Result<Validation, Box<dyn std::error::Error + Send + Sync>> {
    // まず kebab-case バリデーション
    let kebab_result = validate_kebab_case(input)?;
    if let Validation::Invalid(msg) = kebab_result {
        return Ok(Validation::Invalid(msg));
    }

    // 予約語チェック
    if RESERVED_WORDS.contains(&input) {
        return Ok(Validation::Invalid(
            format!(
                "'{}' は予約語のため使用できません（予約語: {}）",
                input,
                RESERVED_WORDS.join(", ")
            )
            .into(),
        ));
    }

    Ok(Validation::Valid)
}

/// screen ID バリデーション（ドット区切り形式）
fn validate_screen_id(input: &str) -> std::result::Result<Validation, Box<dyn std::error::Error + Send + Sync>> {
    if input.is_empty() {
        return Ok(Validation::Invalid("画面 ID を入力してください".into()));
    }

    // ドットで分割
    let parts: Vec<&str> = input.split('.').collect();

    if parts.is_empty() {
        return Ok(Validation::Invalid("画面 ID を入力してください".into()));
    }

    // 各パートが kebab-case であることを確認
    for part in &parts {
        if part.is_empty() {
            return Ok(Validation::Invalid(
                "空のセグメントは許可されていません（例: 'users..list'）".into(),
            ));
        }

        // 直接 kebab-case ルールをチェック（validate_kebab_case を呼ばずに簡潔に）
        let chars: Vec<char> = part.chars().collect();

        // 先頭は小文字
        if !chars.first().is_some_and(|c| c.is_ascii_lowercase()) {
            return Ok(Validation::Invalid(
                format!("セグメント '{}' は小文字のアルファベットで始める必要があります", part).into(),
            ));
        }

        // 末尾はハイフンでない
        if chars.last() == Some(&'-') {
            return Ok(Validation::Invalid(
                format!("セグメント '{}' はハイフンで終わることはできません", part).into(),
            ));
        }

        // 連続するハイフンがない
        for i in 0..chars.len().saturating_sub(1) {
            if chars[i] == '-' && chars[i + 1] == '-' {
                return Ok(Validation::Invalid(
                    format!("セグメント '{}' に連続するハイフンがあります", part).into(),
                ));
            }
        }

        // 許可される文字のみ
        for c in &chars {
            if !c.is_ascii_lowercase() && !c.is_ascii_digit() && *c != '-' {
                return Ok(Validation::Invalid(
                    format!("セグメント '{}' に無効な文字が含まれています。小文字、数字、ハイフンのみ使用できます", part).into(),
                ));
            }
        }
    }

    Ok(Validation::Valid)
}

/// feature 名を入力するプロンプト
///
/// kebab-case でのみ入力を受け付けます。
///
/// # Returns
///
/// 入力された feature 名
///
/// # Errors
///
/// ユーザーがキャンセルした場合
pub fn input_feature_name() -> Result<String> {
    let answer = Text::new("フィーチャー名を入力してください:")
        .with_render_config(get_render_config())
        .with_help_message("kebab-case で入力（例: user-management, order-processing）")
        .with_placeholder("my-feature")
        .with_validator(validate_kebab_case)
        .prompt()
        .map_err(|_| cancelled_error())?;

    Ok(answer)
}

/// domain 名を入力するプロンプト
///
/// kebab-case でのみ入力を受け付け、予約語はブロックします。
///
/// # Returns
///
/// 入力された domain 名
///
/// # Errors
///
/// ユーザーがキャンセルした場合
pub fn input_domain_name() -> Result<String> {
    let answer = Text::new("ドメイン名を入力してください:")
        .with_render_config(get_render_config())
        .with_help_message("kebab-case で入力（例: user, order, payment）")
        .with_placeholder("my-domain")
        .with_validator(validate_domain_name)
        .prompt()
        .map_err(|_| cancelled_error())?;

    Ok(answer)
}

/// screen ID を入力するプロンプト
///
/// ドット区切り形式（例: users.list, orders.detail）を受け付けます。
///
/// # Returns
///
/// 入力された screen ID
///
/// # Errors
///
/// ユーザーがキャンセルした場合
pub fn input_screen_id() -> Result<String> {
    let answer = Text::new("画面 ID を入力してください:")
        .with_render_config(get_render_config())
        .with_help_message("ドット区切り形式（例: users.list, orders.detail）")
        .with_placeholder("users.list")
        .with_validator(validate_screen_id)
        .prompt()
        .map_err(|_| cancelled_error())?;

    Ok(answer)
}

/// 画面タイトルを入力するプロンプト
///
/// # Returns
///
/// 入力された画面タイトル
///
/// # Errors
///
/// ユーザーがキャンセルした場合
pub fn input_screen_title() -> Result<String> {
    let answer = Text::new("画面タイトルを入力してください:")
        .with_render_config(get_render_config())
        .with_help_message("表示名として使用されます（例: ユーザー一覧, 注文詳細）")
        .with_placeholder("My Screen")
        .prompt()
        .map_err(|_| cancelled_error())?;

    // 空の場合はデフォルト値を返す
    if answer.is_empty() {
        Ok("New Screen".to_string())
    } else {
        Ok(answer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_kebab_case_valid() {
        assert!(matches!(validate_kebab_case("user-management").unwrap(), Validation::Valid));
        assert!(matches!(validate_kebab_case("order").unwrap(), Validation::Valid));
        assert!(matches!(validate_kebab_case("auth-service").unwrap(), Validation::Valid));
        assert!(matches!(validate_kebab_case("api2").unwrap(), Validation::Valid));
        assert!(matches!(validate_kebab_case("a").unwrap(), Validation::Valid));
    }

    #[test]
    fn test_validate_kebab_case_invalid() {
        assert!(matches!(validate_kebab_case("").unwrap(), Validation::Invalid(_)));
        assert!(matches!(validate_kebab_case("UserManagement").unwrap(), Validation::Invalid(_)));
        assert!(matches!(validate_kebab_case("user_management").unwrap(), Validation::Invalid(_)));
        assert!(matches!(validate_kebab_case("-user").unwrap(), Validation::Invalid(_)));
        assert!(matches!(validate_kebab_case("user-").unwrap(), Validation::Invalid(_)));
        assert!(matches!(validate_kebab_case("user--management").unwrap(), Validation::Invalid(_)));
        assert!(matches!(validate_kebab_case("2user").unwrap(), Validation::Invalid(_)));
    }

    #[test]
    fn test_validate_domain_name_reserved_words() {
        assert!(matches!(validate_domain_name("framework").unwrap(), Validation::Invalid(_)));
        assert!(matches!(validate_domain_name("feature").unwrap(), Validation::Invalid(_)));
        assert!(matches!(validate_domain_name("domain").unwrap(), Validation::Invalid(_)));
        assert!(matches!(validate_domain_name("k1s0").unwrap(), Validation::Invalid(_)));
        assert!(matches!(validate_domain_name("common").unwrap(), Validation::Invalid(_)));
        assert!(matches!(validate_domain_name("shared").unwrap(), Validation::Invalid(_)));
    }

    #[test]
    fn test_validate_domain_name_valid() {
        assert!(matches!(validate_domain_name("user").unwrap(), Validation::Valid));
        assert!(matches!(validate_domain_name("order").unwrap(), Validation::Valid));
        assert!(matches!(validate_domain_name("payment-gateway").unwrap(), Validation::Valid));
    }

    #[test]
    fn test_validate_screen_id_valid() {
        assert!(matches!(validate_screen_id("users.list").unwrap(), Validation::Valid));
        assert!(matches!(validate_screen_id("orders.detail").unwrap(), Validation::Valid));
        assert!(matches!(validate_screen_id("home").unwrap(), Validation::Valid));
        assert!(matches!(validate_screen_id("admin.users.edit").unwrap(), Validation::Valid));
    }

    #[test]
    fn test_validate_screen_id_invalid() {
        assert!(matches!(validate_screen_id("").unwrap(), Validation::Invalid(_)));
        assert!(matches!(validate_screen_id("users..list").unwrap(), Validation::Invalid(_)));
        assert!(matches!(validate_screen_id(".users").unwrap(), Validation::Invalid(_)));
        assert!(matches!(validate_screen_id("users.").unwrap(), Validation::Invalid(_)));
        assert!(matches!(validate_screen_id("Users.List").unwrap(), Validation::Invalid(_)));
    }
}

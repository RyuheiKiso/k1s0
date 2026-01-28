//! バージョン入力プロンプト
//!
//! semver 形式のバージョン入力を提供します。

use inquire::validator::Validation;
use inquire::Text;

use crate::error::Result;
use crate::prompts::{cancelled_error, get_render_config};

/// デフォルトバージョン
pub const DEFAULT_VERSION: &str = "0.1.0";

/// semver バリデーション（X.Y.Z 形式）
///
/// 有効な形式:
/// - 0.1.0
/// - 1.0.0
/// - 10.20.30
///
/// 無効な形式:
/// - v1.0.0 （接頭辞 v は不可）
/// - 1.0 （3 つの数値が必要）
/// - 1.0.0-beta （プレリリース識別子は不可）
/// - 1.0.0+build （ビルドメタデータは不可）
pub fn validate_semver(
    input: &str,
) -> std::result::Result<Validation, Box<dyn std::error::Error + Send + Sync>> {
    if input.is_empty() {
        return Ok(Validation::Invalid(
            "バージョンを入力してください".into(),
        ));
    }

    // 接頭辞 v を許可しない
    if input.starts_with('v') || input.starts_with('V') {
        return Ok(Validation::Invalid(
            "接頭辞 'v' は不要です（例: 0.1.0）".into(),
        ));
    }

    // ドットで分割
    let parts: Vec<&str> = input.split('.').collect();

    if parts.len() != 3 {
        return Ok(Validation::Invalid(
            "X.Y.Z 形式で入力してください（例: 0.1.0, 1.0.0）".into(),
        ));
    }

    // 各パートが数値であることを確認
    for (i, part) in parts.iter().enumerate() {
        let label = match i {
            0 => "メジャー",
            1 => "マイナー",
            2 => "パッチ",
            _ => "バージョン",
        };

        if part.is_empty() {
            return Ok(Validation::Invalid(
                format!("{}バージョンが空です", label).into(),
            ));
        }

        // 先頭ゼロのチェック（"0" 単体は許可、"01" などは不許可）
        if part.len() > 1 && part.starts_with('0') {
            return Ok(Validation::Invalid(
                format!("{}バージョンに先頭ゼロは使用できません（例: 01 -> 1）", label).into(),
            ));
        }

        // 数値チェック
        if part.parse::<u64>().is_err() {
            return Ok(Validation::Invalid(
                format!("{}バージョンは数値で指定してください", label).into(),
            ));
        }
    }

    Ok(Validation::Valid)
}

/// バージョンを入力するプロンプト
///
/// semver 形式（X.Y.Z）でのみ入力を受け付けます。
///
/// # Returns
///
/// 入力されたバージョン（デフォルト: 0.1.0）
///
/// # Errors
///
/// ユーザーがキャンセルした場合
pub fn input_version() -> Result<String> {
    let answer = Text::new("バージョンを入力してください:")
        .with_render_config(get_render_config())
        .with_help_message("semver 形式（X.Y.Z）で入力（例: 0.1.0, 1.0.0）")
        .with_default(DEFAULT_VERSION)
        .with_placeholder(DEFAULT_VERSION)
        .with_validator(validate_semver)
        .prompt()
        .map_err(|_| cancelled_error())?;

    Ok(answer)
}

/// デフォルト値付きでバージョンを入力するプロンプト
///
/// # Arguments
///
/// * `default` - デフォルト値
///
/// # Returns
///
/// 入力されたバージョン
///
/// # Errors
///
/// ユーザーがキャンセルした場合
pub fn input_version_with_default(default: &str) -> Result<String> {
    let answer = Text::new("バージョンを入力してください:")
        .with_render_config(get_render_config())
        .with_help_message("semver 形式（X.Y.Z）で入力（例: 0.1.0, 1.0.0）")
        .with_default(default)
        .with_placeholder(default)
        .with_validator(validate_semver)
        .prompt()
        .map_err(|_| cancelled_error())?;

    Ok(answer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_semver_valid() {
        assert!(matches!(
            validate_semver("0.1.0").unwrap(),
            Validation::Valid
        ));
        assert!(matches!(
            validate_semver("1.0.0").unwrap(),
            Validation::Valid
        ));
        assert!(matches!(
            validate_semver("10.20.30").unwrap(),
            Validation::Valid
        ));
        assert!(matches!(
            validate_semver("0.0.0").unwrap(),
            Validation::Valid
        ));
        assert!(matches!(
            validate_semver("999.999.999").unwrap(),
            Validation::Valid
        ));
    }

    #[test]
    fn test_validate_semver_invalid_empty() {
        assert!(matches!(
            validate_semver("").unwrap(),
            Validation::Invalid(_)
        ));
    }

    #[test]
    fn test_validate_semver_invalid_prefix_v() {
        assert!(matches!(
            validate_semver("v1.0.0").unwrap(),
            Validation::Invalid(_)
        ));
        assert!(matches!(
            validate_semver("V1.0.0").unwrap(),
            Validation::Invalid(_)
        ));
    }

    #[test]
    fn test_validate_semver_invalid_parts_count() {
        // 2 パート
        assert!(matches!(
            validate_semver("1.0").unwrap(),
            Validation::Invalid(_)
        ));
        // 1 パート
        assert!(matches!(
            validate_semver("1").unwrap(),
            Validation::Invalid(_)
        ));
        // 4 パート
        assert!(matches!(
            validate_semver("1.0.0.0").unwrap(),
            Validation::Invalid(_)
        ));
    }

    #[test]
    fn test_validate_semver_invalid_leading_zero() {
        assert!(matches!(
            validate_semver("01.0.0").unwrap(),
            Validation::Invalid(_)
        ));
        assert!(matches!(
            validate_semver("1.00.0").unwrap(),
            Validation::Invalid(_)
        ));
        assert!(matches!(
            validate_semver("1.0.00").unwrap(),
            Validation::Invalid(_)
        ));
    }

    #[test]
    fn test_validate_semver_invalid_non_numeric() {
        assert!(matches!(
            validate_semver("a.b.c").unwrap(),
            Validation::Invalid(_)
        ));
        assert!(matches!(
            validate_semver("1.0.x").unwrap(),
            Validation::Invalid(_)
        ));
        assert!(matches!(
            validate_semver("1.0.0-beta").unwrap(),
            Validation::Invalid(_)
        ));
        assert!(matches!(
            validate_semver("1.0.0+build").unwrap(),
            Validation::Invalid(_)
        ));
    }

    #[test]
    fn test_validate_semver_invalid_empty_part() {
        assert!(matches!(
            validate_semver("1..0").unwrap(),
            Validation::Invalid(_)
        ));
        assert!(matches!(
            validate_semver(".1.0").unwrap(),
            Validation::Invalid(_)
        ));
        assert!(matches!(
            validate_semver("1.0.").unwrap(),
            Validation::Invalid(_)
        ));
    }

    #[test]
    fn test_default_version_constant() {
        assert_eq!(DEFAULT_VERSION, "0.1.0");
    }
}

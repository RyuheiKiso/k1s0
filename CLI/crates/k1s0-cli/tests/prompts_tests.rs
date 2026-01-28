//! プロンプトモジュールの単体テスト
//!
//! バリデーション関数など、TTY を必要としない部分のテストを行う。
//! 対話式プロンプト自体のテストは TTY が必要なため、このファイルでは
//! バリデーションロジックのみをテストする。

use inquire::validator::Validation;

// ============================================================
// version_input モジュールのテスト
// ============================================================

/// semver バリデーション関数を直接テスト用にインポートできないため、
/// 同等のロジックをテストヘルパーとして再実装
mod version_validation {
    use inquire::validator::Validation;

    /// semver バリデーション（テスト用再実装）
    pub fn validate_semver(
        input: &str,
    ) -> std::result::Result<Validation, Box<dyn std::error::Error + Send + Sync>> {
        if input.is_empty() {
            return Ok(Validation::Invalid(
                "バージョンを入力してください".into(),
            ));
        }

        if input.starts_with('v') || input.starts_with('V') {
            return Ok(Validation::Invalid(
                "接頭辞 'v' は不要です（例: 0.1.0）".into(),
            ));
        }

        let parts: Vec<&str> = input.split('.').collect();

        if parts.len() != 3 {
            return Ok(Validation::Invalid(
                "X.Y.Z 形式で入力してください（例: 0.1.0, 1.0.0）".into(),
            ));
        }

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

            if part.len() > 1 && part.starts_with('0') {
                return Ok(Validation::Invalid(
                    format!(
                        "{}バージョンに先頭ゼロは使用できません（例: 01 -> 1）",
                        label
                    )
                    .into(),
                ));
            }

            if part.parse::<u64>().is_err() {
                return Ok(Validation::Invalid(
                    format!("{}バージョンは数値で指定してください", label).into(),
                ));
            }
        }

        Ok(Validation::Valid)
    }
}

/// kebab-case バリデーション（テスト用再実装）
mod name_validation {
    use inquire::validator::Validation;

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

    pub fn validate_kebab_case(
        input: &str,
    ) -> std::result::Result<Validation, Box<dyn std::error::Error + Send + Sync>> {
        if input.is_empty() {
            return Ok(Validation::Invalid("名前を入力してください".into()));
        }

        let chars: Vec<char> = input.chars().collect();

        if !chars[0].is_ascii_lowercase() {
            return Ok(Validation::Invalid(
                "先頭は小文字のアルファベットで始めてください".into(),
            ));
        }

        if chars.last() == Some(&'-') {
            return Ok(Validation::Invalid(
                "末尾にハイフンは使用できません".into(),
            ));
        }

        for i in 0..chars.len().saturating_sub(1) {
            if chars[i] == '-' && chars[i + 1] == '-' {
                return Ok(Validation::Invalid(
                    "連続するハイフンは使用できません".into(),
                ));
            }
        }

        for c in &chars {
            if !c.is_ascii_lowercase() && !c.is_ascii_digit() && *c != '-' {
                return Ok(Validation::Invalid(
                    "小文字のアルファベット、数字、ハイフンのみ使用できます".into(),
                ));
            }
        }

        Ok(Validation::Valid)
    }

    pub fn validate_domain_name(
        input: &str,
    ) -> std::result::Result<Validation, Box<dyn std::error::Error + Send + Sync>> {
        let kebab_result = validate_kebab_case(input)?;
        if let Validation::Invalid(msg) = kebab_result {
            return Ok(Validation::Invalid(msg));
        }

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

    pub fn validate_screen_id(
        input: &str,
    ) -> std::result::Result<Validation, Box<dyn std::error::Error + Send + Sync>> {
        if input.is_empty() {
            return Ok(Validation::Invalid("画面 ID を入力してください".into()));
        }

        let parts: Vec<&str> = input.split('.').collect();

        if parts.is_empty() {
            return Ok(Validation::Invalid("画面 ID を入力してください".into()));
        }

        for part in &parts {
            if part.is_empty() {
                return Ok(Validation::Invalid(
                    "空のセグメントは許可されていません（例: 'users..list'）".into(),
                ));
            }

            let chars: Vec<char> = part.chars().collect();

            if !chars.first().is_some_and(|c| c.is_ascii_lowercase()) {
                return Ok(Validation::Invalid(
                    format!(
                        "セグメント '{}' は小文字のアルファベットで始める必要があります",
                        part
                    )
                    .into(),
                ));
            }

            if chars.last() == Some(&'-') {
                return Ok(Validation::Invalid(
                    format!(
                        "セグメント '{}' はハイフンで終わることはできません",
                        part
                    )
                    .into(),
                ));
            }

            for i in 0..chars.len().saturating_sub(1) {
                if chars[i] == '-' && chars[i + 1] == '-' {
                    return Ok(Validation::Invalid(
                        format!("セグメント '{}' に連続するハイフンがあります", part).into(),
                    ));
                }
            }

            for c in &chars {
                if !c.is_ascii_lowercase() && !c.is_ascii_digit() && *c != '-' {
                    return Ok(Validation::Invalid(
                        format!(
                            "セグメント '{}' に無効な文字が含まれています。小文字、数字、ハイフンのみ使用できます",
                            part
                        )
                        .into(),
                    ));
                }
            }
        }

        Ok(Validation::Valid)
    }
}

// ============================================================
// semver バリデーションテスト
// ============================================================

mod semver_tests {
    use super::*;

    #[test]
    fn valid_versions() {
        let valid_cases = [
            "0.1.0",
            "1.0.0",
            "0.0.0",
            "10.20.30",
            "100.200.300",
            "999.999.999",
        ];

        for version in &valid_cases {
            let result = version_validation::validate_semver(version).unwrap();
            assert!(
                matches!(result, Validation::Valid),
                "Expected '{}' to be valid",
                version
            );
        }
    }

    #[test]
    fn invalid_empty() {
        let result = version_validation::validate_semver("").unwrap();
        assert!(matches!(result, Validation::Invalid(_)));
    }

    #[test]
    fn invalid_with_v_prefix() {
        let invalid_cases = ["v1.0.0", "V1.0.0", "v0.1.0"];

        for version in &invalid_cases {
            let result = version_validation::validate_semver(version).unwrap();
            assert!(
                matches!(result, Validation::Invalid(_)),
                "Expected '{}' to be invalid (v prefix)",
                version
            );
        }
    }

    #[test]
    fn invalid_wrong_parts_count() {
        let invalid_cases = ["1", "1.0", "1.0.0.0", "1.0.0.0.0"];

        for version in &invalid_cases {
            let result = version_validation::validate_semver(version).unwrap();
            assert!(
                matches!(result, Validation::Invalid(_)),
                "Expected '{}' to be invalid (wrong parts count)",
                version
            );
        }
    }

    #[test]
    fn invalid_leading_zeros() {
        let invalid_cases = ["01.0.0", "1.01.0", "1.0.01", "00.0.0", "0.00.0", "0.0.00"];

        for version in &invalid_cases {
            let result = version_validation::validate_semver(version).unwrap();
            assert!(
                matches!(result, Validation::Invalid(_)),
                "Expected '{}' to be invalid (leading zeros)",
                version
            );
        }
    }

    #[test]
    fn invalid_non_numeric() {
        let invalid_cases = [
            "a.b.c",
            "1.x.0",
            "1.0.x",
            "x.0.0",
            "1.0.0-beta",
            "1.0.0+build",
            "1.0.0-rc.1",
            "one.two.three",
        ];

        for version in &invalid_cases {
            let result = version_validation::validate_semver(version).unwrap();
            assert!(
                matches!(result, Validation::Invalid(_)),
                "Expected '{}' to be invalid (non-numeric)",
                version
            );
        }
    }

    #[test]
    fn invalid_empty_parts() {
        let invalid_cases = ["1..0", ".1.0", "1.0.", "..0", "1..", ".."];

        for version in &invalid_cases {
            let result = version_validation::validate_semver(version).unwrap();
            assert!(
                matches!(result, Validation::Invalid(_)),
                "Expected '{}' to be invalid (empty parts)",
                version
            );
        }
    }

    #[test]
    fn invalid_whitespace() {
        let invalid_cases = [" 1.0.0", "1.0.0 ", "1. 0.0", "1 .0.0"];

        for version in &invalid_cases {
            let result = version_validation::validate_semver(version).unwrap();
            assert!(
                matches!(result, Validation::Invalid(_)),
                "Expected '{}' to be invalid (whitespace)",
                version
            );
        }
    }
}

// ============================================================
// kebab-case バリデーションテスト
// ============================================================

mod kebab_case_tests {
    use super::*;

    #[test]
    fn valid_names() {
        let valid_cases = [
            "user",
            "user-management",
            "order-processing",
            "api2",
            "a",
            "auth-service",
            "order-processing-v2",
        ];

        for name in &valid_cases {
            let result = name_validation::validate_kebab_case(name).unwrap();
            assert!(
                matches!(result, Validation::Valid),
                "Expected '{}' to be valid",
                name
            );
        }
    }

    #[test]
    fn invalid_empty() {
        let result = name_validation::validate_kebab_case("").unwrap();
        assert!(matches!(result, Validation::Invalid(_)));
    }

    #[test]
    fn invalid_uppercase() {
        let invalid_cases = [
            "User",
            "userManagement",
            "UserManagement",
            "USER",
            "USERS-LIST",
        ];

        for name in &invalid_cases {
            let result = name_validation::validate_kebab_case(name).unwrap();
            assert!(
                matches!(result, Validation::Invalid(_)),
                "Expected '{}' to be invalid (uppercase)",
                name
            );
        }
    }

    #[test]
    fn invalid_start_with_number() {
        let invalid_cases = ["2user", "123", "1st-feature"];

        for name in &invalid_cases {
            let result = name_validation::validate_kebab_case(name).unwrap();
            assert!(
                matches!(result, Validation::Invalid(_)),
                "Expected '{}' to be invalid (starts with number)",
                name
            );
        }
    }

    #[test]
    fn invalid_start_with_hyphen() {
        let invalid_cases = ["-user", "-", "--user"];

        for name in &invalid_cases {
            let result = name_validation::validate_kebab_case(name).unwrap();
            assert!(
                matches!(result, Validation::Invalid(_)),
                "Expected '{}' to be invalid (starts with hyphen)",
                name
            );
        }
    }

    #[test]
    fn invalid_end_with_hyphen() {
        let invalid_cases = ["user-", "user-management-"];

        for name in &invalid_cases {
            let result = name_validation::validate_kebab_case(name).unwrap();
            assert!(
                matches!(result, Validation::Invalid(_)),
                "Expected '{}' to be invalid (ends with hyphen)",
                name
            );
        }
    }

    #[test]
    fn invalid_consecutive_hyphens() {
        let invalid_cases = ["user--management", "a--b", "user---management"];

        for name in &invalid_cases {
            let result = name_validation::validate_kebab_case(name).unwrap();
            assert!(
                matches!(result, Validation::Invalid(_)),
                "Expected '{}' to be invalid (consecutive hyphens)",
                name
            );
        }
    }

    #[test]
    fn invalid_underscore() {
        let invalid_cases = ["user_management", "snake_case", "a_b"];

        for name in &invalid_cases {
            let result = name_validation::validate_kebab_case(name).unwrap();
            assert!(
                matches!(result, Validation::Invalid(_)),
                "Expected '{}' to be invalid (underscore)",
                name
            );
        }
    }

    #[test]
    fn invalid_special_characters() {
        let invalid_cases = ["user@management", "user.management", "user/management", "user:management"];

        for name in &invalid_cases {
            let result = name_validation::validate_kebab_case(name).unwrap();
            assert!(
                matches!(result, Validation::Invalid(_)),
                "Expected '{}' to be invalid (special characters)",
                name
            );
        }
    }
}

// ============================================================
// domain 名バリデーションテスト（予約語チェック含む）
// ============================================================

mod domain_name_tests {
    use super::*;

    #[test]
    fn valid_domain_names() {
        let valid_cases = [
            "user",
            "order",
            "payment",
            "inventory",
            "user-management",
            "order-processing",
        ];

        for name in &valid_cases {
            let result = name_validation::validate_domain_name(name).unwrap();
            assert!(
                matches!(result, Validation::Valid),
                "Expected '{}' to be valid",
                name
            );
        }
    }

    #[test]
    fn invalid_reserved_words() {
        let reserved_words = [
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

        for word in &reserved_words {
            let result = name_validation::validate_domain_name(word).unwrap();
            assert!(
                matches!(result, Validation::Invalid(_)),
                "Expected '{}' to be invalid (reserved word)",
                word
            );
        }
    }

    #[test]
    fn invalid_kebab_case_rules_still_apply() {
        // domain 名でも kebab-case ルールは適用される
        let invalid_cases = ["User", "user_management", "-user", "user-"];

        for name in &invalid_cases {
            let result = name_validation::validate_domain_name(name).unwrap();
            assert!(
                matches!(result, Validation::Invalid(_)),
                "Expected '{}' to be invalid (kebab-case rule)",
                name
            );
        }
    }
}

// ============================================================
// screen ID バリデーションテスト
// ============================================================

mod screen_id_tests {
    use super::*;

    #[test]
    fn valid_screen_ids() {
        let valid_cases = [
            "users",
            "users.list",
            "users.detail",
            "orders.edit",
            "admin.users.list",
            "admin.settings.security",
            "home",
            "dashboard",
        ];

        for id in &valid_cases {
            let result = name_validation::validate_screen_id(id).unwrap();
            assert!(
                matches!(result, Validation::Valid),
                "Expected '{}' to be valid",
                id
            );
        }
    }

    #[test]
    fn invalid_empty() {
        let result = name_validation::validate_screen_id("").unwrap();
        assert!(matches!(result, Validation::Invalid(_)));
    }

    #[test]
    fn invalid_empty_segments() {
        let invalid_cases = ["users..list", ".users", "users.", ".."];

        for id in &invalid_cases {
            let result = name_validation::validate_screen_id(id).unwrap();
            assert!(
                matches!(result, Validation::Invalid(_)),
                "Expected '{}' to be invalid (empty segment)",
                id
            );
        }
    }

    #[test]
    fn invalid_segment_uppercase() {
        let invalid_cases = ["Users.list", "users.List", "USERS.LIST"];

        for id in &invalid_cases {
            let result = name_validation::validate_screen_id(id).unwrap();
            assert!(
                matches!(result, Validation::Invalid(_)),
                "Expected '{}' to be invalid (uppercase)",
                id
            );
        }
    }

    #[test]
    fn invalid_segment_start_with_number() {
        let invalid_cases = ["1users.list", "users.2list"];

        for id in &invalid_cases {
            let result = name_validation::validate_screen_id(id).unwrap();
            assert!(
                matches!(result, Validation::Invalid(_)),
                "Expected '{}' to be invalid (starts with number)",
                id
            );
        }
    }

    #[test]
    fn invalid_segment_end_with_hyphen() {
        let invalid_cases = ["users-.list", "users.list-"];

        for id in &invalid_cases {
            let result = name_validation::validate_screen_id(id).unwrap();
            assert!(
                matches!(result, Validation::Invalid(_)),
                "Expected '{}' to be invalid (ends with hyphen)",
                id
            );
        }
    }

    #[test]
    fn invalid_segment_consecutive_hyphens() {
        let invalid_cases = ["users--management.list", "users.list--detail"];

        for id in &invalid_cases {
            let result = name_validation::validate_screen_id(id).unwrap();
            assert!(
                matches!(result, Validation::Invalid(_)),
                "Expected '{}' to be invalid (consecutive hyphens)",
                id
            );
        }
    }
}

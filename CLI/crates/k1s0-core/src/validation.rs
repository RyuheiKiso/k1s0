use std::sync::OnceLock;

/// 名前バリデーション用の正規表現キャッシュ（一度だけコンパイルする）
static NAME_RE: OnceLock<regex::Regex> = OnceLock::new();

/// 名前バリデーション: `[a-z0-9-]+`, 先頭末尾ハイフン禁止
///
/// # Errors
///
/// 名前が空の場合、先頭末尾にハイフンを含む場合、または
/// `[a-z0-9][a-z0-9-]*[a-z0-9]` のパターンに一致しない場合にエラーを返す。
///
/// # Panics
///
/// 名前バリデーション用の静的正規表現の初期化に失敗した場合にパニックする。
/// 正規表現はコンパイル時に検証済みのため、通常はパニックしない。
// MED-005 対応: OnceLock::get_or_init 内の .expect() は静的正規表現の初期化失敗時にのみ発生し、
// コンパイル時に検証済みであるため unwrap_used の許可が必要。
#[allow(clippy::unwrap_used)]
pub fn validate_name(name: &str) -> Result<(), String> {
    // LOW-CLI-002 対応: バリデーション違反の理由を具体的に示すエラーメッセージに改善する

    // 空文字チェック
    if name.is_empty() {
        return Err("名前を入力してください。".into());
    }

    // 最大長チェック: OS のパス長制限を考慮し 64 文字を上限とする
    // プロジェクト名はディレクトリ名として使用されるため、OS パス長の余裕を確保する
    if name.len() > 64 {
        return Err(format!(
            "名前は64文字以内で入力してください（現在 {} 文字）。",
            name.len()
        ));
    }

    // 先頭・末尾のハイフンチェック（正規表現より先に判定して具体的なメッセージを返す）
    if name.starts_with('-') || name.ends_with('-') {
        return Err("名前の先頭と末尾にハイフンは使用できません。".into());
    }

    // OnceLock で正規表現を一度だけコンパイルしてキャッシュする
    let re = NAME_RE.get_or_init(|| {
        regex::Regex::new(r"^[a-z0-9][a-z0-9-]*[a-z0-9]$|^[a-z0-9]$")
            .expect("名前バリデーション用の正規表現は静的に正しい")
    });

    // 使用可能文字チェック
    if !re.is_match(name) {
        return Err("英小文字・ハイフン・数字のみ使用できます。".into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
        // MED-006 対応: 64文字超過はエラー
        assert!(validate_name(&"a".repeat(65)).is_err());
    }

    /// MED-006 対応: 64文字制限のバリデーションを確認する
    #[test]
    fn test_validate_name_max_length() {
        // 64文字はOK
        assert!(validate_name(&"a".repeat(64)).is_ok());
        // 65文字はエラー
        let err = validate_name(&"a".repeat(65))
            .expect_err("65文字の名前はエラーになるべき");
        assert!(err.contains("64文字以内"));
        assert!(err.contains("65 文字"));
    }

    /// LOW-CLI-002 対応: エラーメッセージが具体的な違反理由を示すことを確認する
    #[test]
    fn test_validate_name_error_messages() {
        // 空文字
        let err = validate_name("").expect_err("空文字はエラーになるべき");
        assert_eq!(err, "名前を入力してください。");

        // 先頭ハイフン
        let err = validate_name("-task").expect_err("先頭ハイフンはエラーになるべき");
        assert_eq!(err, "名前の先頭と末尾にハイフンは使用できません。");

        // 末尾ハイフン
        let err = validate_name("task-").expect_err("末尾ハイフンはエラーになるべき");
        assert_eq!(err, "名前の先頭と末尾にハイフンは使用できません。");

        // 使用不可文字（大文字）
        let err = validate_name("Task").expect_err("大文字を含む名前はエラーになるべき");
        assert_eq!(err, "英小文字・ハイフン・数字のみ使用できます。");

        // 使用不可文字（アンダースコア）
        let err = validate_name("task_api").expect_err("アンダースコアを含む名前はエラーになるべき");
        assert_eq!(err, "英小文字・ハイフン・数字のみ使用できます。");
    }
}

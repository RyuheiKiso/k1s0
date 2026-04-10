// タスクドメインサービス。バリデーションロジックを集約する。

/// タイトル文字数上限
const MAX_TITLE_LEN: usize = 500;

pub struct TaskService;

impl TaskService {
    /// タイトルのバリデーション
    pub fn validate_title(title: &str) -> anyhow::Result<()> {
        if title.trim().is_empty() {
            anyhow::bail!("title must not be empty");
        }
        if title.len() > MAX_TITLE_LEN {
            anyhow::bail!("title must be at most {} characters", MAX_TITLE_LEN);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_title_ok() {
        assert!(TaskService::validate_title("Valid Title").is_ok());
    }

    #[test]
    fn test_validate_title_empty() {
        assert!(TaskService::validate_title("").is_err());
        assert!(TaskService::validate_title("   ").is_err());
    }

    #[test]
    fn test_validate_title_too_long() {
        let long = "a".repeat(501);
        assert!(TaskService::validate_title(&long).is_err());
    }

    // 境界値: 500 文字は有効な最大長
    #[test]
    fn test_validate_title_exactly_500_chars() {
        let title = "a".repeat(500);
        assert!(TaskService::validate_title(&title).is_ok());
    }

    // 境界値: 1 文字は最短有効タイトル
    #[test]
    fn test_validate_title_single_char() {
        assert!(TaskService::validate_title("x").is_ok());
    }

    // 空白のみのタイトルはトリム後に空文字になるためエラー
    #[test]
    fn test_validate_title_whitespace_only() {
        assert!(TaskService::validate_title("\t\n  ").is_err());
    }

    // 日本語タイトルが有効であることを確認する
    #[test]
    fn test_validate_title_japanese() {
        assert!(TaskService::validate_title("バグ修正タスク").is_ok());
    }

    // エラーメッセージに期待するキーワードが含まれることを確認する
    #[test]
    fn test_validate_title_empty_error_message() {
        let result = TaskService::validate_title("");
        let err = result.unwrap_err();
        assert!(err.to_string().contains("must not be empty"));
    }

    // エラーメッセージに文字数制限が含まれることを確認する
    #[test]
    fn test_validate_title_too_long_error_message() {
        let result = TaskService::validate_title(&"a".repeat(501));
        let err = result.unwrap_err();
        assert!(err.to_string().contains("500"));
    }

    // マルチバイト文字（絵文字）を含むタイトルが許容されることを確認する
    #[test]
    fn test_validate_title_with_emoji() {
        assert!(TaskService::validate_title("🚀 Deploy service").is_ok());
    }

    // 空白を含む有効なタイトルが許容されることを確認する
    #[test]
    fn test_validate_title_with_leading_trailing_spaces_trimmed() {
        // 前後の空白はトリムして内容が残る場合は有効
        assert!(TaskService::validate_title("  Valid Title  ").is_ok());
    }

    // 数字のみのタイトルが許容されることを確認する
    #[test]
    fn test_validate_title_numbers_only() {
        assert!(TaskService::validate_title("12345").is_ok());
    }

    // 特殊文字を含むタイトルが許容されることを確認する
    #[test]
    fn test_validate_title_special_chars() {
        assert!(TaskService::validate_title("[BUG] Fix null pointer @user #tag").is_ok());
    }
}

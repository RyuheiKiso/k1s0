// アクティビティドメインサービス。
pub struct ActivityService;

impl ActivityService {
    /// アクティビティ作成リクエストのバリデーション（ドメインルール集約）
    pub fn validate_content(content: &Option<String>) -> anyhow::Result<()> {
        if let Some(c) = content {
            if c.len() > 10_000 {
                anyhow::bail!("content must be at most 10000 characters");
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 正常系: content が None の場合はバリデーションが通ることを確認する
    #[test]
    fn test_validate_content_none_ok() {
        assert!(ActivityService::validate_content(&None).is_ok());
    }

    // 正常系: content が 10000 文字以内の場合はバリデーションが通ることを確認する
    #[test]
    fn test_validate_content_within_limit_ok() {
        let content = Some("a".repeat(10_000));
        assert!(ActivityService::validate_content(&content).is_ok());
    }

    // 正常系: content が空文字列の場合はバリデーションが通ることを確認する
    #[test]
    fn test_validate_content_empty_string_ok() {
        let content = Some(String::new());
        assert!(ActivityService::validate_content(&content).is_ok());
    }

    // 異常系: content が 10000 文字を超える場合はバリデーションエラーになることを確認する
    #[test]
    fn test_validate_content_exceeds_limit_error() {
        let content = Some("a".repeat(10_001));
        let result = ActivityService::validate_content(&content);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("10000 characters"));
    }

    // 異常系: content が大幅に上限を超える場合もバリデーションエラーになることを確認する
    #[test]
    fn test_validate_content_far_exceeds_limit_error() {
        let content = Some("x".repeat(50_000));
        let result = ActivityService::validate_content(&content);
        assert!(result.is_err());
    }
}

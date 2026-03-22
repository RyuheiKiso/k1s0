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
}

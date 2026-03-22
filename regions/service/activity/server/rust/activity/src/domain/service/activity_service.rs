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

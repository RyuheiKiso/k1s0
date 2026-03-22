// ボードドメインサービス。
pub struct BoardService;

impl BoardService {
    /// WIP 制限値のバリデーション
    pub fn validate_wip_limit(wip_limit: i32) -> anyhow::Result<()> {
        if wip_limit < 0 {
            anyhow::bail!("wip_limit must be >= 0 (0 means unlimited)");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_wip_limit_ok() {
        assert!(BoardService::validate_wip_limit(0).is_ok());
        assert!(BoardService::validate_wip_limit(5).is_ok());
    }

    #[test]
    fn test_validate_wip_limit_negative() {
        assert!(BoardService::validate_wip_limit(-1).is_err());
    }
}

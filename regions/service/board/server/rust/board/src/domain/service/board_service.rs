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

    // 境界値: -1 のエラーメッセージが期待通りの内容を含むことを確認する
    #[test]
    fn test_validate_wip_limit_negative_error_message() {
        let result = BoardService::validate_wip_limit(-1);
        let err = result.unwrap_err();
        assert!(err.to_string().contains("wip_limit must be >= 0"));
    }

    // 境界値: i32::MAX は有効な WIP 制限値として受け入れられる
    #[test]
    fn test_validate_wip_limit_max_value() {
        assert!(BoardService::validate_wip_limit(i32::MAX).is_ok());
    }

    // 境界値: i32::MIN は無効（負数）
    #[test]
    fn test_validate_wip_limit_min_value() {
        assert!(BoardService::validate_wip_limit(i32::MIN).is_err());
    }

    // 正常系: 大きな正の値も許容される
    #[test]
    fn test_validate_wip_limit_large_positive() {
        assert!(BoardService::validate_wip_limit(1000).is_ok());
        assert!(BoardService::validate_wip_limit(100).is_ok());
        assert!(BoardService::validate_wip_limit(1).is_ok());
    }
}

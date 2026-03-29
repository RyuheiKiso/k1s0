use crate::error::SessionError;

/// セッション TTL の最小値（秒）。
/// 極端に短い TTL（例: 1秒）はセッション管理の意味をなさないため、最低 60 秒を要求する（LOW-08 対応）。
const MIN_TTL_SECONDS: i64 = 60;

pub struct SessionDomainService;

impl SessionDomainService {
    /// セッション作成リクエストのバリデーションを行う。
    /// LOW-08 対応: TTL の下限を MIN_TTL_SECONDS（60秒）に設定し、
    /// 極端に短いセッション TTL を拒否する。
    pub fn validate_create_request(
        device_id: &str,
        ttl: i64,
        max_ttl: i64,
    ) -> Result<(), SessionError> {
        if device_id.trim().is_empty() {
            return Err(SessionError::InvalidInput(
                "device_id is required".to_string(),
            ));
        }
        // TTL の下限チェック: MIN_TTL_SECONDS 未満は拒否する
        if ttl < MIN_TTL_SECONDS || ttl > max_ttl {
            return Err(SessionError::InvalidInput(format!(
                "ttl_seconds must be between {} and {}",
                MIN_TTL_SECONDS, max_ttl
            )));
        }
        Ok(())
    }

    pub fn compute_revoke_count(active_sessions: usize, max_devices: usize) -> usize {
        if active_sessions < max_devices {
            return 0;
        }
        active_sessions - max_devices + 1
    }
}

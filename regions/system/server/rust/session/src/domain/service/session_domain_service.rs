use crate::error::SessionError;

pub struct SessionDomainService;

impl SessionDomainService {
    pub fn validate_create_request(device_id: &str, ttl: i64, max_ttl: i64) -> Result<(), SessionError> {
        if device_id.trim().is_empty() {
            return Err(SessionError::InvalidInput(
                "device_id is required".to_string(),
            ));
        }
        if ttl <= 0 || ttl > max_ttl {
            return Err(SessionError::InvalidInput(format!(
                "ttl_seconds must be between 1 and {}",
                max_ttl
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


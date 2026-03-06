pub enum EventStoreDomainError {
    StreamNotFound,
    StreamAlreadyExists,
    VersionConflict { expected: i64, actual: i64 },
}

pub struct EventStoreDomainService;

impl EventStoreDomainService {
    pub fn validate_append(
        stream_exists: bool,
        current_version: Option<i64>,
        expected_version: i64,
    ) -> Result<(), EventStoreDomainError> {
        if expected_version == -1 {
            if stream_exists {
                return Err(EventStoreDomainError::StreamAlreadyExists);
            }
            return Ok(());
        }

        if !stream_exists {
            return Err(EventStoreDomainError::StreamNotFound);
        }

        let actual = current_version.unwrap_or_default();
        if actual != expected_version {
            return Err(EventStoreDomainError::VersionConflict {
                expected: expected_version,
                actual,
            });
        }

        Ok(())
    }
}

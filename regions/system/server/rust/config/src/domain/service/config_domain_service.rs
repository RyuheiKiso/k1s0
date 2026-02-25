/// DomainError はドメイン層のバリデーションエラーを表す。
#[derive(Debug)]
pub enum DomainError {
    InvalidNamespace(String),
    VersionConflict { expected: i32, actual: i32 },
}

impl std::fmt::Display for DomainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidNamespace(ns) => write!(f, "Invalid namespace format: {}", ns),
            Self::VersionConflict { expected, actual } => {
                write!(
                    f,
                    "Version conflict: expected {}, actual {}",
                    expected, actual
                )
            }
        }
    }
}

/// ConfigDomainService はドメインルールのバリデーションを提供する。
pub struct ConfigDomainService;

impl ConfigDomainService {
    pub fn new() -> Self {
        Self
    }

    /// Validate namespace format: {tier}.{service}.{section}
    /// Namespace は最低2セグメント（tier.service）が必要。
    pub fn validate_namespace(&self, namespace: &str) -> Result<(), DomainError> {
        let parts: Vec<&str> = namespace.split('.').collect();
        if parts.len() < 2 {
            return Err(DomainError::InvalidNamespace(format!(
                "Namespace must have at least 2 segments (tier.service), got: {}",
                namespace
            )));
        }
        let valid_tiers = ["system", "business", "service"];
        if !valid_tiers.contains(&parts[0]) {
            return Err(DomainError::InvalidNamespace(format!(
                "Invalid tier '{}', must be one of: {:?}",
                parts[0], valid_tiers
            )));
        }
        Ok(())
    }

    /// Validate version for optimistic concurrency control.
    pub fn validate_version(
        &self,
        current_version: i32,
        expected_version: i32,
    ) -> Result<(), DomainError> {
        if current_version != expected_version {
            return Err(DomainError::VersionConflict {
                expected: expected_version,
                actual: current_version,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_namespace_valid() {
        let svc = ConfigDomainService::new();
        assert!(svc.validate_namespace("system.auth").is_ok());
        assert!(svc.validate_namespace("system.auth.database").is_ok());
        assert!(svc.validate_namespace("business.billing").is_ok());
        assert!(svc.validate_namespace("service.gateway.rate").is_ok());
    }

    #[test]
    fn test_validate_namespace_single_segment() {
        let svc = ConfigDomainService::new();
        let result = svc.validate_namespace("system");
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::InvalidNamespace(msg) => {
                assert!(msg.contains("at least 2 segments"));
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_validate_namespace_invalid_tier() {
        let svc = ConfigDomainService::new();
        let result = svc.validate_namespace("unknown.auth");
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::InvalidNamespace(msg) => {
                assert!(msg.contains("Invalid tier"));
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_validate_version_ok() {
        let svc = ConfigDomainService::new();
        assert!(svc.validate_version(3, 3).is_ok());
    }

    #[test]
    fn test_validate_version_conflict() {
        let svc = ConfigDomainService::new();
        let result = svc.validate_version(4, 3);
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::VersionConflict { expected, actual } => {
                assert_eq!(expected, 3);
                assert_eq!(actual, 4);
            }
            _ => unreachable!(),
        }
    }
}

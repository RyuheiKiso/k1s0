/// DomainError はドメイン層のバリデーションエラーを表す。
#[derive(Debug)]
pub enum DomainError {
    InvalidNamespace(String),
}

impl std::fmt::Display for DomainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidNamespace(ns) => write!(f, "Invalid namespace format: {}", ns),
        }
    }
}

/// ConfigDomainService はドメインルールのバリデーションを提供する。
pub struct ConfigDomainService;

/// ConfigDomainService の Default 実装（clippy::new_without_default 対応）
impl Default for ConfigDomainService {
    fn default() -> Self {
        Self
    }
}

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
        }
    }
}

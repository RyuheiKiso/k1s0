use crate::error::CodegenError;

/// Tier of the server in the k1s0 architecture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    System,
    Business,
    Service,
}

impl Tier {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Tier::System => "system",
            Tier::Business => "business",
            Tier::Service => "service",
        }
    }
}

/// API style for the generated server.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiStyle {
    Rest,
    Grpc,
    Both,
}

/// Database type for the generated server.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseType {
    Postgres,
    None,
}

/// Configuration for scaffolding a new server.
#[derive(Debug, Clone)]
pub struct ScaffoldConfig {
    /// Server name in kebab-case (e.g. "user-profile").
    pub name: String,
    /// Architecture tier.
    pub tier: Tier,
    /// API style.
    pub api_style: ApiStyle,
    /// Database type.
    pub database: DatabaseType,
    /// Server description for README.
    pub description: String,
    /// Path to .proto file for gRPC services (optional).
    pub proto_path: Option<String>,
    /// Whether to generate a client SDK alongside the server.
    pub generate_client: bool,
}

impl ScaffoldConfig {
    pub fn validate(&self) -> Result<(), CodegenError> {
        if self.name.is_empty() {
            return Err(CodegenError::Validation("name must not be empty".into()));
        }
        if !self
            .name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c == '-' || c.is_ascii_digit())
        {
            return Err(CodegenError::Validation(
                "name must be kebab-case (lowercase ascii, digits, hyphens)".into(),
            ));
        }
        if self.name.starts_with('-') || self.name.ends_with('-') {
            return Err(CodegenError::Validation(
                "name must not start or end with a hyphen".into(),
            ));
        }
        if self.name.contains("--") {
            return Err(CodegenError::Validation(
                "name must not contain consecutive hyphens".into(),
            ));
        }
        Ok(())
    }

    #[must_use]
    pub fn has_grpc(&self) -> bool {
        matches!(self.api_style, ApiStyle::Grpc | ApiStyle::Both)
    }

    #[must_use]
    pub fn has_rest(&self) -> bool {
        matches!(self.api_style, ApiStyle::Rest | ApiStyle::Both)
    }

    #[must_use]
    pub fn has_database(&self) -> bool {
        !matches!(self.database, DatabaseType::None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_config() -> ScaffoldConfig {
        ScaffoldConfig {
            name: "my-server".into(),
            tier: Tier::System,
            api_style: ApiStyle::Rest,
            database: DatabaseType::None,
            description: "test".into(),
            proto_path: None,
            generate_client: false,
        }
    }

    #[test]
    fn valid_name() {
        assert!(valid_config().validate().is_ok());
    }

    #[test]
    fn empty_name() {
        let mut c = valid_config();
        c.name = "".into();
        assert!(c.validate().is_err());
    }

    #[test]
    fn uppercase_name() {
        let mut c = valid_config();
        c.name = "MyServer".into();
        assert!(c.validate().is_err());
    }

    #[test]
    fn leading_hyphen() {
        let mut c = valid_config();
        c.name = "-server".into();
        assert!(c.validate().is_err());
    }

    #[test]
    fn consecutive_hyphens() {
        let mut c = valid_config();
        c.name = "my--server".into();
        assert!(c.validate().is_err());
    }
}

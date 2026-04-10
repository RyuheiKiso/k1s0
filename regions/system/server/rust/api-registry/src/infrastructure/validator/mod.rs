pub mod openapi;
pub mod protobuf;

use async_trait::async_trait;
use openapi::OpenApiSubprocessValidator;
use protobuf::ProtobufSubprocessValidator;

#[derive(Debug)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait SchemaValidator: Send + Sync {
    async fn validate(&self, content: &str) -> anyhow::Result<Vec<ValidationError>>;
}

/// Factory for creating schema validators based on schema type.
pub trait SchemaValidatorFactory: Send + Sync {
    fn create(&self, schema_type: &str) -> Option<Box<dyn SchemaValidator>>;
}

/// Default implementation with well-known validator executable names.
pub struct DefaultSchemaValidatorFactory;

impl SchemaValidatorFactory for DefaultSchemaValidatorFactory {
    fn create(&self, schema_type: &str) -> Option<Box<dyn SchemaValidator>> {
        let factory = ConfigurableSchemaValidatorFactory::new(
            "openapi-spec-validator".to_string(),
            "buf".to_string(),
            10,
        );
        factory.create(schema_type)
    }
}

/// Configurable factory used by `main.rs` based on config.yaml.
pub struct ConfigurableSchemaValidatorFactory {
    openapi_validator_path: String,
    buf_path: String,
    timeout_secs: u64,
}

impl ConfigurableSchemaValidatorFactory {
    #[must_use]
    pub fn new(openapi_validator_path: String, buf_path: String, timeout_secs: u64) -> Self {
        Self {
            openapi_validator_path,
            buf_path,
            timeout_secs,
        }
    }
}

impl SchemaValidatorFactory for ConfigurableSchemaValidatorFactory {
    fn create(&self, schema_type: &str) -> Option<Box<dyn SchemaValidator>> {
        match schema_type {
            "openapi" => Some(Box::new(OpenApiSubprocessValidator::new(
                self.openapi_validator_path.clone(),
                self.timeout_secs,
            ))),
            "protobuf" => Some(Box::new(ProtobufSubprocessValidator::new(
                self.buf_path.clone(),
                self.timeout_secs,
            ))),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_factory_openapi() {
        let factory = DefaultSchemaValidatorFactory;
        assert!(factory.create("openapi").is_some());
    }

    #[test]
    fn test_default_factory_protobuf() {
        let factory = DefaultSchemaValidatorFactory;
        assert!(factory.create("protobuf").is_some());
    }

    #[test]
    fn test_default_factory_unknown() {
        let factory = DefaultSchemaValidatorFactory;
        assert!(factory.create("graphql").is_none());
    }

    #[test]
    fn test_configurable_factory_uses_given_paths() {
        let factory = ConfigurableSchemaValidatorFactory::new(
            "/usr/local/bin/openapi-spec-validator".to_string(),
            "/usr/local/bin/buf".to_string(),
            7,
        );
        assert!(factory.create("openapi").is_some());
        assert!(factory.create("protobuf").is_some());
        assert!(factory.create("unknown").is_none());
    }
}

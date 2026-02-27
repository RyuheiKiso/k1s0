pub mod openapi;
pub mod protobuf;

use async_trait::async_trait;

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

/// Default implementation that performs basic JSON/YAML validation.
pub struct DefaultSchemaValidatorFactory;

impl SchemaValidatorFactory for DefaultSchemaValidatorFactory {
    fn create(&self, schema_type: &str) -> Option<Box<dyn SchemaValidator>> {
        match schema_type {
            "openapi" => Some(Box::new(BasicYamlJsonValidator)),
            "protobuf" => Some(Box::new(BasicProtobufValidator)),
            _ => None,
        }
    }
}

/// Basic YAML/JSON validator for OpenAPI schemas.
struct BasicYamlJsonValidator;

#[async_trait]
impl SchemaValidator for BasicYamlJsonValidator {
    async fn validate(&self, content: &str) -> anyhow::Result<Vec<ValidationError>> {
        // Try YAML parse (which also handles JSON)
        match serde_yaml::from_str::<serde_json::Value>(content) {
            Ok(_) => Ok(vec![]),
            Err(e) => Ok(vec![ValidationError {
                field: "content".to_string(),
                message: format!("YAML/JSON parse error: {}", e),
            }]),
        }
    }
}

/// Basic protobuf syntax validator.
struct BasicProtobufValidator;

#[async_trait]
impl SchemaValidator for BasicProtobufValidator {
    async fn validate(&self, content: &str) -> anyhow::Result<Vec<ValidationError>> {
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return Ok(vec![ValidationError {
                field: "content".to_string(),
                message: "protobuf content is empty".to_string(),
            }]);
        }
        if !trimmed.contains("syntax") {
            return Ok(vec![ValidationError {
                field: "content".to_string(),
                message: "protobuf file must contain a syntax declaration".to_string(),
            }]);
        }
        Ok(vec![])
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

    #[tokio::test]
    async fn test_basic_yaml_validator_valid() {
        let validator = BasicYamlJsonValidator;
        let result = validator.validate("openapi: 3.0.3\ninfo:\n  title: Test\n").await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_basic_yaml_validator_invalid() {
        let validator = BasicYamlJsonValidator;
        let result = validator.validate("{{invalid yaml").await.unwrap();
        assert!(!result.is_empty());
    }

    #[tokio::test]
    async fn test_basic_protobuf_validator_valid() {
        let validator = BasicProtobufValidator;
        let result = validator.validate("syntax = \"proto3\";\nmessage Test {}\n").await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_basic_protobuf_validator_empty() {
        let validator = BasicProtobufValidator;
        let result = validator.validate("").await.unwrap();
        assert!(!result.is_empty());
    }
}

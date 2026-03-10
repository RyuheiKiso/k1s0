use jsonschema::JSONSchema;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;

const CONFIG_SCHEMA_JSON: &str =
    include_str!("../../../../k1s0-cli/templates/config/config-schema-schema.json");

#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigSchemaYaml {
    pub version: u32,
    pub service: String,
    pub namespace_prefix: String,
    pub categories: Vec<CategoryYaml>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CategoryYaml {
    pub id: String,
    pub label: String,
    pub icon: Option<String>,
    pub namespaces: Vec<String>,
    pub fields: Vec<FieldYaml>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FieldYaml {
    pub key: String,
    pub label: String,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub field_type: Option<String>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub options: Option<Vec<String>>,
    pub pattern: Option<String>,
    pub unit: Option<String>,
    pub default: Option<serde_yaml::Value>,
}

pub fn validate_config_schema(path: &str) -> Result<usize, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    println!("Checking config-schema.yaml...");

    let yaml_value: serde_yaml::Value = match serde_yaml::from_str(&content) {
        Ok(value) => {
            println!("  OK YAML parse");
            value
        }
        Err(error) => {
            println!("  ERROR YAML parse: {error}");
            return Ok(1);
        }
    };

    let mut errors = 0usize;
    let schema_json: serde_json::Value = serde_json::from_str(CONFIG_SCHEMA_JSON)?;
    let instance_json = serde_json::to_value(&yaml_value)?;
    let compiled = JSONSchema::compile(&schema_json).map_err(|error| error.to_string())?;
    let json_schema_errors = compiled
        .validate(&instance_json)
        .err()
        .map(|validation_errors| {
            validation_errors
                .map(|error| format!("{error}"))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    if json_schema_errors.is_empty() {
        println!("  OK JSON Schema validation");
    } else {
        for error in json_schema_errors {
            println!("  ERROR JSON Schema: {error}");
            errors += 1;
        }
    }

    let schema: ConfigSchemaYaml = match serde_yaml::from_value(yaml_value.clone()) {
        Ok(schema) => schema,
        Err(error) => {
            if errors == 0 {
                println!("  ERROR schema parse: {error}");
                errors += 1;
            }
            print_summary(errors);
            return Ok(errors);
        }
    };

    errors += validate_namespace_prefixes(&schema);
    errors += validate_unique_category_ids(&schema);
    errors += validate_unique_field_keys(&schema);
    errors += validate_field_types(&schema);
    errors += validate_enum_fields(&schema);
    errors += validate_number_ranges(&schema);
    errors += validate_default_values(&schema);

    print_summary(errors);
    Ok(errors)
}

fn validate_namespace_prefixes(schema: &ConfigSchemaYaml) -> usize {
    let mut errors = 0usize;
    for category in &schema.categories {
        for namespace in &category.namespaces {
            if !namespace.starts_with(&schema.namespace_prefix) {
                println!(
                    "  ERROR namespace prefix: category '{}' namespace '{}' must start with '{}'",
                    category.id, namespace, schema.namespace_prefix
                );
                errors += 1;
            }
        }
    }

    if errors == 0 {
        println!("  OK namespace prefix");
    }

    errors
}

fn validate_unique_category_ids(schema: &ConfigSchemaYaml) -> usize {
    let mut ids = HashSet::new();
    let mut errors = 0usize;

    for category in &schema.categories {
        if !ids.insert(&category.id) {
            println!("  ERROR duplicate category id: '{}'", category.id);
            errors += 1;
        }
    }

    if errors == 0 {
        println!("  OK category ids");
    }

    errors
}

fn validate_unique_field_keys(schema: &ConfigSchemaYaml) -> usize {
    let mut errors = 0usize;

    for category in &schema.categories {
        let mut keys = HashSet::new();
        for field in &category.fields {
            if !keys.insert(&field.key) {
                println!(
                    "  ERROR duplicate field key: category '{}' field '{}'",
                    category.id, field.key
                );
                errors += 1;
            }
        }
    }

    if errors == 0 {
        println!("  OK field keys");
    }

    errors
}

fn validate_field_types(schema: &ConfigSchemaYaml) -> usize {
    const VALID_TYPES: &[&str] = &[
        "string",
        "integer",
        "float",
        "boolean",
        "enum",
        "object",
        "array",
    ];

    let mut errors = 0usize;

    for category in &schema.categories {
        for field in &category.fields {
            match field.field_type.as_deref() {
                Some(field_type) if VALID_TYPES.contains(&field_type) => {}
                Some(field_type) => {
                    println!(
                        "  ERROR field type: category '{}' field '{}' has unsupported type '{}'",
                        category.id, field.key, field_type
                    );
                    errors += 1;
                }
                None => {
                    println!(
                        "  ERROR field type: category '{}' field '{}' is missing type",
                        category.id, field.key
                    );
                    errors += 1;
                }
            }
        }
    }

    if errors == 0 {
        println!("  OK field types");
    }

    errors
}

fn validate_enum_fields(schema: &ConfigSchemaYaml) -> usize {
    let mut errors = 0usize;

    for category in &schema.categories {
        for field in &category.fields {
            if field.field_type.as_deref() != Some("enum") {
                continue;
            }

            let options = field.options.as_ref().filter(|options| !options.is_empty());
            if options.is_none() {
                println!(
                    "  ERROR enum options: category '{}' field '{}' requires options",
                    category.id, field.key
                );
                errors += 1;
                continue;
            }

            if let Some(default) = field.default.as_ref().and_then(serde_yaml::Value::as_str) {
                if !options.unwrap().iter().any(|option| option == default) {
                    println!(
                        "  ERROR enum default: category '{}' field '{}' default '{}' is not in options",
                        category.id, field.key, default
                    );
                    errors += 1;
                }
            }
        }
    }

    if errors == 0 {
        println!("  OK enum options");
    }

    errors
}

fn validate_number_ranges(schema: &ConfigSchemaYaml) -> usize {
    let mut errors = 0usize;

    for category in &schema.categories {
        for field in &category.fields {
            match field.field_type.as_deref() {
                Some("integer") | Some("float") => {
                    if let (Some(min), Some(max)) = (field.min, field.max) {
                        if min > max {
                            println!(
                                "  ERROR numeric range: category '{}' field '{}' has min {} greater than max {}",
                                category.id, field.key, min, max
                            );
                            errors += 1;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    if errors == 0 {
        println!("  OK numeric ranges");
    }

    errors
}

fn validate_default_values(schema: &ConfigSchemaYaml) -> usize {
    let mut errors = 0usize;

    for category in &schema.categories {
        for field in &category.fields {
            let Some(default) = &field.default else {
                continue;
            };

            let Some(field_type) = field.field_type.as_deref() else {
                continue;
            };

            if !default_matches_type(field_type, default) {
                println!(
                    "  ERROR default type: category '{}' field '{}' default does not match type '{}'",
                    category.id, field.key, field_type
                );
                errors += 1;
                continue;
            }

            match field_type {
                "integer" | "float" => {
                    if let Some(value) = default.as_f64() {
                        if let Some(min) = field.min {
                            if value < min {
                                println!(
                                    "  ERROR default range: category '{}' field '{}' default {} is below min {}",
                                    category.id, field.key, value, min
                                );
                                errors += 1;
                            }
                        }
                        if let Some(max) = field.max {
                            if value > max {
                                println!(
                                    "  ERROR default range: category '{}' field '{}' default {} is above max {}",
                                    category.id, field.key, value, max
                                );
                                errors += 1;
                            }
                        }
                    }
                }
                "string" => {
                    if let (Some(pattern), Some(value)) =
                        (field.pattern.as_deref(), default.as_str())
                    {
                        match Regex::new(pattern) {
                            Ok(regex) if regex.is_match(value) => {}
                            Ok(_) => {
                                println!(
                                    "  ERROR default pattern: category '{}' field '{}' default '{}' does not match '{}'",
                                    category.id, field.key, value, pattern
                                );
                                errors += 1;
                            }
                            Err(error) => {
                                println!(
                                    "  ERROR string pattern: category '{}' field '{}' has invalid regex '{}': {}",
                                    category.id, field.key, pattern, error
                                );
                                errors += 1;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    if errors == 0 {
        println!("  OK default values");
    }

    errors
}

fn default_matches_type(field_type: &str, value: &serde_yaml::Value) -> bool {
    match field_type {
        "string" | "enum" => value.as_str().is_some(),
        "integer" => {
            value.as_i64().is_some()
                || value.as_u64().is_some()
                || value
                    .as_f64()
                    .is_some_and(|number| (number.fract() - 0.0).abs() < f64::EPSILON)
        }
        "float" => value.as_f64().is_some(),
        "boolean" => value.as_bool().is_some(),
        "object" => value.as_mapping().is_some(),
        "array" => value.as_sequence().is_some(),
        _ => false,
    }
}

fn print_summary(errors: usize) {
    if errors == 0 {
        println!("\nValidation succeeded. No errors found.");
    } else {
        println!("\nValidation failed. {errors} error(s) found.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_yaml(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file
    }

    #[test]
    fn test_validate_valid_schema() {
        let yaml = r#"
version: 1
service: my-service
namespace_prefix: service.myservice
categories:
  - id: general
    label: General
    namespaces:
      - service.myservice.general
    fields:
      - key: flag
        label: Flag
        type: boolean
        default: false
"#;
        let file = write_yaml(yaml);
        let errors = validate_config_schema(file.path().to_str().unwrap()).unwrap();
        assert_eq!(errors, 0);
    }

    #[test]
    fn test_validate_missing_type() {
        let yaml = r#"
version: 1
service: my-service
namespace_prefix: service.myservice
categories:
  - id: general
    label: General
    namespaces:
      - service.myservice.general
    fields:
      - key: max_retry
        label: Max Retry
"#;
        let file = write_yaml(yaml);
        let errors = validate_config_schema(file.path().to_str().unwrap()).unwrap();
        assert!(errors > 0);
    }

    #[test]
    fn test_validate_invalid_service_name_with_json_schema() {
        let yaml = r#"
version: 1
service: MyService
namespace_prefix: service.myservice
categories:
  - id: general
    label: General
    namespaces:
      - service.myservice.general
    fields:
      - key: flag
        label: Flag
        type: boolean
"#;
        let file = write_yaml(yaml);
        let errors = validate_config_schema(file.path().to_str().unwrap()).unwrap();
        assert!(errors > 0);
    }

    #[test]
    fn test_validate_namespace_mismatch() {
        let yaml = r#"
version: 1
service: my-service
namespace_prefix: service.myservice
categories:
  - id: general
    label: General
    namespaces:
      - other.prefix.general
    fields:
      - key: flag
        label: Flag
        type: boolean
"#;
        let file = write_yaml(yaml);
        let errors = validate_config_schema(file.path().to_str().unwrap()).unwrap();
        assert!(errors > 0);
    }

    #[test]
    fn test_validate_duplicate_category_ids() {
        let yaml = r#"
version: 1
service: my-service
namespace_prefix: service.myservice
categories:
  - id: general
    label: General
    namespaces:
      - service.myservice.general
    fields:
      - key: flag
        label: Flag
        type: boolean
  - id: general
    label: General 2
    namespaces:
      - service.myservice.general2
    fields:
      - key: flag_two
        label: Flag Two
        type: boolean
"#;
        let file = write_yaml(yaml);
        let errors = validate_config_schema(file.path().to_str().unwrap()).unwrap();
        assert!(errors > 0);
    }

    #[test]
    fn test_validate_duplicate_keys() {
        let yaml = r#"
version: 1
service: my-service
namespace_prefix: service.myservice
categories:
  - id: general
    label: General
    namespaces:
      - service.myservice.general
    fields:
      - key: flag
        label: Flag 1
        type: boolean
      - key: flag
        label: Flag 2
        type: boolean
"#;
        let file = write_yaml(yaml);
        let errors = validate_config_schema(file.path().to_str().unwrap()).unwrap();
        assert!(errors > 0);
    }

    #[test]
    fn test_validate_enum_missing_options() {
        let yaml = r#"
version: 1
service: my-service
namespace_prefix: service.myservice
categories:
  - id: general
    label: General
    namespaces:
      - service.myservice.general
    fields:
      - key: log_level
        label: Log Level
        type: enum
"#;
        let file = write_yaml(yaml);
        let errors = validate_config_schema(file.path().to_str().unwrap()).unwrap();
        assert!(errors > 0);
    }

    #[test]
    fn test_validate_default_type_mismatch() {
        let yaml = r#"
version: 1
service: my-service
namespace_prefix: service.myservice
categories:
  - id: general
    label: General
    namespaces:
      - service.myservice.general
    fields:
      - key: count
        label: Count
        type: integer
        default: "not_a_number"
"#;
        let file = write_yaml(yaml);
        let errors = validate_config_schema(file.path().to_str().unwrap()).unwrap();
        assert!(errors > 0);
    }

    #[test]
    fn test_validate_invalid_yaml() {
        let file = write_yaml("{{{{ invalid yaml ::::");
        let errors = validate_config_schema(file.path().to_str().unwrap()).unwrap();
        assert!(errors > 0);
    }
}

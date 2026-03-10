use super::ValidationDiagnostic;
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

pub fn collect_config_schema_diagnostics(
    path: &str,
) -> Result<Vec<ValidationDiagnostic>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let yaml_value: serde_yaml::Value = match serde_yaml::from_str(&content) {
        Ok(value) => value,
        Err(error) => {
            return Ok(vec![ValidationDiagnostic {
                rule: "yaml-parse".to_string(),
                path: "$".to_string(),
                message: error.to_string(),
                line: error.location().map(|location| location.line()),
            }]);
        }
    };

    let mut diagnostics = Vec::new();
    let schema_json: serde_json::Value = serde_json::from_str(CONFIG_SCHEMA_JSON)?;
    let instance_json = serde_json::to_value(&yaml_value)?;
    let compiled = JSONSchema::compile(&schema_json).map_err(|error| error.to_string())?;
    if let Err(validation_errors) = compiled.validate(&instance_json) {
        diagnostics.extend(validation_errors.map(|error| ValidationDiagnostic {
            rule: "json-schema".to_string(),
            path: json_pointer_or_root(error.instance_path.to_string()),
            message: error.to_string(),
            line: None,
        }));
    }

    let schema: ConfigSchemaYaml = match serde_yaml::from_value(yaml_value.clone()) {
        Ok(schema) => schema,
        Err(error) => {
            if diagnostics.is_empty() {
                diagnostics.push(ValidationDiagnostic {
                    rule: "schema-parse".to_string(),
                    path: "$".to_string(),
                    message: error.to_string(),
                    line: None,
                });
            }
            return Ok(diagnostics);
        }
    };

    collect_namespace_prefix_diagnostics(&schema, &mut diagnostics);
    collect_unique_category_id_diagnostics(&schema, &mut diagnostics);
    collect_unique_field_key_diagnostics(&schema, &mut diagnostics);
    collect_field_type_diagnostics(&schema, &mut diagnostics);
    collect_enum_field_diagnostics(&schema, &mut diagnostics);
    collect_number_range_diagnostics(&schema, &mut diagnostics);
    collect_default_value_diagnostics(&schema, &mut diagnostics);

    Ok(diagnostics)
}

pub fn validate_config_schema(path: &str) -> Result<usize, Box<dyn std::error::Error>> {
    println!("Checking config-schema.yaml...");
    let diagnostics = collect_config_schema_diagnostics(path)?;

    if diagnostics.is_empty() {
        println!("  OK YAML parse");
        println!("  OK JSON Schema validation");
        println!("  OK namespace prefix");
        println!("  OK category ids");
        println!("  OK field keys");
        println!("  OK field types");
        println!("  OK enum options");
        println!("  OK numeric ranges");
        println!("  OK default values");
    } else {
        for diagnostic in &diagnostics {
            println!(
                "  ERROR [{}] {}: {}",
                diagnostic.rule, diagnostic.path, diagnostic.message
            );
        }
    }

    print_summary(diagnostics.len());
    Ok(diagnostics.len())
}

fn json_pointer_or_root(pointer: String) -> String {
    if pointer.is_empty() {
        "$".to_string()
    } else {
        pointer
    }
}

fn collect_namespace_prefix_diagnostics(
    schema: &ConfigSchemaYaml,
    diagnostics: &mut Vec<ValidationDiagnostic>,
) {
    for (category_index, category) in schema.categories.iter().enumerate() {
        for (namespace_index, namespace) in category.namespaces.iter().enumerate() {
            if !namespace.starts_with(&schema.namespace_prefix) {
                diagnostics.push(ValidationDiagnostic {
                    rule: "namespace-prefix".to_string(),
                    path: format!("categories[{category_index}].namespaces[{namespace_index}]"),
                    message: format!(
                        "category '{}' namespace '{}' must start with '{}'",
                        category.id, namespace, schema.namespace_prefix
                    ),
                    line: None,
                });
            }
        }
    }
}

fn collect_unique_category_id_diagnostics(
    schema: &ConfigSchemaYaml,
    diagnostics: &mut Vec<ValidationDiagnostic>,
) {
    let mut ids = HashSet::new();

    for (category_index, category) in schema.categories.iter().enumerate() {
        if !ids.insert(category.id.as_str()) {
            diagnostics.push(ValidationDiagnostic {
                rule: "duplicate-category-id".to_string(),
                path: format!("categories[{category_index}].id"),
                message: format!("duplicate category id '{}'", category.id),
                line: None,
            });
        }
    }
}

fn collect_unique_field_key_diagnostics(
    schema: &ConfigSchemaYaml,
    diagnostics: &mut Vec<ValidationDiagnostic>,
) {
    for (category_index, category) in schema.categories.iter().enumerate() {
        let mut keys = HashSet::new();
        for (field_index, field) in category.fields.iter().enumerate() {
            if !keys.insert(field.key.as_str()) {
                diagnostics.push(ValidationDiagnostic {
                    rule: "duplicate-field-key".to_string(),
                    path: format!("categories[{category_index}].fields[{field_index}].key"),
                    message: format!(
                        "category '{}' field '{}' is duplicated",
                        category.id, field.key
                    ),
                    line: None,
                });
            }
        }
    }
}

fn collect_field_type_diagnostics(
    schema: &ConfigSchemaYaml,
    diagnostics: &mut Vec<ValidationDiagnostic>,
) {
    const VALID_TYPES: &[&str] = &[
        "string", "integer", "float", "boolean", "enum", "object", "array",
    ];

    for (category_index, category) in schema.categories.iter().enumerate() {
        for (field_index, field) in category.fields.iter().enumerate() {
            match field.field_type.as_deref() {
                Some(field_type) if VALID_TYPES.contains(&field_type) => {}
                Some(field_type) => diagnostics.push(ValidationDiagnostic {
                    rule: "field-type".to_string(),
                    path: format!("categories[{category_index}].fields[{field_index}].type"),
                    message: format!(
                        "category '{}' field '{}' has unsupported type '{}'",
                        category.id, field.key, field_type
                    ),
                    line: None,
                }),
                None => diagnostics.push(ValidationDiagnostic {
                    rule: "field-type".to_string(),
                    path: format!("categories[{category_index}].fields[{field_index}].type"),
                    message: format!(
                        "category '{}' field '{}' is missing type",
                        category.id, field.key
                    ),
                    line: None,
                }),
            }
        }
    }
}

fn collect_enum_field_diagnostics(
    schema: &ConfigSchemaYaml,
    diagnostics: &mut Vec<ValidationDiagnostic>,
) {
    for (category_index, category) in schema.categories.iter().enumerate() {
        for (field_index, field) in category.fields.iter().enumerate() {
            if field.field_type.as_deref() != Some("enum") {
                continue;
            }

            let options = field.options.as_ref().filter(|options| !options.is_empty());
            if options.is_none() {
                diagnostics.push(ValidationDiagnostic {
                    rule: "enum-options".to_string(),
                    path: format!("categories[{category_index}].fields[{field_index}].options"),
                    message: format!(
                        "category '{}' field '{}' requires options",
                        category.id, field.key
                    ),
                    line: None,
                });
                continue;
            }

            if let Some(default) = field.default.as_ref().and_then(serde_yaml::Value::as_str) {
                if !options
                    .expect("checked above")
                    .iter()
                    .any(|option| option == default)
                {
                    diagnostics.push(ValidationDiagnostic {
                        rule: "enum-default".to_string(),
                        path: format!("categories[{category_index}].fields[{field_index}].default"),
                        message: format!(
                            "category '{}' field '{}' default '{}' is not in options",
                            category.id, field.key, default
                        ),
                        line: None,
                    });
                }
            }
        }
    }
}

fn collect_number_range_diagnostics(
    schema: &ConfigSchemaYaml,
    diagnostics: &mut Vec<ValidationDiagnostic>,
) {
    for (category_index, category) in schema.categories.iter().enumerate() {
        for (field_index, field) in category.fields.iter().enumerate() {
            match field.field_type.as_deref() {
                Some("integer") | Some("float") => {
                    if let (Some(min), Some(max)) = (field.min, field.max) {
                        if min > max {
                            diagnostics.push(ValidationDiagnostic {
                                rule: "numeric-range".to_string(),
                                path: format!("categories[{category_index}].fields[{field_index}]"),
                                message: format!(
                                    "category '{}' field '{}' has min {} greater than max {}",
                                    category.id, field.key, min, max
                                ),
                                line: None,
                            });
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

fn collect_default_value_diagnostics(
    schema: &ConfigSchemaYaml,
    diagnostics: &mut Vec<ValidationDiagnostic>,
) {
    for (category_index, category) in schema.categories.iter().enumerate() {
        for (field_index, field) in category.fields.iter().enumerate() {
            let Some(default) = &field.default else {
                continue;
            };

            let Some(field_type) = field.field_type.as_deref() else {
                continue;
            };

            if !default_matches_type(field_type, default) {
                diagnostics.push(ValidationDiagnostic {
                    rule: "default-type".to_string(),
                    path: format!("categories[{category_index}].fields[{field_index}].default"),
                    message: format!(
                        "category '{}' field '{}' default does not match type '{}'",
                        category.id, field.key, field_type
                    ),
                    line: None,
                });
                continue;
            }

            match field_type {
                "integer" | "float" => {
                    if let Some(value) = default.as_f64() {
                        if let Some(min) = field.min {
                            if value < min {
                                diagnostics.push(ValidationDiagnostic {
                                    rule: "default-range".to_string(),
                                    path: format!(
                                        "categories[{category_index}].fields[{field_index}].default"
                                    ),
                                    message: format!(
                                        "category '{}' field '{}' default {} is below min {}",
                                        category.id, field.key, value, min
                                    ),
                                    line: None,
                                });
                            }
                        }
                        if let Some(max) = field.max {
                            if value > max {
                                diagnostics.push(ValidationDiagnostic {
                                    rule: "default-range".to_string(),
                                    path: format!(
                                        "categories[{category_index}].fields[{field_index}].default"
                                    ),
                                    message: format!(
                                        "category '{}' field '{}' default {} is above max {}",
                                        category.id, field.key, value, max
                                    ),
                                    line: None,
                                });
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
                            Ok(_) => diagnostics.push(ValidationDiagnostic {
                                rule: "default-pattern".to_string(),
                                path: format!(
                                    "categories[{category_index}].fields[{field_index}].default"
                                ),
                                message: format!(
                                    "category '{}' field '{}' default '{}' does not match '{}'",
                                    category.id, field.key, value, pattern
                                ),
                                line: None,
                            }),
                            Err(error) => diagnostics.push(ValidationDiagnostic {
                                rule: "string-pattern".to_string(),
                                path: format!(
                                    "categories[{category_index}].fields[{field_index}].pattern"
                                ),
                                message: format!(
                                    "category '{}' field '{}' has invalid regex '{}': {}",
                                    category.id, field.key, pattern, error
                                ),
                                line: None,
                            }),
                        }
                    }
                }
                _ => {}
            }
        }
    }
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

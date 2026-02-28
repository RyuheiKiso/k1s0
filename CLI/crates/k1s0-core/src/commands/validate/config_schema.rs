use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;

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

/// config-schema.yaml をバリデーションする。
/// 戻り値: エラー数 (0なら成功)
pub fn validate_config_schema(path: &str) -> Result<usize, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    println!("Checking config-schema.yaml...");

    // 1. スキーマバリデーション（YAML パース）
    let schema: ConfigSchemaYaml = match serde_yaml::from_str(&content) {
        Ok(s) => {
            println!("  \u{2705} スキーマバリデーション OK");
            s
        }
        Err(e) => {
            println!("  \u{274c} スキーマバリデーションエラー: {e}");
            return Ok(1);
        }
    };

    let mut errors = 0usize;

    // 2. 必須フィールド存在チェック (service, namespace_prefix, categories)
    if schema.service.is_empty() {
        println!("  \u{274c} service が空です");
        errors += 1;
    }
    if schema.namespace_prefix.is_empty() {
        println!("  \u{274c} namespace_prefix が空です");
        errors += 1;
    }
    if schema.categories.is_empty() {
        println!("  \u{274c} categories が空です");
        errors += 1;
    }

    // 3. namespace prefix 整合性: 全 namespaces が namespace_prefix で始まっている
    let mut ns_ok = true;
    for cat in &schema.categories {
        for ns in &cat.namespaces {
            if !ns.starts_with(&schema.namespace_prefix) {
                println!(
                    "  \u{274c} category '{}' の namespace '{}' が namespace_prefix '{}' で始まっていません",
                    cat.id, ns, schema.namespace_prefix
                );
                errors += 1;
                ns_ok = false;
            }
        }
    }
    if ns_ok {
        println!("  \u{2705} namespace prefix 整合性 OK");
    }

    // 4. field key の重複なし（同カテゴリ内）
    let mut dup_ok = true;
    for cat in &schema.categories {
        let mut keys = HashSet::new();
        for field in &cat.fields {
            if !keys.insert(&field.key) {
                println!(
                    "  \u{274c} category '{}' の field key '{}' が重複しています",
                    cat.id, field.key
                );
                errors += 1;
                dup_ok = false;
            }
        }
    }
    if dup_ok {
        println!("  \u{2705} field key の重複なし");
    }

    // 5. type 必須チェック（全フィールドに type が指定されている）
    let mut type_ok = true;
    for cat in &schema.categories {
        for field in &cat.fields {
            if field.field_type.is_none() || field.field_type.as_deref() == Some("") {
                println!(
                    "  \u{274c} category '{}' の field '{}' に type が未指定",
                    cat.id, field.key
                );
                errors += 1;
                type_ok = false;
            }
        }
    }
    if type_ok {
        println!("  \u{2705} type 指定 OK");
    }

    // 6. enum options チェック（type=enum時にoptionsが空でない）
    let mut enum_ok = true;
    for cat in &schema.categories {
        for field in &cat.fields {
            if field.field_type.as_deref() == Some("enum") {
                let has_options = field
                    .options
                    .as_ref()
                    .is_some_and(|opts| !opts.is_empty());
                if !has_options {
                    println!(
                        "  \u{274c} category '{}' の field '{}' は type=enum ですが options が空です",
                        cat.id, field.key
                    );
                    errors += 1;
                    enum_ok = false;
                }
            }
        }
    }
    if enum_ok {
        println!("  \u{2705} enum options OK");
    }

    // 7. default 型整合性（integer/float -> 数値、boolean -> bool）
    let mut default_ok = true;
    for cat in &schema.categories {
        for field in &cat.fields {
            if let Some(ref default_val) = field.default {
                let type_str = field.field_type.as_deref().unwrap_or("");
                let valid = match type_str {
                    "integer" => default_val.is_number(),
                    "float" => default_val.is_number(),
                    "boolean" => default_val.is_bool(),
                    "string" => default_val.is_string(),
                    _ => true,
                };
                if !valid {
                    println!(
                        "  \u{274c} category '{}' の field '{}' の default 値が type '{}' と不整合です",
                        cat.id, field.key, type_str
                    );
                    errors += 1;
                    default_ok = false;
                }
            }
        }
    }
    if default_ok {
        println!("  \u{2705} default 型整合性 OK");
    }

    if errors == 0 {
        println!("\nバリデーション完了: エラーなし");
    } else {
        println!("\nバリデーション完了: {errors} 件のエラー");
    }

    Ok(errors)
}

// ============================================================================
// テスト
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_yaml(content: &str) -> NamedTempFile {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(content.as_bytes()).unwrap();
        f
    }

    #[test]
    fn test_validate_valid_schema() {
        let yaml = r#"
version: 1
service: my-service
namespace_prefix: service.my_service
categories:
  - id: general
    label: General
    namespaces:
      - service.my_service.general
    fields:
      - key: flag
        label: Flag
        type: boolean
        default: false
"#;
        let f = write_yaml(yaml);
        let errors = validate_config_schema(f.path().to_str().unwrap()).unwrap();
        assert_eq!(errors, 0);
    }

    #[test]
    fn test_validate_missing_type() {
        let yaml = r#"
version: 1
service: my-service
namespace_prefix: service.my_service
categories:
  - id: db
    label: Database
    namespaces:
      - service.my_service.db
    fields:
      - key: max_retry
        label: Max Retry
"#;
        let f = write_yaml(yaml);
        let errors = validate_config_schema(f.path().to_str().unwrap()).unwrap();
        assert!(errors > 0);
    }

    #[test]
    fn test_validate_namespace_mismatch() {
        let yaml = r#"
version: 1
service: my-service
namespace_prefix: service.my_service
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
        let f = write_yaml(yaml);
        let errors = validate_config_schema(f.path().to_str().unwrap()).unwrap();
        assert!(errors > 0);
    }

    #[test]
    fn test_validate_duplicate_keys() {
        let yaml = r#"
version: 1
service: my-service
namespace_prefix: service.my_service
categories:
  - id: general
    label: General
    namespaces:
      - service.my_service.general
    fields:
      - key: flag
        label: Flag 1
        type: boolean
      - key: flag
        label: Flag 2
        type: boolean
"#;
        let f = write_yaml(yaml);
        let errors = validate_config_schema(f.path().to_str().unwrap()).unwrap();
        assert!(errors > 0);
    }

    #[test]
    fn test_validate_enum_missing_options() {
        let yaml = r#"
version: 1
service: my-service
namespace_prefix: service.my_service
categories:
  - id: general
    label: General
    namespaces:
      - service.my_service.general
    fields:
      - key: log_level
        label: Log Level
        type: enum
"#;
        let f = write_yaml(yaml);
        let errors = validate_config_schema(f.path().to_str().unwrap()).unwrap();
        assert!(errors > 0);
    }

    #[test]
    fn test_validate_default_type_mismatch() {
        let yaml = r#"
version: 1
service: my-service
namespace_prefix: service.my_service
categories:
  - id: general
    label: General
    namespaces:
      - service.my_service.general
    fields:
      - key: count
        label: Count
        type: integer
        default: "not_a_number"
"#;
        let f = write_yaml(yaml);
        let errors = validate_config_schema(f.path().to_str().unwrap()).unwrap();
        assert!(errors > 0);
    }

    #[test]
    fn test_validate_invalid_yaml() {
        let yaml = "{{{{ invalid yaml ::::";
        let f = write_yaml(yaml);
        let errors = validate_config_schema(f.path().to_str().unwrap()).unwrap();
        assert!(errors > 0);
    }
}

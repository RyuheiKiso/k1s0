use super::super::validate::config_schema::ConfigSchemaYaml;

/// TypeScript型定義を生成
pub fn generate_typescript_types(schema: &ConfigSchemaYaml) -> String {
    let mut out = String::new();
    out.push_str(
        "// src/config/__generated__/config-types.ts\n\
         // このファイルは CLI が自動生成する。直接編集しないこと。\n\
         // k1s0 generate config-types で再生成できます。\n\n",
    );

    // ConfigKeys const
    out.push_str("export const ConfigKeys = {\n");
    for cat in &schema.categories {
        for field in &cat.fields {
            let const_name = field.key.to_uppercase();
            // namespace は先頭のものを使用
            let ns = cat.namespaces.first().map_or("", String::as_str);
            out.push_str(&format!(
                "  {const_name}: \"{ns}.{}\",\n",
                field.key
            ));
        }
    }
    out.push_str("} as const;\n\n");

    // ConfigValues type
    out.push_str("export type ConfigValues = {\n");
    for cat in &schema.categories {
        for field in &cat.fields {
            let ts_type = match field.field_type.as_deref().unwrap_or("string") {
                "string" => "string".to_string(),
                "integer" | "float" => "number".to_string(),
                "boolean" => "boolean".to_string(),
                "enum" => {
                    if let Some(ref opts) = field.options {
                        opts.iter()
                            .map(|o| format!("\"{o}\""))
                            .collect::<Vec<_>>()
                            .join(" | ")
                    } else {
                        "string".to_string()
                    }
                }
                "object" => "Record<string, unknown>".to_string(),
                "array" => "unknown[]".to_string(),
                other => other.to_string(),
            };
            let ns = cat.namespaces.first().map_or("", String::as_str);
            out.push_str(&format!(
                "  \"{ns}.{}\": {ts_type};\n",
                field.key
            ));
        }
    }
    out.push_str("};\n");

    out
}

/// Dart型定義を生成
pub fn generate_dart_types(schema: &ConfigSchemaYaml) -> String {
    let mut out = String::new();
    out.push_str(
        "// lib/config/__generated__/config_types.dart\n\
         // このファイルは CLI が自動生成する。直接編集しないこと。\n\
         // k1s0 generate config-types で再生成できます。\n\n",
    );

    // enum ConfigKey
    out.push_str("enum ConfigKey {\n");
    for cat in &schema.categories {
        for field in &cat.fields {
            let ns = cat.namespaces.first().map_or("", String::as_str);
            out.push_str(&format!(
                "  {}('{}'),\n",
                to_camel_case(&field.key),
                format!("{ns}.{}", field.key)
            ));
        }
    }
    out.push_str("  ;\n\n");
    out.push_str("  const ConfigKey(this.key);\n");
    out.push_str("  final String key;\n");
    out.push_str("}\n");

    out
}

/// config server にスキーマを push する (同期バージョン)
///
/// 実際の HTTP リクエストは reqwest 等の依存が必要なため、
/// ここではリクエスト情報の構築までを行う。
pub fn build_push_request(
    schema: &ConfigSchemaYaml,
    server_url: &str,
    token: &str,
) -> Result<(String, String, Vec<(String, String)>, String), Box<dyn std::error::Error>> {
    let url = format!(
        "{}/api/v1/config-schema/{}",
        server_url.trim_end_matches('/'),
        schema.service
    );
    let body = serde_yaml::to_string(schema)?;
    let headers = vec![
        ("Authorization".to_string(), format!("Bearer {token}")),
        ("Content-Type".to_string(), "application/yaml".to_string()),
    ];
    Ok(("PUT".to_string(), url, headers, body))
}

fn to_camel_case(snake: &str) -> String {
    let mut result = String::new();
    let mut capitalize = false;
    for ch in snake.chars() {
        if ch == '_' {
            capitalize = true;
        } else if capitalize {
            result.push(ch.to_uppercase().next().unwrap_or(ch));
            capitalize = false;
        } else {
            result.push(ch);
        }
    }
    result
}

// ============================================================================
// テスト
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::validate::config_schema::{CategoryYaml, FieldYaml};

    fn sample_schema() -> ConfigSchemaYaml {
        ConfigSchemaYaml {
            version: 1,
            service: "my-service".to_string(),
            namespace_prefix: "service.my_service".to_string(),
            categories: vec![CategoryYaml {
                id: "general".to_string(),
                label: "General".to_string(),
                icon: Some("settings".to_string()),
                namespaces: vec!["service.my_service.general".to_string()],
                fields: vec![
                    FieldYaml {
                        key: "enable_feature".to_string(),
                        label: "Enable Feature".to_string(),
                        description: None,
                        field_type: Some("boolean".to_string()),
                        min: None,
                        max: None,
                        options: None,
                        pattern: None,
                        unit: None,
                        default: Some(serde_yaml::Value::Bool(false)),
                    },
                    FieldYaml {
                        key: "max_connections".to_string(),
                        label: "Max Connections".to_string(),
                        description: None,
                        field_type: Some("integer".to_string()),
                        min: Some(1.0),
                        max: Some(100.0),
                        options: None,
                        pattern: None,
                        unit: None,
                        default: Some(serde_yaml::Value::Number(serde_yaml::Number::from(10))),
                    },
                    FieldYaml {
                        key: "log_level".to_string(),
                        label: "Log Level".to_string(),
                        description: None,
                        field_type: Some("enum".to_string()),
                        min: None,
                        max: None,
                        options: Some(vec![
                            "debug".to_string(),
                            "info".to_string(),
                            "warn".to_string(),
                            "error".to_string(),
                        ]),
                        pattern: None,
                        unit: None,
                        default: None,
                    },
                ],
            }],
        }
    }

    #[test]
    fn test_generate_typescript_types() {
        let schema = sample_schema();
        let ts = generate_typescript_types(&schema);
        assert!(ts.contains("export const ConfigKeys"));
        assert!(ts.contains("ENABLE_FEATURE"));
        assert!(ts.contains("MAX_CONNECTIONS"));
        assert!(ts.contains("LOG_LEVEL"));
        assert!(ts.contains("export type ConfigValues"));
        assert!(ts.contains("boolean"));
        assert!(ts.contains("number"));
        assert!(ts.contains("\"debug\" | \"info\" | \"warn\" | \"error\""));
    }

    #[test]
    fn test_generate_dart_types() {
        let schema = sample_schema();
        let dart = generate_dart_types(&schema);
        assert!(dart.contains("enum ConfigKey"));
        assert!(dart.contains("enableFeature"));
        assert!(dart.contains("maxConnections"));
        assert!(dart.contains("logLevel"));
        assert!(dart.contains("final String key"));
    }

    #[test]
    fn test_build_push_request() {
        let schema = sample_schema();
        let (method, url, headers, body) =
            build_push_request(&schema, "https://config.example.com", "test-token").unwrap();
        assert_eq!(method, "PUT");
        assert_eq!(
            url,
            "https://config.example.com/api/v1/config-schema/my-service"
        );
        assert!(headers
            .iter()
            .any(|(k, v)| k == "Authorization" && v == "Bearer test-token"));
        assert!(!body.is_empty());
    }

    #[test]
    fn test_to_camel_case() {
        assert_eq!(to_camel_case("enable_feature"), "enableFeature");
        assert_eq!(to_camel_case("max_connections"), "maxConnections");
        assert_eq!(to_camel_case("simple"), "simple");
        assert_eq!(to_camel_case("a_b_c"), "aBC");
    }
}

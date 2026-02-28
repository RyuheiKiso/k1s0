use super::super::validate::config_schema::ConfigSchemaYaml;

/// TypeScript型定義を生成
pub fn generate_typescript_types(schema: &ConfigSchemaYaml) -> String {
    let mut out = String::new();
    out.push_str(
        "// src/config/__generated__/config-types.ts\n\
         // このファイルは CLI が自動生成する。直接編集しないこと。\n\
         // k1s0 generate config-types で再生成できます。\n\n",
    );

    // ConfigKeys const — カテゴリ別 nested オブジェクト
    out.push_str("export const ConfigKeys = {\n");
    for cat in &schema.categories {
        let cat_const = cat.id.to_uppercase().replace('-', "_");
        out.push_str(&format!("  {cat_const}: {{\n"));
        for field in &cat.fields {
            let field_const = field.key.to_uppercase().replace('-', "_");
            out.push_str(&format!("    {field_const}: '{}',\n", field.key));
        }
        out.push_str("  },\n");
    }
    out.push_str("} as const;\n\n");

    // ConfigValues type — '{category_id}.{field_key}': type
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
            out.push_str(&format!("  '{}.{}': {ts_type};\n", cat.id, field.key));
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

    for cat in &schema.categories {
        let cat_pascal = to_pascal_case(&cat.id);

        // カテゴリ別 enum
        out.push_str(&format!("enum {cat_pascal}ConfigKey {{\n"));
        for (i, field) in cat.fields.iter().enumerate() {
            let camel = to_camel_case(&field.key);
            if i < cat.fields.len() - 1 {
                out.push_str(&format!("  {camel},\n"));
            } else {
                out.push_str(&format!("  {camel};\n"));
            }
        }
        out.push_str(&format!("\n  String get key => switch (this) {{\n"));
        for field in &cat.fields {
            let camel = to_camel_case(&field.key);
            out.push_str(&format!(
                "    {cat_pascal}ConfigKey.{camel} => '{}',\n",
                field.key
            ));
        }
        out.push_str("  };\n");
        out.push_str("}\n\n");

        // enum 型フィールドの値 enum
        for field in &cat.fields {
            if field.field_type.as_deref() == Some("enum") {
                if let Some(ref opts) = field.options {
                    let enum_name = to_pascal_case(&field.key);
                    let values: Vec<String> = opts.iter().map(|o| to_camel_case(o)).collect();
                    out.push_str(&format!("enum {enum_name} {{ {} }}\n\n", values.join(", ")));
                }
            }
        }
    }

    out
}

fn to_pascal_case(snake: &str) -> String {
    snake
        .split('_')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect()
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

/// ファイルパスから TypeScript 型定義を生成する
pub fn generate_typescript_types_from_file(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let schema: ConfigSchemaYaml = serde_yaml::from_str(&content)?;
    Ok(generate_typescript_types(&schema))
}

/// ファイルパスから Dart 型定義を生成する
pub fn generate_dart_types_from_file(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let schema: ConfigSchemaYaml = serde_yaml::from_str(&content)?;
    Ok(generate_dart_types(&schema))
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
        // ConfigKeys はカテゴリ別 nested 構造
        assert!(ts.contains("export const ConfigKeys"));
        assert!(ts.contains("GENERAL: {"));
        assert!(ts.contains("    ENABLE_FEATURE: 'enable_feature'"));
        assert!(ts.contains("    MAX_CONNECTIONS: 'max_connections'"));
        assert!(ts.contains("    LOG_LEVEL: 'log_level'"));
        // ConfigValues は '{category_id}.{field_key}' 形式
        assert!(ts.contains("export type ConfigValues"));
        assert!(ts.contains("'general.enable_feature': boolean"));
        assert!(ts.contains("'general.max_connections': number"));
        assert!(ts.contains("\"debug\" | \"info\" | \"warn\" | \"error\""));
    }

    #[test]
    fn test_generate_dart_types() {
        let schema = sample_schema();
        let dart = generate_dart_types(&schema);
        // カテゴリ別 enum
        assert!(dart.contains("enum GeneralConfigKey {"));
        assert!(dart.contains("  enableFeature,"));
        assert!(dart.contains("  maxConnections,"));
        assert!(dart.contains("  logLevel;"));
        // String get key
        assert!(dart.contains("String get key => switch (this) {"));
        assert!(dart.contains("GeneralConfigKey.enableFeature => 'enable_feature'"));
        assert!(dart.contains("GeneralConfigKey.maxConnections => 'max_connections'"));
        assert!(dart.contains("GeneralConfigKey.logLevel => 'log_level'"));
        // 値 enum (enum 型フィールド)
        assert!(dart.contains("enum LogLevel {"));
        assert!(dart.contains("debug, info, warn, error"));
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

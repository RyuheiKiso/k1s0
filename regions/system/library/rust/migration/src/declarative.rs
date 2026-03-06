use crate::error::MigrationError;
use serde::Deserialize;

#[derive(Deserialize)]
struct TableDef {
    table: TableSpec,
}

#[derive(Deserialize)]
struct TableSpec {
    name: String,
    columns: Vec<ColumnSpec>,
}

#[derive(Deserialize)]
struct ColumnSpec {
    name: String,
    #[serde(rename = "type")]
    data_type: String,
    #[serde(default)]
    primary_key: bool,
    #[serde(default = "default_true")]
    nullable: bool,
    #[serde(default)]
    default: Option<String>,
    #[serde(default)]
    unique: bool,
    #[serde(default)]
    references: Option<String>,
}

fn default_true() -> bool {
    true
}

/// TOML定義からCREATE TABLE SQLを生成
pub fn toml_to_create_sql(toml_str: &str) -> Result<String, MigrationError> {
    let def: TableDef = toml::from_str(toml_str)
        .map_err(|e| MigrationError::ParseError(format!("TOML parse error: {e}")))?;

    let mut col_defs = Vec::new();
    let mut primary_keys = Vec::new();

    for col in &def.table.columns {
        let mut parts = vec![format!("{} {}", col.name, col.data_type)];

        if !col.nullable && !col.primary_key {
            parts.push("NOT NULL".to_string());
        }

        if col.unique {
            parts.push("UNIQUE".to_string());
        }

        if let Some(ref default_val) = col.default {
            parts.push(format!("DEFAULT {default_val}"));
        }

        if let Some(ref references) = col.references {
            parts.push(format!("REFERENCES {references}"));
        }

        if col.primary_key {
            primary_keys.push(col.name.clone());
        }

        col_defs.push(parts.join(" "));
    }

    if !primary_keys.is_empty() {
        col_defs.push(format!("PRIMARY KEY ({})", primary_keys.join(", ")));
    }

    Ok(format!(
        "CREATE TABLE {} (\n  {}\n);",
        def.table.name,
        col_defs.join(",\n  ")
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_table() {
        let toml = r#"
[table]
name = "users"

[[table.columns]]
name = "id"
type = "UUID"
primary_key = true
nullable = false

[[table.columns]]
name = "name"
type = "TEXT"
nullable = false

[[table.columns]]
name = "email"
type = "TEXT"
nullable = true
unique = true
"#;
        let sql = toml_to_create_sql(toml).unwrap();
        assert!(sql.contains("CREATE TABLE users"));
        assert!(sql.contains("id UUID"));
        assert!(sql.contains("name TEXT NOT NULL"));
        assert!(sql.contains("email TEXT UNIQUE"));
        assert!(sql.contains("PRIMARY KEY (id)"));
    }

    #[test]
    fn test_column_with_default() {
        let toml = r#"
[table]
name = "settings"

[[table.columns]]
name = "active"
type = "BOOLEAN"
nullable = false
default = "true"
"#;
        let sql = toml_to_create_sql(toml).unwrap();
        assert!(sql.contains("active BOOLEAN NOT NULL DEFAULT true"));
    }

    #[test]
    fn test_column_with_references() {
        let toml = r#"
[table]
name = "orders"

[[table.columns]]
name = "id"
type = "UUID"
primary_key = true
nullable = false

[[table.columns]]
name = "user_id"
type = "UUID"
nullable = false
references = "users(id)"
"#;
        let sql = toml_to_create_sql(toml).unwrap();
        assert!(sql.contains("user_id UUID NOT NULL REFERENCES users(id)"));
    }

    #[test]
    fn test_invalid_toml() {
        let result = toml_to_create_sql("not valid toml {{{}}}");
        assert!(result.is_err());
    }

    #[test]
    fn test_composite_primary_key() {
        let toml = r#"
[table]
name = "order_items"

[[table.columns]]
name = "order_id"
type = "UUID"
primary_key = true
nullable = false

[[table.columns]]
name = "item_id"
type = "UUID"
primary_key = true
nullable = false

[[table.columns]]
name = "quantity"
type = "INT"
nullable = false
"#;
        let sql = toml_to_create_sql(toml).unwrap();
        assert!(sql.contains("PRIMARY KEY (order_id, item_id)"));
    }
}

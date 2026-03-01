use crate::domain::entity::column_definition::ColumnDefinition;
use crate::domain::entity::table_definition::TableDefinition;
use serde_json::Value;

pub struct SchemaGeneratorService;

impl SchemaGeneratorService {
    pub fn generate_json_schema(
        table_def: &TableDefinition,
        columns: &[ColumnDefinition],
    ) -> Value {
        // JSON Schema 生成ロジック
        let mut properties = serde_json::Map::new();
        let mut required = Vec::new();

        for col in columns {
            if col.is_visible_in_form {
                let field_schema = Self::column_to_json_schema(col);
                properties.insert(col.column_name.clone(), field_schema);
                if !col.is_nullable && !col.is_primary_key {
                    required.push(serde_json::Value::String(col.column_name.clone()));
                }
            }
        }

        serde_json::json!({
            "type": "object",
            "title": table_def.display_name,
            "properties": properties,
            "required": required,
        })
    }

    fn column_to_json_schema(col: &ColumnDefinition) -> Value {
        let mut schema = serde_json::Map::new();

        let json_type = match col.data_type.as_str() {
            "text" | "uuid" => "string",
            "integer" => "integer",
            "decimal" => "number",
            "boolean" => "boolean",
            "date" | "datetime" => "string",
            "jsonb" => "object",
            _ => "string",
        };
        schema.insert("type".to_string(), serde_json::Value::String(json_type.to_string()));
        schema.insert("title".to_string(), serde_json::Value::String(col.display_name.clone()));

        if let Some(max_len) = col.max_length {
            schema.insert("maxLength".to_string(), serde_json::Value::Number(max_len.into()));
        }
        if let Some(min) = col.min_value {
            schema.insert("minimum".to_string(), serde_json::json!(min));
        }
        if let Some(max) = col.max_value {
            schema.insert("maximum".to_string(), serde_json::json!(max));
        }
        if let Some(ref pattern) = col.regex_pattern {
            schema.insert("pattern".to_string(), serde_json::Value::String(pattern.clone()));
        }
        if col.data_type == "date" {
            schema.insert("format".to_string(), serde_json::Value::String("date".to_string()));
        } else if col.data_type == "datetime" {
            schema.insert("format".to_string(), serde_json::Value::String("date-time".to_string()));
        }

        serde_json::Value::Object(schema)
    }
}

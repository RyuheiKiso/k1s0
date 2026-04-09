use crate::domain::entity::column_definition::ColumnDefinition;
use crate::domain::entity::table_definition::TableDefinition;
use serde_json::Value;

pub struct SchemaGeneratorService;

impl SchemaGeneratorService {
    #[must_use]
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

        // "text"/"uuid"/"date"/"datetime" アームと wildcard アームが同一の返り値のため統合する
        let json_type = match col.data_type.as_str() {
            "integer" => "integer",
            "decimal" => "number",
            "boolean" => "boolean",
            "jsonb" => "object",
            _ => "string",
        };
        schema.insert(
            "type".to_string(),
            serde_json::Value::String(json_type.to_string()),
        );
        schema.insert(
            "title".to_string(),
            serde_json::Value::String(col.display_name.clone()),
        );

        if let Some(max_len) = col.max_length {
            schema.insert(
                "maxLength".to_string(),
                serde_json::Value::Number(max_len.into()),
            );
        }
        if let Some(min) = col.min_value {
            schema.insert("minimum".to_string(), serde_json::json!(min));
        }
        if let Some(max) = col.max_value {
            schema.insert("maximum".to_string(), serde_json::json!(max));
        }
        if let Some(ref pattern) = col.regex_pattern {
            schema.insert(
                "pattern".to_string(),
                serde_json::Value::String(pattern.clone()),
            );
        }
        if col.data_type == "date" {
            schema.insert(
                "format".to_string(),
                serde_json::Value::String("date".to_string()),
            );
        } else if col.data_type == "datetime" {
            schema.insert(
                "format".to_string(),
                serde_json::Value::String("date-time".to_string()),
            );
        }

        serde_json::Value::Object(schema)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::column_definition::ColumnDefinition;
    use crate::domain::entity::table_definition::TableDefinition;
    use chrono::Utc;
    use uuid::Uuid;

    fn sample_table(name: &str) -> TableDefinition {
        let now = Utc::now();
        TableDefinition {
            id: Uuid::new_v4(),
            name: name.to_string(),
            schema_name: "public".to_string(),
            database_name: "main".to_string(),
            display_name: name.to_string(),
            description: None,
            category: None,
            is_active: true,
            allow_create: true,
            allow_update: true,
            allow_delete: true,
            read_roles: vec![],
            write_roles: vec![],
            admin_roles: vec![],
            sort_order: 0,
            created_by: "system".to_string(),
            created_at: now,
            updated_at: now,
            domain_scope: None,
        }
    }

    fn sample_column(
        table_id: Uuid,
        name: &str,
        data_type: &str,
        is_form: bool,
    ) -> ColumnDefinition {
        let now = Utc::now();
        ColumnDefinition {
            id: Uuid::new_v4(),
            table_id,
            column_name: name.to_string(),
            display_name: name.to_string(),
            data_type: data_type.to_string(),
            is_primary_key: false,
            is_nullable: true,
            is_unique: false,
            default_value: None,
            max_length: None,
            min_value: None,
            max_value: None,
            regex_pattern: None,
            display_order: 0,
            is_searchable: false,
            is_sortable: false,
            is_filterable: false,
            is_visible_in_list: true,
            is_visible_in_form: is_form,
            is_readonly: false,
            input_type: "text".to_string(),
            select_options: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// カラムなしの場合はpropertiesが空のスキーマを生成する
    #[test]
    fn generate_json_schema_no_columns() {
        let table = sample_table("empty_table");
        let schema = SchemaGeneratorService::generate_json_schema(&table, &[]);
        assert_eq!(schema["type"], "object");
        assert_eq!(schema["title"], "empty_table");
        assert!(schema["properties"].as_object().unwrap().is_empty());
    }

    /// is_visible_in_form=trueのカラムのみスキーマに含まれる
    #[test]
    fn generate_json_schema_includes_only_form_visible_columns() {
        let table = sample_table("products");
        let col_visible = sample_column(table.id, "name", "text", true);
        let col_hidden = sample_column(table.id, "internal_code", "text", false);
        let schema =
            SchemaGeneratorService::generate_json_schema(&table, &[col_visible, col_hidden]);
        let props = schema["properties"].as_object().unwrap();
        assert!(props.contains_key("name"));
        assert!(!props.contains_key("internal_code"));
    }

    /// integerカラムのJSON Schema型がintegerになる
    #[test]
    fn generate_json_schema_integer_column_type() {
        let table = sample_table("orders");
        let col = sample_column(table.id, "quantity", "integer", true);
        let schema = SchemaGeneratorService::generate_json_schema(&table, &[col]);
        assert_eq!(schema["properties"]["quantity"]["type"], "integer");
    }

    /// dateカラムのフォーマットがdate、datetimeはdate-timeになる
    #[test]
    fn generate_json_schema_date_format() {
        let table = sample_table("events");
        let col_date = sample_column(table.id, "event_date", "date", true);
        let col_datetime = sample_column(table.id, "created_at", "datetime", true);
        let schema =
            SchemaGeneratorService::generate_json_schema(&table, &[col_date, col_datetime]);
        assert_eq!(schema["properties"]["event_date"]["format"], "date");
        assert_eq!(schema["properties"]["created_at"]["format"], "date-time");
    }
}

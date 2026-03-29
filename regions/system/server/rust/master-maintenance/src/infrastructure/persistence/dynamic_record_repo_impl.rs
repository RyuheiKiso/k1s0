use crate::domain::entity::column_definition::ColumnDefinition;
use crate::domain::entity::table_definition::TableDefinition;
use crate::domain::repository::dynamic_record_repository::DynamicRecordRepository;
use async_trait::async_trait;
// M-02 監査対応: LIKE/ILIKE ワイルドカードエスケープのため server-common のユーティリティを使用する
use k1s0_server_common::escape_like_pattern;
use serde_json::Value;
use sqlx::{postgres::PgRow, PgPool, Row};

pub struct DynamicRecordPostgresRepository {
    pool: PgPool,
}

impl DynamicRecordPostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// Quote a SQL identifier to prevent injection.
/// Doubles any internal double-quotes and wraps in double-quotes.
fn quote_identifier(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

/// Validate that a name is a safe SQL identifier (alphanumeric + underscore).
fn validate_identifier(name: &str) -> anyhow::Result<()> {
    if name.is_empty() {
        anyhow::bail!("Identifier cannot be empty");
    }
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        anyhow::bail!("Invalid identifier: {}", name);
    }
    Ok(())
}

/// Build the fully qualified table name with quoting.
fn build_table_name(table_def: &TableDefinition) -> anyhow::Result<String> {
    validate_identifier(&table_def.schema_name)?;
    validate_identifier(&table_def.name)?;
    Ok(format!(
        "{}.{}",
        quote_identifier(&table_def.schema_name),
        quote_identifier(&table_def.name)
    ))
}

fn postgres_cast_type(data_type: &str) -> anyhow::Result<&'static str> {
    match data_type.to_lowercase().as_str() {
        "uuid" => Ok("uuid"),
        "integer" | "int" | "int4" | "serial" => Ok("integer"),
        "bigint" | "int8" | "bigserial" => Ok("bigint"),
        "smallint" | "int2" => Ok("smallint"),
        "boolean" | "bool" => Ok("boolean"),
        "real" | "float4" => Ok("real"),
        "double precision" | "float8" => Ok("double precision"),
        "numeric" | "decimal" => Ok("numeric"),
        "json" => Ok("json"),
        "jsonb" => Ok("jsonb"),
        "date" => Ok("date"),
        "timestamp"
        | "timestamptz"
        | "timestamp with time zone"
        | "timestamp without time zone"
        | "datetime" => Ok("timestamptz"),
        "text" => Ok("text"),
        other => anyhow::bail!("unsupported column data type for SQL casting: {}", other),
    }
}

fn typed_placeholder(column: &ColumnDefinition, param_idx: u32) -> anyhow::Result<String> {
    Ok(format!(
        "CAST(${} AS {})",
        param_idx,
        postgres_cast_type(&column.data_type)?
    ))
}

/// Find the primary key column from column definitions.
fn find_primary_key_column(columns: &[ColumnDefinition]) -> anyhow::Result<&ColumnDefinition> {
    columns
        .iter()
        .find(|c| c.is_primary_key)
        .ok_or_else(|| anyhow::anyhow!("No primary key column found"))
}

/// Convert a PgRow to a serde_json::Value object using column definitions.
fn row_to_json(row: &PgRow, columns: &[ColumnDefinition]) -> Value {
    let mut map = serde_json::Map::new();
    for col in columns {
        let col_name = col.column_name.as_str();
        // Try to get the value based on data type
        let value = match col.data_type.to_lowercase().as_str() {
            "uuid" => row
                .try_get::<uuid::Uuid, _>(col_name)
                .map(|v| Value::String(v.to_string()))
                .unwrap_or(Value::Null),
            "integer" | "int" | "int4" | "serial" => row
                .try_get::<i32, _>(col_name)
                .map(|v| Value::Number(v.into()))
                .unwrap_or(Value::Null),
            "bigint" | "int8" | "bigserial" => row
                .try_get::<i64, _>(col_name)
                .map(|v| Value::Number(v.into()))
                .unwrap_or(Value::Null),
            "smallint" | "int2" => row
                .try_get::<i16, _>(col_name)
                .map(|v| Value::Number(v.into()))
                .unwrap_or(Value::Null),
            "boolean" | "bool" => row
                .try_get::<bool, _>(col_name)
                .map(Value::Bool)
                .unwrap_or(Value::Null),
            "real" | "float4" => row
                .try_get::<f32, _>(col_name)
                .map(|v| {
                    serde_json::Number::from_f64(v as f64)
                        .map(Value::Number)
                        .unwrap_or(Value::Null)
                })
                .unwrap_or(Value::Null),
            "double precision" | "float8" | "numeric" | "decimal" => row
                .try_get::<f64, _>(col_name)
                .map(|v| {
                    serde_json::Number::from_f64(v)
                        .map(Value::Number)
                        .unwrap_or(Value::Null)
                })
                .unwrap_or(Value::Null),
            "json" | "jsonb" => row.try_get::<Value, _>(col_name).unwrap_or(Value::Null),
            "timestamp"
            | "timestamptz"
            | "timestamp with time zone"
            | "timestamp without time zone" => row
                .try_get::<chrono::DateTime<chrono::Utc>, _>(col_name)
                .map(|v| Value::String(v.to_rfc3339()))
                .or_else(|_| {
                    row.try_get::<chrono::NaiveDateTime, _>(col_name)
                        .map(|v| Value::String(v.to_string()))
                })
                .unwrap_or(Value::Null),
            "date" => row
                .try_get::<chrono::NaiveDate, _>(col_name)
                .map(|v| Value::String(v.to_string()))
                .unwrap_or(Value::Null),
            _ => {
                // Default: try as string
                row.try_get::<String, _>(col_name)
                    .map(Value::String)
                    .unwrap_or(Value::Null)
            }
        };
        map.insert(col_name.to_string(), value);
    }
    Value::Object(map)
}

#[async_trait]
impl DynamicRecordRepository for DynamicRecordPostgresRepository {
    async fn find_all(
        &self,
        table_def: &TableDefinition,
        columns: &[ColumnDefinition],
        page: i32,
        page_size: i32,
        sort: Option<&str>,
        filter: Option<&str>,
        search: Option<&str>,
    ) -> anyhow::Result<(Vec<Value>, i64)> {
        let table_name = build_table_name(table_def)?;
        let offset = (page - 1).max(0) * page_size;

        // Build column list
        let col_list: Vec<String> = columns
            .iter()
            .map(|c| quote_identifier(&c.column_name))
            .collect();
        let select_cols = col_list.join(", ");

        // Build WHERE clause
        let mut where_clauses: Vec<String> = Vec::new();
        let mut bind_values: Vec<String> = Vec::new();
        let mut param_idx = 1u32;

        // Filter: "column:value" format
        if let Some(f) = filter {
            for part in f.split(',') {
                if let Some((col_name, val)) = part.split_once(':') {
                    let col_name = col_name.trim();
                    let val = val.trim();
                    // カラム定義に存在することを確認済みなので find で取得する
                    // any() で存在確認済みだが、万が一見つからない場合はエラーとして伝播する
                    if columns.iter().any(|c| c.column_name == col_name) {
                        let column = columns
                            .iter()
                            .find(|c| c.column_name == col_name)
                            .ok_or_else(|| {
                                anyhow::anyhow!("カラム '{}' が定義に見つかりません", col_name)
                            })?;
                        validate_identifier(col_name)?;
                        where_clauses.push(format!(
                            "{} = {}",
                            quote_identifier(col_name),
                            typed_placeholder(column, param_idx)?
                        ));
                        bind_values.push(val.to_string());
                        param_idx += 1;
                    }
                }
            }
        }

        // Search: ILIKE on searchable columns
        // M-02 監査対応: ワイルドカード特殊文字（\, %, _）をエスケープし意図しない全件マッチを防ぐ
        if let Some(s) = search {
            if !s.is_empty() {
                let searchable: Vec<String> = columns
                    .iter()
                    .filter(|c| c.is_searchable)
                    .map(|c| {
                        format!(
                            "{}::text ILIKE ${} ESCAPE '\\\\'",
                            quote_identifier(&c.column_name),
                            param_idx
                        )
                    })
                    .collect();
                if !searchable.is_empty() {
                    where_clauses.push(format!("({})", searchable.join(" OR ")));
                    // H-02 監査対応: needless_borrow 修正 - &s は自動 deref されるため s を直接渡す
                    bind_values.push(format!("%{}%", escape_like_pattern(s)));
                }
            }
        }

        let where_sql = if where_clauses.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", where_clauses.join(" AND "))
        };

        // Build ORDER BY clause
        let order_sql = if let Some(s) = sort {
            let mut parts = Vec::new();
            for item in s.split(',') {
                let item = item.trim();
                let (col_name, direction) = if let Some(stripped) = item.strip_prefix('-') {
                    (stripped, "DESC")
                } else {
                    (item, "ASC")
                };
                if columns
                    .iter()
                    .any(|c| c.column_name == col_name && c.is_sortable)
                {
                    validate_identifier(col_name)?;
                    parts.push(format!("{} {}", quote_identifier(col_name), direction));
                }
            }
            if parts.is_empty() {
                String::new()
            } else {
                format!(" ORDER BY {}", parts.join(", "))
            }
        } else {
            String::new()
        };

        // Count query
        let count_sql = format!("SELECT COUNT(*) FROM {}{}", table_name, where_sql);
        let mut count_q = sqlx::query_scalar::<_, i64>(&count_sql);
        for val in &bind_values {
            count_q = count_q.bind(val.clone());
        }
        let total = count_q.fetch_one(&self.pool).await?;

        // Data query
        let data_sql = format!(
            "SELECT {} FROM {}{}{} LIMIT {} OFFSET {}",
            select_cols, table_name, where_sql, order_sql, page_size, offset
        );
        let mut data_q = sqlx::query(&data_sql);
        for val in &bind_values {
            data_q = data_q.bind(val.clone());
        }
        let rows = data_q.fetch_all(&self.pool).await?;
        let results: Vec<Value> = rows.iter().map(|r| row_to_json(r, columns)).collect();

        Ok((results, total))
    }

    async fn find_by_id(
        &self,
        table_def: &TableDefinition,
        columns: &[ColumnDefinition],
        record_id: &str,
    ) -> anyhow::Result<Option<Value>> {
        let table_name = build_table_name(table_def)?;
        let pk_col = find_primary_key_column(columns)?;
        validate_identifier(&pk_col.column_name)?;

        let col_list: Vec<String> = columns
            .iter()
            .map(|c| quote_identifier(&c.column_name))
            .collect();
        let select_cols = col_list.join(", ");

        let sql = format!(
            "SELECT {} FROM {} WHERE {} = CAST($1 AS {})",
            select_cols,
            table_name,
            quote_identifier(&pk_col.column_name),
            postgres_cast_type(&pk_col.data_type)?
        );

        let row = sqlx::query(&sql)
            .bind(record_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.as_ref().map(|r| row_to_json(r, columns)))
    }

    async fn create(
        &self,
        table_def: &TableDefinition,
        columns: &[ColumnDefinition],
        data: &Value,
    ) -> anyhow::Result<Value> {
        let table_name = build_table_name(table_def)?;
        let obj = data
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("Data must be a JSON object"))?;

        let mut col_names: Vec<String> = Vec::new();
        let mut placeholders: Vec<String> = Vec::new();
        // NULL 値は Option::None として保持し、SQL NULL でバインドする
        let mut values: Vec<Option<String>> = Vec::new();
        let mut param_idx = 1u32;

        for col in columns {
            if col.is_readonly && !col.is_primary_key {
                continue;
            }
            if let Some(val) = obj.get(&col.column_name) {
                // NULL でも明示的に指定された場合はカラムに含める（SQL NULL を挿入）
                validate_identifier(&col.column_name)?;
                col_names.push(quote_identifier(&col.column_name));
                placeholders.push(typed_placeholder(col, param_idx)?);
                values.push(json_value_to_string(val));
                param_idx += 1;
            }
        }

        if col_names.is_empty() {
            anyhow::bail!("No valid columns provided for insert");
        }

        let returning_cols: Vec<String> = columns
            .iter()
            .map(|c| quote_identifier(&c.column_name))
            .collect();

        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({}) RETURNING {}",
            table_name,
            col_names.join(", "),
            placeholders.join(", "),
            returning_cols.join(", ")
        );

        let mut q = sqlx::query(&sql);
        for val in &values {
            q = q.bind(val.clone());
        }

        let row = q.fetch_one(&self.pool).await?;
        Ok(row_to_json(&row, columns))
    }

    async fn update(
        &self,
        table_def: &TableDefinition,
        columns: &[ColumnDefinition],
        record_id: &str,
        data: &Value,
    ) -> anyhow::Result<Value> {
        let table_name = build_table_name(table_def)?;
        let pk_col = find_primary_key_column(columns)?;
        validate_identifier(&pk_col.column_name)?;

        let obj = data
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("Data must be a JSON object"))?;

        let mut set_clauses: Vec<String> = Vec::new();
        // NULL 値は Option::None として保持し、SQL NULL でバインドする
        let mut values: Vec<Option<String>> = Vec::new();
        let mut param_idx = 1u32;

        for col in columns {
            if col.is_primary_key || col.is_readonly {
                continue;
            }
            if let Some(val) = obj.get(&col.column_name) {
                validate_identifier(&col.column_name)?;
                set_clauses.push(format!(
                    "{} = {}",
                    quote_identifier(&col.column_name),
                    typed_placeholder(col, param_idx)?
                ));
                // json_value_to_string が None を返す場合は SQL NULL としてバインドする
                values.push(json_value_to_string(val));
                param_idx += 1;
            }
        }

        if set_clauses.is_empty() {
            anyhow::bail!("No valid columns provided for update");
        }

        let returning_cols: Vec<String> = columns
            .iter()
            .map(|c| quote_identifier(&c.column_name))
            .collect();

        let sql = format!(
            "UPDATE {} SET {} WHERE {} = CAST(${} AS {}) RETURNING {}",
            table_name,
            set_clauses.join(", "),
            quote_identifier(&pk_col.column_name),
            param_idx,
            postgres_cast_type(&pk_col.data_type)?,
            returning_cols.join(", ")
        );
        // WHERE 句のプレースホルダー用に record_id を Option でラップしてバインドリストへ追加する
        values.push(Some(record_id.to_string()));

        let mut q = sqlx::query(&sql);
        for val in &values {
            q = q.bind(val.clone());
        }

        let row = q.fetch_one(&self.pool).await?;
        Ok(row_to_json(&row, columns))
    }

    async fn delete(&self, table_def: &TableDefinition, record_id: &str) -> anyhow::Result<()> {
        let table_name = build_table_name(table_def)?;

        // We need to find the PK column name. Since we don't have columns here,
        // we query the column_definitions table to find it.
        let pk_col_name: Option<String> = sqlx::query_scalar(
            "SELECT column_name FROM master_maintenance.column_definitions WHERE table_id = $1 AND is_primary_key = true LIMIT 1"
        )
        .bind(table_def.id)
        .fetch_optional(&self.pool)
        .await?;

        let pk_col = pk_col_name.ok_or_else(|| {
            anyhow::anyhow!("No primary key column found for table {}", table_def.name)
        })?;
        validate_identifier(&pk_col)?;

        let sql = format!(
            "DELETE FROM {} WHERE {} = CAST($1 AS {})",
            table_name,
            quote_identifier(&pk_col),
            postgres_cast_type(
                &sqlx::query_scalar::<_, String>(
                    "SELECT data_type FROM master_maintenance.column_definitions WHERE table_id = $1 AND column_name = $2 LIMIT 1"
                )
                .bind(table_def.id)
                .bind(&pk_col)
                .fetch_one(&self.pool)
                .await?
            )?
        );
        sqlx::query(&sql)
            .bind(record_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

/// JSON 値を SQL バインド用の文字列に変換する。
/// Value::Null の場合は None を返し、呼び出し元が SQL NULL としてバインドする。
fn json_value_to_string(val: &Value) -> Option<String> {
    match val {
        Value::Null => None,
        Value::String(s) => Some(s.clone()),
        Value::Bool(b) => Some(b.to_string()),
        Value::Number(n) => Some(n.to_string()),
        // 配列・オブジェクトは JSON 文字列として扱う
        _ => Some(val.to_string()),
    }
}

use crate::domain::entity::column_definition::{ColumnDefinition, CreateColumnDefinition};
use crate::domain::entity::table_definition::{CreateTableDefinition, TableDefinition};
use crate::domain::entity::table_relationship::TableRelationship;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

/// `SchemaManager` はテーブル・カラム・リレーションシップの物理スキーマ操作を抽象化するトレイト。
/// テスト時にスタブ実装で差し替えられるように定義する。
#[async_trait]
pub trait SchemaManager: Send + Sync {
    /// テーブルを物理的に作成する。
    async fn create_table(&self, input: &CreateTableDefinition) -> anyhow::Result<()>;
    /// テーブルを物理的に削除する。
    async fn delete_table(&self, table: &TableDefinition) -> anyhow::Result<()>;
    /// カラムをテーブルに追加する。
    async fn add_columns(
        &self,
        table: &TableDefinition,
        columns: &[CreateColumnDefinition],
    ) -> anyhow::Result<()>;
    /// カラム定義を更新する。
    async fn update_column(
        &self,
        table: &TableDefinition,
        existing: &ColumnDefinition,
        input: &CreateColumnDefinition,
    ) -> anyhow::Result<()>;
    /// カラムを削除する。
    async fn delete_column(&self, table: &TableDefinition, column_name: &str)
        -> anyhow::Result<()>;
    /// テーブル間リレーションシップを作成する。
    async fn create_relationship(
        &self,
        source_table: &TableDefinition,
        target_table: &TableDefinition,
        relationship: &TableRelationship,
    ) -> anyhow::Result<()>;
    /// テーブル間リレーションシップを更新する。
    async fn update_relationship(
        &self,
        source_table: &TableDefinition,
        target_table: &TableDefinition,
        relationship: &TableRelationship,
    ) -> anyhow::Result<()>;
    /// テーブル間リレーションシップを削除する。
    async fn delete_relationship(
        &self,
        source_table: &TableDefinition,
        relationship_id: Uuid,
    ) -> anyhow::Result<()>;
}

/// RUST-002 監査対応: `on_delete` を enum 型で定義し、型安全性を確保する
enum OnDeleteAction {
    Cascade,
    Restrict,
}

impl OnDeleteAction {
    fn as_sql(&self) -> &'static str {
        match self {
            Self::Cascade => "CASCADE",
            Self::Restrict => "RESTRICT",
        }
    }
}

pub struct PhysicalSchemaManager {
    pool: PgPool,
}

impl PhysicalSchemaManager {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_table(&self, input: &CreateTableDefinition) -> anyhow::Result<()> {
        validate_identifier(&input.schema_name)?;
        validate_identifier(&input.name)?;

        let create_schema_sql = format!(
            "CREATE SCHEMA IF NOT EXISTS {}",
            quote_identifier(&input.schema_name)
        );
        sqlx::query(&create_schema_sql).execute(&self.pool).await?;

        let create_table_sql = format!(
            "CREATE TABLE IF NOT EXISTS {}.{} ()",
            quote_identifier(&input.schema_name),
            quote_identifier(&input.name)
        );
        sqlx::query(&create_table_sql).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn delete_table(&self, table: &TableDefinition) -> anyhow::Result<()> {
        let qualified_table = qualified_table_name(table)?;
        let sql = format!("DROP TABLE IF EXISTS {qualified_table} CASCADE");
        sqlx::query(&sql).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn add_columns(
        &self,
        table: &TableDefinition,
        columns: &[CreateColumnDefinition],
    ) -> anyhow::Result<()> {
        let qualified_table = qualified_table_name(table)?;

        for column in columns {
            validate_identifier(&column.column_name)?;
            let data_type = postgres_type(&column.data_type)?;

            let mut sql = format!(
                "ALTER TABLE {} ADD COLUMN IF NOT EXISTS {} {}",
                qualified_table,
                quote_identifier(&column.column_name),
                data_type
            );

            if let Some(default_sql) = default_value_sql(column)? {
                sql.push_str(" DEFAULT ");
                sql.push_str(&default_sql);
            }

            if !column.is_nullable.unwrap_or(true) || column.is_primary_key.unwrap_or(false) {
                sql.push_str(" NOT NULL");
            }

            sqlx::query(&sql).execute(&self.pool).await?;

            if column.is_unique.unwrap_or(false) {
                let unique_constraint =
                    format!("{}_{}_unique", table.name, column.column_name).replace('-', "_");
                let unique_sql = format!(
                    "ALTER TABLE {} ADD CONSTRAINT {} UNIQUE ({})",
                    qualified_table,
                    quote_identifier(&unique_constraint),
                    quote_identifier(&column.column_name)
                );
                let _ = sqlx::query(&unique_sql).execute(&self.pool).await;
            }

            if column.is_primary_key.unwrap_or(false) {
                let pk_constraint =
                    format!("{}_{}_pk", table.name, column.column_name).replace('-', "_");
                let pk_sql = format!(
                    "ALTER TABLE {} ADD CONSTRAINT {} PRIMARY KEY ({})",
                    qualified_table,
                    quote_identifier(&pk_constraint),
                    quote_identifier(&column.column_name)
                );
                let _ = sqlx::query(&pk_sql).execute(&self.pool).await;
            }
        }

        Ok(())
    }

    pub async fn update_column(
        &self,
        table: &TableDefinition,
        existing: &ColumnDefinition,
        input: &CreateColumnDefinition,
    ) -> anyhow::Result<()> {
        let qualified_table = qualified_table_name(table)?;
        validate_identifier(&existing.column_name)?;
        let quoted_column = quote_identifier(&existing.column_name);

        if existing.data_type != input.data_type {
            let sql = format!(
                "ALTER TABLE {} ALTER COLUMN {} TYPE {} USING {}::{}",
                qualified_table,
                quoted_column,
                postgres_type(&input.data_type)?,
                quoted_column,
                postgres_type(&input.data_type)?
            );
            sqlx::query(&sql).execute(&self.pool).await?;
        }

        if existing.default_value != input.default_value {
            let sql = if let Some(default_sql) = default_value_sql(input)? {
                format!(
                    "ALTER TABLE {qualified_table} ALTER COLUMN {quoted_column} SET DEFAULT {default_sql}"
                )
            } else {
                format!("ALTER TABLE {qualified_table} ALTER COLUMN {quoted_column} DROP DEFAULT")
            };
            sqlx::query(&sql).execute(&self.pool).await?;
        }

        if existing.is_nullable != input.is_nullable.unwrap_or(existing.is_nullable) {
            let sql = if input.is_nullable.unwrap_or(existing.is_nullable) {
                format!("ALTER TABLE {qualified_table} ALTER COLUMN {quoted_column} DROP NOT NULL")
            } else {
                format!("ALTER TABLE {qualified_table} ALTER COLUMN {quoted_column} SET NOT NULL")
            };
            sqlx::query(&sql).execute(&self.pool).await?;
        }

        Ok(())
    }

    pub async fn delete_column(
        &self,
        table: &TableDefinition,
        column_name: &str,
    ) -> anyhow::Result<()> {
        let qualified_table = qualified_table_name(table)?;
        validate_identifier(column_name)?;
        let sql = format!(
            "ALTER TABLE {} DROP COLUMN IF EXISTS {}",
            qualified_table,
            quote_identifier(column_name)
        );
        sqlx::query(&sql).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn create_relationship(
        &self,
        source_table: &TableDefinition,
        target_table: &TableDefinition,
        relationship: &TableRelationship,
    ) -> anyhow::Result<()> {
        let source_table_name = qualified_table_name(source_table)?;
        validate_identifier(&relationship.source_column)?;
        validate_identifier(&relationship.target_column)?;
        let target_table_name = qualified_table_name(target_table)?;
        let constraint_name = relationship_constraint_name(relationship.id);
        let on_delete = if relationship.is_cascade_delete {
            OnDeleteAction::Cascade
        } else {
            OnDeleteAction::Restrict
        };
        let sql = format!(
            "ALTER TABLE {} ADD CONSTRAINT {} FOREIGN KEY ({}) REFERENCES {} ({}) ON DELETE {}",
            source_table_name,
            quote_identifier(&constraint_name),
            quote_identifier(&relationship.source_column),
            target_table_name,
            quote_identifier(&relationship.target_column),
            on_delete.as_sql()
        );
        sqlx::query(&sql).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn update_relationship(
        &self,
        source_table: &TableDefinition,
        target_table: &TableDefinition,
        relationship: &TableRelationship,
    ) -> anyhow::Result<()> {
        self.delete_relationship(source_table, relationship.id)
            .await?;
        self.create_relationship(source_table, target_table, relationship)
            .await
    }

    pub async fn delete_relationship(
        &self,
        source_table: &TableDefinition,
        relationship_id: uuid::Uuid,
    ) -> anyhow::Result<()> {
        let source_table_name = qualified_table_name(source_table)?;
        let constraint_name = relationship_constraint_name(relationship_id);
        let sql = format!(
            "ALTER TABLE {} DROP CONSTRAINT IF EXISTS {}",
            source_table_name,
            quote_identifier(&constraint_name)
        );
        sqlx::query(&sql).execute(&self.pool).await?;
        Ok(())
    }
}

/// `PhysicalSchemaManager` に `SchemaManager` トレイトを実装する。
#[async_trait]
impl SchemaManager for PhysicalSchemaManager {
    async fn create_table(&self, input: &CreateTableDefinition) -> anyhow::Result<()> {
        Self::create_table(self, input).await
    }
    async fn delete_table(&self, table: &TableDefinition) -> anyhow::Result<()> {
        Self::delete_table(self, table).await
    }
    async fn add_columns(
        &self,
        table: &TableDefinition,
        columns: &[CreateColumnDefinition],
    ) -> anyhow::Result<()> {
        Self::add_columns(self, table, columns).await
    }
    async fn update_column(
        &self,
        table: &TableDefinition,
        existing: &ColumnDefinition,
        input: &CreateColumnDefinition,
    ) -> anyhow::Result<()> {
        Self::update_column(self, table, existing, input).await
    }
    async fn delete_column(
        &self,
        table: &TableDefinition,
        column_name: &str,
    ) -> anyhow::Result<()> {
        Self::delete_column(self, table, column_name).await
    }
    async fn create_relationship(
        &self,
        source_table: &TableDefinition,
        target_table: &TableDefinition,
        relationship: &TableRelationship,
    ) -> anyhow::Result<()> {
        Self::create_relationship(self, source_table, target_table, relationship).await
    }
    async fn update_relationship(
        &self,
        source_table: &TableDefinition,
        target_table: &TableDefinition,
        relationship: &TableRelationship,
    ) -> anyhow::Result<()> {
        Self::update_relationship(self, source_table, target_table, relationship).await
    }
    async fn delete_relationship(
        &self,
        source_table: &TableDefinition,
        relationship_id: Uuid,
    ) -> anyhow::Result<()> {
        Self::delete_relationship(self, source_table, relationship_id).await
    }
}

fn qualified_table_name(table: &TableDefinition) -> anyhow::Result<String> {
    validate_identifier(&table.schema_name)?;
    validate_identifier(&table.name)?;
    Ok(format!(
        "{}.{}",
        quote_identifier(&table.schema_name),
        quote_identifier(&table.name)
    ))
}

fn quote_identifier(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

/// `PostgreSQL` 識別子の最大長（バイト数）。
/// `PostgreSQL` は識別子を NAMEDATALEN-1 = 63 バイトに切り詰めるため、
/// それを超える識別子は予期しない重複の原因となる。
const PG_MAX_IDENTIFIER_LEN: usize = 63;

/// 識別子が `PostgreSQL` の命名規則に従っているか検証する。
/// CRITICAL-RUST-002 監査対応:
/// - 空文字禁止
/// - ASCII英数字とアンダースコアのみ許可（動的 DDL の SQL インジェクション対策）
/// - `PostgreSQL` の最大識別子長（63 バイト）を超えてはならない
fn validate_identifier(name: &str) -> anyhow::Result<()> {
    if name.is_empty() {
        anyhow::bail!("identifier cannot be empty");
    }
    // CRITICAL-RUST-002 監査対応: PostgreSQL 最大識別子長チェック（63 バイト制限）
    // UTF-8 でも識別子は ASCII 文字のみ許可するため len() == byte 数
    if name.len() > PG_MAX_IDENTIFIER_LEN {
        anyhow::bail!(
            "identifier '{}' exceeds PostgreSQL maximum length of {} bytes (got {})",
            name,
            PG_MAX_IDENTIFIER_LEN,
            name.len()
        );
    }
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        anyhow::bail!("invalid identifier: {name}");
    }
    Ok(())
}

fn postgres_type(data_type: &str) -> anyhow::Result<&'static str> {
    match data_type {
        "text" => Ok("TEXT"),
        "integer" => Ok("INTEGER"),
        "decimal" => Ok("NUMERIC"),
        "boolean" => Ok("BOOLEAN"),
        "date" => Ok("DATE"),
        "datetime" => Ok("TIMESTAMPTZ"),
        "uuid" => Ok("UUID"),
        "jsonb" => Ok("JSONB"),
        other => anyhow::bail!("unsupported data type: {other}"),
    }
}

/// カラム定義からデフォルト値の SQL フラグメントを生成する。
/// CRITICAL-RUST-002 監査対応:
/// - integer/decimal のデフォルト値を数値としてパース検証し、
///   任意の SQL が注入されないことを保証する。
fn default_value_sql(column: &CreateColumnDefinition) -> anyhow::Result<Option<String>> {
    let Some(default_value) = column.default_value.as_deref() else {
        return Ok(None);
    };

    let sql = match column.data_type.as_str() {
        "integer" => {
            // CRITICAL-RUST-002 監査対応: integer デフォルト値を i64 としてパースして
            // 不正な値（SQL 注入など）が混入しないことを検証する。
            default_value.parse::<i64>().map_err(|_| {
                anyhow::anyhow!(
                    "invalid integer default value '{default_value}': must be a valid integer"
                )
            })?;
            default_value.to_string()
        }
        "decimal" => {
            // CRITICAL-RUST-002 監査対応: decimal デフォルト値を f64 としてパースして
            // 不正な値が混入しないことを検証する。
            default_value.parse::<f64>().map_err(|_| {
                anyhow::anyhow!(
                    "invalid decimal default value '{default_value}': must be a valid number"
                )
            })?;
            default_value.to_string()
        }
        "boolean" => match default_value {
            "true" | "false" => default_value.to_string(),
            _ => anyhow::bail!("invalid boolean default: {default_value}"),
        },
        "uuid" if default_value.eq_ignore_ascii_case("gen_random_uuid()") => {
            "gen_random_uuid()".to_string()
        }
        "uuid" => format!("'{}'::uuid", escape_literal(default_value)),
        "date" => format!("'{}'::date", escape_literal(default_value)),
        "datetime" => format!("'{}'::timestamptz", escape_literal(default_value)),
        "jsonb" => format!("'{}'::jsonb", escape_literal(default_value)),
        _ => format!("'{}'", escape_literal(default_value)),
    };

    Ok(Some(sql))
}

fn escape_literal(value: &str) -> String {
    value.replace('\'', "''")
}

fn relationship_constraint_name(id: uuid::Uuid) -> String {
    format!("mm_rel_{}", id.simple())
}

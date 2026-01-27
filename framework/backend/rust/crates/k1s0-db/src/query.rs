//! 型安全なクエリビルダー
//!
//! SQLクエリを型安全に構築するためのビルダーパターンを提供する。
//!
//! # 機能
//!
//! - **SelectBuilder**: SELECT クエリの構築
//! - **InsertBuilder**: INSERT クエリの構築
//! - **UpdateBuilder**: UPDATE クエリの構築
//! - **DeleteBuilder**: DELETE クエリの構築
//! - **WhereClause**: 条件句の構築
//!
//! # 使用例
//!
//! ```rust
//! use k1s0_db::query::{SelectBuilder, WhereClause, Operator};
//!
//! let query = SelectBuilder::new("users")
//!     .columns(&["id", "name", "email"])
//!     .where_clause(WhereClause::new("status", Operator::Eq, "active"))
//!     .where_clause(WhereClause::new("age", Operator::Gte, "18"))
//!     .order_by("name", true)
//!     .limit(10)
//!     .offset(0)
//!     .build();
//!
//! assert!(query.sql.contains("SELECT"));
//! assert!(query.sql.contains("WHERE"));
//! ```

use crate::repository::{Pagination, SortBy, SortDirection};

/// 比較演算子
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    /// 等しい
    Eq,
    /// 等しくない
    Ne,
    /// より大きい
    Gt,
    /// 以上
    Gte,
    /// より小さい
    Lt,
    /// 以下
    Lte,
    /// LIKE
    Like,
    /// ILIKE (case-insensitive LIKE)
    ILike,
    /// IN
    In,
    /// NOT IN
    NotIn,
    /// IS NULL
    IsNull,
    /// IS NOT NULL
    IsNotNull,
    /// BETWEEN
    Between,
}

impl Operator {
    /// SQL演算子文字列を取得
    pub fn to_sql(&self) -> &'static str {
        match self {
            Operator::Eq => "=",
            Operator::Ne => "<>",
            Operator::Gt => ">",
            Operator::Gte => ">=",
            Operator::Lt => "<",
            Operator::Lte => "<=",
            Operator::Like => "LIKE",
            Operator::ILike => "ILIKE",
            Operator::In => "IN",
            Operator::NotIn => "NOT IN",
            Operator::IsNull => "IS NULL",
            Operator::IsNotNull => "IS NOT NULL",
            Operator::Between => "BETWEEN",
        }
    }

    /// 値が必要かどうか
    pub fn needs_value(&self) -> bool {
        !matches!(self, Operator::IsNull | Operator::IsNotNull)
    }
}

/// WHERE句の条件
#[derive(Debug, Clone)]
pub struct WhereClause {
    /// カラム名
    pub column: String,
    /// 演算子
    pub operator: Operator,
    /// 値（プレースホルダインデックス用）
    pub value: String,
}

impl WhereClause {
    /// 新しい条件を作成
    pub fn new(column: impl Into<String>, operator: Operator, value: impl Into<String>) -> Self {
        Self {
            column: column.into(),
            operator,
            value: value.into(),
        }
    }

    /// 等価条件を作成
    pub fn eq(column: impl Into<String>, value: impl Into<String>) -> Self {
        Self::new(column, Operator::Eq, value)
    }

    /// 不等条件を作成
    pub fn ne(column: impl Into<String>, value: impl Into<String>) -> Self {
        Self::new(column, Operator::Ne, value)
    }

    /// IS NULL 条件を作成
    pub fn is_null(column: impl Into<String>) -> Self {
        Self::new(column, Operator::IsNull, "")
    }

    /// IS NOT NULL 条件を作成
    pub fn is_not_null(column: impl Into<String>) -> Self {
        Self::new(column, Operator::IsNotNull, "")
    }

    /// LIKE 条件を作成
    pub fn like(column: impl Into<String>, pattern: impl Into<String>) -> Self {
        Self::new(column, Operator::Like, pattern)
    }

    /// SQL文字列を生成（プレースホルダ番号付き）
    pub fn to_sql(&self, param_index: usize) -> String {
        if !self.operator.needs_value() {
            format!("{} {}", self.column, self.operator.to_sql())
        } else {
            format!("{} {} ${}", self.column, self.operator.to_sql(), param_index)
        }
    }
}

/// ビルドされたクエリ
#[derive(Debug, Clone)]
pub struct BuiltQuery {
    /// SQL文字列
    pub sql: String,
    /// パラメータ値
    pub params: Vec<String>,
}

impl BuiltQuery {
    /// 新しいビルド済みクエリを作成
    pub fn new(sql: String, params: Vec<String>) -> Self {
        Self { sql, params }
    }
}

/// SELECTクエリビルダー
#[derive(Debug, Clone)]
pub struct SelectBuilder {
    table: String,
    columns: Vec<String>,
    where_clauses: Vec<WhereClause>,
    order_by: Vec<(String, bool)>,
    limit: Option<u64>,
    offset: Option<u64>,
    joins: Vec<String>,
    group_by: Vec<String>,
    having: Vec<WhereClause>,
    distinct: bool,
}

impl SelectBuilder {
    /// 新しいSELECTビルダーを作成
    pub fn new(table: impl Into<String>) -> Self {
        Self {
            table: table.into(),
            columns: Vec::new(),
            where_clauses: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            offset: None,
            joins: Vec::new(),
            group_by: Vec::new(),
            having: Vec::new(),
            distinct: false,
        }
    }

    /// カラムを設定
    pub fn columns(mut self, columns: &[&str]) -> Self {
        self.columns = columns.iter().map(|s| s.to_string()).collect();
        self
    }

    /// 全カラムを選択
    pub fn all_columns(mut self) -> Self {
        self.columns = vec!["*".to_string()];
        self
    }

    /// WHERE句を追加
    pub fn where_clause(mut self, clause: WhereClause) -> Self {
        self.where_clauses.push(clause);
        self
    }

    /// WHERE条件を複数追加
    pub fn where_clauses(mut self, clauses: Vec<WhereClause>) -> Self {
        self.where_clauses.extend(clauses);
        self
    }

    /// ORDER BYを追加
    pub fn order_by(mut self, column: impl Into<String>, ascending: bool) -> Self {
        self.order_by.push((column.into(), ascending));
        self
    }

    /// SortByからORDER BYを設定
    pub fn order_by_sort(mut self, sort_by: &[SortBy]) -> Self {
        for sort in sort_by {
            self.order_by.push((
                sort.column.clone(),
                sort.direction == SortDirection::Asc,
            ));
        }
        self
    }

    /// LIMITを設定
    pub fn limit(mut self, limit: u64) -> Self {
        self.limit = Some(limit);
        self
    }

    /// OFFSETを設定
    pub fn offset(mut self, offset: u64) -> Self {
        self.offset = Some(offset);
        self
    }

    /// ページネーションを適用
    pub fn paginate(mut self, pagination: &Pagination) -> Self {
        self.limit = Some(pagination.limit());
        self.offset = Some(pagination.offset());
        self
    }

    /// JOINを追加
    pub fn join(mut self, join_clause: impl Into<String>) -> Self {
        self.joins.push(join_clause.into());
        self
    }

    /// LEFT JOINを追加
    pub fn left_join(self, table: &str, condition: &str) -> Self {
        self.join(format!("LEFT JOIN {} ON {}", table, condition))
    }

    /// INNER JOINを追加
    pub fn inner_join(self, table: &str, condition: &str) -> Self {
        self.join(format!("INNER JOIN {} ON {}", table, condition))
    }

    /// GROUP BYを追加
    pub fn group_by(mut self, columns: &[&str]) -> Self {
        self.group_by = columns.iter().map(|s| s.to_string()).collect();
        self
    }

    /// HAVING句を追加
    pub fn having(mut self, clause: WhereClause) -> Self {
        self.having.push(clause);
        self
    }

    /// DISTINCTを有効化
    pub fn distinct(mut self) -> Self {
        self.distinct = true;
        self
    }

    /// クエリをビルド
    pub fn build(self) -> BuiltQuery {
        let mut sql = String::new();
        let mut params = Vec::new();
        let mut param_index = 1;

        // SELECT
        sql.push_str("SELECT ");
        if self.distinct {
            sql.push_str("DISTINCT ");
        }

        if self.columns.is_empty() {
            sql.push('*');
        } else {
            sql.push_str(&self.columns.join(", "));
        }

        // FROM
        sql.push_str(" FROM ");
        sql.push_str(&self.table);

        // JOINs
        for join in &self.joins {
            sql.push(' ');
            sql.push_str(join);
        }

        // WHERE
        if !self.where_clauses.is_empty() {
            sql.push_str(" WHERE ");
            let conditions: Vec<String> = self
                .where_clauses
                .iter()
                .map(|clause| {
                    let condition = clause.to_sql(param_index);
                    if clause.operator.needs_value() {
                        params.push(clause.value.clone());
                        param_index += 1;
                    }
                    condition
                })
                .collect();
            sql.push_str(&conditions.join(" AND "));
        }

        // GROUP BY
        if !self.group_by.is_empty() {
            sql.push_str(" GROUP BY ");
            sql.push_str(&self.group_by.join(", "));
        }

        // HAVING
        if !self.having.is_empty() {
            sql.push_str(" HAVING ");
            let conditions: Vec<String> = self
                .having
                .iter()
                .map(|clause| {
                    let condition = clause.to_sql(param_index);
                    if clause.operator.needs_value() {
                        params.push(clause.value.clone());
                        param_index += 1;
                    }
                    condition
                })
                .collect();
            sql.push_str(&conditions.join(" AND "));
        }

        // ORDER BY
        if !self.order_by.is_empty() {
            sql.push_str(" ORDER BY ");
            let orders: Vec<String> = self
                .order_by
                .iter()
                .map(|(col, asc)| {
                    format!("{} {}", col, if *asc { "ASC" } else { "DESC" })
                })
                .collect();
            sql.push_str(&orders.join(", "));
        }

        // LIMIT
        if let Some(limit) = self.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        // OFFSET
        if let Some(offset) = self.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        BuiltQuery::new(sql, params)
    }

    /// COUNT用のクエリをビルド
    pub fn build_count(self) -> BuiltQuery {
        let mut builder = self;
        builder.columns = vec!["COUNT(*)".to_string()];
        builder.order_by.clear();
        builder.limit = None;
        builder.offset = None;
        builder.build()
    }
}

/// INSERTクエリビルダー
#[derive(Debug, Clone)]
pub struct InsertBuilder {
    table: String,
    columns: Vec<String>,
    values: Vec<String>,
    returning: Vec<String>,
    on_conflict: Option<String>,
}

impl InsertBuilder {
    /// 新しいINSERTビルダーを作成
    pub fn new(table: impl Into<String>) -> Self {
        Self {
            table: table.into(),
            columns: Vec::new(),
            values: Vec::new(),
            returning: Vec::new(),
            on_conflict: None,
        }
    }

    /// カラムと値を追加
    pub fn column(mut self, column: impl Into<String>, value: impl Into<String>) -> Self {
        self.columns.push(column.into());
        self.values.push(value.into());
        self
    }

    /// 複数のカラムと値を設定
    pub fn columns_values(mut self, columns: &[&str], values: &[&str]) -> Self {
        self.columns = columns.iter().map(|s| s.to_string()).collect();
        self.values = values.iter().map(|s| s.to_string()).collect();
        self
    }

    /// RETURNING句を追加
    pub fn returning(mut self, columns: &[&str]) -> Self {
        self.returning = columns.iter().map(|s| s.to_string()).collect();
        self
    }

    /// ON CONFLICTを設定
    pub fn on_conflict(mut self, clause: impl Into<String>) -> Self {
        self.on_conflict = Some(clause.into());
        self
    }

    /// ON CONFLICT DO NOTHINGを設定
    pub fn on_conflict_do_nothing(self) -> Self {
        self.on_conflict("DO NOTHING")
    }

    /// クエリをビルド
    pub fn build(self) -> BuiltQuery {
        let mut sql = String::new();
        let params = self.values.clone();

        // INSERT INTO
        sql.push_str("INSERT INTO ");
        sql.push_str(&self.table);
        sql.push_str(" (");
        sql.push_str(&self.columns.join(", "));
        sql.push_str(") VALUES (");

        // プレースホルダ
        let placeholders: Vec<String> = (1..=self.values.len())
            .map(|i| format!("${}", i))
            .collect();
        sql.push_str(&placeholders.join(", "));
        sql.push(')');

        // ON CONFLICT
        if let Some(ref on_conflict) = self.on_conflict {
            sql.push_str(" ON CONFLICT ");
            sql.push_str(on_conflict);
        }

        // RETURNING
        if !self.returning.is_empty() {
            sql.push_str(" RETURNING ");
            sql.push_str(&self.returning.join(", "));
        }

        BuiltQuery::new(sql, params)
    }
}

/// UPDATEクエリビルダー
#[derive(Debug, Clone)]
pub struct UpdateBuilder {
    table: String,
    set_clauses: Vec<(String, String)>,
    where_clauses: Vec<WhereClause>,
    returning: Vec<String>,
}

impl UpdateBuilder {
    /// 新しいUPDATEビルダーを作成
    pub fn new(table: impl Into<String>) -> Self {
        Self {
            table: table.into(),
            set_clauses: Vec::new(),
            where_clauses: Vec::new(),
            returning: Vec::new(),
        }
    }

    /// SET句を追加
    pub fn set(mut self, column: impl Into<String>, value: impl Into<String>) -> Self {
        self.set_clauses.push((column.into(), value.into()));
        self
    }

    /// WHERE句を追加
    pub fn where_clause(mut self, clause: WhereClause) -> Self {
        self.where_clauses.push(clause);
        self
    }

    /// RETURNING句を追加
    pub fn returning(mut self, columns: &[&str]) -> Self {
        self.returning = columns.iter().map(|s| s.to_string()).collect();
        self
    }

    /// クエリをビルド
    pub fn build(self) -> BuiltQuery {
        let mut sql = String::new();
        let mut params = Vec::new();
        let mut param_index = 1;

        // UPDATE
        sql.push_str("UPDATE ");
        sql.push_str(&self.table);
        sql.push_str(" SET ");

        // SET clauses
        let set_parts: Vec<String> = self
            .set_clauses
            .iter()
            .map(|(col, val)| {
                params.push(val.clone());
                let part = format!("{} = ${}", col, param_index);
                param_index += 1;
                part
            })
            .collect();
        sql.push_str(&set_parts.join(", "));

        // WHERE
        if !self.where_clauses.is_empty() {
            sql.push_str(" WHERE ");
            let conditions: Vec<String> = self
                .where_clauses
                .iter()
                .map(|clause| {
                    let condition = clause.to_sql(param_index);
                    if clause.operator.needs_value() {
                        params.push(clause.value.clone());
                        param_index += 1;
                    }
                    condition
                })
                .collect();
            sql.push_str(&conditions.join(" AND "));
        }

        // RETURNING
        if !self.returning.is_empty() {
            sql.push_str(" RETURNING ");
            sql.push_str(&self.returning.join(", "));
        }

        BuiltQuery::new(sql, params)
    }
}

/// DELETEクエリビルダー
#[derive(Debug, Clone)]
pub struct DeleteBuilder {
    table: String,
    where_clauses: Vec<WhereClause>,
    returning: Vec<String>,
}

impl DeleteBuilder {
    /// 新しいDELETEビルダーを作成
    pub fn new(table: impl Into<String>) -> Self {
        Self {
            table: table.into(),
            where_clauses: Vec::new(),
            returning: Vec::new(),
        }
    }

    /// WHERE句を追加
    pub fn where_clause(mut self, clause: WhereClause) -> Self {
        self.where_clauses.push(clause);
        self
    }

    /// RETURNING句を追加
    pub fn returning(mut self, columns: &[&str]) -> Self {
        self.returning = columns.iter().map(|s| s.to_string()).collect();
        self
    }

    /// クエリをビルド
    pub fn build(self) -> BuiltQuery {
        let mut sql = String::new();
        let mut params = Vec::new();
        let mut param_index = 1;

        // DELETE FROM
        sql.push_str("DELETE FROM ");
        sql.push_str(&self.table);

        // WHERE
        if !self.where_clauses.is_empty() {
            sql.push_str(" WHERE ");
            let conditions: Vec<String> = self
                .where_clauses
                .iter()
                .map(|clause| {
                    let condition = clause.to_sql(param_index);
                    if clause.operator.needs_value() {
                        params.push(clause.value.clone());
                        param_index += 1;
                    }
                    condition
                })
                .collect();
            sql.push_str(&conditions.join(" AND "));
        }

        // RETURNING
        if !self.returning.is_empty() {
            sql.push_str(" RETURNING ");
            sql.push_str(&self.returning.join(", "));
        }

        BuiltQuery::new(sql, params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_builder_basic() {
        let query = SelectBuilder::new("users")
            .columns(&["id", "name", "email"])
            .build();

        assert_eq!(query.sql, "SELECT id, name, email FROM users");
        assert!(query.params.is_empty());
    }

    #[test]
    fn test_select_builder_with_where() {
        let query = SelectBuilder::new("users")
            .columns(&["id", "name"])
            .where_clause(WhereClause::eq("status", "active"))
            .build();

        assert_eq!(
            query.sql,
            "SELECT id, name FROM users WHERE status = $1"
        );
        assert_eq!(query.params, vec!["active"]);
    }

    #[test]
    fn test_select_builder_with_multiple_where() {
        let query = SelectBuilder::new("users")
            .columns(&["id"])
            .where_clause(WhereClause::eq("status", "active"))
            .where_clause(WhereClause::new("age", Operator::Gte, "18"))
            .build();

        assert!(query.sql.contains("WHERE status = $1 AND age >= $2"));
        assert_eq!(query.params.len(), 2);
    }

    #[test]
    fn test_select_builder_with_order_and_limit() {
        let query = SelectBuilder::new("users")
            .all_columns()
            .order_by("name", true)
            .order_by("created_at", false)
            .limit(10)
            .offset(20)
            .build();

        assert!(query.sql.contains("ORDER BY name ASC, created_at DESC"));
        assert!(query.sql.contains("LIMIT 10"));
        assert!(query.sql.contains("OFFSET 20"));
    }

    #[test]
    fn test_select_builder_with_join() {
        let query = SelectBuilder::new("users")
            .columns(&["users.id", "orders.total"])
            .left_join("orders", "orders.user_id = users.id")
            .build();

        assert!(query.sql.contains("LEFT JOIN orders ON orders.user_id = users.id"));
    }

    #[test]
    fn test_select_builder_distinct() {
        let query = SelectBuilder::new("users")
            .distinct()
            .columns(&["email"])
            .build();

        assert!(query.sql.starts_with("SELECT DISTINCT email"));
    }

    #[test]
    fn test_select_builder_count() {
        let query = SelectBuilder::new("users")
            .where_clause(WhereClause::eq("status", "active"))
            .build_count();

        assert!(query.sql.contains("SELECT COUNT(*)"));
        assert!(!query.sql.contains("ORDER BY"));
        assert!(!query.sql.contains("LIMIT"));
    }

    #[test]
    fn test_insert_builder() {
        let query = InsertBuilder::new("users")
            .column("name", "John")
            .column("email", "john@example.com")
            .returning(&["id"])
            .build();

        assert!(query.sql.contains("INSERT INTO users"));
        assert!(query.sql.contains("(name, email)"));
        assert!(query.sql.contains("VALUES ($1, $2)"));
        assert!(query.sql.contains("RETURNING id"));
    }

    #[test]
    fn test_insert_builder_on_conflict() {
        let query = InsertBuilder::new("users")
            .column("name", "John")
            .on_conflict_do_nothing()
            .build();

        assert!(query.sql.contains("ON CONFLICT DO NOTHING"));
    }

    #[test]
    fn test_update_builder() {
        let query = UpdateBuilder::new("users")
            .set("name", "John Doe")
            .set("updated_at", "NOW()")
            .where_clause(WhereClause::eq("id", "123"))
            .build();

        assert!(query.sql.contains("UPDATE users SET"));
        assert!(query.sql.contains("name = $1"));
        assert!(query.sql.contains("updated_at = $2"));
        assert!(query.sql.contains("WHERE id = $3"));
    }

    #[test]
    fn test_delete_builder() {
        let query = DeleteBuilder::new("users")
            .where_clause(WhereClause::eq("id", "123"))
            .returning(&["id"])
            .build();

        assert!(query.sql.contains("DELETE FROM users"));
        assert!(query.sql.contains("WHERE id = $1"));
        assert!(query.sql.contains("RETURNING id"));
    }

    #[test]
    fn test_where_clause_is_null() {
        let query = SelectBuilder::new("users")
            .columns(&["id"])
            .where_clause(WhereClause::is_null("deleted_at"))
            .build();

        assert!(query.sql.contains("WHERE deleted_at IS NULL"));
        assert!(query.params.is_empty());
    }

    #[test]
    fn test_pagination() {
        let pagination = Pagination::new(2, 25);
        let query = SelectBuilder::new("users")
            .all_columns()
            .paginate(&pagination)
            .build();

        assert!(query.sql.contains("LIMIT 25"));
        assert!(query.sql.contains("OFFSET 25"));
    }
}

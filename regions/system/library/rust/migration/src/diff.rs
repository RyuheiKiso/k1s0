use crate::schema::{Column, Schema, Table};

#[derive(Debug, Clone, PartialEq)]
pub enum SchemaDiff {
    TableAdded(Table),
    TableDropped(String),
    ColumnAdded {
        table: String,
        column: Column,
    },
    ColumnDropped {
        table: String,
        column: String,
    },
    ColumnChanged {
        table: String,
        column: String,
        from: Column,
        to: Column,
    },
}

pub fn diff_schemas(old: &Schema, new: &Schema) -> Vec<SchemaDiff> {
    let mut diffs = Vec::new();

    let old_tables: std::collections::HashMap<&str, &Table> =
        old.tables.iter().map(|t| (t.name.as_str(), t)).collect();
    let new_tables: std::collections::HashMap<&str, &Table> =
        new.tables.iter().map(|t| (t.name.as_str(), t)).collect();

    // Detect dropped tables
    for (name, table) in &old_tables {
        if !new_tables.contains_key(name) {
            diffs.push(SchemaDiff::TableDropped(table.name.clone()));
        }
    }

    // Detect added tables and column changes
    for (name, new_table) in &new_tables {
        match old_tables.get(name) {
            None => {
                diffs.push(SchemaDiff::TableAdded((*new_table).clone()));
            }
            Some(old_table) => {
                diff_columns(&mut diffs, name, old_table, new_table);
            }
        }
    }

    diffs
}

fn diff_columns(diffs: &mut Vec<SchemaDiff>, table: &str, old: &Table, new: &Table) {
    let old_cols: std::collections::HashMap<&str, &Column> =
        old.columns.iter().map(|c| (c.name.as_str(), c)).collect();
    let new_cols: std::collections::HashMap<&str, &Column> =
        new.columns.iter().map(|c| (c.name.as_str(), c)).collect();

    for (col_name, _) in &old_cols {
        if !new_cols.contains_key(col_name) {
            diffs.push(SchemaDiff::ColumnDropped {
                table: table.to_string(),
                column: col_name.to_string(),
            });
        }
    }

    for (col_name, new_col) in &new_cols {
        match old_cols.get(col_name) {
            None => {
                diffs.push(SchemaDiff::ColumnAdded {
                    table: table.to_string(),
                    column: (*new_col).clone(),
                });
            }
            Some(old_col) => {
                if old_col != new_col {
                    diffs.push(SchemaDiff::ColumnChanged {
                        table: table.to_string(),
                        column: col_name.to_string(),
                        from: (*old_col).clone(),
                        to: (*new_col).clone(),
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::*;

    fn make_column(name: &str, data_type: &str, nullable: bool) -> Column {
        Column {
            name: name.to_string(),
            data_type: data_type.to_string(),
            nullable,
            default: None,
        }
    }

    fn make_table(name: &str, columns: Vec<Column>) -> Table {
        Table {
            name: name.to_string(),
            columns,
            indexes: Vec::new(),
            constraints: Vec::new(),
        }
    }

    // 新しいテーブルが追加された場合に TableAdded の差分が検出されることを確認する。
    #[test]
    fn test_table_added() {
        let old = Schema { tables: vec![] };
        let new = Schema {
            tables: vec![make_table("users", vec![make_column("id", "UUID", false)])],
        };
        let diffs = diff_schemas(&old, &new);
        assert_eq!(diffs.len(), 1);
        assert!(matches!(&diffs[0], SchemaDiff::TableAdded(t) if t.name == "users"));
    }

    // テーブルが削除された場合に TableDropped の差分が検出されることを確認する。
    #[test]
    fn test_table_dropped() {
        let old = Schema {
            tables: vec![make_table("users", vec![make_column("id", "UUID", false)])],
        };
        let new = Schema { tables: vec![] };
        let diffs = diff_schemas(&old, &new);
        assert_eq!(diffs.len(), 1);
        assert!(matches!(&diffs[0], SchemaDiff::TableDropped(name) if name == "users"));
    }

    // カラムが追加された場合に ColumnAdded の差分が検出されることを確認する。
    #[test]
    fn test_column_added() {
        let old = Schema {
            tables: vec![make_table("users", vec![make_column("id", "UUID", false)])],
        };
        let new = Schema {
            tables: vec![make_table(
                "users",
                vec![
                    make_column("id", "UUID", false),
                    make_column("email", "TEXT", true),
                ],
            )],
        };
        let diffs = diff_schemas(&old, &new);
        assert_eq!(diffs.len(), 1);
        assert!(
            matches!(&diffs[0], SchemaDiff::ColumnAdded { table, column } if table == "users" && column.name == "email")
        );
    }

    // カラムが削除された場合に ColumnDropped の差分が検出されることを確認する。
    #[test]
    fn test_column_dropped() {
        let old = Schema {
            tables: vec![make_table(
                "users",
                vec![
                    make_column("id", "UUID", false),
                    make_column("email", "TEXT", true),
                ],
            )],
        };
        let new = Schema {
            tables: vec![make_table("users", vec![make_column("id", "UUID", false)])],
        };
        let diffs = diff_schemas(&old, &new);
        assert_eq!(diffs.len(), 1);
        assert!(
            matches!(&diffs[0], SchemaDiff::ColumnDropped { table, column } if table == "users" && column == "email")
        );
    }

    // カラムの型や制約が変更された場合に ColumnChanged の差分が検出されることを確認する。
    #[test]
    fn test_column_changed() {
        let old = Schema {
            tables: vec![make_table("users", vec![make_column("name", "TEXT", true)])],
        };
        let new = Schema {
            tables: vec![make_table(
                "users",
                vec![make_column("name", "VARCHAR", false)],
            )],
        };
        let diffs = diff_schemas(&old, &new);
        assert_eq!(diffs.len(), 1);
        assert!(
            matches!(&diffs[0], SchemaDiff::ColumnChanged { table, column, from, to }
                if table == "users" && column == "name" && from.data_type == "TEXT" && to.data_type == "VARCHAR")
        );
    }

    // 同一スキーマを比較した場合に差分が検出されないことを確認する。
    #[test]
    fn test_no_changes() {
        let schema = Schema {
            tables: vec![make_table("users", vec![make_column("id", "UUID", false)])],
        };
        let diffs = diff_schemas(&schema, &schema);
        assert!(diffs.is_empty());
    }
}

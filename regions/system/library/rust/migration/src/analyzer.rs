use sqlparser::ast::{AlterTableOperation, Statement};
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;

#[derive(Debug, Clone, PartialEq)]
pub enum BreakingChange {
    ColumnDropped {
        table: String,
        column: String,
    },
    ColumnTypeChanged {
        table: String,
        column: String,
        from: String,
        to: String,
    },
    TableDropped {
        table: String,
    },
    NotNullAdded {
        table: String,
        column: String,
    },
    ColumnRenamed {
        table: String,
        from: String,
        to: String,
    },
}

impl std::fmt::Display for BreakingChange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ColumnDropped { table, column } => {
                write!(f, "Column {table}.{column} dropped")
            }
            Self::ColumnTypeChanged {
                table,
                column,
                from,
                to,
            } => write!(
                f,
                "Column {table}.{column} type changed from {from} to {to}"
            ),
            Self::TableDropped { table } => write!(f, "Table {table} dropped"),
            Self::NotNullAdded { table, column } => {
                write!(f, "NOT NULL added to {table}.{column}")
            }
            Self::ColumnRenamed { table, from, to } => {
                write!(f, "Column {table}.{from} renamed to {to}")
            }
        }
    }
}

/// SQLから破壊的変更を検出する
pub fn detect_breaking_changes(sql: &str) -> Vec<BreakingChange> {
    let dialect = GenericDialect {};
    let statements = match Parser::parse_sql(&dialect, sql) {
        Ok(stmts) => stmts,
        Err(_) => return Vec::new(),
    };

    let mut changes = Vec::new();

    for stmt in &statements {
        match stmt {
            Statement::Drop {
                object_type, names, ..
            } => {
                use sqlparser::ast::ObjectType;
                if *object_type == ObjectType::Table {
                    for name in names {
                        changes.push(BreakingChange::TableDropped {
                            table: name.to_string(),
                        });
                    }
                }
            }
            Statement::AlterTable {
                name, operations, ..
            } => {
                let table = name.to_string();
                for op in operations {
                    match op {
                        AlterTableOperation::DropColumn { column_name, .. } => {
                            changes.push(BreakingChange::ColumnDropped {
                                table: table.clone(),
                                column: column_name.to_string(),
                            });
                        }
                        AlterTableOperation::AlterColumn {
                            column_name, op, ..
                        } => {
                            use sqlparser::ast::AlterColumnOperation;
                            match op {
                                AlterColumnOperation::SetNotNull => {
                                    changes.push(BreakingChange::NotNullAdded {
                                        table: table.clone(),
                                        column: column_name.to_string(),
                                    });
                                }
                                AlterColumnOperation::SetDataType {
                                    data_type, ..
                                } => {
                                    changes.push(BreakingChange::ColumnTypeChanged {
                                        table: table.clone(),
                                        column: column_name.to_string(),
                                        from: "unknown".to_string(),
                                        to: data_type.to_string(),
                                    });
                                }
                                _ => {}
                            }
                        }
                        AlterTableOperation::RenameColumn {
                            old_column_name,
                            new_column_name,
                        } => {
                            changes.push(BreakingChange::ColumnRenamed {
                                table: table.clone(),
                                from: old_column_name.to_string(),
                                to: new_column_name.to_string(),
                            });
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    changes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_drop_table() {
        let sql = "DROP TABLE users;";
        let changes = detect_breaking_changes(sql);
        assert_eq!(changes.len(), 1);
        assert_eq!(
            changes[0],
            BreakingChange::TableDropped {
                table: "users".to_string()
            }
        );
    }

    #[test]
    fn test_detect_drop_column() {
        let sql = "ALTER TABLE users DROP COLUMN email;";
        let changes = detect_breaking_changes(sql);
        assert_eq!(changes.len(), 1);
        assert_eq!(
            changes[0],
            BreakingChange::ColumnDropped {
                table: "users".to_string(),
                column: "email".to_string()
            }
        );
    }

    #[test]
    fn test_detect_set_not_null() {
        let sql = "ALTER TABLE users ALTER COLUMN email SET NOT NULL;";
        let changes = detect_breaking_changes(sql);
        assert_eq!(changes.len(), 1);
        assert_eq!(
            changes[0],
            BreakingChange::NotNullAdded {
                table: "users".to_string(),
                column: "email".to_string()
            }
        );
    }

    #[test]
    fn test_detect_type_change() {
        let sql = "ALTER TABLE users ALTER COLUMN age SET DATA TYPE BIGINT;";
        let changes = detect_breaking_changes(sql);
        assert_eq!(changes.len(), 1);
        match &changes[0] {
            BreakingChange::ColumnTypeChanged { table, column, to, .. } => {
                assert_eq!(table, "users");
                assert_eq!(column, "age");
                assert_eq!(to, "BIGINT");
            }
            other => panic!("Expected ColumnTypeChanged, got {:?}", other),
        }
    }

    #[test]
    fn test_detect_rename_column() {
        let sql = "ALTER TABLE users RENAME COLUMN old_name TO new_name;";
        let changes = detect_breaking_changes(sql);
        assert_eq!(changes.len(), 1);
        assert_eq!(
            changes[0],
            BreakingChange::ColumnRenamed {
                table: "users".to_string(),
                from: "old_name".to_string(),
                to: "new_name".to_string()
            }
        );
    }

    #[test]
    fn test_no_breaking_changes() {
        let sql = "ALTER TABLE users ADD COLUMN email TEXT;";
        let changes = detect_breaking_changes(sql);
        assert!(changes.is_empty());
    }

    #[test]
    fn test_display_formatting() {
        let change = BreakingChange::ColumnDropped {
            table: "users".to_string(),
            column: "email".to_string(),
        };
        assert_eq!(change.to_string(), "Column users.email dropped");
    }

    #[test]
    fn test_invalid_sql_returns_empty() {
        let changes = detect_breaking_changes("NOT VALID SQL AT ALL !!!");
        assert!(changes.is_empty());
    }
}

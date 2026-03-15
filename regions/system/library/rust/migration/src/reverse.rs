use crate::error::MigrationError;
use sqlparser::ast::Statement;
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;

/// UP SQLからDOWN SQLを自動生成する
pub fn generate_down_sql(up_sql: &str) -> Result<String, MigrationError> {
    let dialect = GenericDialect {};
    let statements = Parser::parse_sql(&dialect, up_sql)
        .map_err(|e| MigrationError::ParseError(format!("SQL parse error: {e}")))?;

    let mut down_statements = Vec::new();

    for stmt in &statements {
        match stmt {
            Statement::CreateTable(create_table) => {
                let table_name = &create_table.name;
                down_statements.push(format!("DROP TABLE IF EXISTS {table_name} CASCADE;"));
            }
            Statement::CreateIndex(create_index) => {
                if let Some(ref index_name) = create_index.name {
                    down_statements.push(format!("DROP INDEX IF EXISTS {index_name};"));
                }
            }
            Statement::AlterTable {
                name, operations, ..
            } => {
                for op in operations {
                    use sqlparser::ast::AlterTableOperation;
                    match op {
                        AlterTableOperation::AddColumn { column_def, .. } => {
                            let col_name = &column_def.name;
                            down_statements
                                .push(format!("ALTER TABLE {name} DROP COLUMN {col_name};"));
                        }
                        AlterTableOperation::AddConstraint(constraint) => {
                            use sqlparser::ast::TableConstraint;
                            match constraint {
                                TableConstraint::Unique {
                                    name: Some(cname), ..
                                }
                                | TableConstraint::ForeignKey {
                                    name: Some(cname), ..
                                }
                                | TableConstraint::Check {
                                    name: Some(cname), ..
                                }
                                | TableConstraint::PrimaryKey {
                                    name: Some(cname), ..
                                } => {
                                    down_statements.push(format!(
                                        "ALTER TABLE {name} DROP CONSTRAINT {cname};"
                                    ));
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    // Reverse order so drops happen in reverse dependency order
    down_statements.reverse();
    Ok(down_statements.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    // CREATE TABLE 文から DROP TABLE IF EXISTS … CASCADE; が生成されることを確認する。
    #[test]
    fn test_create_table_generates_drop() {
        let up = "CREATE TABLE users (id UUID PRIMARY KEY, name TEXT NOT NULL);";
        let down = generate_down_sql(up).unwrap();
        assert!(down.contains("DROP TABLE IF EXISTS users CASCADE;"));
    }

    // ADD COLUMN 文から ALTER TABLE … DROP COLUMN … が生成されることを確認する。
    #[test]
    fn test_add_column_generates_drop_column() {
        let up = "ALTER TABLE users ADD COLUMN email TEXT;";
        let down = generate_down_sql(up).unwrap();
        assert!(down.contains("ALTER TABLE users DROP COLUMN email;"));
    }

    // CREATE INDEX 文から DROP INDEX IF EXISTS … が生成されることを確認する。
    #[test]
    fn test_create_index_generates_drop_index() {
        let up = "CREATE INDEX idx_users_name ON users (name);";
        let down = generate_down_sql(up).unwrap();
        assert!(down.contains("DROP INDEX IF EXISTS idx_users_name;"));
    }

    // CREATE UNIQUE INDEX 文から DROP INDEX IF EXISTS … が生成されることを確認する。
    #[test]
    fn test_create_unique_index_generates_drop_index() {
        let up = "CREATE UNIQUE INDEX idx_users_email ON users (email);";
        let down = generate_down_sql(up).unwrap();
        assert!(down.contains("DROP INDEX IF EXISTS idx_users_email;"));
    }

    // 複数の UP 文から逆順の DOWN 文が生成されることを確認する。
    #[test]
    fn test_multiple_statements_reversed() {
        let up =
            "CREATE TABLE users (id UUID PRIMARY KEY);\nCREATE INDEX idx_users_id ON users (id);";
        let down = generate_down_sql(up).unwrap();
        let lines: Vec<&str> = down.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("DROP INDEX"));
        assert!(lines[1].contains("DROP TABLE"));
    }

    // 空文字列の SQL に generate_down_sql を適用すると空文字列が返ることを確認する。
    #[test]
    fn test_empty_sql() {
        let down = generate_down_sql("").unwrap();
        assert!(down.is_empty());
    }
}

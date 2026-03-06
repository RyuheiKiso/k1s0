/// マイグレーション実行モジュール。
///
/// dev up 時に各サービスの migrations/ ディレクトリを検出し、
/// マイグレーションを実行する。
use anyhow::Result;
use std::path::Path;

use super::types::PortAssignments;

/// dev up 時のマイグレーションを実行する。
///
/// 各サービスの migrations/ ディレクトリを検出し、
/// SQL ファイルを PostgreSQL に対して実行する。
///
/// # Errors
///
/// マイグレーションファイルの読み込みまたは SQL 実行に失敗した場合にエラーを返す。
pub fn run_dev_migrations(service_paths: &[String], ports: &PortAssignments) -> Result<()> {
    for service_path in service_paths {
        let path = Path::new(service_path);
        let migrations_dir = path.join("migrations");

        if !migrations_dir.is_dir() {
            continue;
        }

        println!(
            "  マイグレーション実行中: {}",
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
        );

        run_migrations_for_service(&migrations_dir, ports)?;
    }

    Ok(())
}

/// 指定ディレクトリ内のマイグレーションファイルを実行する。
fn run_migrations_for_service(migrations_dir: &Path, ports: &PortAssignments) -> Result<()> {
    let mut sql_files: Vec<_> = std::fs::read_dir(migrations_dir)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("sql") {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    // ファイル名でソート（マイグレーション順序を保証）
    sql_files.sort();

    for sql_file in &sql_files {
        let file_name = sql_file
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        println!("    適用中: {file_name}");

        let content = std::fs::read_to_string(sql_file)?;

        // psql を使ってマイグレーションを実行
        let status = std::process::Command::new("psql")
            .args([
                "-h",
                "localhost",
                "-p",
                &ports.postgres.to_string(),
                "-U",
                "app",
                "-c",
                &content,
            ])
            .env("PGPASSWORD", "password")
            .output();

        match status {
            Ok(output) if output.status.success() => {
                println!("    完了: {file_name}");
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("    警告: マイグレーション {file_name} でエラーが発生しました: {stderr}");
            }
            Err(e) => {
                eprintln!(
                    "    警告: psql コマンドの実行に失敗しました: {e}（psql がインストールされていない場合はスキップします）"
                );
                break;
            }
        }
    }

    Ok(())
}

/// 指定ディレクトリにマイグレーションファイルが存在するか確認する。
pub fn has_migrations(service_path: &Path) -> bool {
    let migrations_dir = service_path.join("migrations");
    if !migrations_dir.is_dir() {
        return false;
    }
    std::fs::read_dir(&migrations_dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .any(|e| {
                    e.path()
                        .extension()
                        .and_then(|ext| ext.to_str())
                        == Some("sql")
                })
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// migrations/ がないサービスはスキップされる。
    #[test]
    fn test_has_migrations_no_dir() {
        let tmp = TempDir::new().unwrap();
        assert!(!has_migrations(tmp.path()));
    }

    /// migrations/ に SQL ファイルがある場合は true を返す。
    #[test]
    fn test_has_migrations_with_sql() {
        let tmp = TempDir::new().unwrap();
        let migrations_dir = tmp.path().join("migrations");
        std::fs::create_dir_all(&migrations_dir).unwrap();
        std::fs::write(
            migrations_dir.join("001_init.sql"),
            "CREATE TABLE test (id INT);",
        )
        .unwrap();

        assert!(has_migrations(tmp.path()));
    }

    /// migrations/ に SQL 以外のファイルしかない場合は false を返す。
    #[test]
    fn test_has_migrations_no_sql() {
        let tmp = TempDir::new().unwrap();
        let migrations_dir = tmp.path().join("migrations");
        std::fs::create_dir_all(&migrations_dir).unwrap();
        std::fs::write(migrations_dir.join("README.md"), "# Migrations").unwrap();

        assert!(!has_migrations(tmp.path()));
    }
}

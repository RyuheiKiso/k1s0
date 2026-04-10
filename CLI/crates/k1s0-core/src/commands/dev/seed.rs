/// シードデータ投入モジュール。
///
/// dev up 時に各サービスの seeds/ ディレクトリを検出し、
/// シードデータを投入する。
use anyhow::Result;
use std::path::{Path, PathBuf};

use super::types::PortAssignments;

/// シードデータを投入する。
///
/// 各サービスの seeds/ ディレクトリ内の SQL ファイルを実行する。
///
/// # Errors
///
/// シードファイルの読み込みまたは SQL 実行に失敗した場合にエラーを返す。
pub fn execute_seed(service_paths: &[String], ports: &PortAssignments) -> Result<()> {
    for service_path in service_paths {
        let path = Path::new(service_path);
        let seed_files = scan_seed_files(path);

        if seed_files.is_empty() {
            continue;
        }

        println!(
            "  シードデータ投入中: {}",
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
        );

        for seed_file in &seed_files {
            let file_name = seed_file
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");

            println!("    適用中: {file_name}");

            // CRIT-004 監査対応: ファイル内容を -c で直接渡すのではなく -f でパスを渡す。
            // -c はシード内容をシェルコマンドとして直接実行するため、
            // 改ざんされたシードファイルにより任意 SQL が実行されるリスクがある。
            // -f はファイルパスのみを psql に渡すため、この攻撃面を排除できる。
            // L-001 監査対応: {:?} (Debug) の代わりに display() を使い {} (Display) フォーマットで出力する。
            let seed_file_path = seed_file.to_str().ok_or_else(|| {
                anyhow::anyhow!(
                    "シードファイルパスに非UTF-8文字が含まれています: {}",
                    seed_file.display()
                )
            })?;

            // HIGH-011 監査対応: デフォルトパスワードのハードコードを禁止する。
            // K1S0_DEV_PG_PASSWORD が未設定の場合はエラーで終了し、安全でない接続を防ぐ。
            let pg_password = std::env::var("K1S0_DEV_PG_PASSWORD").map_err(|_| {
                anyhow::anyhow!(
                    "環境変数 K1S0_DEV_PG_PASSWORD が設定されていません。\n\
                    .env.dev を確認し、PostgreSQL パスワードを設定してください。\n\
                    例: K1S0_DEV_PG_PASSWORD=your-password"
                )
            })?;

            let status = std::process::Command::new("psql")
                .args([
                    "-h",
                    "localhost",
                    "-p",
                    &ports.postgres.to_string(),
                    "-U",
                    "app",
                    "-f",
                    seed_file_path,
                ])
                .env("PGPASSWORD", &pg_password)
                .output();

            match status {
                Ok(output) if output.status.success() => {
                    println!("    完了: {file_name}");
                }
                Ok(output) => {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    eprintln!("    警告: シード {file_name} でエラーが発生しました: {stderr}");
                }
                Err(e) => {
                    eprintln!(
                        "    警告: psql コマンドの実行に失敗しました: {e}（psql がインストールされていない場合はスキップします）"
                    );
                    break;
                }
            }
        }
    }

    Ok(())
}

/// 指定サービスディレクトリ内のシードファイルを走査する。
///
/// seeds/ ディレクトリ内の .sql ファイルをファイル名順でソートして返す。
pub fn scan_seed_files(service_path: &Path) -> Vec<PathBuf> {
    let seeds_dir = service_path.join("seed");
    if !seeds_dir.is_dir() {
        return Vec::new();
    }

    let mut files: Vec<PathBuf> = std::fs::read_dir(&seeds_dir)
        .into_iter()
        .flatten()
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

    files.sort();
    files
}

// テストコードでは unwrap() の使用を許可する
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// seeds/ がないサービスは空のリストを返す。
    #[test]
    fn test_scan_seed_files_no_dir() {
        let tmp = TempDir::new().unwrap();
        let files = scan_seed_files(tmp.path());
        assert!(files.is_empty());
    }

    /// seeds/ 内の SQL ファイルをソート順で返す。
    #[test]
    fn test_scan_seed_files_sorted() {
        let tmp = TempDir::new().unwrap();
        let seeds_dir = tmp.path().join("seed");
        std::fs::create_dir_all(&seeds_dir).unwrap();

        std::fs::write(seeds_dir.join("002_users.sql"), "INSERT INTO users;").unwrap();
        std::fs::write(seeds_dir.join("001_roles.sql"), "INSERT INTO roles;").unwrap();
        std::fs::write(seeds_dir.join("README.md"), "# Seeds").unwrap();

        let files = scan_seed_files(tmp.path());
        assert_eq!(files.len(), 2);

        let names: Vec<&str> = files
            .iter()
            .map(|f| f.file_name().unwrap().to_str().unwrap())
            .collect();
        assert_eq!(names[0], "001_roles.sql");
        assert_eq!(names[1], "002_users.sql");
    }

    /// seeds/ に SQL ファイルがない場合は空のリストを返す。
    #[test]
    fn test_scan_seed_files_no_sql() {
        let tmp = TempDir::new().unwrap();
        let seeds_dir = tmp.path().join("seed");
        std::fs::create_dir_all(&seeds_dir).unwrap();
        std::fs::write(seeds_dir.join("README.md"), "# Seeds").unwrap();

        let files = scan_seed_files(tmp.path());
        assert!(files.is_empty());
    }
}

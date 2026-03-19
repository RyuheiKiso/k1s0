/// マイグレーション対象の走査とファイル解析。
use std::fs;
use std::path::Path;
use std::sync::OnceLock;

use anyhow::{Context, Result};
use regex::Regex;

use super::types::{Direction, Language, MigrateTarget, MigrationFile};

/// マイグレーションファイル名パターンの正規表現キャッシュ
static MIGRATION_FILE_RE: OnceLock<Regex> = OnceLock::new();
/// マイグレーション名バリデーション用の正規表現キャッシュ
static MIGRATION_NAME_RE: OnceLock<Regex> = OnceLock::new();

/// regions/ 配下のサービスを走査し、migrations/ ディレクトリがあるものを返す。
pub fn scan_migrate_targets(base_dir: &Path) -> Vec<MigrateTarget> {
    scan_migrate_targets_at(base_dir)
}

/// 指定ディレクトリを基点にマイグレーション対象を走査する（テスト用）。
pub fn scan_migrate_targets_at(base_dir: &Path) -> Vec<MigrateTarget> {
    let mut targets = Vec::new();
    let regions = base_dir.join("regions");
    if !regions.is_dir() {
        return targets;
    }
    scan_targets_recursive(&regions, &mut targets);
    targets.sort_by(|a, b| a.service_name.cmp(&b.service_name));
    targets
}

/// 再帰的にサービスディレクトリを走査する。
fn scan_targets_recursive(path: &Path, targets: &mut Vec<MigrateTarget>) {
    if !path.is_dir() {
        return;
    }

    // migrations/ ディレクトリが存在するか確認
    let migrations_dir = path.join("migrations");
    if migrations_dir.is_dir() {
        // 言語を検出
        if let Some(language) = detect_language(path) {
            let service_name = extract_service_name(path);
            let tier = extract_tier(path);
            let db_name = detect_db_name(path).unwrap_or_else(|| format!("{service_name}_db"));

            targets.push(MigrateTarget {
                service_name,
                tier,
                language,
                migrations_dir,
                db_name,
            });
        }
        return; // migrations/ があるならこれ以上深くは潜らない
    }

    // サブディレクトリを走査
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                scan_targets_recursive(&entry.path(), targets);
            }
        }
    }
}

/// プロジェクトの言語を検出する。
fn detect_language(path: &Path) -> Option<Language> {
    if path.join("Cargo.toml").exists() {
        Some(Language::Rust)
    } else if path.join("go.mod").exists() {
        Some(Language::Go)
    } else {
        None
    }
}

/// パスからサービス名を抽出する。
fn extract_service_name(path: &Path) -> String {
    path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string()
}

/// パスからティアを抽出する。
fn extract_tier(path: &Path) -> String {
    let path_str = path.to_string_lossy().replace('\\', "/");
    let parts: Vec<&str> = path_str.split('/').collect();
    for (i, part) in parts.iter().enumerate() {
        if *part == "regions" && i + 1 < parts.len() {
            let tier = parts[i + 1];
            if tier == "system" || tier == "business" || tier == "service" {
                return tier.to_string();
            }
        }
    }
    "unknown".to_string()
}

/// config/config.yaml からデータベース名を検出する。
fn detect_db_name(path: &Path) -> Option<String> {
    let config_path = path.join("config").join("config.yaml");
    if !config_path.exists() {
        return None;
    }
    let content = fs::read_to_string(&config_path).ok()?;
    // RuntimeConfig を使わずにシンプルにパースする
    let config: serde_yaml::Value = serde_yaml::from_str(&content).ok()?;
    config
        .get("database")?
        .get("name")?
        .as_str()
        .map(String::from)
}

/// migrations/ 内のSQLファイルを走査する。
///
/// ファイル名の形式: `{NNN}_{description}.{up|down}.sql`
///
/// # Errors
///
/// ディレクトリの読み込みに失敗した場合にエラーを返す。
pub fn scan_migration_files(migrations_dir: &Path) -> Result<Vec<MigrationFile>> {
    let mut files = Vec::new();

    if !migrations_dir.is_dir() {
        return Ok(files);
    }

    // OnceLock で正規表現を一度だけコンパイルしてキャッシュする
    let pattern = MIGRATION_FILE_RE.get_or_init(|| {
        Regex::new(r"^(\d{3})_([a-z0-9_]+)\.(up|down)\.sql$")
            .expect("マイグレーションファイル名の正規表現は静的に正しい")
    });

    let entries = fs::read_dir(migrations_dir).with_context(|| {
        format!(
            "ディレクトリの読み込みに失敗しました: {}",
            migrations_dir.display()
        )
    })?;

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let file_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name.to_string(),
            None => continue,
        };

        if let Some(caps) = pattern.captures(&file_name) {
            let number: u32 = caps[1].parse().unwrap_or(0);
            let description = caps[2].to_string();
            let direction = match &caps[3] {
                "up" => Direction::Up,
                "down" => Direction::Down,
                _ => continue,
            };

            files.push(MigrationFile {
                number,
                description,
                direction,
                path,
            });
        }
    }

    // 番号順、同番号ならup→downの順にソート
    files.sort_by(|a, b| {
        a.number
            .cmp(&b.number)
            .then_with(|| match (&a.direction, &b.direction) {
                (Direction::Up, Direction::Down) => std::cmp::Ordering::Less,
                (Direction::Down, Direction::Up) => std::cmp::Ordering::Greater,
                _ => std::cmp::Ordering::Equal,
            })
    });

    Ok(files)
}

/// 既存ファイルから次の連番を算出する。
pub fn next_sequence_number(files: &[MigrationFile]) -> u32 {
    files.iter().map(|f| f.number).max().unwrap_or(0) + 1
}

/// マイグレーション名のバリデーション。
///
/// 英小文字・数字・アンダースコアのみ許可（[a-z0-9_]+）。
/// Validate a migration name.
///
/// Only lowercase alphanumeric characters and underscores are allowed.
///
/// # Errors
///
/// Returns an error when the name is empty, contains unsupported characters,
/// or the validation pattern cannot be constructed.
pub fn validate_migration_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("マイグレーション名を入力してください".to_string());
    }
    // OnceLock で正規表現を一度だけコンパイルしてキャッシュする
    let re = MIGRATION_NAME_RE.get_or_init(|| {
        Regex::new(r"^[a-z0-9_]+$")
            .expect("マイグレーション名バリデーション用の正規表現は静的に正しい")
    });
    if !re.is_match(name) {
        return Err(
            "マイグレーション名は英小文字・数字・アンダースコアのみ使用できます（[a-z0-9_]+）"
                .to_string(),
        );
    }
    Ok(())
}

// テストコードでは unwrap() の使用を許可する
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // --- validate_migration_name ---

    #[test]
    fn test_validate_migration_name_valid() {
        assert!(validate_migration_name("create_users").is_ok());
        assert!(validate_migration_name("add_column").is_ok());
        assert!(validate_migration_name("v1").is_ok());
        assert!(validate_migration_name("a").is_ok());
        assert!(validate_migration_name("test_123").is_ok());
        assert!(validate_migration_name("abc123def").is_ok());
    }

    #[test]
    fn test_validate_migration_name_invalid() {
        assert!(validate_migration_name("").is_err());
        assert!(validate_migration_name("Create_Users").is_err());
        assert!(validate_migration_name("add-column").is_err());
        assert!(validate_migration_name("has space").is_err());
        assert!(validate_migration_name("dot.name").is_err());
        assert!(validate_migration_name("UPPER").is_err());
    }

    // --- next_sequence_number ---

    #[test]
    fn test_next_sequence_number_empty() {
        assert_eq!(next_sequence_number(&[]), 1);
    }

    #[test]
    fn test_next_sequence_number_with_files() {
        let files = vec![
            MigrationFile {
                number: 1,
                description: "create_users".to_string(),
                direction: Direction::Up,
                path: "001_create_users.up.sql".into(),
            },
            MigrationFile {
                number: 1,
                description: "create_users".to_string(),
                direction: Direction::Down,
                path: "001_create_users.down.sql".into(),
            },
            MigrationFile {
                number: 2,
                description: "add_email".to_string(),
                direction: Direction::Up,
                path: "002_add_email.up.sql".into(),
            },
            MigrationFile {
                number: 2,
                description: "add_email".to_string(),
                direction: Direction::Down,
                path: "002_add_email.down.sql".into(),
            },
        ];
        assert_eq!(next_sequence_number(&files), 3);
    }

    // --- scan_migration_files ---

    #[test]
    fn test_scan_migration_files_empty_dir() {
        let tmp = TempDir::new().unwrap();
        let migrations = tmp.path().join("migrations");
        fs::create_dir_all(&migrations).unwrap();

        let files = scan_migration_files(&migrations).unwrap();
        assert!(files.is_empty());
    }

    #[test]
    fn test_scan_migration_files_valid() {
        let tmp = TempDir::new().unwrap();
        let migrations = tmp.path().join("migrations");
        fs::create_dir_all(&migrations).unwrap();

        fs::write(
            migrations.join("001_create_users.up.sql"),
            "CREATE TABLE users (id INT);",
        )
        .unwrap();
        fs::write(
            migrations.join("001_create_users.down.sql"),
            "DROP TABLE users;",
        )
        .unwrap();
        fs::write(
            migrations.join("002_add_email.up.sql"),
            "ALTER TABLE users ADD COLUMN email VARCHAR;",
        )
        .unwrap();
        fs::write(
            migrations.join("002_add_email.down.sql"),
            "ALTER TABLE users DROP COLUMN email;",
        )
        .unwrap();

        let files = scan_migration_files(&migrations).unwrap();
        assert_eq!(files.len(), 4);
        assert_eq!(files[0].number, 1);
        assert_eq!(files[0].direction, Direction::Up);
        assert_eq!(files[1].number, 1);
        assert_eq!(files[1].direction, Direction::Down);
        assert_eq!(files[2].number, 2);
        assert_eq!(files[2].direction, Direction::Up);
        assert_eq!(files[3].number, 2);
        assert_eq!(files[3].direction, Direction::Down);
    }

    #[test]
    fn test_scan_migration_files_ignores_invalid() {
        let tmp = TempDir::new().unwrap();
        let migrations = tmp.path().join("migrations");
        fs::create_dir_all(&migrations).unwrap();

        // 正しい形式
        fs::write(migrations.join("001_create_users.up.sql"), "CREATE TABLE;").unwrap();
        // 不正な形式（無視されるべき）
        fs::write(migrations.join("README.md"), "# Migrations").unwrap();
        fs::write(migrations.join("invalid.sql"), "SELECT 1;").unwrap();
        fs::write(migrations.join("1_no_padding.up.sql"), "SELECT 1;").unwrap();

        let files = scan_migration_files(&migrations).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].number, 1);
    }

    #[test]
    fn test_scan_migration_files_nonexistent_dir() {
        let files = scan_migration_files(Path::new("/nonexistent/dir")).unwrap();
        assert!(files.is_empty());
    }

    // --- scan_migrate_targets ---

    #[test]
    fn test_scan_migrate_targets_empty() {
        let tmp = TempDir::new().unwrap();
        let targets = scan_migrate_targets_at(tmp.path());
        assert!(targets.is_empty());
    }

    #[test]
    fn test_scan_migrate_targets_rust_service() {
        let tmp = TempDir::new().unwrap();

        // Rustサービスをセットアップ
        let service_dir = tmp.path().join("regions/system/server/rust/auth");
        fs::create_dir_all(&service_dir).unwrap();
        fs::write(service_dir.join("Cargo.toml"), "[package]\nname = \"auth\"").unwrap();
        fs::create_dir_all(service_dir.join("migrations")).unwrap();

        let targets = scan_migrate_targets_at(tmp.path());
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].service_name, "auth");
        assert_eq!(targets[0].tier, "system");
        assert_eq!(targets[0].language, Language::Rust);
    }

    #[test]
    fn test_scan_migrate_targets_go_service() {
        let tmp = TempDir::new().unwrap();

        // Goサービスをセットアップ
        let service_dir = tmp.path().join("regions/business/server/go/ledger");
        fs::create_dir_all(&service_dir).unwrap();
        fs::write(service_dir.join("go.mod"), "module ledger").unwrap();
        fs::create_dir_all(service_dir.join("migrations")).unwrap();

        let targets = scan_migrate_targets_at(tmp.path());
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].service_name, "ledger");
        assert_eq!(targets[0].tier, "business");
        assert_eq!(targets[0].language, Language::Go);
    }

    #[test]
    fn test_scan_migrate_targets_with_config() {
        let tmp = TempDir::new().unwrap();

        let service_dir = tmp.path().join("regions/system/server/rust/auth");
        fs::create_dir_all(&service_dir).unwrap();
        fs::write(service_dir.join("Cargo.toml"), "[package]").unwrap();
        fs::create_dir_all(service_dir.join("migrations")).unwrap();

        // config.yaml にデータベース名を設定
        let config_dir = service_dir.join("config");
        fs::create_dir_all(&config_dir).unwrap();
        fs::write(
            config_dir.join("config.yaml"),
            "database:\n  name: auth_database\n",
        )
        .unwrap();

        let targets = scan_migrate_targets_at(tmp.path());
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].db_name, "auth_database");
    }

    #[test]
    fn test_scan_migrate_targets_excludes_no_migrations() {
        let tmp = TempDir::new().unwrap();

        // migrations/ がないサービスは除外
        let service_dir = tmp.path().join("regions/system/server/rust/auth");
        fs::create_dir_all(&service_dir).unwrap();
        fs::write(service_dir.join("Cargo.toml"), "[package]").unwrap();

        let targets = scan_migrate_targets_at(tmp.path());
        assert!(targets.is_empty());
    }

    // --- detect_language ---

    #[test]
    fn test_detect_language_rust() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("Cargo.toml"), "[package]").unwrap();
        assert_eq!(detect_language(tmp.path()), Some(Language::Rust));
    }

    #[test]
    fn test_detect_language_go() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("go.mod"), "module test").unwrap();
        assert_eq!(detect_language(tmp.path()), Some(Language::Go));
    }

    #[test]
    fn test_detect_language_none() {
        let tmp = TempDir::new().unwrap();
        assert_eq!(detect_language(tmp.path()), None);
    }

    // --- extract_tier ---

    #[test]
    fn test_extract_tier() {
        assert_eq!(
            extract_tier(Path::new("regions/system/server/rust/auth")),
            "system"
        );
        assert_eq!(
            extract_tier(Path::new("regions/business/server/go/ledger")),
            "business"
        );
        assert_eq!(
            extract_tier(Path::new("regions/service/order/server/rust")),
            "service"
        );
        assert_eq!(extract_tier(Path::new("unknown/path")), "unknown");
    }
}

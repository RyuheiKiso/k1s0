use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

use super::executor::backup_project_dir;
use walkdir::WalkDir;

use super::parser::BACKUP_DIR_NAME;

/// バックアップからファイルを復元する。
///
/// # Errors
///
/// ファイル操作に失敗した場合にエラーを返す。
pub fn rollback(project_dir: &Path, backup_dir: &Path) -> Result<()> {
    let backup_project = backup_project_dir(backup_dir);
    clear_project_dir(project_dir)?;
    restore_project_dir(&backup_project, project_dir)?;
    Ok(())
}

/// 利用可能なバックアップの一覧を返す（新しい順）。
///
/// # Errors
///
/// ディレクトリの読み込みに失敗した場合にエラーを返す。
pub fn list_backups(project_dir: &Path) -> Result<Vec<String>> {
    let backup_root = project_dir.join(BACKUP_DIR_NAME);
    if !backup_root.exists() {
        return Ok(Vec::new());
    }

    let mut backups = Vec::new();
    for entry in fs::read_dir(&backup_root)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            if let Some(name) = entry.file_name().to_str() {
                backups.push(name.to_string());
            }
        }
    }

    backups.sort();
    backups.reverse();
    Ok(backups)
}

/// バックアップ ID からバックアップディレクトリを解決する。
pub fn backup_dir(project_dir: &Path, backup_id: &str) -> PathBuf {
    project_dir.join(BACKUP_DIR_NAME).join(backup_id)
}

fn clear_project_dir(project_dir: &Path) -> Result<()> {
    for entry in fs::read_dir(project_dir)? {
        let entry = entry?;
        if entry.file_name() == BACKUP_DIR_NAME {
            continue;
        }

        let path = entry.path();
        if entry.file_type()?.is_dir() {
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_file(path)?;
        }
    }

    Ok(())
}

fn restore_project_dir(source: &Path, destination: &Path) -> Result<()> {
    for entry in WalkDir::new(source)
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        let relative = entry.path().strip_prefix(source)?;
        if relative.as_os_str().is_empty() {
            continue;
        }

        let target = destination.join(relative);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target)?;
            continue;
        }

        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(entry.path(), target)?;
    }

    Ok(())
}

// テストコードでは unwrap() の使用を許可する
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn list_backups_returns_newest_first() {
        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join(BACKUP_DIR_NAME).join("20260311_010101")).unwrap();
        fs::create_dir_all(temp.path().join(BACKUP_DIR_NAME).join("20260312_010101")).unwrap();

        let backups = list_backups(temp.path()).unwrap();
        assert_eq!(
            backups,
            vec!["20260312_010101".to_string(), "20260311_010101".to_string()]
        );
    }

    #[test]
    fn rollback_restores_project_tree() {
        let temp = TempDir::new().unwrap();
        let backup = backup_dir(temp.path(), "20260312_010101");
        fs::create_dir_all(backup_project_dir(&backup).join("src")).unwrap();
        fs::write(
            backup_project_dir(&backup).join("src/main.rs"),
            "fn main() {}\n",
        )
        .unwrap();

        fs::create_dir_all(temp.path().join("src")).unwrap();
        fs::write(temp.path().join("src/main.rs"), "fn main() { panic!() }\n").unwrap();
        fs::write(temp.path().join("scratch.txt"), "remove me").unwrap();

        rollback(temp.path(), &backup).unwrap();

        assert_eq!(
            fs::read_to_string(temp.path().join("src/main.rs")).unwrap(),
            "fn main() {}\n"
        );
        assert!(!temp.path().join("scratch.txt").exists());
    }
}

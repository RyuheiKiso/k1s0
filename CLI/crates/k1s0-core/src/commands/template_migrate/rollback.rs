use anyhow::Result;
use std::path::Path;
use walkdir::WalkDir;

/// バックアップからファイルを復元する。
///
/// # Errors
///
/// ファイル操作に失敗した場合にエラーを返す。
pub fn rollback(project_dir: &Path, backup_dir: &Path) -> Result<()> {
    for entry in WalkDir::new(backup_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let relative = entry.path().strip_prefix(backup_dir)?;
        let target = project_dir.join(relative);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(entry.path(), &target)?;
    }
    Ok(())
}

/// 利用可能なバックアップの一覧を返す（新しい順）。
///
/// # Errors
///
/// ディレクトリの読み込みに失敗した場合にエラーを返す。
pub fn list_backups(project_dir: &Path) -> Result<Vec<String>> {
    let backup_root = project_dir.join(".k1s0-backup");
    if !backup_root.exists() {
        return Ok(Vec::new());
    }
    let mut backups = Vec::new();
    for entry in std::fs::read_dir(&backup_root)? {
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

//! ファイル操作ユーティリティ
//!
//! ファイル生成・上書き・衝突検知の共通関数を提供する。

use std::path::Path;

use crate::Result;

/// ファイルを安全に書き込む（親ディレクトリを自動作成）
pub fn write_file<P: AsRef<Path>>(path: P, content: &str) -> Result<()> {
    let path = path.as_ref();

    // 親ディレクトリを作成
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::write(path, content)?;
    Ok(())
}

/// ファイルが存在するかどうかを確認
pub fn file_exists<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().exists()
}

/// ディレクトリを再帰的に作成
pub fn create_dir_all<P: AsRef<Path>>(path: P) -> Result<()> {
    std::fs::create_dir_all(path)?;
    Ok(())
}

/// ファイルをバックアップ（.bak を付けてコピー）
pub fn backup_file<P: AsRef<Path>>(path: P) -> Result<Option<String>> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(None);
    }

    let backup_path = format!("{}.bak", path.display());
    std::fs::copy(path, &backup_path)?;
    Ok(Some(backup_path))
}

/// ファイルの衝突を検知する
///
/// checksum が異なる場合、手動変更されたと判断する
pub fn detect_conflict<P: AsRef<Path>>(
    path: P,
    expected_checksum: &str,
) -> Result<bool> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(false);
    }

    let actual_checksum = crate::fingerprint::calculate_file_checksum(path)?;
    Ok(actual_checksum != expected_checksum)
}

/// 指定パターンにマッチするファイルを列挙する
pub fn glob_files<P: AsRef<Path>>(
    base_dir: P,
    pattern: &str,
) -> Result<Vec<String>> {
    let base_dir = base_dir.as_ref();
    let full_pattern = base_dir.join(pattern);

    let mut results = Vec::new();
    for entry in glob::glob(&full_pattern.to_string_lossy())? {
        if let Ok(path) = entry {
            if let Ok(relative) = path.strip_prefix(base_dir) {
                results.push(relative.to_string_lossy().to_string());
            }
        }
    }

    Ok(results)
}

impl From<glob::PatternError> for crate::Error {
    fn from(e: glob::PatternError) -> Self {
        crate::Error::Other(e.to_string())
    }
}

impl From<glob::GlobError> for crate::Error {
    fn from(e: glob::GlobError) -> Self {
        crate::Error::Other(e.to_string())
    }
}

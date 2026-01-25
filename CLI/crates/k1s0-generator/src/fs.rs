//! ファイル操作ユーティリティ
//!
//! ファイル生成・上書き・衝突検知の共通関数を提供する。

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::Result;

/// ファイル操作の結果
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WriteResult {
    /// 新規作成
    Created,
    /// 上書き
    Overwritten,
    /// スキップ（既に同一内容）
    Skipped,
}

/// ファイル衝突の情報
#[derive(Debug, Clone)]
pub struct ConflictInfo {
    /// ファイルパス
    pub path: PathBuf,
    /// 期待されるチェックサム
    pub expected_checksum: String,
    /// 実際のチェックサム
    pub actual_checksum: String,
}

/// ファイルを安全に書き込む（親ディレクトリを自動作成）
pub fn write_file<P: AsRef<Path>>(path: P, content: &str) -> Result<WriteResult> {
    let path = path.as_ref();

    // 既存ファイルと同一内容ならスキップ
    if path.exists() {
        let existing = std::fs::read_to_string(path)?;
        if existing == content {
            return Ok(WriteResult::Skipped);
        }
    }

    let result = if path.exists() {
        WriteResult::Overwritten
    } else {
        WriteResult::Created
    };

    // 親ディレクトリを作成
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::write(path, content)?;
    Ok(result)
}

/// バイナリファイルを安全に書き込む
pub fn write_file_bytes<P: AsRef<Path>>(path: P, content: &[u8]) -> Result<WriteResult> {
    let path = path.as_ref();

    // 既存ファイルと同一内容ならスキップ
    if path.exists() {
        let existing = std::fs::read(path)?;
        if existing == content {
            return Ok(WriteResult::Skipped);
        }
    }

    let result = if path.exists() {
        WriteResult::Overwritten
    } else {
        WriteResult::Created
    };

    // 親ディレクトリを作成
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::write(path, content)?;
    Ok(result)
}

/// アトミックな書き込み（一時ファイル経由）
///
/// 書き込み中にクラッシュしても、元のファイルが壊れないことを保証する。
pub fn write_file_atomic<P: AsRef<Path>>(path: P, content: &str) -> Result<WriteResult> {
    let path = path.as_ref();

    // 既存ファイルと同一内容ならスキップ
    if path.exists() {
        let existing = std::fs::read_to_string(path)?;
        if existing == content {
            return Ok(WriteResult::Skipped);
        }
    }

    let result = if path.exists() {
        WriteResult::Overwritten
    } else {
        WriteResult::Created
    };

    // 親ディレクトリを作成
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // 一時ファイルに書き込み
    let temp_path = path.with_extension("tmp");
    {
        let mut file = File::create(&temp_path)?;
        file.write_all(content.as_bytes())?;
        file.sync_all()?;
    }

    // リネームでアトミックに置換
    std::fs::rename(&temp_path, path)?;
    Ok(result)
}

/// ファイルが存在するかどうかを確認
pub fn file_exists<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().exists()
}

/// ファイルかどうかを確認
pub fn is_file<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().is_file()
}

/// ディレクトリかどうかを確認
pub fn is_dir<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().is_dir()
}

/// ディレクトリを再帰的に作成
pub fn create_dir_all<P: AsRef<Path>>(path: P) -> Result<()> {
    std::fs::create_dir_all(path)?;
    Ok(())
}

/// ファイルを読み込む
pub fn read_file<P: AsRef<Path>>(path: P) -> Result<String> {
    let content = std::fs::read_to_string(path)?;
    Ok(content)
}

/// バイナリファイルを読み込む
pub fn read_file_bytes<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
    let content = std::fs::read(path)?;
    Ok(content)
}

/// ファイルをバックアップ（.bak を付けてコピー）
pub fn backup_file<P: AsRef<Path>>(path: P) -> Result<Option<PathBuf>> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(None);
    }

    let backup_path = path.with_extension(format!(
        "{}.bak",
        path.extension()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or_default()
    ));
    std::fs::copy(path, &backup_path)?;
    Ok(Some(backup_path))
}

/// タイムスタンプ付きバックアップ
pub fn backup_file_with_timestamp<P: AsRef<Path>>(path: P) -> Result<Option<PathBuf>> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(None);
    }

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let ext = path
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let backup_name = format!("{}.{}{}", stem, timestamp, ext);

    let backup_path = path.with_file_name(backup_name);
    std::fs::copy(path, &backup_path)?;
    Ok(Some(backup_path))
}

/// ファイルの衝突を検知する
///
/// 期待されるチェックサムと実際のチェックサムを比較し、
/// 異なる場合は手動変更されたと判断する。
pub fn detect_conflict<P: AsRef<Path>>(
    path: P,
    expected_checksum: &str,
) -> Result<Option<ConflictInfo>> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(None);
    }

    let actual_checksum = crate::fingerprint::calculate_file_checksum(path)?;
    if actual_checksum != expected_checksum {
        return Ok(Some(ConflictInfo {
            path: path.to_path_buf(),
            expected_checksum: expected_checksum.to_string(),
            actual_checksum,
        }));
    }
    Ok(None)
}

/// 複数ファイルの衝突を一括検知
pub fn detect_conflicts<P: AsRef<Path>>(
    base_dir: P,
    checksums: &[(String, String)], // (path, checksum)
) -> Result<Vec<ConflictInfo>> {
    let base_dir = base_dir.as_ref();
    let mut conflicts = Vec::new();

    for (path, expected_checksum) in checksums {
        let full_path = base_dir.join(path);
        if let Some(conflict) = detect_conflict(&full_path, expected_checksum)? {
            conflicts.push(conflict);
        }
    }

    Ok(conflicts)
}

/// 2つのファイルの内容を比較
pub fn files_equal<P: AsRef<Path>, Q: AsRef<Path>>(path1: P, path2: Q) -> Result<bool> {
    let content1 = std::fs::read(path1)?;
    let content2 = std::fs::read(path2)?;
    Ok(content1 == content2)
}

/// ディレクトリを再帰的にコピー
pub fn copy_dir_all<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> Result<()> {
    let src = src.as_ref();
    let dst = dst.as_ref();

    std::fs::create_dir_all(dst)?;

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

/// ファイルを削除（存在しない場合は無視）
pub fn remove_file<P: AsRef<Path>>(path: P) -> Result<bool> {
    let path = path.as_ref();
    if path.exists() {
        std::fs::remove_file(path)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// ディレクトリを再帰的に削除（存在しない場合は無視）
pub fn remove_dir_all<P: AsRef<Path>>(path: P) -> Result<bool> {
    let path = path.as_ref();
    if path.exists() {
        std::fs::remove_dir_all(path)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// 指定パターンにマッチするファイルを列挙する
pub fn glob_files<P: AsRef<Path>>(base_dir: P, pattern: &str) -> Result<Vec<PathBuf>> {
    let base_dir = base_dir.as_ref();
    let full_pattern = base_dir.join(pattern);

    let mut results = Vec::new();
    for entry in glob::glob(&full_pattern.to_string_lossy())? {
        if let Ok(path) = entry {
            results.push(path);
        }
    }

    // ソートして決定論的な順序にする
    results.sort();
    Ok(results)
}

/// 相対パスでファイルを列挙する
pub fn glob_files_relative<P: AsRef<Path>>(base_dir: P, pattern: &str) -> Result<Vec<String>> {
    let base_dir = base_dir.as_ref();
    let full_pattern = base_dir.join(pattern);

    let mut results = Vec::new();
    for entry in glob::glob(&full_pattern.to_string_lossy())? {
        if let Ok(path) = entry {
            if let Ok(relative) = path.strip_prefix(base_dir) {
                // パス区切りを統一（Windows対応）
                let normalized = relative
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy().to_string())
                    .collect::<Vec<_>>()
                    .join("/");
                results.push(normalized);
            }
        }
    }

    // ソートして決定論的な順序にする
    results.sort();
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_write_file_creates_parent_dirs() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("a/b/c/file.txt");

        let result = write_file(&path, "hello").unwrap();
        assert_eq!(result, WriteResult::Created);
        assert!(path.exists());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "hello");
    }

    #[test]
    fn test_write_file_skips_same_content() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("file.txt");

        write_file(&path, "hello").unwrap();
        let result = write_file(&path, "hello").unwrap();
        assert_eq!(result, WriteResult::Skipped);
    }

    #[test]
    fn test_write_file_overwrites_different_content() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("file.txt");

        write_file(&path, "hello").unwrap();
        let result = write_file(&path, "world").unwrap();
        assert_eq!(result, WriteResult::Overwritten);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "world");
    }

    #[test]
    fn test_backup_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("file.txt");
        std::fs::write(&path, "content").unwrap();

        let backup = backup_file(&path).unwrap();
        assert!(backup.is_some());
        let backup_path = backup.unwrap();
        assert!(backup_path.exists());
    }

    #[test]
    fn test_files_equal() {
        let dir = tempdir().unwrap();
        let path1 = dir.path().join("file1.txt");
        let path2 = dir.path().join("file2.txt");
        let path3 = dir.path().join("file3.txt");

        std::fs::write(&path1, "same").unwrap();
        std::fs::write(&path2, "same").unwrap();
        std::fs::write(&path3, "different").unwrap();

        assert!(files_equal(&path1, &path2).unwrap());
        assert!(!files_equal(&path1, &path3).unwrap());
    }

    #[test]
    fn test_copy_dir_all() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("src");
        let dst = dir.path().join("dst");

        std::fs::create_dir_all(src.join("sub")).unwrap();
        std::fs::write(src.join("file.txt"), "root").unwrap();
        std::fs::write(src.join("sub/nested.txt"), "nested").unwrap();

        copy_dir_all(&src, &dst).unwrap();

        assert!(dst.join("file.txt").exists());
        assert!(dst.join("sub/nested.txt").exists());
        assert_eq!(std::fs::read_to_string(dst.join("file.txt")).unwrap(), "root");
        assert_eq!(
            std::fs::read_to_string(dst.join("sub/nested.txt")).unwrap(),
            "nested"
        );
    }
}

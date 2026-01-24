//! 差分計算・表示
//!
//! テンプレート更新時の差分計算と表示を提供する。

use std::path::Path;

use crate::Result;

/// 差分の種類
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffKind {
    /// 追加
    Added,
    /// 削除
    Removed,
    /// 変更
    Modified,
    /// 衝突（手動変更あり）
    Conflict,
}

/// ファイルの差分
#[derive(Debug, Clone)]
pub struct FileDiff {
    /// ファイルパス
    pub path: String,
    /// 差分の種類
    pub kind: DiffKind,
    /// 変更内容（詳細）
    pub content: Option<String>,
}

/// 差分の計算結果
#[derive(Debug, Default)]
pub struct DiffResult {
    /// 追加されたファイル
    pub added: Vec<FileDiff>,
    /// 削除されたファイル
    pub removed: Vec<FileDiff>,
    /// 変更されたファイル
    pub modified: Vec<FileDiff>,
    /// 衝突したファイル
    pub conflicts: Vec<FileDiff>,
}

impl DiffResult {
    /// 変更があるかどうか
    pub fn has_changes(&self) -> bool {
        !self.added.is_empty()
            || !self.removed.is_empty()
            || !self.modified.is_empty()
    }

    /// 衝突があるかどうか
    pub fn has_conflicts(&self) -> bool {
        !self.conflicts.is_empty()
    }

    /// 変更の総数
    pub fn total_changes(&self) -> usize {
        self.added.len() + self.removed.len() + self.modified.len()
    }
}

/// 2つのディレクトリ間の差分を計算する
pub fn calculate_diff<P: AsRef<Path>, Q: AsRef<Path>>(
    _old_dir: P,
    _new_dir: Q,
) -> Result<DiffResult> {
    // TODO: フェーズ32 で実装
    Ok(DiffResult::default())
}

/// managed_paths に含まれるかどうかを判定する
pub fn is_managed_path(path: &str, managed_paths: &[String]) -> bool {
    for pattern in managed_paths {
        if pattern.ends_with('/') {
            // ディレクトリパターン
            if path.starts_with(pattern) || path.starts_with(pattern.trim_end_matches('/')) {
                return true;
            }
        } else {
            // ファイルパターン
            if path == pattern {
                return true;
            }
        }
    }
    false
}

/// protected_paths に含まれるかどうかを判定する
pub fn is_protected_path(path: &str, protected_paths: &[String]) -> bool {
    for pattern in protected_paths {
        if pattern.ends_with('/') {
            if path.starts_with(pattern) || path.starts_with(pattern.trim_end_matches('/')) {
                return true;
            }
        } else if path == pattern {
            return true;
        }
    }
    false
}

/// 差分結果を表示する
pub fn print_diff(diff: &DiffResult) {
    if !diff.added.is_empty() {
        println!("追加されるファイル:");
        for f in &diff.added {
            println!("  + {}", f.path);
        }
        println!();
    }

    if !diff.removed.is_empty() {
        println!("削除されるファイル:");
        for f in &diff.removed {
            println!("  - {}", f.path);
        }
        println!();
    }

    if !diff.modified.is_empty() {
        println!("変更されるファイル:");
        for f in &diff.modified {
            println!("  ~ {}", f.path);
        }
        println!();
    }

    if !diff.conflicts.is_empty() {
        println!("衝突（手動解決が必要）:");
        for f in &diff.conflicts {
            println!("  ! {}", f.path);
        }
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_managed_path() {
        let managed = vec![
            "deploy/".to_string(),
            "config/".to_string(),
            "Cargo.toml".to_string(),
        ];

        assert!(is_managed_path("deploy/base/deployment.yaml", &managed));
        assert!(is_managed_path("config/default.yaml", &managed));
        assert!(is_managed_path("Cargo.toml", &managed));
        assert!(!is_managed_path("src/main.rs", &managed));
    }

    #[test]
    fn test_is_protected_path() {
        let protected = vec![
            "src/domain/".to_string(),
            "src/application/".to_string(),
        ];

        assert!(is_protected_path("src/domain/entities/user.rs", &protected));
        assert!(is_protected_path("src/application/usecases/mod.rs", &protected));
        assert!(!is_protected_path("src/main.rs", &protected));
    }
}

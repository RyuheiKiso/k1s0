//! 差分計算・表示
//!
//! テンプレート更新時の差分計算と表示を提供する。

use std::collections::HashSet;
use std::path::Path;

use crate::walker::Walker;
use crate::Result;

/// 差分の種類
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffKind {
    /// 追加
    Added,
    /// 削除
    Removed,
    /// 変更
    Modified,
    /// 衝突（手動変更あり）
    Conflict,
    /// 変更なし
    Unchanged,
}

impl DiffKind {
    /// 表示用のプレフィックス
    pub fn prefix(&self) -> &'static str {
        match self {
            DiffKind::Added => "+",
            DiffKind::Removed => "-",
            DiffKind::Modified => "~",
            DiffKind::Conflict => "!",
            DiffKind::Unchanged => " ",
        }
    }

    /// 表示用のラベル
    pub fn label(&self) -> &'static str {
        match self {
            DiffKind::Added => "追加",
            DiffKind::Removed => "削除",
            DiffKind::Modified => "変更",
            DiffKind::Conflict => "衝突",
            DiffKind::Unchanged => "変更なし",
        }
    }
}

/// ファイルの差分
#[derive(Debug, Clone)]
pub struct FileDiff {
    /// ファイルパス（相対パス）
    pub path: String,
    /// 差分の種類
    pub kind: DiffKind,
    /// 変更前の内容（Modified/Removed の場合）
    pub old_content: Option<String>,
    /// 変更後の内容（Modified/Added の場合）
    pub new_content: Option<String>,
    /// 期待されるチェックサム（衝突検知用）
    pub expected_checksum: Option<String>,
    /// 実際のチェックサム（衝突検知用）
    pub actual_checksum: Option<String>,
}

impl FileDiff {
    /// 追加されたファイルを作成
    pub fn added(path: impl Into<String>, content: Option<String>) -> Self {
        Self {
            path: path.into(),
            kind: DiffKind::Added,
            old_content: None,
            new_content: content,
            expected_checksum: None,
            actual_checksum: None,
        }
    }

    /// 削除されたファイルを作成
    pub fn removed(path: impl Into<String>, content: Option<String>) -> Self {
        Self {
            path: path.into(),
            kind: DiffKind::Removed,
            old_content: content,
            new_content: None,
            expected_checksum: None,
            actual_checksum: None,
        }
    }

    /// 変更されたファイルを作成
    pub fn modified(
        path: impl Into<String>,
        old_content: Option<String>,
        new_content: Option<String>,
    ) -> Self {
        Self {
            path: path.into(),
            kind: DiffKind::Modified,
            old_content,
            new_content,
            expected_checksum: None,
            actual_checksum: None,
        }
    }

    /// 衝突したファイルを作成
    pub fn conflict(
        path: impl Into<String>,
        expected_checksum: String,
        actual_checksum: String,
    ) -> Self {
        Self {
            path: path.into(),
            kind: DiffKind::Conflict,
            old_content: None,
            new_content: None,
            expected_checksum: Some(expected_checksum),
            actual_checksum: Some(actual_checksum),
        }
    }

    /// 変更なしのファイルを作成
    pub fn unchanged(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            kind: DiffKind::Unchanged,
            old_content: None,
            new_content: None,
            expected_checksum: None,
            actual_checksum: None,
        }
    }
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
    /// 変更なしのファイル
    pub unchanged: Vec<FileDiff>,
}

impl DiffResult {
    /// 変更があるかどうか
    pub fn has_changes(&self) -> bool {
        !self.added.is_empty() || !self.removed.is_empty() || !self.modified.is_empty()
    }

    /// 衝突があるかどうか
    pub fn has_conflicts(&self) -> bool {
        !self.conflicts.is_empty()
    }

    /// 変更の総数
    pub fn total_changes(&self) -> usize {
        self.added.len() + self.removed.len() + self.modified.len()
    }

    /// すべての差分を取得（変更のあるもののみ）
    pub fn all_changes(&self) -> Vec<&FileDiff> {
        let mut all = Vec::new();
        all.extend(self.added.iter());
        all.extend(self.removed.iter());
        all.extend(self.modified.iter());
        all.extend(self.conflicts.iter());
        all
    }

    /// パスでフィルタした差分を取得
    pub fn filter_by_paths(&self, paths: &[String]) -> DiffResult {
        let path_set: HashSet<_> = paths.iter().collect();

        DiffResult {
            added: self
                .added
                .iter()
                .filter(|d| matches_path_patterns(&d.path, &path_set))
                .cloned()
                .collect(),
            removed: self
                .removed
                .iter()
                .filter(|d| matches_path_patterns(&d.path, &path_set))
                .cloned()
                .collect(),
            modified: self
                .modified
                .iter()
                .filter(|d| matches_path_patterns(&d.path, &path_set))
                .cloned()
                .collect(),
            conflicts: self
                .conflicts
                .iter()
                .filter(|d| matches_path_patterns(&d.path, &path_set))
                .cloned()
                .collect(),
            unchanged: self
                .unchanged
                .iter()
                .filter(|d| matches_path_patterns(&d.path, &path_set))
                .cloned()
                .collect(),
        }
    }

    /// サマリーを文字列で取得
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();
        if !self.added.is_empty() {
            parts.push(format!("{} 追加", self.added.len()));
        }
        if !self.modified.is_empty() {
            parts.push(format!("{} 変更", self.modified.len()));
        }
        if !self.removed.is_empty() {
            parts.push(format!("{} 削除", self.removed.len()));
        }
        if !self.conflicts.is_empty() {
            parts.push(format!("{} 衝突", self.conflicts.len()));
        }
        if parts.is_empty() {
            "変更なし".to_string()
        } else {
            parts.join(", ")
        }
    }
}

/// パスがパターンにマッチするか
fn matches_path_patterns(path: &str, patterns: &HashSet<&String>) -> bool {
    for pattern in patterns {
        if pattern.ends_with('/') {
            // ディレクトリパターン
            let prefix = pattern.trim_end_matches('/');
            if path.starts_with(prefix) {
                return true;
            }
        } else if path == *pattern {
            return true;
        }
    }
    false
}

/// 2つのディレクトリ間の差分を計算する
pub fn calculate_diff<P: AsRef<Path>, Q: AsRef<Path>>(
    old_dir: P,
    new_dir: Q,
) -> Result<DiffResult> {
    let old_dir = old_dir.as_ref();
    let new_dir = new_dir.as_ref();

    let old_files: HashSet<_> = Walker::new(old_dir).file_paths()?.into_iter().collect();
    let new_files: HashSet<_> = Walker::new(new_dir).file_paths()?.into_iter().collect();

    let mut result = DiffResult::default();

    // 追加されたファイル
    for path in new_files.difference(&old_files) {
        let content = std::fs::read_to_string(new_dir.join(path)).ok();
        result.added.push(FileDiff::added(path.clone(), content));
    }

    // 削除されたファイル
    for path in old_files.difference(&new_files) {
        let content = std::fs::read_to_string(old_dir.join(path)).ok();
        result.removed.push(FileDiff::removed(path.clone(), content));
    }

    // 両方に存在するファイル
    for path in old_files.intersection(&new_files) {
        let old_content = std::fs::read(old_dir.join(path))?;
        let new_content = std::fs::read(new_dir.join(path))?;

        if old_content != new_content {
            result.modified.push(FileDiff::modified(
                path.clone(),
                String::from_utf8(old_content).ok(),
                String::from_utf8(new_content).ok(),
            ));
        } else {
            result.unchanged.push(FileDiff::unchanged(path.clone()));
        }
    }

    // ソート
    result.added.sort_by(|a, b| a.path.cmp(&b.path));
    result.removed.sort_by(|a, b| a.path.cmp(&b.path));
    result.modified.sort_by(|a, b| a.path.cmp(&b.path));
    result.unchanged.sort_by(|a, b| a.path.cmp(&b.path));

    Ok(result)
}

/// 衝突を検知しながら差分を計算する
pub fn calculate_diff_with_conflicts<P: AsRef<Path>, Q: AsRef<Path>>(
    old_dir: P,
    new_dir: Q,
    checksums: &[(String, String)], // (path, expected_checksum)
) -> Result<DiffResult> {
    let old_dir = old_dir.as_ref();
    let new_dir = new_dir.as_ref();

    let mut result = calculate_diff(old_dir, new_dir)?;

    // チェックサムによる衝突検知
    let checksum_map: std::collections::HashMap<_, _> = checksums.iter().cloned().collect();

    // modified から衝突を分離
    let mut conflicts = Vec::new();
    let mut non_conflicts = Vec::new();

    for diff in result.modified.drain(..) {
        if let Some(expected) = checksum_map.get(&diff.path) {
            let actual = crate::fingerprint::calculate_file_checksum(old_dir.join(&diff.path))?;
            if &actual != expected {
                conflicts.push(FileDiff::conflict(
                    diff.path,
                    expected.clone(),
                    actual,
                ));
            } else {
                non_conflicts.push(diff);
            }
        } else {
            non_conflicts.push(diff);
        }
    }

    result.modified = non_conflicts;
    result.conflicts = conflicts;

    Ok(result)
}

/// managed_paths に含まれるかどうかを判定する
pub fn is_managed_path(path: &str, managed_paths: &[String]) -> bool {
    for pattern in managed_paths {
        if pattern.ends_with('/') {
            // ディレクトリパターン
            let prefix = pattern.trim_end_matches('/');
            if path.starts_with(prefix) || path == prefix {
                return true;
            }
        } else if path == pattern {
            return true;
        }
    }
    false
}

/// protected_paths に含まれるかどうかを判定する
pub fn is_protected_path(path: &str, protected_paths: &[String]) -> bool {
    for pattern in protected_paths {
        if pattern.ends_with('/') {
            let prefix = pattern.trim_end_matches('/');
            if path.starts_with(prefix) || path == prefix {
                return true;
            }
        } else if path == pattern {
            return true;
        }
    }
    false
}

/// 差分結果をテキストで出力する
pub fn format_diff(diff: &DiffResult) -> String {
    let mut output = String::new();

    if !diff.added.is_empty() {
        output.push_str("追加されるファイル:\n");
        for f in &diff.added {
            output.push_str(&format!("  + {}\n", f.path));
        }
        output.push('\n');
    }

    if !diff.removed.is_empty() {
        output.push_str("削除されるファイル:\n");
        for f in &diff.removed {
            output.push_str(&format!("  - {}\n", f.path));
        }
        output.push('\n');
    }

    if !diff.modified.is_empty() {
        output.push_str("変更されるファイル:\n");
        for f in &diff.modified {
            output.push_str(&format!("  ~ {}\n", f.path));
        }
        output.push('\n');
    }

    if !diff.conflicts.is_empty() {
        output.push_str("衝突（手動解決が必要）:\n");
        for f in &diff.conflicts {
            output.push_str(&format!("  ! {}\n", f.path));
        }
        output.push('\n');
    }

    output
}

/// 差分結果を標準エラー出力に表示する（非推奨: format_diff を使用）
pub fn print_diff(diff: &DiffResult) {
    eprint!("{}", format_diff(diff));
}

/// 行単位の差分を計算する
pub fn line_diff(old: &str, new: &str) -> Vec<LineDiff> {
    let old_lines: Vec<&str> = old.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();

    let mut result = Vec::new();
    let mut old_idx = 0;
    let mut new_idx = 0;

    // 簡易的な LCS ベースの差分計算
    while old_idx < old_lines.len() || new_idx < new_lines.len() {
        if old_idx >= old_lines.len() {
            // 新しい行を追加
            result.push(LineDiff::Added(new_lines[new_idx].to_string()));
            new_idx += 1;
        } else if new_idx >= new_lines.len() {
            // 古い行を削除
            result.push(LineDiff::Removed(old_lines[old_idx].to_string()));
            old_idx += 1;
        } else if old_lines[old_idx] == new_lines[new_idx] {
            // 同じ行
            result.push(LineDiff::Unchanged(old_lines[old_idx].to_string()));
            old_idx += 1;
            new_idx += 1;
        } else {
            // 変更あり - 先読みで同期点を探す
            let sync_point = find_sync_point(&old_lines[old_idx..], &new_lines[new_idx..]);
            match sync_point {
                Some((old_skip, new_skip)) => {
                    // 削除
                    for line in old_lines.iter().skip(old_idx).take(old_skip) {
                        result.push(LineDiff::Removed(line.to_string()));
                    }
                    old_idx += old_skip;
                    // 追加
                    for line in new_lines.iter().skip(new_idx).take(new_skip) {
                        result.push(LineDiff::Added(line.to_string()));
                    }
                    new_idx += new_skip;
                }
                None => {
                    // 同期点が見つからない - 残りをすべて変更として扱う
                    for line in old_lines.iter().skip(old_idx) {
                        result.push(LineDiff::Removed(line.to_string()));
                    }
                    for line in new_lines.iter().skip(new_idx) {
                        result.push(LineDiff::Added(line.to_string()));
                    }
                    break;
                }
            }
        }
    }

    result
}

/// 同期点を探す（簡易実装）
fn find_sync_point(old: &[&str], new: &[&str]) -> Option<(usize, usize)> {
    let max_lookahead = 10.min(old.len()).min(new.len());

    for skip in 1..=max_lookahead {
        // old[skip] が new のどこかにあるか
        for (j, new_line) in new.iter().enumerate().take(skip + 1) {
            if old.get(skip) == Some(new_line) {
                return Some((skip, j));
            }
        }
        // new[skip] が old のどこかにあるか
        for (i, old_line) in old.iter().enumerate().take(skip + 1) {
            if new.get(skip) == Some(old_line) {
                return Some((i, skip));
            }
        }
    }

    None
}

/// 行の差分
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LineDiff {
    /// 追加された行
    Added(String),
    /// 削除された行
    Removed(String),
    /// 変更なしの行
    Unchanged(String),
}

impl LineDiff {
    /// 差分を文字列でフォーマット
    pub fn format(&self) -> String {
        match self {
            LineDiff::Added(line) => format!("+ {}", line),
            LineDiff::Removed(line) => format!("- {}", line),
            LineDiff::Unchanged(line) => format!("  {}", line),
        }
    }
}

/// 行差分をテキストでフォーマット
pub fn format_line_diff(diff: &[LineDiff], context: usize) -> String {
    if context == 0 {
        // コンテキストなし - すべて表示
        return diff.iter().map(|d| d.format()).collect::<Vec<_>>().join("\n");
    }

    let mut output = Vec::new();
    let mut last_change_idx: Option<usize> = None;
    let mut pending_unchanged = Vec::new();

    for (i, line) in diff.iter().enumerate() {
        match line {
            LineDiff::Unchanged(content) => {
                if let Some(last_idx) = last_change_idx {
                    // 変更の後のコンテキスト
                    if i - last_idx <= context {
                        output.push(line.format());
                    } else {
                        pending_unchanged.push((i, content.clone()));
                    }
                } else {
                    pending_unchanged.push((i, content.clone()));
                }
            }
            _ => {
                // 変更行の前のコンテキスト
                let start = if pending_unchanged.len() > context {
                    pending_unchanged.len() - context
                } else {
                    0
                };

                if !pending_unchanged.is_empty() && start > 0 && !output.is_empty() {
                    output.push("...".to_string());
                }

                for (_, content) in pending_unchanged.drain(..).skip(start) {
                    output.push(format!("  {}", content));
                }

                output.push(line.format());
                last_change_idx = Some(i);
            }
        }
    }

    output.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

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
        assert!(is_protected_path(
            "src/application/usecases/mod.rs",
            &protected
        ));
        assert!(!is_protected_path("src/main.rs", &protected));
    }

    #[test]
    fn test_calculate_diff() {
        let old_dir = tempdir().unwrap();
        let new_dir = tempdir().unwrap();

        // old のファイル
        std::fs::write(old_dir.path().join("common.txt"), "old content").unwrap();
        std::fs::write(old_dir.path().join("removed.txt"), "to be removed").unwrap();
        std::fs::write(old_dir.path().join("unchanged.txt"), "same").unwrap();

        // new のファイル
        std::fs::write(new_dir.path().join("common.txt"), "new content").unwrap();
        std::fs::write(new_dir.path().join("added.txt"), "new file").unwrap();
        std::fs::write(new_dir.path().join("unchanged.txt"), "same").unwrap();

        let diff = calculate_diff(old_dir.path(), new_dir.path()).unwrap();

        assert_eq!(diff.added.len(), 1);
        assert_eq!(diff.added[0].path, "added.txt");

        assert_eq!(diff.removed.len(), 1);
        assert_eq!(diff.removed[0].path, "removed.txt");

        assert_eq!(diff.modified.len(), 1);
        assert_eq!(diff.modified[0].path, "common.txt");

        assert_eq!(diff.unchanged.len(), 1);
        assert_eq!(diff.unchanged[0].path, "unchanged.txt");
    }

    #[test]
    fn test_diff_result_summary() {
        let mut diff = DiffResult::default();
        assert_eq!(diff.summary(), "変更なし");

        diff.added.push(FileDiff::added("a.txt", None));
        diff.added.push(FileDiff::added("b.txt", None));
        diff.modified.push(FileDiff::modified("c.txt", None, None));

        assert_eq!(diff.summary(), "2 追加, 1 変更");
    }

    #[test]
    fn test_line_diff() {
        let old = "line1\nline2\nline3";
        let new = "line1\nmodified\nline3\nline4";

        let diff = line_diff(old, new);

        assert!(matches!(diff[0], LineDiff::Unchanged(_)));
        assert!(matches!(diff[1], LineDiff::Removed(_)));
        assert!(matches!(diff[2], LineDiff::Added(_)));
    }
}

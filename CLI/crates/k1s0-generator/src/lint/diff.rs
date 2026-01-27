//! Git 差分ベースの lint フィルタリング
//!
//! `--diff <base>` フラグで変更されたファイルのみを対象に lint を実行する。

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::{LintResult, Violation};

/// Git 差分情報
#[derive(Debug, Clone)]
pub struct GitDiff {
    /// ベースブランチ/コミット
    pub base: String,
    /// 変更されたファイル
    pub changed_files: Vec<PathBuf>,
    /// 追加されたファイル
    pub added_files: Vec<PathBuf>,
    /// 削除されたファイル
    pub deleted_files: Vec<PathBuf>,
    /// 変更された行情報（ファイルパス -> 行番号の集合）
    pub changed_lines: std::collections::HashMap<PathBuf, HashSet<usize>>,
}

/// Git 差分取得エラー
#[derive(Debug, thiserror::Error)]
pub enum DiffError {
    /// Git コマンド実行エラー
    #[error("Git command failed: {0}")]
    GitCommand(String),

    /// Git リポジトリでない
    #[error("Not a git repository")]
    NotGitRepo,

    /// ベースブランチが見つからない
    #[error("Base branch/commit not found: {0}")]
    BaseNotFound(String),

    /// パースエラー
    #[error("Failed to parse git output: {0}")]
    ParseError(String),
}

/// 差分フィルター
pub struct DiffFilter {
    /// リポジトリルート
    repo_root: PathBuf,
}

impl DiffFilter {
    /// 新しい差分フィルターを作成
    pub fn new(repo_root: impl AsRef<Path>) -> Self {
        Self {
            repo_root: repo_root.as_ref().to_path_buf(),
        }
    }

    /// Git リポジトリかどうか確認
    pub fn is_git_repo(&self) -> bool {
        self.repo_root.join(".git").exists()
            || Command::new("git")
                .args(["rev-parse", "--git-dir"])
                .current_dir(&self.repo_root)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
    }

    /// 指定されたベースとの差分を取得
    pub fn get_diff(&self, base: &str) -> Result<GitDiff, DiffError> {
        if !self.is_git_repo() {
            return Err(DiffError::NotGitRepo);
        }

        // ベースが存在するか確認
        let verify = Command::new("git")
            .args(["rev-parse", "--verify", base])
            .current_dir(&self.repo_root)
            .output()
            .map_err(|e| DiffError::GitCommand(e.to_string()))?;

        if !verify.status.success() {
            return Err(DiffError::BaseNotFound(base.to_string()));
        }

        // 変更ファイル一覧を取得
        let diff_output = Command::new("git")
            .args(["diff", "--name-status", base])
            .current_dir(&self.repo_root)
            .output()
            .map_err(|e| DiffError::GitCommand(e.to_string()))?;

        if !diff_output.status.success() {
            return Err(DiffError::GitCommand(
                String::from_utf8_lossy(&diff_output.stderr).to_string(),
            ));
        }

        let output = String::from_utf8_lossy(&diff_output.stdout);
        let mut changed_files = Vec::new();
        let mut added_files = Vec::new();
        let mut deleted_files = Vec::new();

        for line in output.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 2 {
                let status = parts[0];
                let file = PathBuf::from(parts[1]);

                match status {
                    "A" => added_files.push(file.clone()),
                    "D" => deleted_files.push(file.clone()),
                    "M" | "R" | "C" => changed_files.push(file.clone()),
                    _ => {
                        // Renamed files (R100, R095, etc.)
                        if status.starts_with('R') && parts.len() >= 3 {
                            changed_files.push(PathBuf::from(parts[2]));
                        } else {
                            changed_files.push(file.clone());
                        }
                    }
                }
            }
        }

        // 詳細な行差分を取得（オプション）
        let changed_lines = self.get_changed_lines(base, &changed_files)?;

        Ok(GitDiff {
            base: base.to_string(),
            changed_files,
            added_files,
            deleted_files,
            changed_lines,
        })
    }

    /// 変更された行を取得
    fn get_changed_lines(
        &self,
        base: &str,
        files: &[PathBuf],
    ) -> Result<std::collections::HashMap<PathBuf, HashSet<usize>>, DiffError> {
        let mut result = std::collections::HashMap::new();

        for file in files {
            let diff_output = Command::new("git")
                .args([
                    "diff",
                    "--unified=0",
                    base,
                    "--",
                    file.to_str().unwrap_or(""),
                ])
                .current_dir(&self.repo_root)
                .output()
                .map_err(|e| DiffError::GitCommand(e.to_string()))?;

            if diff_output.status.success() {
                let output = String::from_utf8_lossy(&diff_output.stdout);
                let lines = parse_diff_lines(&output);
                if !lines.is_empty() {
                    result.insert(file.clone(), lines);
                }
            }
        }

        Ok(result)
    }

    /// lint 結果から差分に関連する違反のみをフィルタ
    pub fn filter_violations(&self, result: &LintResult, diff: &GitDiff) -> LintResult {
        let all_changed: HashSet<&PathBuf> = diff
            .changed_files
            .iter()
            .chain(diff.added_files.iter())
            .collect();

        let filtered_violations: Vec<Violation> = result
            .violations
            .iter()
            .filter(|v| {
                // パスが指定されていない違反は常に含める（manifest 等）
                let Some(path_str) = &v.path else {
                    return true;
                };

                let path = PathBuf::from(path_str);

                // ファイルが変更対象かどうか
                if !all_changed.contains(&path) {
                    return false;
                }

                // 行番号が指定されている場合、変更行かどうか確認
                if let Some(line) = v.line {
                    if let Some(changed_lines) = diff.changed_lines.get(&path) {
                        return changed_lines.contains(&line);
                    }
                }

                true
            })
            .cloned()
            .collect();

        let mut filtered_result = LintResult::new(result.path.clone());
        for v in filtered_violations {
            filtered_result.add_violation(v);
        }

        filtered_result
    }
}

/// diff 出力から変更行番号を解析
fn parse_diff_lines(diff_output: &str) -> HashSet<usize> {
    let mut lines = HashSet::new();

    for line in diff_output.lines() {
        // @@ -old_start,old_count +new_start,new_count @@
        if line.starts_with("@@") && line.contains('+') {
            if let Some(plus_part) = line.split('+').nth(1) {
                let nums: Vec<&str> = plus_part.split(&[',', ' '][..]).collect();
                if let Some(start_str) = nums.first() {
                    if let Ok(start) = start_str.parse::<usize>() {
                        let count = nums
                            .get(1)
                            .and_then(|s| s.parse::<usize>().ok())
                            .unwrap_or(1);

                        for i in 0..count {
                            lines.insert(start + i);
                        }
                    }
                }
            }
        }
    }

    lines
}

/// HEAD との差分を取得（ステージングエリアの変更）
pub fn diff_from_head(repo_root: impl AsRef<Path>) -> Result<GitDiff, DiffError> {
    DiffFilter::new(repo_root).get_diff("HEAD")
}

/// main/master ブランチとの差分を取得
pub fn diff_from_main(repo_root: impl AsRef<Path>) -> Result<GitDiff, DiffError> {
    let filter = DiffFilter::new(&repo_root);

    // main ブランチを試す
    if let Ok(diff) = filter.get_diff("origin/main") {
        return Ok(diff);
    }
    if let Ok(diff) = filter.get_diff("main") {
        return Ok(diff);
    }

    // master ブランチを試す
    if let Ok(diff) = filter.get_diff("origin/master") {
        return Ok(diff);
    }
    if let Ok(diff) = filter.get_diff("master") {
        return Ok(diff);
    }

    Err(DiffError::BaseNotFound(
        "main or master branch".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_diff_lines_simple() {
        let diff = "@@ -1,3 +1,5 @@";
        let lines = parse_diff_lines(diff);

        assert!(lines.contains(&1));
        assert!(lines.contains(&2));
        assert!(lines.contains(&3));
        assert!(lines.contains(&4));
        assert!(lines.contains(&5));
    }

    #[test]
    fn test_parse_diff_lines_single() {
        let diff = "@@ -10 +10 @@";
        let lines = parse_diff_lines(diff);

        assert!(lines.contains(&10));
        assert!(!lines.contains(&9));
        assert!(!lines.contains(&11));
    }

    #[test]
    fn test_parse_diff_lines_multiple() {
        let diff = r#"@@ -1,2 +1,3 @@
+new line
@@ -10,1 +11,2 @@
+another line"#;
        let lines = parse_diff_lines(diff);

        assert!(lines.contains(&1));
        assert!(lines.contains(&2));
        assert!(lines.contains(&3));
        assert!(lines.contains(&11));
        assert!(lines.contains(&12));
    }

    #[test]
    fn test_diff_filter_not_git_repo() {
        use tempfile::tempdir;

        let temp = tempdir().unwrap();
        let filter = DiffFilter::new(temp.path());

        assert!(!filter.is_git_repo());
    }
}

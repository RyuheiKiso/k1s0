//! ディレクトリ走査ユーティリティ
//!
//! 対象/除外の規約に基づいたディレクトリ走査機能を提供する。

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::Result;

/// デフォルトの除外パターン
pub const DEFAULT_EXCLUDE_PATTERNS: &[&str] = &[
    // バージョン管理
    ".git",
    ".svn",
    ".hg",
    // ビルド成果物
    "target",
    "build",
    "dist",
    "out",
    ".next",
    // 依存関係
    "node_modules",
    "vendor",
    ".cargo",
    // IDE/エディタ
    ".idea",
    ".vscode",
    "*.swp",
    "*.swo",
    // OS 生成ファイル
    ".DS_Store",
    "Thumbs.db",
    "desktop.ini",
    // k1s0 内部
    ".k1s0",
];

/// ディレクトリ走査の設定
#[derive(Debug, Clone)]
pub struct WalkConfig {
    /// 除外パターン（ディレクトリ名/ファイル名）
    pub exclude_patterns: Vec<String>,
    /// 除外するパス（相対パス）
    pub exclude_paths: Vec<String>,
    /// 対象とするパス（相対パス）。空の場合はすべてが対象
    pub include_paths: Vec<String>,
    /// 隠しファイルを含めるか
    pub include_hidden: bool,
    /// シンボリックリンクをたどるか
    pub follow_symlinks: bool,
    /// 最大深度（0 = 無制限）
    pub max_depth: usize,
}

impl Default for WalkConfig {
    fn default() -> Self {
        Self {
            exclude_patterns: DEFAULT_EXCLUDE_PATTERNS
                .iter()
                .map(|s| s.to_string())
                .collect(),
            exclude_paths: Vec::new(),
            include_paths: Vec::new(),
            include_hidden: false,
            follow_symlinks: false,
            max_depth: 0,
        }
    }
}

impl WalkConfig {
    /// 新しい設定を作成
    pub fn new() -> Self {
        Self::default()
    }

    /// 除外パターンを追加
    pub fn exclude(mut self, pattern: &str) -> Self {
        self.exclude_patterns.push(pattern.to_string());
        self
    }

    /// 除外パスを追加
    pub fn exclude_path(mut self, path: &str) -> Self {
        self.exclude_paths.push(path.to_string());
        self
    }

    /// 対象パスを追加
    pub fn include_path(mut self, path: &str) -> Self {
        self.include_paths.push(path.to_string());
        self
    }

    /// 隠しファイルを含める
    pub fn with_hidden(mut self) -> Self {
        self.include_hidden = true;
        self
    }

    /// シンボリックリンクをたどる
    pub fn with_symlinks(mut self) -> Self {
        self.follow_symlinks = true;
        self
    }

    /// 最大深度を設定
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }
}

/// 走査されたエントリ
#[derive(Debug, Clone)]
pub struct WalkEntry {
    /// 絶対パス
    pub path: PathBuf,
    /// 相対パス（基点からの）
    pub relative_path: String,
    /// ファイルかどうか
    pub is_file: bool,
    /// ディレクトリかどうか
    pub is_dir: bool,
    /// ファイルサイズ（ファイルの場合）
    pub size: Option<u64>,
}

/// ディレクトリウォーカー
pub struct Walker {
    base_dir: PathBuf,
    config: WalkConfig,
}

impl Walker {
    /// 新しいウォーカーを作成
    pub fn new<P: AsRef<Path>>(base_dir: P) -> Self {
        Self {
            base_dir: base_dir.as_ref().to_path_buf(),
            config: WalkConfig::default(),
        }
    }

    /// 設定を指定して作成
    pub fn with_config<P: AsRef<Path>>(base_dir: P, config: WalkConfig) -> Self {
        Self {
            base_dir: base_dir.as_ref().to_path_buf(),
            config,
        }
    }

    /// ファイルのみを列挙
    pub fn files(&self) -> Result<Vec<WalkEntry>> {
        self.walk_filtered(true, false)
    }

    /// ディレクトリのみを列挙
    pub fn directories(&self) -> Result<Vec<WalkEntry>> {
        self.walk_filtered(false, true)
    }

    /// すべてのエントリを列挙
    pub fn all(&self) -> Result<Vec<WalkEntry>> {
        self.walk_filtered(true, true)
    }

    /// ファイルパスのみを列挙（相対パス）
    pub fn file_paths(&self) -> Result<Vec<String>> {
        Ok(self.files()?.into_iter().map(|e| e.relative_path).collect())
    }

    /// フィルタ付きで走査
    fn walk_filtered(&self, include_files: bool, include_dirs: bool) -> Result<Vec<WalkEntry>> {
        let mut entries = Vec::new();
        let include_set: HashSet<_> = self.config.include_paths.iter().collect();

        let mut walker = WalkDir::new(&self.base_dir).follow_links(self.config.follow_symlinks);

        if self.config.max_depth > 0 {
            walker = walker.max_depth(self.config.max_depth);
        }

        for entry in walker.into_iter().filter_entry(|e| self.should_include(e)) {
            let entry = entry?;
            let path = entry.path();

            // 基点自体はスキップ
            if path == self.base_dir {
                continue;
            }

            let is_file = entry.file_type().is_file();
            let is_dir = entry.file_type().is_dir();

            // ファイル/ディレクトリのフィルタ
            if (is_file && !include_files) || (is_dir && !include_dirs) {
                continue;
            }

            // 相対パスを計算
            let relative_path = path
                .strip_prefix(&self.base_dir)
                .map(|p| {
                    p.components()
                        .map(|c| c.as_os_str().to_string_lossy().to_string())
                        .collect::<Vec<_>>()
                        .join("/")
                })
                .unwrap_or_default();

            // include_paths が指定されている場合、マッチするもののみ
            if !include_set.is_empty() && !self.matches_include(&relative_path) {
                continue;
            }

            // exclude_paths のチェック
            if self.matches_exclude_path(&relative_path) {
                continue;
            }

            let size = if is_file {
                entry.metadata().ok().map(|m| m.len())
            } else {
                None
            };

            entries.push(WalkEntry {
                path: path.to_path_buf(),
                relative_path,
                is_file,
                is_dir,
                size,
            });
        }

        // ソートして決定論的な順序にする
        entries.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
        Ok(entries)
    }

    /// エントリを含めるべきかどうか
    fn should_include(&self, entry: &walkdir::DirEntry) -> bool {
        let name = entry.file_name().to_string_lossy();

        // 隠しファイルのチェック
        if !self.config.include_hidden && name.starts_with('.') {
            // .k1s0 は常に除外（隠しファイル設定に関係なく）
            if name == ".k1s0" {
                return false;
            }
            // 隠しファイルを含めない設定で、デフォルト除外パターンにない隠しファイル
            if !self
                .config
                .exclude_patterns
                .iter()
                .any(|p| name == *p || p.starts_with('.'))
            {
                return false;
            }
        }

        // 除外パターンのチェック
        for pattern in &self.config.exclude_patterns {
            if pattern.contains('*') {
                // ワイルドカードパターン
                if matches_wildcard(&name, pattern) {
                    return false;
                }
            } else if name == *pattern {
                return false;
            }
        }

        true
    }

    /// include_paths にマッチするか
    fn matches_include(&self, relative_path: &str) -> bool {
        for include in &self.config.include_paths {
            if include.ends_with('/') {
                // ディレクトリパターン
                let prefix = include.trim_end_matches('/');
                if relative_path.starts_with(prefix) {
                    return true;
                }
            } else if relative_path == include || relative_path.starts_with(&format!("{}/", include))
            {
                return true;
            }
        }
        false
    }

    /// exclude_paths にマッチするか
    fn matches_exclude_path(&self, relative_path: &str) -> bool {
        for exclude in &self.config.exclude_paths {
            if exclude.ends_with('/') {
                let prefix = exclude.trim_end_matches('/');
                if relative_path.starts_with(prefix) {
                    return true;
                }
            } else if relative_path == exclude {
                return true;
            }
        }
        false
    }
}

impl From<walkdir::Error> for crate::Error {
    fn from(e: walkdir::Error) -> Self {
        crate::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    }
}

/// シンプルなワイルドカードマッチ（* のみサポート）
fn matches_wildcard(name: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    if let Some(suffix) = pattern.strip_prefix('*') {
        // *.ext パターン
        return name.ends_with(suffix);
    }

    if let Some(prefix) = pattern.strip_suffix('*') {
        // prefix* パターン
        return name.starts_with(prefix);
    }

    // *を含む中間パターン
    if let Some(idx) = pattern.find('*') {
        let prefix = &pattern[..idx];
        let suffix = &pattern[idx + 1..];
        return name.starts_with(prefix) && name.ends_with(suffix);
    }

    name == pattern
}

/// ディレクトリ内のファイルを簡易列挙
pub fn list_files<P: AsRef<Path>>(dir: P) -> Result<Vec<String>> {
    Walker::new(dir).file_paths()
}

/// ディレクトリ内のファイルを設定付きで列挙
pub fn list_files_with_config<P: AsRef<Path>>(dir: P, config: WalkConfig) -> Result<Vec<String>> {
    Walker::with_config(dir, config).file_paths()
}

/// 2つのディレクトリのファイルリストを比較
pub fn compare_directories<P: AsRef<Path>, Q: AsRef<Path>>(
    dir1: P,
    dir2: Q,
) -> Result<DirectoryComparison> {
    let files1: HashSet<_> = Walker::new(&dir1).file_paths()?.into_iter().collect();
    let files2: HashSet<_> = Walker::new(&dir2).file_paths()?.into_iter().collect();

    let only_in_first: Vec<_> = files1.difference(&files2).cloned().collect();
    let only_in_second: Vec<_> = files2.difference(&files1).cloned().collect();
    let in_both: Vec<_> = files1.intersection(&files2).cloned().collect();

    Ok(DirectoryComparison {
        only_in_first,
        only_in_second,
        in_both,
    })
}

/// ディレクトリ比較の結果
#[derive(Debug, Clone)]
pub struct DirectoryComparison {
    /// 最初のディレクトリにのみ存在
    pub only_in_first: Vec<String>,
    /// 2番目のディレクトリにのみ存在
    pub only_in_second: Vec<String>,
    /// 両方に存在
    pub in_both: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_walker_basic() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("file1.txt"), "").unwrap();
        std::fs::write(dir.path().join("file2.rs"), "").unwrap();
        std::fs::create_dir(dir.path().join("subdir")).unwrap();
        std::fs::write(dir.path().join("subdir/nested.txt"), "").unwrap();

        let files = Walker::new(dir.path()).file_paths().unwrap();
        assert_eq!(files.len(), 3);
        assert!(files.contains(&"file1.txt".to_string()));
        assert!(files.contains(&"file2.rs".to_string()));
        assert!(files.contains(&"subdir/nested.txt".to_string()));
    }

    #[test]
    fn test_walker_excludes_patterns() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("file.txt"), "").unwrap();
        std::fs::create_dir(dir.path().join("node_modules")).unwrap();
        std::fs::write(dir.path().join("node_modules/pkg.js"), "").unwrap();

        let files = Walker::new(dir.path()).file_paths().unwrap();
        assert_eq!(files.len(), 1);
        assert!(files.contains(&"file.txt".to_string()));
    }

    #[test]
    fn test_walker_custom_exclude() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("keep.txt"), "").unwrap();
        std::fs::write(dir.path().join("skip.tmp"), "").unwrap();

        let config = WalkConfig::new().exclude("*.tmp");
        let files = Walker::with_config(dir.path(), config)
            .file_paths()
            .unwrap();
        assert_eq!(files.len(), 1);
        assert!(files.contains(&"keep.txt".to_string()));
    }

    #[test]
    fn test_walker_include_paths() {
        let dir = tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        std::fs::create_dir_all(dir.path().join("tests")).unwrap();
        std::fs::write(dir.path().join("src/main.rs"), "").unwrap();
        std::fs::write(dir.path().join("tests/test.rs"), "").unwrap();
        std::fs::write(dir.path().join("README.md"), "").unwrap();

        let config = WalkConfig::new().include_path("src/");
        let files = Walker::with_config(dir.path(), config)
            .file_paths()
            .unwrap();
        assert_eq!(files.len(), 1);
        assert!(files.contains(&"src/main.rs".to_string()));
    }

    #[test]
    fn test_matches_wildcard() {
        assert!(matches_wildcard("file.txt", "*.txt"));
        assert!(matches_wildcard("test.rs", "*.rs"));
        assert!(!matches_wildcard("file.txt", "*.rs"));
        assert!(matches_wildcard("prefix_test", "prefix*"));
        assert!(matches_wildcard("anything", "*"));
    }

    #[test]
    fn test_compare_directories() {
        let dir1 = tempdir().unwrap();
        let dir2 = tempdir().unwrap();

        std::fs::write(dir1.path().join("common.txt"), "").unwrap();
        std::fs::write(dir1.path().join("only1.txt"), "").unwrap();
        std::fs::write(dir2.path().join("common.txt"), "").unwrap();
        std::fs::write(dir2.path().join("only2.txt"), "").unwrap();

        let comparison = compare_directories(dir1.path(), dir2.path()).unwrap();
        assert!(comparison.only_in_first.contains(&"only1.txt".to_string()));
        assert!(comparison.only_in_second.contains(&"only2.txt".to_string()));
        assert!(comparison.in_both.contains(&"common.txt".to_string()));
    }
}

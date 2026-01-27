//! ファイル監視機能
//!
//! `--watch` フラグで継続的な lint 実行をサポートする。

use std::path::{Path, PathBuf};

#[cfg(feature = "watch")]
use std::sync::mpsc::{channel, Receiver, Sender};
#[cfg(feature = "watch")]
use std::time::Duration;

#[cfg(feature = "watch")]
use notify::RecursiveMode;
#[cfg(feature = "watch")]
use notify_debouncer_mini::{new_debouncer, DebouncedEvent, DebouncedEventKind};

use super::{LintConfig, LintResult, Linter};

/// ファイル変更イベント
#[derive(Debug, Clone)]
pub struct FileChangeEvent {
    /// 変更されたファイルパス
    pub paths: Vec<PathBuf>,
    /// イベント種別
    pub kind: FileChangeKind,
}

/// ファイル変更種別
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileChangeKind {
    /// 作成
    Created,
    /// 変更
    Modified,
    /// 削除
    Removed,
    /// その他
    Other,
}

/// 監視設定
#[derive(Debug, Clone)]
pub struct WatchConfig {
    /// デバウンス間隔（ミリ秒）
    pub debounce_ms: u64,
    /// 監視対象のパターン（glob）
    pub include_patterns: Vec<String>,
    /// 除外パターン（glob）
    pub exclude_patterns: Vec<String>,
    /// 再帰的に監視するかどうか
    pub recursive: bool,
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            debounce_ms: 500,
            include_patterns: vec!["**/*".to_string()],
            exclude_patterns: vec![
                "**/target/**".to_string(),
                "**/node_modules/**".to_string(),
                "**/.git/**".to_string(),
                "**/*.lock".to_string(),
            ],
            recursive: true,
        }
    }
}

/// ファイル監視ウォッチャー
pub struct LintWatcher {
    /// lint 設定
    #[allow(dead_code)]
    lint_config: LintConfig,
    /// 監視設定
    watch_config: WatchConfig,
    /// 監視対象ディレクトリ
    #[allow(dead_code)]
    root_path: PathBuf,
}

impl LintWatcher {
    /// 新しいウォッチャーを作成
    pub fn new(root_path: impl AsRef<Path>, lint_config: LintConfig) -> Self {
        Self {
            lint_config,
            watch_config: WatchConfig::default(),
            root_path: root_path.as_ref().to_path_buf(),
        }
    }

    /// 監視設定を変更
    pub fn with_watch_config(mut self, config: WatchConfig) -> Self {
        self.watch_config = config;
        self
    }

    /// パスが除外対象かどうか
    #[allow(dead_code)]
    fn is_excluded(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        for pattern in &self.watch_config.exclude_patterns {
            if let Ok(glob_pattern) = glob::Pattern::new(pattern) {
                if glob_pattern.matches(&path_str) {
                    return true;
                }
            }
        }

        false
    }

    /// lint を実行する
    #[allow(dead_code)]
    fn run_lint(&self) -> LintResult {
        let linter = Linter::new(self.lint_config.clone());
        linter.lint(&self.root_path)
    }

    /// ファイル監視を開始（ブロッキング）
    ///
    /// 変更が検出されるたびにコールバックが呼ばれる。
    /// `callback` は `LintResult` を受け取り、継続するかどうかを返す。
    /// `false` を返すと監視を終了する。
    #[cfg(feature = "watch")]
    pub fn watch<F>(&self, mut callback: F) -> anyhow::Result<()>
    where
        F: FnMut(LintResult, Option<&FileChangeEvent>) -> bool,
    {
        use anyhow::Context;

        // 初回 lint 実行
        let initial_result = self.run_lint();
        if !callback(initial_result, None) {
            return Ok(());
        }

        // デバウンサー付きウォッチャーを作成
        let (tx, rx): (Sender<Result<Vec<DebouncedEvent>, _>>, Receiver<_>) = channel();

        let mut debouncer = new_debouncer(
            Duration::from_millis(self.watch_config.debounce_ms),
            tx,
        )
        .context("Failed to create file watcher")?;

        // 監視を開始
        let mode = if self.watch_config.recursive {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };

        debouncer
            .watcher()
            .watch(&self.root_path, mode)
            .context("Failed to start watching")?;

        // イベントループ
        loop {
            match rx.recv() {
                Ok(Ok(events)) => {
                    // 除外対象をフィルタ
                    let relevant_paths: Vec<PathBuf> = events
                        .iter()
                        .filter(|e| !self.is_excluded(&e.path))
                        .map(|e| e.path.clone())
                        .collect();

                    if relevant_paths.is_empty() {
                        continue;
                    }

                    // 変更イベントを作成
                    let change_event = FileChangeEvent {
                        paths: relevant_paths,
                        kind: events
                            .first()
                            .map(|e| match e.kind {
                                DebouncedEventKind::Any => FileChangeKind::Modified,
                                DebouncedEventKind::AnyContinuous => FileChangeKind::Modified,
                                _ => FileChangeKind::Other,
                            })
                            .unwrap_or(FileChangeKind::Other),
                    };

                    // lint を再実行
                    let result = self.run_lint();
                    if !callback(result, Some(&change_event)) {
                        break;
                    }
                }
                Ok(Err(e)) => {
                    eprintln!("Watch error: {:?}", e);
                }
                Err(_) => {
                    // チャンネルが閉じられた
                    break;
                }
            }
        }

        Ok(())
    }

    /// ファイル監視を開始（watch feature が無効な場合）
    #[cfg(not(feature = "watch"))]
    pub fn watch<F>(&self, _callback: F) -> anyhow::Result<()>
    where
        F: FnMut(LintResult, Option<&FileChangeEvent>) -> bool,
    {
        anyhow::bail!("Watch mode is not available. Rebuild with `--features watch`")
    }
}

/// 変更されたファイルのみを対象に lint する設定を生成
#[allow(dead_code)]
pub fn filter_changed_files(
    original_config: &LintConfig,
    changed_files: &[PathBuf],
) -> LintConfig {
    let config = original_config.clone();

    // 変更ファイルを allowlist として設定（将来的な拡張用）
    // 現在は lint 全体を実行するが、将来的にはファイル単位の lint をサポート
    let _ = changed_files;

    config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watch_config_default() {
        let config = WatchConfig::default();

        assert_eq!(config.debounce_ms, 500);
        assert!(config.recursive);
        assert!(!config.exclude_patterns.is_empty());
    }

    #[test]
    fn test_is_excluded() {
        let watcher = LintWatcher::new(".", LintConfig::default());

        // 除外対象
        assert!(watcher.is_excluded(Path::new("target/debug/main")));
        assert!(watcher.is_excluded(Path::new("node_modules/foo/bar.js")));
        assert!(watcher.is_excluded(Path::new(".git/objects/abc")));

        // 除外対象でない
        assert!(!watcher.is_excluded(Path::new("src/main.rs")));
        assert!(!watcher.is_excluded(Path::new("config/dev.yaml")));
    }

    #[test]
    fn test_file_change_kind() {
        assert_eq!(FileChangeKind::Created, FileChangeKind::Created);
        assert_ne!(FileChangeKind::Created, FileChangeKind::Modified);
    }
}

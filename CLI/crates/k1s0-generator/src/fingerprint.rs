//! テンプレートの fingerprint 算出
//!
//! テンプレートディレクトリから SHA-256 ハッシュを算出する。
//! 詳細は ADR-0003 を参照。

use sha2::{Digest, Sha256};
use std::path::Path;
use walkdir::WalkDir;

use crate::Result;

/// 除外するディレクトリ/ファイル名パターン
///
/// これらのパターンに一致するパスは fingerprint 算出から除外される。
/// 詳細は ADR-0003 を参照。
const EXCLUDE_PATTERNS: &[&str] = &[
    // バージョン管理
    ".git",
    ".svn",
    ".hg",
    // ビルド成果物
    "target",       // Rust
    "node_modules", // Node.js
    "dist",         // 一般的なビルド出力
    "build",        // 一般的なビルド出力
    "__pycache__",  // Python
    ".dart_tool",   // Dart/Flutter
    // OS メタデータ
    ".DS_Store",
    "Thumbs.db",
    "Desktop.ini",
    // IDE/エディタ
    ".idea",
    ".vscode",
    // k1s0 メタデータ（生成後のもの）
    ".k1s0",
];

/// 除外するファイル拡張子
const EXCLUDE_EXTENSIONS: &[&str] = &[".pyc", ".pyo", ".log", ".tmp", ".bak", ".swp", ".swo"];

/// 除外するファイル名（完全一致）
const EXCLUDE_FILES: &[&str] = &[".env", ".env.local"];

/// ディレクトリの fingerprint を算出する
///
/// テンプレートディレクトリ内のファイルから決定論的なハッシュを算出する。
/// - ファイルは相対パスでソートされる
/// - パス区切りは `/` に正規化される（OS 非依存）
/// - 除外パターンに一致するファイルはスキップされる
pub fn calculate_fingerprint<P: AsRef<Path>>(dir: P) -> Result<String> {
    let dir = dir.as_ref();
    let mut hasher = Sha256::new();

    // ファイルをソートして決定論的な順序にする
    let mut entries: Vec<_> = WalkDir::new(dir)
        .into_iter()
        .filter_entry(|e| !should_exclude(e.path()))
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .collect();

    entries.sort_by(|a, b| a.path().cmp(b.path()));

    for entry in entries {
        let path = entry.path();
        let relative_path = path.strip_prefix(dir).unwrap_or(path);

        // パス区切りを `/` に正規化（OS 非依存）
        let normalized_path = normalize_path(relative_path);

        // パスをハッシュに含める
        hasher.update(normalized_path.as_bytes());
        hasher.update(b"\0");

        // ファイル内容をハッシュに含める
        let content = std::fs::read(path)?;
        hasher.update(&content);
        hasher.update(b"\0");
    }

    let result = hasher.finalize();
    Ok(hex::encode(result))
}

/// パス区切りを `/` に正規化する（OS 非依存）
fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// ファイルの checksum を算出する
pub fn calculate_file_checksum<P: AsRef<Path>>(path: P) -> Result<String> {
    let content = std::fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&content);
    let result = hasher.finalize();
    Ok(hex::encode(result))
}

/// 除外すべきパスかどうかを判定する
fn should_exclude(path: &Path) -> bool {
    // ディレクトリ/ファイル名パターンのチェック
    for pattern in EXCLUDE_PATTERNS {
        if path
            .components()
            .any(|c| c.as_os_str().to_string_lossy() == *pattern)
        {
            return true;
        }
    }

    // ファイル名の完全一致チェック
    if let Some(file_name) = path.file_name() {
        let file_name_str = file_name.to_string_lossy();
        if EXCLUDE_FILES.iter().any(|&f| file_name_str == f) {
            return true;
        }
    }

    // 拡張子のチェック
    if let Some(file_name) = path.file_name() {
        let file_name_str = file_name.to_string_lossy();
        for ext in EXCLUDE_EXTENSIONS {
            if file_name_str.ends_with(ext) {
                return true;
            }
        }
    }

    false
}

// hex エンコード用のヘルパー
mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes
            .as_ref()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_exclude_patterns() {
        // バージョン管理
        assert!(should_exclude(Path::new(".git/config")));
        assert!(should_exclude(Path::new(".svn/entries")));
        assert!(should_exclude(Path::new(".hg/store")));

        // ビルド成果物
        assert!(should_exclude(Path::new("target/debug/k1s0")));
        assert!(should_exclude(Path::new("node_modules/pkg/index.js")));
        assert!(should_exclude(Path::new("dist/bundle.js")));
        assert!(should_exclude(Path::new("build/output.js")));
        assert!(should_exclude(Path::new("__pycache__/module.cpython-39.pyc")));
        assert!(should_exclude(Path::new(".dart_tool/package_config.json")));

        // OS メタデータ
        assert!(should_exclude(Path::new(".DS_Store")));
        assert!(should_exclude(Path::new("subdir/.DS_Store")));
        assert!(should_exclude(Path::new("Thumbs.db")));
        assert!(should_exclude(Path::new("Desktop.ini")));

        // IDE/エディタ
        assert!(should_exclude(Path::new(".idea/workspace.xml")));
        assert!(should_exclude(Path::new(".vscode/settings.json")));

        // k1s0 メタデータ
        assert!(should_exclude(Path::new(".k1s0/manifest.json")));

        // 通常のファイルは除外しない
        assert!(!should_exclude(Path::new("src/main.rs")));
        assert!(!should_exclude(Path::new("README.md")));
    }

    #[test]
    fn test_should_exclude_extensions() {
        assert!(should_exclude(Path::new("module.pyc")));
        assert!(should_exclude(Path::new("cache.pyo")));
        assert!(should_exclude(Path::new("debug.log")));
        assert!(should_exclude(Path::new("temp.tmp")));
        assert!(should_exclude(Path::new("config.bak")));
        assert!(should_exclude(Path::new("file.swp")));
        assert!(should_exclude(Path::new("file.swo")));

        // 通常の拡張子は除外しない
        assert!(!should_exclude(Path::new("main.rs")));
        assert!(!should_exclude(Path::new("index.ts")));
        assert!(!should_exclude(Path::new("style.css")));
    }

    #[test]
    fn test_should_exclude_files() {
        assert!(should_exclude(Path::new(".env")));
        assert!(should_exclude(Path::new(".env.local")));
        assert!(should_exclude(Path::new("subdir/.env")));
        assert!(should_exclude(Path::new("subdir/.env.local")));

        // 類似名は除外しない
        assert!(!should_exclude(Path::new(".env.example")));
        assert!(!should_exclude(Path::new(".env.production")));
    }

    #[test]
    fn test_normalize_path() {
        assert_eq!(normalize_path(Path::new("src/main.rs")), "src/main.rs");
        assert_eq!(normalize_path(Path::new("src\\main.rs")), "src/main.rs");
        assert_eq!(
            normalize_path(Path::new("deep\\nested\\path\\file.txt")),
            "deep/nested/path/file.txt"
        );
    }

    #[test]
    fn test_hex_encode() {
        assert_eq!(hex::encode([0x00, 0xff, 0x10, 0xab]), "00ff10ab");
        assert_eq!(hex::encode([]), "");
    }
}

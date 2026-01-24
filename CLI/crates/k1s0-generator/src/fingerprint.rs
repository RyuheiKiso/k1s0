//! テンプレートの fingerprint 算出
//!
//! テンプレートディレクトリから SHA-256 ハッシュを算出する。

use sha2::{Digest, Sha256};
use std::path::Path;
use walkdir::WalkDir;

use crate::Result;

/// 除外するパターン
const EXCLUDE_PATTERNS: &[&str] = &[
    ".git",
    "target",
    "node_modules",
    ".DS_Store",
    "Thumbs.db",
];

/// ディレクトリの fingerprint を算出する
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

        // パスをハッシュに含める
        hasher.update(relative_path.to_string_lossy().as_bytes());
        hasher.update(b"\0");

        // ファイル内容をハッシュに含める
        let content = std::fs::read(path)?;
        hasher.update(&content);
        hasher.update(b"\0");
    }

    let result = hasher.finalize();
    Ok(hex::encode(result))
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
    for pattern in EXCLUDE_PATTERNS {
        if path
            .components()
            .any(|c| c.as_os_str().to_string_lossy() == *pattern)
        {
            return true;
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
    fn test_should_exclude() {
        assert!(should_exclude(Path::new(".git/config")));
        assert!(should_exclude(Path::new("target/debug/k1s0")));
        assert!(should_exclude(Path::new("node_modules/pkg/index.js")));
        assert!(!should_exclude(Path::new("src/main.rs")));
    }
}

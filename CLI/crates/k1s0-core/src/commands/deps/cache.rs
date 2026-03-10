//! 依存関係マップのキャッシュ機能。
//!
//! `.k1s0/deps-cache.json` にキャッシュを保存し、
//! ファイルハッシュ（SHA256）で変更検出を行う。

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::Result;
use sha2::{Digest, Sha256};

use super::types::{Dependency, DepsCache, Violation};

/// キャッシュバージョン。フォーマット変更時にインクリメントする。
const CACHE_VERSION: u32 = 1;

/// キャッシュファイル名。
const CACHE_FILE: &str = "deps-cache.json";

/// キャッシュを読み込む。
///
/// キャッシュファイルが存在しない場合や読み込みに失敗した場合は `None` を返す。
pub fn load_cache(cache_dir: &Path) -> Option<DepsCache> {
    let cache_path = cache_dir.join(CACHE_FILE);
    let content = fs::read_to_string(cache_path).ok()?;
    let cache: DepsCache = serde_json::from_str(&content).ok()?;

    // バージョンチェック
    if cache.version != CACHE_VERSION {
        return None;
    }

    Some(cache)
}

/// キャッシュを保存する。
///
/// # Errors
///
/// ファイル書き込みに失敗した場合にエラーを返す。
pub fn save_cache(
    cache_dir: &Path,
    dependencies: &[Dependency],
    violations: &[Violation],
    file_hashes: &HashMap<String, String>,
) -> Result<()> {
    fs::create_dir_all(cache_dir)?;

    let cache = DepsCache {
        version: CACHE_VERSION,
        generated_at: chrono::Utc::now().to_rfc3339(),
        file_hashes: file_hashes.clone(),
        dependencies: dependencies.to_vec(),
        violations: violations.to_vec(),
    };

    let content = serde_json::to_string_pretty(&cache)?;
    fs::write(cache_dir.join(CACHE_FILE), content)?;

    Ok(())
}

/// キャッシュが有効かどうかを判定する。
///
/// 現在のファイルハッシュとキャッシュ内のハッシュを比較し、
/// すべて一致していれば有効と判定する。
pub fn is_cache_valid(cache: &DepsCache, current_hashes: &HashMap<String, String>) -> bool {
    if cache.version != CACHE_VERSION {
        return false;
    }

    // ファイル数が異なる場合は無効
    if cache.file_hashes.len() != current_hashes.len() {
        return false;
    }

    // すべてのハッシュが一致するか
    for (path, hash) in &cache.file_hashes {
        match current_hashes.get(path) {
            Some(current_hash) if current_hash == hash => {}
            _ => return false,
        }
    }

    true
}

/// 解析対象ファイルのSHA256ハッシュを計算する。
///
/// 対象ファイル:
/// - `*.proto`
/// - `config.yaml`
/// - `Cargo.toml`, `go.mod`, `package.json`, `pubspec.yaml`
/// - `*.rs`, `*.go`, `*.ts`, `*.dart`
///
/// # Errors
///
/// ファイル読み込みに失敗した場合にエラーを返す。
pub fn compute_file_hashes(base_dir: &Path) -> Result<HashMap<String, String>> {
    let mut hashes = HashMap::new();
    let regions = base_dir.join("regions");
    if !regions.is_dir() {
        return Ok(hashes);
    }

    compute_hashes_recursive(&regions, base_dir, &mut hashes)?;
    Ok(hashes)
}

/// 再帰的にファイルハッシュを計算する。
fn compute_hashes_recursive(
    dir: &Path,
    base_dir: &Path,
    hashes: &mut HashMap<String, String>,
) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(dir)?.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let dir_name = path.file_name().unwrap_or_default().to_string_lossy();
            if dir_name == "target"
                || dir_name == "node_modules"
                || dir_name == ".git"
                || dir_name == "vendor"
            {
                continue;
            }
            compute_hashes_recursive(&path, base_dir, hashes)?;
        } else if is_hashable_file(&path) {
            let content = fs::read(&path)?;
            let hash = format!("sha256:{:x}", Sha256::digest(&content));
            let relative = path
                .strip_prefix(base_dir)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();
            hashes.insert(relative, hash);
        }
    }

    Ok(())
}

/// ハッシュ計算対象のファイルかどうかを判定する。
fn is_hashable_file(path: &Path) -> bool {
    let file_name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    // 特定のファイル名
    if matches!(
        file_name.as_str(),
        "config.yaml" | "Cargo.toml" | "go.mod" | "package.json" | "pubspec.yaml"
    ) {
        return true;
    }

    // 拡張子
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy();
        return matches!(ext_str.as_ref(), "proto" | "rs" | "go" | "ts" | "dart");
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // ========================================================================
    // load_cache テスト
    // ========================================================================

    #[test]
    fn test_load_cache_no_file() {
        let tmp = TempDir::new().unwrap();
        let result = load_cache(tmp.path());
        assert!(result.is_none());
    }

    #[test]
    fn test_load_cache_invalid_json() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join(CACHE_FILE), "invalid json").unwrap();
        let result = load_cache(tmp.path());
        assert!(result.is_none());
    }

    #[test]
    fn test_load_cache_wrong_version() {
        let tmp = TempDir::new().unwrap();
        let cache = DepsCache {
            version: 999,
            generated_at: "2026-01-01T00:00:00Z".to_string(),
            file_hashes: HashMap::new(),
            dependencies: vec![],
            violations: vec![],
        };
        let content = serde_json::to_string(&cache).unwrap();
        fs::write(tmp.path().join(CACHE_FILE), content).unwrap();

        let result = load_cache(tmp.path());
        assert!(result.is_none(), "バージョン不一致の場合はNoneを返すこと");
    }

    #[test]
    fn test_load_cache_valid() {
        let tmp = TempDir::new().unwrap();
        let cache = DepsCache {
            version: CACHE_VERSION,
            generated_at: "2026-01-01T00:00:00Z".to_string(),
            file_hashes: HashMap::new(),
            dependencies: vec![],
            violations: vec![],
        };
        let content = serde_json::to_string(&cache).unwrap();
        fs::write(tmp.path().join(CACHE_FILE), content).unwrap();

        let result = load_cache(tmp.path());
        assert!(result.is_some());
        assert_eq!(result.unwrap().version, CACHE_VERSION);
    }

    // ========================================================================
    // save_cache テスト
    // ========================================================================

    #[test]
    fn test_save_and_load_cache() {
        let tmp = TempDir::new().unwrap();
        let hashes: HashMap<String, String> =
            [("file.rs".to_string(), "sha256:abc123".to_string())]
                .into_iter()
                .collect();

        save_cache(tmp.path(), &[], &[], &hashes).unwrap();

        let loaded = load_cache(tmp.path()).unwrap();
        assert_eq!(loaded.version, CACHE_VERSION);
        assert_eq!(loaded.file_hashes.get("file.rs").unwrap(), "sha256:abc123");
    }

    #[test]
    fn test_save_cache_creates_directory() {
        let tmp = TempDir::new().unwrap();
        let cache_dir = tmp.path().join(".k1s0");

        save_cache(&cache_dir, &[], &[], &HashMap::new()).unwrap();
        assert!(cache_dir.join(CACHE_FILE).exists());
    }

    // ========================================================================
    // is_cache_valid テスト
    // ========================================================================

    #[test]
    fn test_is_cache_valid_matching() {
        let hashes: HashMap<String, String> = [
            ("a.rs".to_string(), "hash_a".to_string()),
            ("b.proto".to_string(), "hash_b".to_string()),
        ]
        .into_iter()
        .collect();

        let cache = DepsCache {
            version: CACHE_VERSION,
            generated_at: String::new(),
            file_hashes: hashes.clone(),
            dependencies: vec![],
            violations: vec![],
        };

        assert!(is_cache_valid(&cache, &hashes));
    }

    #[test]
    fn test_is_cache_valid_hash_changed() {
        let old_hashes: HashMap<String, String> = [("a.rs".to_string(), "old_hash".to_string())]
            .into_iter()
            .collect();
        let new_hashes: HashMap<String, String> = [("a.rs".to_string(), "new_hash".to_string())]
            .into_iter()
            .collect();

        let cache = DepsCache {
            version: CACHE_VERSION,
            generated_at: String::new(),
            file_hashes: old_hashes,
            dependencies: vec![],
            violations: vec![],
        };

        assert!(!is_cache_valid(&cache, &new_hashes));
    }

    #[test]
    fn test_is_cache_valid_file_added() {
        let old_hashes: HashMap<String, String> = [("a.rs".to_string(), "hash_a".to_string())]
            .into_iter()
            .collect();
        let new_hashes: HashMap<String, String> = [
            ("a.rs".to_string(), "hash_a".to_string()),
            ("b.rs".to_string(), "hash_b".to_string()),
        ]
        .into_iter()
        .collect();

        let cache = DepsCache {
            version: CACHE_VERSION,
            generated_at: String::new(),
            file_hashes: old_hashes,
            dependencies: vec![],
            violations: vec![],
        };

        assert!(!is_cache_valid(&cache, &new_hashes));
    }

    #[test]
    fn test_is_cache_valid_file_removed() {
        let old_hashes: HashMap<String, String> = [
            ("a.rs".to_string(), "hash_a".to_string()),
            ("b.rs".to_string(), "hash_b".to_string()),
        ]
        .into_iter()
        .collect();
        let new_hashes: HashMap<String, String> = [("a.rs".to_string(), "hash_a".to_string())]
            .into_iter()
            .collect();

        let cache = DepsCache {
            version: CACHE_VERSION,
            generated_at: String::new(),
            file_hashes: old_hashes,
            dependencies: vec![],
            violations: vec![],
        };

        assert!(!is_cache_valid(&cache, &new_hashes));
    }

    #[test]
    fn test_is_cache_valid_wrong_version() {
        let hashes = HashMap::new();
        let cache = DepsCache {
            version: 999,
            generated_at: String::new(),
            file_hashes: hashes.clone(),
            dependencies: vec![],
            violations: vec![],
        };

        assert!(!is_cache_valid(&cache, &hashes));
    }

    // ========================================================================
    // compute_file_hashes テスト
    // ========================================================================

    #[test]
    fn test_compute_file_hashes_empty() {
        let tmp = TempDir::new().unwrap();
        let hashes = compute_file_hashes(tmp.path()).unwrap();
        assert!(hashes.is_empty());
    }

    #[test]
    fn test_compute_file_hashes_includes_proto() {
        let tmp = TempDir::new().unwrap();
        let proto_dir = tmp.path().join("regions/system/server/rust/auth/src");
        fs::create_dir_all(&proto_dir).unwrap();
        fs::write(proto_dir.join("service.proto"), "syntax = \"proto3\";").unwrap();

        let hashes = compute_file_hashes(tmp.path()).unwrap();
        assert_eq!(hashes.len(), 1);
        assert!(hashes.keys().next().unwrap().contains("service.proto"));
        // ハッシュ値は sha256: プレフィックス付き
        let hash_value = hashes.values().next().unwrap();
        assert!(
            hash_value.starts_with("sha256:"),
            "ハッシュ値はsha256:プレフィックス付きであること: {hash_value}"
        );
    }

    #[test]
    fn test_compute_file_hashes_includes_cargo_toml() {
        let tmp = TempDir::new().unwrap();
        let server_dir = tmp.path().join("regions/system/server/rust/auth");
        fs::create_dir_all(&server_dir).unwrap();
        fs::write(server_dir.join("Cargo.toml"), "[package]\nname = \"auth\"").unwrap();

        let hashes = compute_file_hashes(tmp.path()).unwrap();
        assert_eq!(hashes.len(), 1);
    }

    #[test]
    fn test_compute_file_hashes_deterministic() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("regions/system/test");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("main.rs"), "fn main() {}").unwrap();

        let hashes1 = compute_file_hashes(tmp.path()).unwrap();
        let hashes2 = compute_file_hashes(tmp.path()).unwrap();
        assert_eq!(hashes1, hashes2, "同じ内容のハッシュは一致すること");
    }

    #[test]
    fn test_compute_file_hashes_changes_on_content_change() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("regions/system/test");
        fs::create_dir_all(&dir).unwrap();

        fs::write(dir.join("main.rs"), "version 1").unwrap();
        let hashes1 = compute_file_hashes(tmp.path()).unwrap();

        fs::write(dir.join("main.rs"), "version 2").unwrap();
        let hashes2 = compute_file_hashes(tmp.path()).unwrap();

        assert_ne!(
            hashes1.values().next(),
            hashes2.values().next(),
            "内容変更後のハッシュは異なること"
        );
    }

    // ========================================================================
    // is_hashable_file テスト
    // ========================================================================

    #[test]
    fn test_is_hashable_file() {
        assert!(is_hashable_file(Path::new("service.proto")));
        assert!(is_hashable_file(Path::new("main.rs")));
        assert!(is_hashable_file(Path::new("main.go")));
        assert!(is_hashable_file(Path::new("index.ts")));
        assert!(is_hashable_file(Path::new("main.dart")));
        assert!(is_hashable_file(Path::new("Cargo.toml")));
        assert!(is_hashable_file(Path::new("go.mod")));
        assert!(is_hashable_file(Path::new("package.json")));
        assert!(is_hashable_file(Path::new("pubspec.yaml")));
        assert!(is_hashable_file(Path::new("config.yaml")));

        assert!(!is_hashable_file(Path::new("readme.md")));
        assert!(!is_hashable_file(Path::new("image.png")));
        assert!(!is_hashable_file(Path::new("data.csv")));
    }
}

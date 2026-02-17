use anyhow::Result;
use std::fs;
use std::path::Path;

use crate::prompt;
use super::types::{DbInfo, Rdbms, RDBMS_LABELS, ALL_RDBMS};

/// 既存ディレクトリを走査して名前一覧を返す。
pub(super) fn scan_existing_dirs(base: &str) -> Vec<String> {
    let path = Path::new(base);
    if !path.is_dir() {
        return Vec::new();
    }
    let mut names = Vec::new();
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    names.push(name.to_string());
                }
            }
        }
    }
    names.sort();
    names
}

/// 既存データベースを走査する。
pub(super) fn scan_existing_databases() -> Vec<DbInfo> {
    let mut dbs = Vec::new();
    let search_paths = &[
        "regions/system/database",
        "regions/business",
        "regions/service",
    ];

    for base in search_paths {
        scan_db_recursive(Path::new(base), &mut dbs);
    }

    dbs
}

fn scan_db_recursive(path: &Path, dbs: &mut Vec<DbInfo>) {
    if !path.is_dir() {
        return;
    }
    // database.yaml を探す
    let config_path = path.join("database.yaml");
    if config_path.is_file() {
        if let Ok(content) = fs::read_to_string(&config_path) {
            // 簡易パース
            let mut name = String::new();
            let mut rdbms_str = String::new();
            for line in content.lines() {
                if let Some(v) = line.strip_prefix("name: ") {
                    name = v.trim().to_string();
                }
                if let Some(v) = line.strip_prefix("rdbms: ") {
                    rdbms_str = v.trim().to_string();
                }
            }
            if !name.is_empty() {
                let rdbms = match rdbms_str.as_str() {
                    "MySQL" => Rdbms::MySQL,
                    "SQLite" => Rdbms::SQLite,
                    _ => Rdbms::PostgreSQL,
                };
                dbs.push(DbInfo { name, rdbms });
            }
        }
    }

    // 再帰的にサブディレクトリを探索
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                scan_db_recursive(&entry.path(), dbs);
            }
        }
    }
}

/// 名前の入力 or 既存選択。
///
/// 新規作成の場合、既存ディレクトリとの重複チェックを行う。
pub(super) fn prompt_name_or_select(
    select_prompt_text: &str,
    input_prompt_text: &str,
    existing: &[String],
) -> Result<Option<String>> {
    let mut items: Vec<&str> = vec!["(新規作成)"];
    for name in existing {
        items.push(name.as_str());
    }

    let idx = prompt::select_prompt(select_prompt_text, &items)?;
    match idx {
        None => Ok(None),
        Some(0) => {
            // 新規作成: 名前バリデーション + 重複チェック
            let existing_names: Vec<String> = existing.to_vec();
            match prompt::input_with_validation(input_prompt_text, move |input: &String| {
                // まず名前バリデーション
                prompt::validate_name(input)?;
                // 重複チェック
                if existing_names.iter().any(|n| n == input) {
                    return Err(format!("'{}' は既に存在します。別の名前を入力してください。", input));
                }
                Ok(())
            }) {
                Ok(name) => Ok(Some(name)),
                Err(_) => Ok(None),
            }
        }
        Some(i) => Ok(Some(existing[i - 1].clone())),
    }
}

/// DB選択 (既存 or 新規作成)。
pub(super) fn prompt_db_selection(existing: &[DbInfo]) -> Result<Option<DbInfo>> {
    let mut items: Vec<String> = vec!["(新規作成)".to_string()];
    for db in existing {
        items.push(format!("{} ({})", db.name, db.rdbms.as_str()));
    }
    let items_ref: Vec<&str> = items.iter().map(|s| s.as_str()).collect();

    let idx = prompt::select_prompt("データベース名を入力または選択してください", &items_ref)?;
    match idx {
        None => Ok(None),
        Some(0) => {
            // 新規作成
            let name = match prompt::input_prompt("データベース名を入力してください") {
                Ok(n) => n,
                Err(_) => return Ok(None),
            };
            let rdbms_idx = prompt::select_prompt("RDBMS を選択してください", RDBMS_LABELS)?;
            match rdbms_idx {
                Some(i) => Ok(Some(DbInfo {
                    name,
                    rdbms: ALL_RDBMS[i],
                })),
                None => Ok(None),
            }
        }
        Some(i) => Ok(Some(existing[i - 1].clone())),
    }
}

// ============================================================================
// テスト
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_existing_dirs_nonexistent() {
        let dirs = scan_existing_dirs("/nonexistent/path");
        assert!(dirs.is_empty());
    }

    #[test]
    fn test_scan_existing_dirs_with_dirs() {
        let tmp = tempfile::TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("base/aaa")).unwrap();
        fs::create_dir_all(tmp.path().join("base/bbb")).unwrap();
        fs::write(tmp.path().join("base/file.txt"), "").unwrap();

        let dirs = scan_existing_dirs(tmp.path().join("base").to_str().unwrap());
        assert_eq!(dirs.len(), 2);
        assert!(dirs.contains(&"aaa".to_string()));
        assert!(dirs.contains(&"bbb".to_string()));
    }
}

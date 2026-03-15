// 配置先スキャンのテスト。
// scan_placements_at が各 Tier に対して正しいディレクトリ一覧を返すことを検証する。

use std::fs;
use tempfile::TempDir;

use crate::commands::generate::execute::scan_placements_at;
use crate::commands::generate::types::Tier;

#[test]
fn test_scan_placements_at_empty_dir() {
    let tmp = TempDir::new().unwrap();
    let result = scan_placements_at(&Tier::Business, tmp.path());
    assert!(result.is_empty());
}

#[test]
fn test_scan_placements_at_system_returns_empty() {
    let tmp = TempDir::new().unwrap();
    // System Tier 配下にディレクトリがあっても返さない
    fs::create_dir_all(tmp.path().join("regions/system/some-dir")).unwrap();
    let result = scan_placements_at(&Tier::System, tmp.path());
    assert!(result.is_empty(), "System tier should always return empty");
}

#[test]
fn test_scan_placements_at_business_with_dirs() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join("regions/business/accounting")).unwrap();
    fs::create_dir_all(tmp.path().join("regions/business/fa")).unwrap();
    fs::create_dir_all(tmp.path().join("regions/business/hr")).unwrap();
    // ファイルは含まれないことを確認
    fs::write(tmp.path().join("regions/business/.gitkeep"), "").unwrap();
    let result = scan_placements_at(&Tier::Business, tmp.path());
    assert_eq!(result, vec!["accounting", "fa", "hr"]);
}

#[test]
fn test_scan_placements_at_service_with_dirs() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join("regions/service/order")).unwrap();
    fs::create_dir_all(tmp.path().join("regions/service/payment")).unwrap();
    let result = scan_placements_at(&Tier::Service, tmp.path());
    assert_eq!(result, vec!["order", "payment"]);
}

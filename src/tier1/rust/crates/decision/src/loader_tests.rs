// 本ファイルは loader.rs のテスト。
//
// 観点（FR-T1-DECISION-004）:
//   - 起動時 load: ディレクトリ配下の *.json を全件 register
//   - リロード: 同 rule_id の 2 度目 register が新 version を生成（registry 仕様）
//   - hidden ファイル / 非 .json ファイルはスキップ
//   - 不正 JSON は登録失敗で warn のみ（呼出側は他ファイルを継続処理）

use super::*;
use crate::registry::RuleRegistry;
use std::sync::Arc;

/// 最小有効 JDM（registry_tests.rs 同等の 3 ノード構造）。
fn minimal_jdm() -> Vec<u8> {
    br#"{
      "nodes": [
        {"id":"in","type":"inputNode","name":"input","content":{}},
        {"id":"out","type":"outputNode","name":"output","content":{}}
      ],
      "edges": [
        {"id":"e1","sourceId":"in","targetId":"out"}
      ]
    }"#
    .to_vec()
}

#[test]
fn path_to_rule_id_uses_file_stem() {
    let p = std::path::Path::new("/etc/k1s0/decisions/payment-approval.json");
    assert_eq!(path_to_rule_id(p), Some("payment-approval".to_string()));
}

#[test]
fn is_jdm_file_filters_hidden_and_non_json() {
    let dir = tempfile::tempdir().unwrap();
    let visible = dir.path().join("visible.json");
    std::fs::write(&visible, b"{}").unwrap();
    let hidden = dir.path().join(".hidden.json");
    std::fs::write(&hidden, b"{}").unwrap();
    let txt = dir.path().join("note.txt");
    std::fs::write(&txt, b"x").unwrap();
    assert!(is_jdm_file(&visible));
    assert!(!is_jdm_file(&hidden));
    assert!(!is_jdm_file(&txt));
}

#[test]
fn load_initial_registers_all_jdm_files() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("rule-a.json"), minimal_jdm()).unwrap();
    std::fs::write(dir.path().join("rule-b.json"), minimal_jdm()).unwrap();
    // 不正 JSON は登録失敗 → errors に拾われる。
    std::fs::write(dir.path().join("invalid.json"), b"not json").unwrap();
    // 関係ないファイルは無視される。
    std::fs::write(dir.path().join("note.txt"), b"x").unwrap();
    let registry = RuleRegistry::new();
    // loader はシステム名前空間 "system" にロードする想定で監査する。
    let (ok, errors) = load_initial(&registry, dir.path(), "system", "loader-test").unwrap();
    assert_eq!(ok, 2, "rule-a + rule-b should succeed");
    assert_eq!(errors.len(), 1, "invalid.json should fail");
    let metas_a = registry.list_versions("system", "rule-a").unwrap();
    assert_eq!(metas_a.len(), 1);
    assert!(
        metas_a[0].commit_hash.starts_with("mtime-"),
        "commit_hash should be mtime-derived"
    );
}

#[test]
fn load_initial_returns_zero_for_missing_dir() {
    let path = std::path::PathBuf::from("/nonexistent/dir/path");
    let registry = RuleRegistry::new();
    let (ok, errors) = load_initial(&registry, &path, "system", "test").unwrap();
    assert_eq!(ok, 0);
    assert_eq!(errors.len(), 0);
}

#[test]
fn re_register_same_file_bumps_version() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("payment.json");
    std::fs::write(&path, minimal_jdm()).unwrap();
    let registry = RuleRegistry::new();
    let v1 = load_one(&registry, &path, "system", "test").unwrap();
    assert_eq!(v1, "v1");
    // 再 register（ホットリロードのシミュレーション）。
    let v2 = load_one(&registry, &path, "system", "test").unwrap();
    assert_eq!(v2, "v2");
    let metas = registry.list_versions("system", "payment").unwrap();
    assert_eq!(metas.len(), 2);
}

#[tokio::test]
async fn watcher_picks_up_new_file_within_short_window() {
    // notify backend (inotify on Linux) は実 OS イベントを使うため、CI 環境によっては
    // タイミングにばらつきがある。ここでは「watcher 起動 → ファイル作成 → 数秒待つ」
    // を成立させ、新 rule_id が registry に乗ることだけを確認する。
    let dir = tempfile::tempdir().unwrap();
    let registry = Arc::new(RuleRegistry::new());
    let _handle = spawn_watcher(
        registry.clone(),
        dir.path().to_path_buf(),
        "system".into(),
        "watcher-test".into(),
    )
    .unwrap();
    // 監視開始後にファイル作成。
    let path = dir.path().join("hot-rule.json");
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    std::fs::write(&path, minimal_jdm()).unwrap();
    // 反映待ち（複数 backend のレイテンシマージン）。
    let mut found = false;
    for _ in 0..40 {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        if let Ok(metas) = registry.list_versions("system", "hot-rule") {
            if !metas.is_empty() {
                found = true;
                break;
            }
        }
    }
    assert!(found, "watcher should pick up new file within 4 seconds");
}

/// NFR-E-AC-003: loader が system_tenant_id="system" でロードした rule は、
/// クライアント tenant ("tenant-A") から evaluate しても見えない（構造的越境防止）。
#[tokio::test]
async fn loader_rules_isolated_from_client_tenants() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("admin-rule.json"), minimal_jdm()).unwrap();
    let registry = RuleRegistry::new();
    let (ok, _errors) = load_initial(&registry, dir.path(), "system", "test").unwrap();
    assert_eq!(ok, 1);
    // tenant-A から evaluate → NotFound。
    let err = registry
        .evaluate("tenant-A", "admin-rule", "v1", br#"{}"#, false)
        .await
        .unwrap_err();
    match err {
        crate::registry::RegistryError::NotFound { tenant_id, .. } => {
            assert_eq!(tenant_id, "tenant-A")
        }
        other => panic!("expected NotFound for cross-tenant, got {:?}", other),
    }
    // system tenant 名で呼ぶと取れる（admin 管理経路のみ参照可）。
    let outcome = registry
        .evaluate("system", "admin-rule", "v1", br#"{}"#, false)
        .await
        .unwrap();
    assert!(!outcome.output_json.is_empty());
}

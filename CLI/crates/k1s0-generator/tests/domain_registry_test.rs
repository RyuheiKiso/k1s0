//! ドメインレジストリの統合テスト
//!
//! fixtures/domain-registry/ 配下のサンプル manifest.json を使用して
//! scan_domains, scan_features, build_catalog, DomainGraph をテストする。

use std::path::PathBuf;

use k1s0_generator::domain::catalog::{build_catalog, format_json, format_table};
use k1s0_generator::domain::graph::DomainGraph;
use k1s0_generator::domain::scanner::{scan_domains, scan_features};

/// テスト用 fixtures ルートパスを取得する
fn fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("domain-registry")
}

// ============================================================
// scan_domains テスト
// ============================================================

#[test]
fn test_scan_domains_finds_all_domains() {
    let root = fixtures_root();
    let domains = scan_domains(&root).expect("scan_domains should succeed");

    // cycle-a, cycle-b, order-processing, user-management (rust) + notification (go) = 5
    assert_eq!(domains.len(), 5, "Expected 5 domains, got {}", domains.len());
}

#[test]
fn test_scan_domains_sorted_by_name() {
    let root = fixtures_root();
    let domains = scan_domains(&root).expect("scan_domains should succeed");
    let names: Vec<&str> = domains.iter().map(|d| d.name.as_str()).collect();

    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted, "Domains should be sorted alphabetically");
}

#[test]
fn test_scan_domains_user_management_details() {
    let root = fixtures_root();
    let domains = scan_domains(&root).expect("scan_domains should succeed");
    let um = domains
        .iter()
        .find(|d| d.name == "user-management")
        .expect("user-management domain should exist");

    assert_eq!(um.version, "1.2.0");
    assert_eq!(um.domain_type, "backend-rust");
    assert_eq!(um.language, "rust");
    assert!(um.deprecated.is_none());
    assert!(um.dependencies.is_empty());
    assert_eq!(um.min_framework_version.as_deref(), Some("0.1.0"));
}

#[test]
fn test_scan_domains_order_processing_depends_on_user_management() {
    let root = fixtures_root();
    let domains = scan_domains(&root).expect("scan_domains should succeed");
    let op = domains
        .iter()
        .find(|d| d.name == "order-processing")
        .expect("order-processing domain should exist");

    assert_eq!(op.version, "0.3.0");
    assert_eq!(op.dependencies.len(), 1);
    assert_eq!(
        op.dependencies.get("user-management"),
        Some(&"^1.0.0".to_string())
    );
}

#[test]
fn test_scan_domains_notification_deprecated() {
    let root = fixtures_root();
    let domains = scan_domains(&root).expect("scan_domains should succeed");
    let notif = domains
        .iter()
        .find(|d| d.name == "notification")
        .expect("notification domain should exist");

    assert_eq!(notif.version, "2.0.0");
    assert_eq!(notif.domain_type, "backend-go");
    assert_eq!(notif.language, "go");

    let dep_info = notif.deprecated.as_ref().expect("should be deprecated");
    assert!(dep_info.message.contains("notification-v2"));
    assert_eq!(dep_info.alternative.as_deref(), Some("notification-v2"));
}

// ============================================================
// scan_features テスト
// ============================================================

#[test]
fn test_scan_features_finds_user_service() {
    let root = fixtures_root();
    let features = scan_features(&root).expect("scan_features should succeed");

    assert_eq!(features.len(), 1, "Expected 1 feature");
    let f = &features[0];
    assert_eq!(f.name, "user-service");
    assert_eq!(f.feature_type, "backend-rust");
}

#[test]
fn test_scan_features_domain_dependencies() {
    let root = fixtures_root();
    let features = scan_features(&root).expect("scan_features should succeed");
    let f = &features[0];

    assert_eq!(f.domain_dependencies.len(), 1);
    assert_eq!(
        f.domain_dependencies.get("user-management"),
        Some(&"^1.0.0".to_string())
    );
}

// ============================================================
// build_catalog テスト
// ============================================================

#[test]
fn test_build_catalog_summary() {
    let root = fixtures_root();
    let domains = scan_domains(&root).unwrap();
    let features = scan_features(&root).unwrap();
    let catalog = build_catalog(&domains, &features);

    assert_eq!(catalog.summary.total, 5);
    assert_eq!(catalog.summary.deprecated, 1); // notification
    assert_eq!(catalog.summary.active, 4);
}

#[test]
fn test_build_catalog_dependent_counts() {
    let root = fixtures_root();
    let domains = scan_domains(&root).unwrap();
    let features = scan_features(&root).unwrap();
    let catalog = build_catalog(&domains, &features);

    let um_entry = catalog
        .domains
        .iter()
        .find(|e| e.info.name == "user-management")
        .expect("user-management entry should exist");
    // user-service depends on user-management
    assert_eq!(um_entry.dependent_count, 1);
    assert_eq!(um_entry.status, "active");

    let notif_entry = catalog
        .domains
        .iter()
        .find(|e| e.info.name == "notification")
        .expect("notification entry should exist");
    assert_eq!(notif_entry.dependent_count, 0);
    assert_eq!(notif_entry.status, "deprecated");
}

#[test]
fn test_build_catalog_by_language() {
    let root = fixtures_root();
    let domains = scan_domains(&root).unwrap();
    let features = scan_features(&root).unwrap();
    let catalog = build_catalog(&domains, &features);

    assert_eq!(catalog.summary.by_language.get("rust"), Some(&4));
    assert_eq!(catalog.summary.by_language.get("go"), Some(&1));
}

// ============================================================
// format_table / format_json テスト
// ============================================================

#[test]
fn test_format_table_contains_all_domains() {
    let root = fixtures_root();
    let domains = scan_domains(&root).unwrap();
    let features = scan_features(&root).unwrap();
    let catalog = build_catalog(&domains, &features);
    let table = format_table(&catalog);

    assert!(table.contains("user-management"));
    assert!(table.contains("order-processing"));
    assert!(table.contains("notification"));
    assert!(table.contains("cycle-a"));
    assert!(table.contains("cycle-b"));
    assert!(table.contains("Total: 5"));
    assert!(table.contains("Active: 4"));
    assert!(table.contains("Deprecated: 1"));
}

#[test]
fn test_format_json_valid() {
    let root = fixtures_root();
    let domains = scan_domains(&root).unwrap();
    let features = scan_features(&root).unwrap();
    let catalog = build_catalog(&domains, &features);
    let json_str = format_json(&catalog);

    let parsed: serde_json::Value =
        serde_json::from_str(&json_str).expect("format_json should produce valid JSON");
    let domains_arr = parsed["domains"].as_array().expect("domains should be array");
    assert_eq!(domains_arr.len(), 5);

    let summary = &parsed["summary"];
    assert_eq!(summary["total"], 5);
    assert_eq!(summary["active"], 4);
    assert_eq!(summary["deprecated"], 1);
}

// ============================================================
// DomainGraph テスト
// ============================================================

#[test]
fn test_domain_graph_cycle_detection_with_fixtures() {
    let root = fixtures_root();
    let domains = scan_domains(&root).unwrap();
    let features = scan_features(&root).unwrap();
    let graph = DomainGraph::from_domains(&domains, &features);

    let cycles = graph.detect_cycles();
    assert!(
        !cycles.is_empty(),
        "Should detect the cycle-a <-> cycle-b cycle"
    );

    // Find the cycle containing cycle-a and cycle-b
    let has_ab_cycle = cycles.iter().any(|cycle| {
        cycle.contains(&"cycle-a".to_string()) && cycle.contains(&"cycle-b".to_string())
    });
    assert!(has_ab_cycle, "Should detect cycle between cycle-a and cycle-b");
}

#[test]
fn test_domain_graph_no_false_positive_cycles() {
    let root = fixtures_root();
    let domains = scan_domains(&root).unwrap();
    let features = scan_features(&root).unwrap();
    let graph = DomainGraph::from_domains(&domains, &features);

    let cycles = graph.detect_cycles();
    // Only the cycle-a <-> cycle-b pair should form a cycle
    for cycle in &cycles {
        for name in cycle {
            assert!(
                name == "cycle-a" || name == "cycle-b",
                "Only cycle-a and cycle-b should be in cycles, found: {}",
                name
            );
        }
    }
}

#[test]
fn test_domain_graph_to_mermaid() {
    let root = fixtures_root();
    let domains = scan_domains(&root).unwrap();
    let features = scan_features(&root).unwrap();
    let graph = DomainGraph::from_domains(&domains, &features);
    let mermaid = graph.to_mermaid();

    assert!(mermaid.starts_with("graph TD"), "Should start with graph TD");
    assert!(mermaid.contains("Domain Layer"), "Should contain Domain Layer subgraph");
    assert!(mermaid.contains("Feature Layer"), "Should contain Feature Layer subgraph");
    // Check domain nodes exist
    assert!(mermaid.contains("user_management"), "Should contain user_management node");
    assert!(mermaid.contains("order_processing"), "Should contain order_processing node");
    assert!(mermaid.contains("notification"), "Should contain notification node");
    // Check feature node
    assert!(mermaid.contains("user_service") || mermaid.contains("f_user_service"),
        "Should contain user_service feature node");
    // Check deprecated styling (notification is deprecated)
    assert!(mermaid.contains("fill:#ffcccc"), "Deprecated node should have red fill");
    // Check edges exist (order-processing -> user-management)
    assert!(mermaid.contains("order_processing"), "Should have order_processing edge");
}

#[test]
fn test_domain_graph_to_dot() {
    let root = fixtures_root();
    let domains = scan_domains(&root).unwrap();
    let features = scan_features(&root).unwrap();
    let graph = DomainGraph::from_domains(&domains, &features);
    let dot = graph.to_dot();

    assert!(dot.starts_with("digraph domain_dependencies"), "Should start with digraph");
    assert!(dot.contains("cluster_domain"), "Should contain domain cluster");
    assert!(dot.contains("cluster_feature"), "Should contain feature cluster");
    assert!(dot.contains("user_management"), "Should contain user_management node");
    assert!(dot.contains("notification"), "Should contain notification node");
    // Deprecated node style
    assert!(dot.contains("#ffcccc"), "Deprecated node should have red fill color");
    // Feature node style
    assert!(dot.contains("#cce5ff"), "Feature node should have blue fill color");
    // Active node style
    assert!(dot.contains("#ccffcc"), "Active node should have green fill color");
    // Check closing brace
    assert!(dot.ends_with('}'), "Should end with closing brace");
}

#[test]
fn test_domain_graph_subgraph_from_fixtures() {
    let root = fixtures_root();
    let domains = scan_domains(&root).unwrap();
    let features = scan_features(&root).unwrap();
    let graph = DomainGraph::from_domains(&domains, &features);

    // order-processing depends on user-management, so subgraph should include both
    let sub = graph
        .subgraph("order-processing")
        .expect("subgraph should succeed");
    let mermaid = sub.to_mermaid();
    assert!(mermaid.contains("order_processing"));
    assert!(mermaid.contains("user_management"));
    // notification should NOT be in this subgraph
    assert!(!mermaid.contains("notification"));
}

#[test]
fn test_domain_graph_subgraph_not_found() {
    let root = fixtures_root();
    let domains = scan_domains(&root).unwrap();
    let features = scan_features(&root).unwrap();
    let graph = DomainGraph::from_domains(&domains, &features);

    let result = graph.subgraph("nonexistent-domain");
    assert!(result.is_err(), "Should return error for nonexistent domain");
}

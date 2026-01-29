//! ドメインカタログ構築・フォーマット

use std::collections::HashMap;

use super::types::{DomainCatalog, DomainCatalogEntry, DomainInfo, DomainSummary, FeatureInfo};

/// ドメインカタログを構築する
pub fn build_catalog(domains: &[DomainInfo], features: &[FeatureInfo]) -> DomainCatalog {
    // feature ごとのドメイン依存カウントを計算
    let mut dependent_counts: HashMap<String, usize> = HashMap::new();
    for feature in features {
        for domain_name in feature.domain_dependencies.keys() {
            *dependent_counts.entry(domain_name.clone()).or_insert(0) += 1;
        }
    }

    let mut entries: Vec<DomainCatalogEntry> = domains
        .iter()
        .map(|d| {
            let dependent_count = dependent_counts.get(&d.name).copied().unwrap_or(0);
            let status = if d.deprecated.is_some() {
                "deprecated".to_string()
            } else {
                "active".to_string()
            };
            DomainCatalogEntry {
                info: d.clone(),
                dependent_count,
                status,
            }
        })
        .collect();

    entries.sort_by(|a, b| a.info.name.cmp(&b.info.name));

    let summary = build_summary(domains);

    DomainCatalog {
        domains: entries,
        summary,
    }
}

/// サマリーを構築する
fn build_summary(domains: &[DomainInfo]) -> DomainSummary {
    let total = domains.len();
    let deprecated = domains.iter().filter(|d| d.deprecated.is_some()).count();
    let active = total - deprecated;

    let mut by_language: HashMap<String, usize> = HashMap::new();
    let mut by_type: HashMap<String, usize> = HashMap::new();

    for d in domains {
        *by_language.entry(d.language.clone()).or_insert(0) += 1;
        *by_type.entry(d.domain_type.clone()).or_insert(0) += 1;
    }

    DomainSummary {
        total,
        active,
        deprecated,
        by_language,
        by_type,
    }
}

/// テーブル形式にフォーマットする
pub fn format_table(catalog: &DomainCatalog) -> String {
    let mut lines = Vec::new();

    // ヘッダー
    lines.push(format!(
        "{:<25} {:<18} {:<12} {:<10} {:<12} {:<10}",
        "NAME", "TYPE", "LANGUAGE", "VERSION", "STATUS", "DEPENDENTS"
    ));
    lines.push("-".repeat(90));

    // 行
    for entry in &catalog.domains {
        let status_display = if entry.status == "deprecated" {
            "deprecated"
        } else {
            "active"
        };
        lines.push(format!(
            "{:<25} {:<18} {:<12} {:<10} {:<12} {:<10}",
            entry.info.name,
            entry.info.domain_type,
            entry.info.language,
            entry.info.version,
            status_display,
            entry.dependent_count,
        ));
    }

    // サマリー
    lines.push(String::new());
    lines.push(format!(
        "Total: {}  Active: {}  Deprecated: {}",
        catalog.summary.total, catalog.summary.active, catalog.summary.deprecated
    ));

    lines.join("\n")
}

/// JSON 形式にフォーマットする
pub fn format_json(catalog: &DomainCatalog) -> String {
    serde_json::to_string_pretty(catalog).unwrap_or_else(|_| "{}".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn sample_domain(name: &str, deprecated: bool) -> DomainInfo {
        DomainInfo {
            name: name.to_string(),
            version: "0.1.0".to_string(),
            domain_type: "backend-rust".to_string(),
            language: "rust".to_string(),
            path: PathBuf::from(format!("domain/backend/rust/{}", name)),
            dependencies: HashMap::new(),
            min_framework_version: None,
            deprecated: if deprecated {
                Some(super::super::types::DeprecatedInfo {
                    message: "Use new-domain instead".to_string(),
                    alternative: Some("new-domain".to_string()),
                })
            } else {
                None
            },
            breaking_changes: None,
        }
    }

    fn sample_feature(name: &str, domain_deps: &[(&str, &str)]) -> FeatureInfo {
        let mut deps = HashMap::new();
        for (d, v) in domain_deps {
            deps.insert(d.to_string(), v.to_string());
        }
        FeatureInfo {
            name: name.to_string(),
            feature_type: "backend-rust".to_string(),
            path: PathBuf::from(format!("feature/backend/rust/{}", name)),
            domain_dependencies: deps,
        }
    }

    #[test]
    fn test_build_catalog() {
        let domains = vec![
            sample_domain("user-management", false),
            sample_domain("old-domain", true),
        ];
        let features = vec![
            sample_feature("user-service", &[("user-management", "^0.1.0")]),
            sample_feature("admin-service", &[("user-management", "^0.1.0")]),
        ];

        let catalog = build_catalog(&domains, &features);
        assert_eq!(catalog.summary.total, 2);
        assert_eq!(catalog.summary.active, 1);
        assert_eq!(catalog.summary.deprecated, 1);
        assert_eq!(catalog.domains[1].dependent_count, 2); // user-management
        assert_eq!(catalog.domains[0].dependent_count, 0); // old-domain
    }

    #[test]
    fn test_format_table() {
        let domains = vec![sample_domain("test-domain", false)];
        let features = vec![];
        let catalog = build_catalog(&domains, &features);
        let table = format_table(&catalog);
        assert!(table.contains("test-domain"));
        assert!(table.contains("Total: 1"));
    }

    #[test]
    fn test_format_json() {
        let domains = vec![sample_domain("test-domain", false)];
        let features = vec![];
        let catalog = build_catalog(&domains, &features);
        let json = format_json(&catalog);
        assert!(json.contains("test-domain"));
    }
}

//! Dependency output helpers.
//!
//! Provides terminal and Mermaid renderers for dependency analysis results.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::Result;

use super::types::{Dependency, DependencyType, DepsResult, Severity, Violation};

/// 依存関係の分析結果をターミナルに出力する。
pub fn print_terminal(result: &DepsResult) {
    println!();

    let tier_order = ["system", "business", "service"];
    let mut by_tier: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for service in &result.services {
        by_tier
            .entry(&service.tier)
            .or_default()
            .push(&service.name);
    }

    println!("=== 依存関係マップ ===");

    for tier in &tier_order {
        if let Some(services) = by_tier.get(tier) {
            println!("\n[{tier} tier]");
            for service_name in services {
                println!("  {service_name}");
                for dep in &result.dependencies {
                    if dep.source == *service_name {
                        let tier_label = if dep.target_tier == *tier {
                            String::new()
                        } else {
                            format!(" [{}]", dep.target_tier)
                        };
                        let detail = dep
                            .detail
                            .as_ref()
                            .map(|d| format!("  {d}"))
                            .unwrap_or_default();
                        let dep_type_str = dep.dep_type.to_string();
                        println!(
                            "    -> ({dep_type_str:<8}) {}{tier_label}{detail}",
                            dep.target
                        );
                    }
                    if dep.target == *service_name && dep.source_tier == *tier {
                        let detail = dep
                            .detail
                            .as_ref()
                            .map(|d| format!("  {d}"))
                            .unwrap_or_default();
                        let dep_type_str = dep.dep_type.to_string();
                        println!("    <- ({dep_type_str:<8}) {}{detail}", dep.source);
                    }
                }
            }
        }
    }

    if result.violations.is_empty() {
        println!("\n=== ルール違反: なし ===");
    } else {
        println!("\n=== ルール違反 ===");

        for violation in &result.violations {
            let prefix = match violation.severity {
                Severity::Error => "\n  ✗",
                Severity::Warning => "\n  ⚠",
                Severity::Info => "\n  (i)",
            };
            println!(
                "{} {} ({}) -> {} ({})",
                prefix,
                violation.source,
                violation.source_tier,
                violation.target,
                violation.target_tier
            );
            println!("    {}", violation.message);
            if let Some(loc) = &violation.location {
                println!("    場所: {loc}");
            }
            println!("    推奨: {}", violation.recommendation);
        }
    }

    let error_count = result
        .violations
        .iter()
        .filter(|v| v.severity == Severity::Error)
        .count();
    let warning_count = result
        .violations
        .iter()
        .filter(|v| v.severity == Severity::Warning)
        .count();

    println!("\n=== サマリー ===");
    println!("  対象サービス: {}", result.services.len());
    println!("  依存関係:     {}", result.dependencies.len());
    println!("  エラー:       {error_count}");
    println!("  警告:         {warning_count}");
    println!();
}

/// Mermaid 形式の依存関係ダイアグラムを生成する。
pub fn generate_mermaid(result: &DepsResult) -> String {
    let mut lines = vec![
        "# 依存関係マップ".to_string(),
        String::new(),
        "```mermaid".to_string(),
        "graph TD".to_string(),
    ];
    let tiers = collect_mermaid_tiers(result);
    append_mermaid_subgraphs(&mut lines, &tiers);

    let violation_pairs = collect_violation_pairs(result);
    let violation_link_indices =
        append_mermaid_links(&mut lines, &result.dependencies, &violation_pairs);
    append_violation_link_styles(&mut lines, &violation_link_indices);

    lines.push("```".to_string());
    append_violation_summary(&mut lines, &result.violations);
    lines.push(String::new());
    lines.join("\n")
}

fn collect_mermaid_tiers(result: &DepsResult) -> BTreeMap<String, BTreeSet<String>> {
    let mut tiers: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for service in &result.services {
        tiers
            .entry(service.tier.clone())
            .or_default()
            .insert(service.name.clone());
    }
    for dep in &result.dependencies {
        if dep.dep_type == DependencyType::Library {
            continue;
        }
        tiers
            .entry(dep.source_tier.clone())
            .or_default()
            .insert(dep.source.clone());
        tiers
            .entry(dep.target_tier.clone())
            .or_default()
            .insert(dep.target.clone());
    }
    tiers
}

fn append_mermaid_subgraphs(lines: &mut Vec<String>, tiers: &BTreeMap<String, BTreeSet<String>>) {
    let tier_labels: BTreeMap<&str, &str> = [
        ("system", "System Tier"),
        ("business", "Business Tier"),
        ("service", "Service Tier"),
    ]
    .into_iter()
    .collect();

    for (tier, services) in tiers {
        let tier_str = tier.as_str();
        let default_label = tier_str;
        let label = tier_labels.get(tier_str).unwrap_or(&default_label);
        lines.push(format!("    subgraph {tier}[\"{label}\"]"));
        for service in services {
            let node_id = sanitize_mermaid_id(service);
            let display = service.replace("-server", "");
            lines.push(format!("        {node_id}[{display}]"));
        }
        lines.push("    end".to_string());
    }
}

fn collect_violation_pairs(result: &DepsResult) -> BTreeSet<(String, String)> {
    result
        .violations
        .iter()
        .filter(|v| v.severity == Severity::Error)
        .map(|v| (v.source.clone(), v.target.clone()))
        .collect()
}

fn append_mermaid_links(
    lines: &mut Vec<String>,
    dependencies: &[Dependency],
    violation_pairs: &BTreeSet<(String, String)>,
) -> Vec<usize> {
    let mut link_index = 0;
    let mut violation_link_indices = Vec::new();

    for dep in dependencies {
        if dep.dep_type == DependencyType::Library {
            continue;
        }
        let source_id = sanitize_mermaid_id(&dep.source);
        let target_id = sanitize_mermaid_id(&dep.target);
        let label = dep.dep_type.to_string();
        let arrow = match dep.dep_type {
            DependencyType::Kafka => format!("{source_id} -.->|{label}| {target_id}"),
            _ => format!("{source_id} -->|{label}| {target_id}"),
        };
        lines.push(format!("    {arrow}"));

        if violation_pairs.contains(&(dep.source.clone(), dep.target.clone())) {
            violation_link_indices.push(link_index);
        }
        link_index += 1;
    }

    violation_link_indices
}

fn append_violation_link_styles(lines: &mut Vec<String>, violation_link_indices: &[usize]) {
    if violation_link_indices.is_empty() {
        return;
    }

    lines.push(String::new());
    lines.push("    %% 違反".to_string());
    for idx in violation_link_indices {
        lines.push(format!("    linkStyle {idx} stroke:red,stroke-width:3px"));
    }
}

fn append_violation_summary(lines: &mut Vec<String>, violations: &[Violation]) {
    if violations.is_empty() {
        return;
    }

    lines.push(String::new());
    lines.push("## ルール違反".to_string());
    lines.push(String::new());

    for violation in violations {
        let icon = match violation.severity {
            Severity::Error => "x",
            Severity::Warning => "!",
            Severity::Info => "i",
        };
        lines.push(format!(
            "- [{icon}] **{}**: {} ({}) -> {} ({})",
            violation.severity,
            violation.source,
            violation.source_tier,
            violation.target,
            violation.target_tier,
        ));
        lines.push(format!("  - {}", violation.message));
        lines.push(format!("  - 推奨: {}", violation.recommendation));
    }
}

/// Mermaid 形式の依存関係ダイアグラムをファイルに書き出す。
///
/// # Errors
///
/// 親ディレクトリの作成失敗またはファイルの書き込み失敗の場合にエラーを返す。
pub fn write_mermaid(result: &DepsResult, path: &Path) -> Result<()> {
    let content = generate_mermaid(result);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, content)?;
    println!("Mermaid diagram written: {}", path.display());
    Ok(())
}

fn sanitize_mermaid_id(name: &str) -> String {
    name.replace('-', "_")
}

// テストコードでは unwrap() の使用を許可する
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::commands::deps::types::*;
    use tempfile::TempDir;

    fn sample_result() -> DepsResult {
        DepsResult {
            services: vec![
                ServiceInfo {
                    name: "auth-server".to_string(),
                    tier: "system".to_string(),
                    domain: None,
                    language: "rust".to_string(),
                    path: std::path::PathBuf::from("regions/system/server/rust/auth"),
                },
                ServiceInfo {
                    name: "task-server".to_string(),
                    tier: "service".to_string(),
                    domain: Some("task".to_string()),
                    language: "rust".to_string(),
                    path: std::path::PathBuf::from("regions/service/task/server/rust"),
                },
            ],
            dependencies: vec![Dependency {
                source: "task-server".to_string(),
                source_tier: "service".to_string(),
                target: "auth-server".to_string(),
                target_tier: "system".to_string(),
                dep_type: DependencyType::Grpc,
                locations: vec!["proto/auth.proto".to_string()],
                detail: None,
            }],
            violations: vec![],
        }
    }

    #[test]
    fn test_generate_mermaid_basic() {
        let result = sample_result();
        let mermaid = generate_mermaid(&result);

        assert!(mermaid.contains("```mermaid"));
        assert!(mermaid.contains("graph TD"));
        assert!(mermaid.contains("System Tier"));
        assert!(mermaid.contains("Service Tier"));
        assert!(mermaid.contains("auth_server[auth]"));
        assert!(mermaid.contains("task_server[task]"));
        assert!(mermaid.contains("-->|gRPC|"));
    }

    #[test]
    fn test_generate_mermaid_kafka_dashed_arrow() {
        let result = DepsResult {
            services: vec![],
            dependencies: vec![Dependency {
                source: "task-server".to_string(),
                source_tier: "service".to_string(),
                target: "activity-server".to_string(),
                target_tier: "service".to_string(),
                dep_type: DependencyType::Kafka,
                locations: vec![],
                detail: Some("k1s0.service.task.created.v1".to_string()),
            }],
            violations: vec![],
        };

        let mermaid = generate_mermaid(&result);
        assert!(mermaid.contains("-.->|Kafka|"));
    }

    #[test]
    fn test_generate_mermaid_with_violations() {
        let mut result = sample_result();
        result.dependencies.push(Dependency {
            source: "auth-server".to_string(),
            source_tier: "system".to_string(),
            target: "task-server".to_string(),
            target_tier: "service".to_string(),
            dep_type: DependencyType::Grpc,
            locations: vec![],
            detail: None,
        });
        result.violations.push(Violation {
            severity: Severity::Error,
            source: "auth-server".to_string(),
            source_tier: "system".to_string(),
            target: "task-server".to_string(),
            target_tier: "service".to_string(),
            dep_type: DependencyType::Grpc,
            message: "上位 tier から下位 tier への依存です".to_string(),
            location: None,
            recommendation: "イベント駆動に変更してください".to_string(),
        });

        let mermaid = generate_mermaid(&result);
        assert!(mermaid.contains("## ルール違反"));
        assert!(mermaid.contains("**ERROR**"));
        assert!(mermaid.contains("linkStyle"));
        assert!(mermaid.contains("stroke:red"));
    }

    #[test]
    fn test_generate_mermaid_empty() {
        let result = DepsResult::default();
        let mermaid = generate_mermaid(&result);
        assert!(mermaid.contains("```mermaid"));
        assert!(mermaid.contains("graph TD"));
    }

    #[test]
    fn test_generate_mermaid_library_excluded() {
        let result = DepsResult {
            services: vec![],
            dependencies: vec![Dependency {
                source: "auth-server".to_string(),
                source_tier: "system".to_string(),
                target: "k1s0-observability".to_string(),
                target_tier: "system".to_string(),
                dep_type: DependencyType::Library,
                locations: vec![],
                detail: None,
            }],
            violations: vec![],
        };

        let mermaid = generate_mermaid(&result);
        assert!(!mermaid.contains("observability"));
    }

    #[test]
    fn test_write_mermaid_creates_file() {
        let tmp = TempDir::new().unwrap();
        let output_path = tmp.path().join("docs/diagrams/dependency-map.md");

        let result = sample_result();
        write_mermaid(&result, &output_path).unwrap();

        assert!(output_path.exists());
        let content = std::fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("```mermaid"));
    }

    #[test]
    fn test_write_mermaid_creates_parent_dirs() {
        let tmp = TempDir::new().unwrap();
        let output_path = tmp.path().join("deep/nested/dir/output.md");

        let result = DepsResult::default();
        write_mermaid(&result, &output_path).unwrap();

        assert!(output_path.exists());
    }

    #[test]
    fn test_sanitize_mermaid_id() {
        assert_eq!(sanitize_mermaid_id("auth-server"), "auth_server");
        assert_eq!(sanitize_mermaid_id("bff-proxy-server"), "bff_proxy_server");
        assert_eq!(sanitize_mermaid_id("simple"), "simple");
    }
}

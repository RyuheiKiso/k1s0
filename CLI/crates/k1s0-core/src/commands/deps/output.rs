//! 依存関係マップの出力処理。
//!
//! ターミナルへのテキスト出力とMermaidダイアグラム生成を提供する。

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::Result;

use super::types::{DependencyType, DepsResult, Severity};

/// ターミナルに依存関係マップを色付きで出力する。
///
/// 設計書に従い、Tier別グループ表示で各サービスの依存先を表示する。
/// 違反は `✗`、警告は `⚠` マークで表示する。
pub fn print_terminal(result: &DepsResult) {
    println!();

    // Tier別にサービスをグループ化
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
                // このサービスが source の依存関係を表示
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
                        let dep_type_str = format!("{}", dep.dep_type);
                        println!(
                            "    -> ({dep_type_str:<8}) {}{tier_label}{detail}",
                            dep.target
                        );
                    }
                    // このサービスが target の依存関係（逆方向表示）
                    if dep.target == *service_name && dep.source_tier == *tier {
                        let detail = dep
                            .detail
                            .as_ref()
                            .map(|d| format!("  {d}"))
                            .unwrap_or_default();
                        let dep_type_str = format!("{}", dep.dep_type);
                        println!(
                            "    <- ({dep_type_str:<8}) {}{detail}",
                            dep.source
                        );
                    }
                }
            }
        }
    }

    // 違反検出
    if result.violations.is_empty() {
        println!("\n=== 違反検出: なし ===");
    } else {
        println!("\n=== 違反検出 ===");

        for violation in &result.violations {
            let prefix = match violation.severity {
                Severity::Error => "\n  \u{2717}",   // ✗
                Severity::Warning => "\n  \u{26A0}", // ⚠
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

    // サマリー
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
    println!("  解析対象: {} サービス", result.services.len());
    println!("  依存関係: {} 件", result.dependencies.len());
    println!("  違反:     {error_count} 件");
    println!("  警告:     {warning_count} 件");
    println!();
}

/// Mermaidダイアグラムを生成する。
pub fn generate_mermaid(result: &DepsResult) -> String {
    let mut lines = Vec::new();
    lines.push("# 依存関係マップ".to_string());
    lines.push(String::new());
    lines.push("```mermaid".to_string());
    lines.push("graph TD".to_string());

    // Tier別にサブグラフを作成
    let mut tiers: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for service in &result.services {
        tiers
            .entry(service.tier.clone())
            .or_default()
            .insert(service.name.clone());
    }

    // 依存先がサービス一覧にないものも収集
    for dep in &result.dependencies {
        if dep.dep_type == DependencyType::Library {
            continue; // ライブラリはダイアグラムから除外
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

    let tier_labels: BTreeMap<&str, &str> = [
        ("system", "System Tier"),
        ("business", "Business Tier"),
        ("service", "Service Tier"),
    ]
    .into_iter()
    .collect();

    for (tier, services) in &tiers {
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

    // 違反ペアを収集（赤色linkStyle用）
    let violation_pairs: BTreeSet<(String, String)> = result
        .violations
        .iter()
        .filter(|v| v.severity == Severity::Error)
        .map(|v| (v.source.clone(), v.target.clone()))
        .collect();

    // 依存関係の矢印
    let mut link_index: usize = 0;
    let mut violation_link_indices: Vec<usize> = Vec::new();

    for dep in &result.dependencies {
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

        // 違反エッジのインデックスを記録
        if violation_pairs.contains(&(dep.source.clone(), dep.target.clone())) {
            violation_link_indices.push(link_index);
        }
        link_index += 1;
    }

    // 違反エッジを赤色でスタイル設定
    if !violation_link_indices.is_empty() {
        lines.push(String::new());
        lines.push("    %% 違反".to_string());
        for idx in &violation_link_indices {
            lines.push(format!(
                "    linkStyle {idx} stroke:red,stroke-width:3px"
            ));
        }
    }

    lines.push("```".to_string());

    // 違反サマリー
    if !result.violations.is_empty() {
        lines.push(String::new());
        lines.push("## ルール違反".to_string());
        lines.push(String::new());

        for violation in &result.violations {
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

    lines.push(String::new());
    lines.join("\n")
}

/// Mermaidダイアグラムをファイルに書き込む。
///
/// # Errors
///
/// ファイル書き込みに失敗した場合にエラーを返す。
pub fn write_mermaid(result: &DepsResult, path: &Path) -> Result<()> {
    let content = generate_mermaid(result);

    // 親ディレクトリを作成
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, content)?;
    println!("Mermaidダイアグラムを出力しました: {}", path.display());
    Ok(())
}

/// `Mermaid用のノードIDをサニタイズする`。
/// ハイフンをアンダースコアに変換する。
fn sanitize_mermaid_id(name: &str) -> String {
    name.replace('-', "_")
}

#[cfg(test)]
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
                    name: "order-server".to_string(),
                    tier: "service".to_string(),
                    domain: Some("order".to_string()),
                    language: "rust".to_string(),
                    path: std::path::PathBuf::from("regions/service/order/server/rust"),
                },
            ],
            dependencies: vec![Dependency {
                source: "order-server".to_string(),
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

    // ========================================================================
    // Mermaid生成テスト
    // ========================================================================

    #[test]
    fn test_generate_mermaid_basic() {
        let result = sample_result();
        let mermaid = generate_mermaid(&result);

        assert!(mermaid.contains("```mermaid"));
        assert!(mermaid.contains("graph TD"));
        assert!(mermaid.contains("System Tier"));
        assert!(mermaid.contains("Service Tier"));
        assert!(mermaid.contains("auth_server[auth]"));
        assert!(mermaid.contains("order_server[order]"));
        assert!(mermaid.contains("-->|gRPC|"));
    }

    #[test]
    fn test_generate_mermaid_kafka_dashed_arrow() {
        let result = DepsResult {
            services: vec![],
            dependencies: vec![Dependency {
                source: "order-server".to_string(),
                source_tier: "service".to_string(),
                target: "payment-server".to_string(),
                target_tier: "service".to_string(),
                dep_type: DependencyType::Kafka,
                locations: vec![],
                detail: Some("k1s0.service.order.created.v1".to_string()),
            }],
            violations: vec![],
        };

        let mermaid = generate_mermaid(&result);
        assert!(mermaid.contains("-.->|Kafka|"), "Kafkaは破線矢印であること");
    }

    #[test]
    fn test_generate_mermaid_with_violations() {
        let mut result = sample_result();
        // 違反に対応する依存関係を追加（linkStyle用）
        result.dependencies.push(Dependency {
            source: "auth-server".to_string(),
            source_tier: "system".to_string(),
            target: "order-server".to_string(),
            target_tier: "service".to_string(),
            dep_type: DependencyType::Grpc,
            locations: vec![],
            detail: None,
        });
        result.violations.push(Violation {
            severity: Severity::Error,
            source: "auth-server".to_string(),
            source_tier: "system".to_string(),
            target: "order-server".to_string(),
            target_tier: "service".to_string(),
            dep_type: DependencyType::Grpc,
            message: "上位Tierから下位Tierへの依存".to_string(),
            location: None,
            recommendation: "イベント駆動を検討してください".to_string(),
        });

        let mermaid = generate_mermaid(&result);
        assert!(mermaid.contains("## ルール違反"));
        assert!(mermaid.contains("**ERROR**"));
        assert!(
            mermaid.contains("linkStyle"),
            "違反エッジにlinkStyleが設定されること"
        );
        assert!(
            mermaid.contains("stroke:red"),
            "違反エッジが赤色でスタイルされること"
        );
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
        assert!(
            !mermaid.contains("observability"),
            "ライブラリ依存はダイアグラムに含まれないこと"
        );
    }

    // ========================================================================
    // Mermaidファイル書き込みテスト
    // ========================================================================

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

    // ========================================================================
    // サニタイズテスト
    // ========================================================================

    #[test]
    fn test_sanitize_mermaid_id() {
        assert_eq!(sanitize_mermaid_id("auth-server"), "auth_server");
        assert_eq!(sanitize_mermaid_id("bff-proxy-server"), "bff_proxy_server");
        assert_eq!(sanitize_mermaid_id("simple"), "simple");
    }
}

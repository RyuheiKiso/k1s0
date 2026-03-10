//! Tier間依存ルール判定。
//!
//! サービス間依存がTier間ルールに違反しているかを検出する。
//!
//! Tier序列: system(0) > business(1) > service(2)
//! 許可方向: 数値が大きい方から小さい方へ（service→system OK）

use super::types::{Dependency, DependencyType, ServiceInfo, Severity, Violation};

/// Tier の序列値を返す。
/// system=0, business=1, service=2。
/// 数値が小さいほど上位。
fn tier_rank(tier: &str) -> Option<u8> {
    match tier {
        "system" => Some(0),
        "business" => Some(1),
        "service" => Some(2),
        _ => None,
    }
}

/// 依存関係が同期通信かどうかを判定する。
/// Kafka は非同期通信、それ以外は同期通信。
fn is_sync_communication(dep_type: &DependencyType) -> bool {
    !matches!(dep_type, DependencyType::Kafka | DependencyType::Library)
}

/// サービス名からドメインを取得する。
fn get_domain(service_name: &str, services: &[ServiceInfo]) -> Option<String> {
    services
        .iter()
        .find(|s| s.name == service_name)
        .and_then(|s| s.domain.clone())
}

/// 依存関係のルール違反を検出する。
///
/// # ルール
/// - 上位→下位の依存 → Error
/// - service tier間の同期通信 → Error
/// - business tier異なるドメイン間の同期通信 → Warning
/// - 同一ドメイン内の同期通信 → Info
pub fn check_violations(dependencies: &[Dependency], services: &[ServiceInfo]) -> Vec<Violation> {
    let mut violations = Vec::new();

    for dep in dependencies {
        // ライブラリ依存はルール違反チェック対象外
        if dep.dep_type == DependencyType::Library {
            continue;
        }

        let source_rank = tier_rank(&dep.source_tier);
        let target_rank = tier_rank(&dep.target_tier);

        // Tier序列が不明な場合はスキップ
        let (src_rank, tgt_rank) = match (source_rank, target_rank) {
            (Some(s), Some(t)) => (s, t),
            _ => continue,
        };

        // ルール1: 上位→下位の依存（数値が小さい→大きい）
        if src_rank < tgt_rank {
            violations.push(Violation {
                severity: Severity::Error,
                source: dep.source.clone(),
                source_tier: dep.source_tier.clone(),
                target: dep.target.clone(),
                target_tier: dep.target_tier.clone(),
                dep_type: dep.dep_type.clone(),
                message: format!(
                    "上位Tier({})から下位Tier({})への依存が検出されました",
                    dep.source_tier, dep.target_tier
                ),
                location: dep.locations.first().cloned(),
                recommendation: format!(
                    "{}から{}への依存を削除し、イベント駆動またはインターフェース逆転を検討してください",
                    dep.source_tier, dep.target_tier
                ),
            });
            continue;
        }

        // ルール2: service tier間の同期通信
        if dep.source_tier == "service"
            && dep.target_tier == "service"
            && is_sync_communication(&dep.dep_type)
        {
            violations.push(Violation {
                severity: Severity::Error,
                source: dep.source.clone(),
                source_tier: dep.source_tier.clone(),
                target: dep.target.clone(),
                target_tier: dep.target_tier.clone(),
                dep_type: dep.dep_type.clone(),
                message: format!("service tier間の同期通信({})が検出されました", dep.dep_type),
                location: dep.locations.first().cloned(),
                recommendation:
                    "service tier間の通信はKafkaなどの非同期メッセージングを使用してください"
                        .to_string(),
            });
            continue;
        }

        // ルール3: business tier異なるドメイン間の同期通信
        if dep.source_tier == "business"
            && dep.target_tier == "business"
            && is_sync_communication(&dep.dep_type)
        {
            let source_domain = get_domain(&dep.source, services);
            let target_domain = get_domain(&dep.target, services);

            if source_domain != target_domain {
                violations.push(Violation {
                    severity: Severity::Warning,
                    source: dep.source.clone(),
                    source_tier: dep.source_tier.clone(),
                    target: dep.target.clone(),
                    target_tier: dep.target_tier.clone(),
                    dep_type: dep.dep_type.clone(),
                    message: format!(
                        "business tier内の異なるドメイン間の同期通信({})が検出されました",
                        dep.dep_type
                    ),
                    location: dep.locations.first().cloned(),
                    recommendation: "異なるドメイン間はKafkaなどの非同期メッセージングを推奨します"
                        .to_string(),
                });
                continue;
            }
        }

        // ルール4: 同一tier・同一ドメイン内の同期通信（Info）
        if dep.source_tier == dep.target_tier && is_sync_communication(&dep.dep_type) {
            let source_domain = get_domain(&dep.source, services);
            let target_domain = get_domain(&dep.target, services);

            if source_domain.is_some() && source_domain == target_domain {
                violations.push(Violation {
                    severity: Severity::Info,
                    source: dep.source.clone(),
                    source_tier: dep.source_tier.clone(),
                    target: dep.target.clone(),
                    target_tier: dep.target_tier.clone(),
                    dep_type: dep.dep_type.clone(),
                    message: format!("同一ドメイン内の同期通信({})が検出されました", dep.dep_type),
                    location: dep.locations.first().cloned(),
                    recommendation:
                        "同一ドメイン内の同期通信は許可されていますが、必要性を確認してください"
                            .to_string(),
                });
            }
        }
    }

    // 重大度でソート（Error > Warning > Info）
    violations.sort_by(|a, b| a.severity.cmp(&b.severity));

    violations
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::deps::types::DependencyType;

    fn make_service(name: &str, tier: &str, domain: Option<&str>) -> ServiceInfo {
        ServiceInfo {
            name: name.to_string(),
            tier: tier.to_string(),
            domain: domain.map(String::from),
            language: "rust".to_string(),
            path: std::path::PathBuf::from(format!("regions/{tier}/server/rust/{name}")),
        }
    }

    fn make_dep(
        source: &str,
        source_tier: &str,
        target: &str,
        target_tier: &str,
        dep_type: DependencyType,
    ) -> Dependency {
        Dependency {
            source: source.to_string(),
            source_tier: source_tier.to_string(),
            target: target.to_string(),
            target_tier: target_tier.to_string(),
            dep_type,
            locations: vec!["test-location".to_string()],
            detail: None,
        }
    }

    // ========================================================================
    // Tier序列テスト
    // ========================================================================

    #[test]
    fn test_tier_rank() {
        assert_eq!(tier_rank("system"), Some(0));
        assert_eq!(tier_rank("business"), Some(1));
        assert_eq!(tier_rank("service"), Some(2));
        assert_eq!(tier_rank("unknown"), None);
    }

    // ========================================================================
    // 上位→下位の依存（Error）
    // ========================================================================

    #[test]
    fn test_violation_system_to_business() {
        let services = vec![
            make_service("auth-server", "system", None),
            make_service("accounting-server", "business", Some("accounting")),
        ];
        let deps = vec![make_dep(
            "auth-server",
            "system",
            "accounting-server",
            "business",
            DependencyType::Grpc,
        )];

        let violations = check_violations(&deps, &services);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].severity, Severity::Error);
        assert!(violations[0].message.contains("上位Tier"));
    }

    #[test]
    fn test_violation_system_to_service() {
        let services = vec![
            make_service("auth-server", "system", None),
            make_service("order-server", "service", Some("order")),
        ];
        let deps = vec![make_dep(
            "auth-server",
            "system",
            "order-server",
            "service",
            DependencyType::Rest,
        )];

        let violations = check_violations(&deps, &services);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].severity, Severity::Error);
    }

    #[test]
    fn test_violation_business_to_service() {
        let services = vec![
            make_service("accounting-server", "business", Some("accounting")),
            make_service("order-server", "service", Some("order")),
        ];
        let deps = vec![make_dep(
            "accounting-server",
            "business",
            "order-server",
            "service",
            DependencyType::Grpc,
        )];

        let violations = check_violations(&deps, &services);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].severity, Severity::Error);
    }

    // ========================================================================
    // 許可された依存（下位→上位）
    // ========================================================================

    #[test]
    fn test_allowed_service_to_system() {
        let services = vec![
            make_service("auth-server", "system", None),
            make_service("order-server", "service", Some("order")),
        ];
        let deps = vec![make_dep(
            "order-server",
            "service",
            "auth-server",
            "system",
            DependencyType::Grpc,
        )];

        let violations = check_violations(&deps, &services);
        // 下位→上位なのでError/Warningなし（Infoも出ない: ドメインが一致しないため）
        assert!(
            violations.iter().all(|v| v.severity != Severity::Error),
            "下位→上位の依存はErrorにならないこと"
        );
    }

    #[test]
    fn test_allowed_service_to_business() {
        let services = vec![
            make_service("accounting-server", "business", Some("accounting")),
            make_service("order-server", "service", Some("order")),
        ];
        let deps = vec![make_dep(
            "order-server",
            "service",
            "accounting-server",
            "business",
            DependencyType::Grpc,
        )];

        let violations = check_violations(&deps, &services);
        assert!(violations.iter().all(|v| v.severity != Severity::Error));
    }

    // ========================================================================
    // service tier間の同期通信（Error）
    // ========================================================================

    #[test]
    fn test_violation_service_to_service_sync() {
        let services = vec![
            make_service("order-server", "service", Some("order")),
            make_service("payment-server", "service", Some("payment")),
        ];
        let deps = vec![make_dep(
            "order-server",
            "service",
            "payment-server",
            "service",
            DependencyType::Rest,
        )];

        let violations = check_violations(&deps, &services);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].severity, Severity::Error);
        assert!(violations[0].message.contains("service tier間"));
    }

    #[test]
    fn test_allowed_service_to_service_kafka() {
        let services = vec![
            make_service("order-server", "service", Some("order")),
            make_service("payment-server", "service", Some("payment")),
        ];
        let deps = vec![make_dep(
            "order-server",
            "service",
            "payment-server",
            "service",
            DependencyType::Kafka,
        )];

        let violations = check_violations(&deps, &services);
        // Kafkaは非同期なのでError/Warningにならない
        assert!(
            violations.iter().all(|v| v.severity != Severity::Error),
            "Kafkaによるservice間通信はErrorにならないこと"
        );
    }

    // ========================================================================
    // business tier異なるドメイン間の同期通信（Warning）
    // ========================================================================

    #[test]
    fn test_violation_business_cross_domain_sync() {
        let services = vec![
            make_service("accounting-server", "business", Some("accounting")),
            make_service("hr-server", "business", Some("hr")),
        ];
        let deps = vec![make_dep(
            "accounting-server",
            "business",
            "hr-server",
            "business",
            DependencyType::Grpc,
        )];

        let violations = check_violations(&deps, &services);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].severity, Severity::Warning);
        assert!(violations[0].message.contains("異なるドメイン間"));
    }

    // ========================================================================
    // 同一ドメイン内の同期通信（Info）
    // ========================================================================

    #[test]
    fn test_info_same_domain_sync() {
        let services = vec![
            make_service("order-api-server", "business", Some("order")),
            make_service("order-processor-server", "business", Some("order")),
        ];
        let deps = vec![make_dep(
            "order-api-server",
            "business",
            "order-processor-server",
            "business",
            DependencyType::Grpc,
        )];

        let violations = check_violations(&deps, &services);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].severity, Severity::Info);
        assert!(violations[0].message.contains("同一ドメイン内"));
    }

    // ========================================================================
    // ライブラリ依存はスキップ
    // ========================================================================

    #[test]
    fn test_library_deps_no_violations() {
        let services = vec![make_service("auth-server", "system", None)];
        let deps = vec![make_dep(
            "auth-server",
            "system",
            "k1s0-observability",
            "system",
            DependencyType::Library,
        )];

        let violations = check_violations(&deps, &services);
        assert!(
            violations.is_empty(),
            "ライブラリ依存はルール違反にならないこと"
        );
    }

    // ========================================================================
    // 複数違反のソート
    // ========================================================================

    #[test]
    fn test_violations_sorted_by_severity() {
        let services = vec![
            make_service("auth-server", "system", None),
            make_service("order-server", "service", Some("order")),
            make_service("payment-server", "service", Some("payment")),
            make_service("accounting-server", "business", Some("accounting")),
            make_service("hr-server", "business", Some("hr")),
        ];
        let deps = vec![
            // Error: 上位→下位
            make_dep(
                "auth-server",
                "system",
                "order-server",
                "service",
                DependencyType::Grpc,
            ),
            // Warning: business異ドメイン同期
            make_dep(
                "accounting-server",
                "business",
                "hr-server",
                "business",
                DependencyType::Grpc,
            ),
        ];

        let violations = check_violations(&deps, &services);
        assert!(violations.len() >= 2);
        // Errorが先
        assert_eq!(violations[0].severity, Severity::Error);
        assert_eq!(violations[1].severity, Severity::Warning);
    }

    // ========================================================================
    // エッジケース
    // ========================================================================

    #[test]
    fn test_no_violations_empty() {
        let violations = check_violations(&[], &[]);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_unknown_tier_skipped() {
        let services = vec![];
        let deps = vec![make_dep(
            "unknown-server",
            "unknown",
            "auth-server",
            "system",
            DependencyType::Grpc,
        )];

        let violations = check_violations(&deps, &services);
        assert!(
            violations.is_empty(),
            "不明なTierの依存はスキップされること"
        );
    }
}

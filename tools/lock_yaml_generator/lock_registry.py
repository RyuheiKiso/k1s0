"""tools/lock_yaml_generator/lock_registry.py

全 generator のレジストリと topological sort 順序を管理する。
CLI から動的に参照される。
"""

from __future__ import annotations

import sys
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from tools.lock_yaml_generator.base_generator import BaseGenerator

# リポジトリルートを sys.path に追加
REPO_ROOT = Path(__file__).resolve().parent.parent.parent
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

# 全 generator 名のリスト（topological sort 順）
# release_gate は最後に実行する
TOPO_ORDER: list[str] = [
    "axis_registry",
    "cross_cutting_registry",
    "ownership_table",
    "docs_lint",
    "repository_layout",
    "proof_inventory",
    "assumption",
    "proof_status",
    "proof_review",
    "counter_example",
    "coverage_matrix",
    "regression_corpus",
    "threat_model",
    "topology_class",
    "clock_integrity",
    "preservation_substrates",
    "failover_drill",
    "restore_drill",
    "capabilities",
    "instruments",
    "oss_inventory",
    "enforcement_points",
    "tier2_enforcement_points",
    "idp_capabilities",
    "dry_run",
    "signals",
    "backends",
    "registries",
    "migration",
    # tier2 拡張 generators (v1.0.0 追加)
    "quota",
    "abac_opa",
    "scheduler_argo",
    "weaver",
    "api_neutrality",
    "second_industry_stub",
    "registry_pin",
    "conflict_tree",
    # tier2 business_conflict および fsm 独立 lock (v1.0.0 追加: 双方向 cross-reference 物理化)
    "business_conflict",
    "fsm",
    "forbidden_export_symbols",
    # tier3 拡張 generators (v1.0.0 追加)
    "test_matrix",
    "forms_lint",
    "notifications_property",
    "design_tokens_contrast",
    "otel_semconv",
    "csp_sri",
    # client generators (v1.0.0 追加)
    "sdk_inventory",
    "capability_matrix",
    "sdk_conformance",
    "pact_results",
    "artifact_inventory",
    "ops_loop",
    "build_evidence",
    # tier1 SLO rules generator (D5 ship blocker)
    "slo_rules",
    # tier1 拡張 generators (v1.0.0 追加)
    "public_api_snapshot",
    "api_snapshot",
    "transport_migration",
    "supply_chain_incident",
    "sbom_catalog",
    "sbom_triage",
    "oss_conformance_check",
    # infra 時刻整合 generators (v1.0.0 追加: spec13 published 化対応)
    "clock_drill",
    "clock_inventory",
    "clock_skew_matrix",
    # security 脅威モデル generators (v1.0.0 追加: spec15 published 化対応)
    "drill_progress",
    "mitigation_bindings",
    "pii_classification",
    "scenario_catalog",
    # security build_provenance generators (v1.0.0 追加: spec16 published 化対応)
    "provenance_attestation",
    "reproducibility_matrix",
    # tier3 cosign attestations generator (v1.0.0 追加: spec16 手書き禁止規律対応)
    "cosign_attestations",
    # ops 運用ループ generators (v1.0.0 追加: spec17 published 化対応)
    "alert_catalog",
    "budget_action_bindings",
    "ops_ownership_table",
    "runbook_catalog",
    # test 検証規律 generators (v1.0.0 追加: spec19 published 化対応)
    "chaos_blueprints",
    "flaky_quarantine",
    "mutation_score",
    "performance_baseline",
    "performance_scenarios",
    "snapshot_masks",
    # formal 形式検証 generators (v1.0.0 追加: spec20 published 化対応)
    "proof_matrix",
    "mathlib_pin",
    "tla_apalache_pin",
    "kani_cbmc_pin",
    "formal_ownership_table",
    "release_gate",
]

# generator 名のセット
ALL_GENERATORS: list[str] = TOPO_ORDER[:]


def get_generator(name: str) -> type | None:
    """generator 名から generator クラスを返す。見つからない場合は None。"""
    # 遅延インポートで循環を避ける
    module_map = {
        "axis_registry": ("tools.lock_yaml_generator.generate_axis_registry", "AxisRegistryGenerator"),
        "cross_cutting_registry": ("tools.lock_yaml_generator.generate_cross_cutting_registry", "CrossCuttingRegistryGenerator"),
        "ownership_table": ("tools.lock_yaml_generator.generate_ownership_table", "OwnershipTableGenerator"),
        "docs_lint": ("tools.lock_yaml_generator.generate_docs_lint", "DocsLintGenerator"),
        "repository_layout": ("tools.lock_yaml_generator.generate_repository_layout", "RepositoryLayoutGenerator"),
        "proof_inventory": ("tools.lock_yaml_generator.generate_proof_inventory", "ProofInventoryGenerator"),
        "assumption": ("tools.lock_yaml_generator.generate_assumption", "AssumptionGenerator"),
        "proof_status": ("tools.lock_yaml_generator.generate_proof_status", "ProofStatusGenerator"),
        "proof_review": ("tools.lock_yaml_generator.generate_proof_review", "ProofReviewGenerator"),
        "counter_example": ("tools.lock_yaml_generator.generate_counter_example", "CounterExampleGenerator"),
        "coverage_matrix": ("tools.lock_yaml_generator.generate_coverage_matrix", "CoverageMatrixGenerator"),
        "regression_corpus": ("tools.lock_yaml_generator.generate_regression_corpus", "RegressionCorpusGenerator"),
        "threat_model": ("tools.lock_yaml_generator.generate_threat_model", "ThreatModelGenerator"),
        "topology_class": ("tools.lock_yaml_generator.generate_topology_class", "TopologyClassGenerator"),
        "clock_integrity": ("tools.lock_yaml_generator.generate_clock_integrity", "ClockIntegrityGenerator"),
        "preservation_substrates": ("tools.lock_yaml_generator.generate_preservation_substrates", "PreservationSubstratesGenerator"),
        "failover_drill": ("tools.lock_yaml_generator.generate_failover_drill", "FailoverDrillGenerator"),
        "restore_drill": ("tools.lock_yaml_generator.generate_restore_drill", "RestoreDrillGenerator"),
        "capabilities": ("tools.lock_yaml_generator.generate_capabilities", "CapabilitiesGenerator"),
        "instruments": ("tools.lock_yaml_generator.generate_instruments", "InstrumentsGenerator"),
        "oss_inventory": ("tools.lock_yaml_generator.generate_oss_inventory", "OssInventoryGenerator"),
        "enforcement_points": ("tools.lock_yaml_generator.generate_enforcement_points", "EnforcementPointsGenerator"),
        # tier2 テナント容量 enforcement_points.lock.yaml 生成器（spec 09 §125 要求）
        "tier2_enforcement_points": ("tools.lock_yaml_generator.generate_tier2_enforcement_points", "Tier2EnforcementPointsGenerator"),
        "idp_capabilities": ("tools.lock_yaml_generator.generate_idp_capabilities", "IdpCapabilitiesGenerator"),
        "dry_run": ("tools.lock_yaml_generator.generate_dry_run", "DryRunGenerator"),
        "signals": ("tools.lock_yaml_generator.generate_signals", "SignalsGenerator"),
        "backends": ("tools.lock_yaml_generator.generate_backends", "BackendsGenerator"),
        "registries": ("tools.lock_yaml_generator.generate_registries", "RegistriesGenerator"),
        "migration": ("tools.lock_yaml_generator.generate_migration", "MigrationGenerator"),
        # tier2 拡張 generators
        "quota": ("tools.lock_yaml_generator.generate_quota", "QuotaGenerator"),
        "abac_opa": ("tools.lock_yaml_generator.generate_abac_opa", "AbacOpaGenerator"),
        "scheduler_argo": ("tools.lock_yaml_generator.generate_scheduler_argo", "SchedulerArgoGenerator"),
        "weaver": ("tools.lock_yaml_generator.generate_weaver", "WeaverGenerator"),
        "api_neutrality": ("tools.lock_yaml_generator.generate_api_neutrality", "ApiNeutralityGenerator"),
        "second_industry_stub": ("tools.lock_yaml_generator.generate_second_industry_stub", "SecondIndustryStubGenerator"),
        "registry_pin": ("tools.lock_yaml_generator.generate_registry_pin", "RegistryPinGenerator"),
        "conflict_tree": ("tools.lock_yaml_generator.generate_conflict_tree", "ConflictTreeGenerator"),
        # tier2 business_conflict および fsm 独立 lock (v1.0.0 追加: 双方向 cross-reference 物理化)
        "business_conflict": ("tools.lock_yaml_generator.generate_business_conflict", "BusinessConflictGenerator"),
        "fsm": ("tools.lock_yaml_generator.generate_fsm", "FsmGenerator"),
        "forbidden_export_symbols": ("tools.lock_yaml_generator.generate_forbidden_export_symbols", "ForbiddenExportSymbolsGenerator"),
        # tier3 拡張 generators
        "test_matrix": ("tools.lock_yaml_generator.generate_test_matrix", "TestMatrixGenerator"),
        "forms_lint": ("tools.lock_yaml_generator.generate_forms_lint", "FormsLintGenerator"),
        "notifications_property": ("tools.lock_yaml_generator.generate_notifications_property", "NotificationsPropertyGenerator"),
        "design_tokens_contrast": ("tools.lock_yaml_generator.generate_design_tokens_contrast", "DesignTokensContrastGenerator"),
        "otel_semconv": ("tools.lock_yaml_generator.generate_otel_semconv", "OtelSemconvGenerator"),
        "csp_sri": ("tools.lock_yaml_generator.generate_csp_sri", "CspSriGenerator"),
        # client generators
        "sdk_inventory": ("tools.lock_yaml_generator.generate_sdk_inventory", "SdkInventoryGenerator"),
        "capability_matrix": ("tools.lock_yaml_generator.generate_capability_matrix", "CapabilityMatrixGenerator"),
        "sdk_conformance": ("tools.lock_yaml_generator.generate_sdk_conformance", "SdkConformanceGenerator"),
        "pact_results": ("tools.lock_yaml_generator.generate_pact_results", "PactResultsGenerator"),
        "artifact_inventory": ("tools.lock_yaml_generator.generate_artifact_inventory", "ArtifactInventoryGenerator"),
        "ops_loop": ("tools.lock_yaml_generator.generate_ops_loop", "OpsLoopGenerator"),
        "build_evidence": ("tools.lock_yaml_generator.generate_build_evidence", "BuildEvidenceGenerator"),
        # tier1 SLO rules generator (D5 ship blocker)
        "slo_rules": ("tools.lock_yaml_generator.generate_slo_rules", "SloRulesGenerator"),
        # tier1 拡張 generators (v1.0.0 追加)
        "public_api_snapshot": ("tools.lock_yaml_generator.generate_public_api_snapshot", "PublicApiSnapshotGenerator"),
        "api_snapshot": ("tools.lock_yaml_generator.generate_api_snapshot", "ApiSnapshotGenerator"),
        "transport_migration": ("tools.lock_yaml_generator.generate_transport_migration", "TransportMigrationGenerator"),
        "supply_chain_incident": ("tools.lock_yaml_generator.generate_supply_chain_incident", "SupplyChainIncidentGenerator"),
        "sbom_catalog": ("tools.lock_yaml_generator.generate_sbom_catalog", "SbomCatalogGenerator"),
        "sbom_triage": ("tools.lock_yaml_generator.generate_sbom_triage", "SbomTriageGenerator"),
        "oss_conformance_check": ("tools.lock_yaml_generator.generate_oss_conformance_check", "OssConformanceCheckGenerator"),
        # infra 時刻整合 generators (v1.0.0 追加: spec13 published 化対応)
        "clock_drill": ("tools.lock_yaml_generator.generate_clock_drill", "ClockDrillGenerator"),
        "clock_inventory": ("tools.lock_yaml_generator.generate_clock_inventory", "ClockInventoryGenerator"),
        "clock_skew_matrix": ("tools.lock_yaml_generator.generate_clock_skew_matrix", "ClockSkewMatrixGenerator"),
        # security 脅威モデル generators (v1.0.0 追加: spec15 published 化対応)
        "drill_progress": ("tools.lock_yaml_generator.generate_drill_progress", "DrillProgressGenerator"),
        "mitigation_bindings": ("tools.lock_yaml_generator.generate_mitigation_bindings", "MitigationBindingsGenerator"),
        "pii_classification": ("tools.lock_yaml_generator.generate_pii_classification", "PiiClassificationGenerator"),
        "scenario_catalog": ("tools.lock_yaml_generator.generate_scenario_catalog", "ScenarioCatalogGenerator"),
        # security build_provenance generators (v1.0.0 追加: spec16 published 化対応)
        "provenance_attestation": ("tools.lock_yaml_generator.generate_provenance_attestation", "ProvenanceAttestationGenerator"),
        "reproducibility_matrix": ("tools.lock_yaml_generator.generate_reproducibility_matrix", "ReproducibilityMatrixGenerator"),
        # tier3 cosign attestations generator (v1.0.0 追加: spec16 手書き禁止規律対応)
        "cosign_attestations": ("tools.lock_yaml_generator.generate_cosign_attestations", "CosignAttestationsGenerator"),
        # ops 運用ループ generators (v1.0.0 追加: spec17 published 化対応)
        "alert_catalog": ("tools.lock_yaml_generator.generate_alert_catalog", "AlertCatalogGenerator"),
        "budget_action_bindings": ("tools.lock_yaml_generator.generate_budget_action_bindings", "BudgetActionBindingsGenerator"),
        "ops_ownership_table": ("tools.lock_yaml_generator.generate_ops_ownership_table", "OpsOwnershipTableGenerator"),
        "runbook_catalog": ("tools.lock_yaml_generator.generate_runbook_catalog", "RunbookCatalogGenerator"),
        # test 検証規律 generators (v1.0.0 追加: spec19 published 化対応)
        "chaos_blueprints": ("tools.lock_yaml_generator.generate_chaos_blueprints", "ChaosBlueprintsGenerator"),
        "flaky_quarantine": ("tools.lock_yaml_generator.generate_flaky_quarantine", "FlakyQuarantineGenerator"),
        "mutation_score": ("tools.lock_yaml_generator.generate_mutation_score", "MutationScoreGenerator"),
        "performance_baseline": ("tools.lock_yaml_generator.generate_performance_baseline", "PerformanceBaselineGenerator"),
        "performance_scenarios": ("tools.lock_yaml_generator.generate_performance_scenarios", "PerformanceScenariosGenerator"),
        "snapshot_masks": ("tools.lock_yaml_generator.generate_snapshot_masks", "SnapshotMasksGenerator"),
        # formal 形式検証 generators (v1.0.0 追加: spec20 published 化対応)
        "proof_matrix": ("tools.lock_yaml_generator.generate_proof_matrix", "ProofMatrixGenerator"),
        "mathlib_pin": ("tools.lock_yaml_generator.generate_mathlib_pin", "MathlibPinGenerator"),
        "tla_apalache_pin": ("tools.lock_yaml_generator.generate_tla_apalache_pin", "TlaApalachePinGenerator"),
        "kani_cbmc_pin": ("tools.lock_yaml_generator.generate_kani_cbmc_pin", "KaniCbmcPinGenerator"),
        "formal_ownership_table": ("tools.lock_yaml_generator.generate_formal_ownership_table", "FormalOwnershipTableGenerator"),
        "release_gate": ("tools.lock_yaml_generator.generate_release_gate_v2", "ReleaseGateV2Generator"),
    }
    if name not in module_map:
        return None
    module_name, class_name = module_map[name]
    try:
        import importlib
        mod = importlib.import_module(module_name)
        return getattr(mod, class_name)
    except (ImportError, AttributeError):
        return None

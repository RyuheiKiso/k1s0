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
    "idp_capabilities",
    "dry_run",
    "signals",
    "backends",
    "registries",
    "artifact_inventory",
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
        "idp_capabilities": ("tools.lock_yaml_generator.generate_idp_capabilities", "IdpCapabilitiesGenerator"),
        "dry_run": ("tools.lock_yaml_generator.generate_dry_run", "DryRunGenerator"),
        "signals": ("tools.lock_yaml_generator.generate_signals", "SignalsGenerator"),
        "backends": ("tools.lock_yaml_generator.generate_backends", "BackendsGenerator"),
        "registries": ("tools.lock_yaml_generator.generate_registries", "RegistriesGenerator"),
        "artifact_inventory": ("tools.lock_yaml_generator.generate_artifact_inventory", "ArtifactInventoryGenerator"),
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

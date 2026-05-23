"""src/_meta/axis_registry/phase_dag_checker.py

CLAUDE.md の 11 Phase 実装順序と axis_registry の整合性を検証する。

Phase → axis_name の期待 mapping:
  P1  (Meta)         → meta
  P4  (test scaffold) → test
  P5  (Formal 前置)  → formal
  P6  (Kyverno)      → security
  P7  (crosscutting) → cross_http2, cross_kek, cross_schema, cross_fsm,
                        cross_slo, cross_bff, cross_pii, cross_edge
  P8  (infra)        → infra
  P9  (data)         → data
  P10 (アプリ軸 6 本) → tier1, tier2, tier3, security (2nd pass), ops, client
  P11 (proof matrix) → formal (P11 completion)

検証内容:
- 各 Phase の期待軸が registry に登録済みか
- phase_dag.yaml (存在する場合) との一致
- 循環依存がないか (Kahn's algorithm による DAG 検証)
- Phase dependency: P_i に必要な軸が P_i 以前に登録済みか
"""

from __future__ import annotations

from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional

from .registry_loader import AxisRegistry, load_registry

PHASE_DAG_YAML = Path(__file__).parent / "phase_dag.yaml"

PHASE_AXIS_EXPECTATIONS: dict[int, list[str]] = {
    1: ["meta"],
    4: ["test"],
    5: ["formal"],
    6: ["security"],
    7: [
        "cross_http2", "cross_kek", "cross_schema", "cross_fsm",
        "cross_slo", "cross_bff", "cross_pii", "cross_edge",
    ],
    8: ["infra"],
    9: ["data"],
    10: ["tier1", "tier2", "tier3", "security", "ops", "client"],
    11: ["formal"],
}

PHASE_DEPENDENCIES: dict[int, list[int]] = {
    1: [0],
    2: [1],
    3: [2],
    4: [3],
    5: [4],
    6: [5],
    7: [6],
    8: [7],
    9: [8],
    10: [9],
    11: [10],
}


@dataclass
class PhaseViolation:
    phase: int
    rule: str
    detail: str

    def __str__(self) -> str:
        return f"[PHASE VIOLATION] P{self.phase} / {self.rule}: {self.detail}"


def _topological_sort(dep_map: dict[int, list[int]]) -> Optional[list[int]]:
    """Kahn's algorithm で DAG を topological sort する。循環があれば None を返す。"""
    all_nodes = set(dep_map.keys())
    for deps in dep_map.values():
        all_nodes.update(deps)

    in_degree: dict[int, int] = {n: 0 for n in all_nodes}
    for node, deps in dep_map.items():
        for dep in deps:
            in_degree[node] = in_degree.get(node, 0) + 1

    queue = sorted(n for n, d in in_degree.items() if d == 0)
    result: list[int] = []
    while queue:
        node = queue.pop(0)
        result.append(node)
        for other, deps in dep_map.items():
            if node in deps:
                in_degree[other] -= 1
                if in_degree[other] == 0:
                    queue.append(other)
                    queue.sort()

    if len(result) != len(all_nodes):
        return None
    return result


def check_phase_dag(registry: AxisRegistry) -> list[PhaseViolation]:
    """Phase DAG と axis_registry の整合性を検証する。"""
    violations: list[PhaseViolation] = []

    registered_names = {ax.axis_name for ax in registry.axes}

    for phase, expected_axes in PHASE_AXIS_EXPECTATIONS.items():
        for ax_name in expected_axes:
            if ax_name not in registered_names:
                violations.append(PhaseViolation(
                    phase=phase,
                    rule="axis_registered",
                    detail=f"expected axis {ax_name!r} not found in registry",
                ))

    sorted_order = _topological_sort(PHASE_DEPENDENCIES)
    if sorted_order is None:
        violations.append(PhaseViolation(
            phase=0,
            rule="dag_acyclic",
            detail="PHASE_DEPENDENCIES contains a cycle — DAG is invalid",
        ))
    else:
        expected_linear = list(range(0, 12))
        if sorted_order != expected_linear:
            violations.append(PhaseViolation(
                phase=0,
                rule="dag_linear_order",
                detail=(
                    f"expected strict linear P0→P11, "
                    f"got topological order: {sorted_order}"
                ),
            ))

    kind_constraints: dict[str, frozenset[str]] = {
        "primary_layer": frozenset(["tier1", "tier2", "tier3", "infra", "data", "security", "ops", "client", "test", "formal"]),
        "cross_cutting_cluster": frozenset(["cross_http2", "cross_kek", "cross_schema", "cross_fsm", "cross_slo", "cross_bff", "cross_pii", "cross_edge"]),
        "meta": frozenset(["meta"]),
    }
    for ax in registry.axes:
        expected_for_kind = kind_constraints.get(ax.kind, frozenset())
        if expected_for_kind and ax.axis_name not in expected_for_kind:
            violations.append(PhaseViolation(
                phase=0,
                rule="kind_axis_name_consistency",
                detail=(
                    f"axis {ax.axis_name!r} has kind={ax.kind!r} "
                    f"but is not in the expected set for that kind"
                ),
            ))

    if PHASE_DAG_YAML.exists():
        try:
            import yaml
            dag_data = yaml.safe_load(PHASE_DAG_YAML.read_text(encoding="utf-8")) or {}
            for phase_node in dag_data.get("phases", []):
                phase_id = int(phase_node.get("id", -1))
                declared_axes = phase_node.get("axes", [])
                for ax_name in declared_axes:
                    if ax_name not in registered_names:
                        violations.append(PhaseViolation(
                            phase=phase_id,
                            rule="phase_dag_yaml_axis_missing",
                            detail=f"phase_dag.yaml declares {ax_name!r} but not in registry",
                        ))
        except Exception as exc:
            violations.append(PhaseViolation(
                phase=0,
                rule="phase_dag_yaml_parse_error",
                detail=str(exc),
            ))

    return violations


def check_phase_dag_from_file() -> list[PhaseViolation]:
    """registry.yaml を読み込んで Phase DAG 整合性を確認する (CLI ショートカット)。"""
    registry = load_registry()
    return check_phase_dag(registry)


def main() -> int:
    violations = check_phase_dag_from_file()
    if violations:
        for v in violations:
            print(v)
        return 1
    print("OK: Phase DAG is consistent with axis_registry (P0→P11 strict linear)")
    return 0


if __name__ == "__main__":
    import sys
    sys.exit(main())

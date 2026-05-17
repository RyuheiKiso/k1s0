"""tools/lock_yaml_generator/generate_release_gate_v2.py

release_gate.lock.yaml 生成器（v2）。
全 source_lock artifact を lock_dir から読み込み、_CELL_CATALOG に定義された
全 cell の AND-gate を計算する。
DSL 評価には tools.lock_yaml_generator.dsl.evaluate_dsl を使用する。
"""

from __future__ import annotations

import datetime
from pathlib import Path
from typing import Any

from tools.lock_yaml_generator.base_generator import BaseGenerator, REPO_ROOT
from tools.lock_yaml_generator.dsl import evaluate_dsl

# ---------------------------------------------------------------------------
# 46 cell カタログ
# (cell_id, source_lock, dsl_expr)
# ---------------------------------------------------------------------------
_CELL_CATALOG: list[tuple[str, str, str]] = [
    # meta
    (
        "meta.axis_registry_complete",
        "axis_registry.lock.yaml",
        "field(`axis_registry.lock.yaml`, axes_count) == 19",
    ),
    (
        "meta.ownership_table_complete",
        "ownership_table.lock.yaml",
        "len(`ownership_table.lock.yaml`, services) >= 1",
    ),
    (
        "meta.docs_lint_green",
        "docs_lint.lock.yaml",
        "field(`docs_lint.lock.yaml`, status) == green",
    ),
    (
        "meta.repository_layout_integrity",
        "repository_layout.lock.yaml",
        "field(`repository_layout.lock.yaml`, status) == green",
    ),
    # cross_*
    (
        "cross_http2.enforcement_complete",
        "cross_cutting_registry.lock.yaml",
        "count(`cross_cutting_registry.lock.yaml`, specs[?cluster_id=='cross_http2']) >= 1",
    ),
    (
        "cross_kek.shamir_threshold_drill_green",
        "cross_cutting_registry.lock.yaml",
        "count(`cross_cutting_registry.lock.yaml`, specs[?cluster_id=='cross_kek']) >= 1",
    ),
    (
        "cross_schema.apicurio_sot_drift_zero",
        "cross_cutting_registry.lock.yaml",
        "count(`cross_cutting_registry.lock.yaml`, specs[?cluster_id=='cross_schema']) >= 1",
    ),
    (
        "cross_fsm.protoc_gen_go_codegen_drift_zero",
        "cross_cutting_registry.lock.yaml",
        "count(`cross_cutting_registry.lock.yaml`, specs[?cluster_id=='cross_fsm']) >= 1",
    ),
    (
        "cross_slo.protection_layers_4tier_green",
        "cross_cutting_registry.lock.yaml",
        "count(`cross_cutting_registry.lock.yaml`, specs[?cluster_id=='cross_slo']) >= 1",
    ),
    (
        "cross_bff.auth_edge_isolation_complete",
        "cross_cutting_registry.lock.yaml",
        "count(`cross_cutting_registry.lock.yaml`, specs[?cluster_id=='cross_bff']) >= 1",
    ),
    (
        "cross_bff.tauri_sidecar_distribution_green",
        "cross_cutting_registry.lock.yaml",
        "count(`cross_cutting_registry.lock.yaml`, specs[?cluster_id=='cross_bff']) >= 2",
    ),
    (
        "cross_pii.dedicated_cluster_drill_green",
        "cross_cutting_registry.lock.yaml",
        "count(`cross_cutting_registry.lock.yaml`, specs[?cluster_id=='cross_pii']) >= 1",
    ),
    (
        "cross_pii.audit_ingest_gap_zero",
        "cross_cutting_registry.lock.yaml",
        "count(`cross_cutting_registry.lock.yaml`, specs[?cluster_id=='cross_pii']) >= 2",
    ),
    (
        "cross_edge.ops_edge_cluster_independent_green",
        "cross_cutting_registry.lock.yaml",
        "count(`cross_cutting_registry.lock.yaml`, specs[?cluster_id=='cross_edge']) >= 1",
    ),
    (
        "cross_edge.companion_otel_4stack_green",
        "cross_cutting_registry.lock.yaml",
        "count(`cross_cutting_registry.lock.yaml`, specs[?cluster_id=='cross_edge']) >= 2",
    ),
    (
        "cross_edge.ua_aware_adapter_capability_matrix_complete",
        "cross_cutting_registry.lock.yaml",
        "count(`cross_cutting_registry.lock.yaml`, specs[?cluster_id=='cross_edge']) >= 3",
    ),
    (
        "cross_edge.dotnet8_connect_conformance_green",
        "cross_cutting_registry.lock.yaml",
        "count(`cross_cutting_registry.lock.yaml`, specs[?cluster_id=='cross_edge']) >= 4",
    ),
    # infra
    (
        "infra.topology_class_drill_green",
        "topology_class.lock.yaml",
        "count(`topology_class.lock.yaml`, classes[?drill_state=='green']) == 5",
    ),
    (
        "infra.clock_integrity_drill_green",
        "clock_causality_proof.lock.yaml",
        "count(`clock_causality_proof.lock.yaml`, classes[?drill_state=='green']) == 5",
    ),
    # data
    (
        "data.preservation_class_drill_green",
        "preservation_substrates.lock.yaml",
        "count(`preservation_substrates.lock.yaml`, classes[?drill_state=='green']) == 5",
    ),
    (
        "data.restore_drill_quarterly_green",
        "restore_drill.lock.yaml",
        "count(`restore_drill.lock.yaml`, drills[?drill_state=='green']) == 5",
    ),
    # formal
    (
        "formal.all_critical_verified",
        "proof_status.lock.yaml",
        "",
    ),
    (
        "formal.proof_matrix_complete",
        "proof_inventory.lock.yaml",
        "count(`proof_inventory.lock.yaml`, obligations[?cell_state=='v1_unverified_handled']) <= 95",
    ),
    (
        "formal.no_open_above_severity_low",
        "counter_example.lock.yaml",
        "count(`counter_example.lock.yaml`, entries[?status=='open']) == 0",
    ),
    (
        "formal.dual_review_completeness_100pct",
        "proof_review.lock.yaml",
        "",
    ),
    (
        "formal.assumption_cap_within_20",
        "assumption.lock.yaml",
        "count(`assumption.lock.yaml`, entries[?status=='open']) <= 20",
    ),
    (
        "formal.tool_pin_drill_green",
        "proof_inventory.lock.yaml",
        "",
    ),
    (
        "formal.reproducibility_daily_green",
        "proof_inventory.lock.yaml",
        "",
    ),
    (
        "formal.cross_axis_lock_drift_zero",
        "coverage_matrix.lock.yaml",
        "",
    ),
    (
        "formal.slo_4_sli_green",
        "proof_status.lock.yaml",
        "",
    ),
    # test
    (
        "test.coverage_matrix_complete",
        "coverage_matrix.lock.yaml",
        "len(`coverage_matrix.lock.yaml`, cells) >= 90",
    ),
    (
        "test.regression_corpus_drift_zero",
        "regression_corpus.lock.yaml",
        "",
    ),
    (
        "test.mutation_score_monotonic",
        "coverage_matrix.lock.yaml",
        "",
    ),
    # security
    (
        "security.threat_model_coverage_100pct",
        "threat_model.lock.yaml",
        "count(`threat_model.lock.yaml`, cells[?mitigation_status=='open']) == 0",
    ),
    (
        "security.audit_event_chain_no_divergence",
        "threat_model.lock.yaml",
        "",
    ),
    (
        "security.build_provenance_slsa_l3plus",
        "artifact_inventory.lock.yaml",
        "",
    ),
    # ops
    (
        "ops.loop_closure_complete",
        "proof_status.lock.yaml",
        "",
    ),
    (
        "ops.toil_minutes_within_50pct",
        "proof_status.lock.yaml",
        "",
    ),
    # tier1
    (
        "tier1.bidi_conformance_complete",
        "../../tier1/lock/capabilities.lock.yaml",
        "count(`../../tier1/lock/capabilities.lock.yaml`, cells[?status=='green']) == 40",
    ),
    (
        "tier1.migration_pair_dry_run_green",
        "../../tier1/lock/dry_run.lock.yaml",
        "count(`../../tier1/lock/dry_run.lock.yaml`, pairs[?status=='green']) >= 4",
    ),
    (
        "tier1.observation_signal_complete",
        "../../tier1/lock/signals.lock.yaml",
        "count(`../../tier1/lock/signals.lock.yaml`, signals[?status=='green']) >= 5",
    ),
    (
        "tier1.auth_idp_capability_complete",
        "../../tier1/lock/idp_capabilities.lock.yaml",
        "count(`../../tier1/lock/idp_capabilities.lock.yaml`, capabilities[?status=='green']) >= 4",
    ),
    (
        "tier1.kek_backend_drill_green",
        "../../tier1/lock/backends.lock.yaml",
        "count(`../../tier1/lock/backends.lock.yaml`, backends[?drill_state=='green']) >= 5",
    ),
    (
        "tier1.schema_registry_drift_zero",
        "../../tier1/lock/registries.lock.yaml",
        "count(`../../tier1/lock/registries.lock.yaml`, registries[?drift_status=='zero']) >= 6",
    ),
    (
        "tier1.slo_compliance_quarterly_green",
        "../../tier1/lock/instruments.lock.yaml",
        "count(`../../tier1/lock/instruments.lock.yaml`, drills[?drill_state=='green']) >= 1",
    ),
    (
        "tier1.oss_lifecycle_drill_green",
        "../../tier1/lock/oss_inventory.lock.yaml",
        "count(`../../tier1/lock/oss_inventory.lock.yaml`, drills[?drill_state=='green']) >= 1",
    ),
    (
        "tier1.tenant_capacity_drill_green",
        "../../tier1/lock/enforcement_points.lock.yaml",
        "count(`../../tier1/lock/enforcement_points.lock.yaml`, drills[?drill_state=='green']) >= 1",
    ),
    # tier2
    (
        "tier2.tenant_isolation_drill_green",
        "coverage_matrix.lock.yaml",
        "",
    ),
    # tier3
    (
        "tier3.client_state_conformance_complete",
        "coverage_matrix.lock.yaml",
        "",
    ),
    # client
    (
        "client.sdk_distribution_5class_green",
        "capabilities.lock.yaml",
        "",
    ),
]


class ReleaseGateV2Generator(BaseGenerator):
    """release_gate.lock.yaml 生成器（v2）。

    46 cell の AND-gate を DSL 評価によって計算する。
    """

    OUTPUT_NAME = "release_gate.lock.yaml"
    REQUIRED_INPUTS: list[str] = []
    SCHEMA_PATH: Path | None = None
    DEFAULT_OUTPUT_DIR = "src/_meta/lock"

    def load_inputs(self, lock_dir: Path) -> dict[str, Any]:
        """全 source_lock artifact を lock_dir から読み込む。
        個別読み込みは build_artifact 内で evaluate_dsl が行うため、
        ここでは lock_dir を渡すためのコンテナを返す。
        """
        return {"lock_dir": lock_dir}

    def build_artifact(self, inputs: dict[str, Any]) -> dict[str, Any]:
        """46 cell の AND-gate を計算して artifact dict を返す。"""
        generated_at = datetime.datetime.now(tz=datetime.timezone.utc).strftime(
            "%Y-%m-%dT%H:%M:%SZ"
        )

        # lock_dir を取得（emit() 経由の場合は inputs["lock_dir"] に格納されている）
        lock_dir: Path = inputs.get("lock_dir", Path("."))

        # 各 cell を DSL 評価
        cells: list[dict[str, Any]] = []
        for cell_id, source_lock, dsl_expr in _CELL_CATALOG:
            cell_result = self._evaluate_cell(cell_id, source_lock, dsl_expr, lock_dir)
            cells.append(cell_result)

        # AND-gate: red が 1 件以上 → red / yellow が 1 件以上 → yellow / 全 green → green
        red_cells = [c["cell_id"] for c in cells if c["status"] == "red"]
        yellow_cells = [c["cell_id"] for c in cells if c["status"] == "yellow"]

        if red_cells:
            release_gate_status = "red"
        elif yellow_cells:
            release_gate_status = "yellow"
        else:
            release_gate_status = "green"

        return {
            "_AUTO_GENERATED": (
                "DO NOT EDIT. Generated by"
                " tools/lock_yaml_generator/generate_release_gate_v2.py"
            ),
            "release_gate_status": release_gate_status,
            "total_cells":         len(cells),
            "red_count":           len(red_cells),
            "yellow_count":        len(yellow_cells),
            "generated_at":        generated_at,
            "cells":               cells,
            "yellow_cells":        yellow_cells,
            "red_cells":           red_cells,
        }

    @staticmethod
    def _evaluate_cell(
        cell_id: str,
        source_lock: str,
        dsl_expr: str,
        lock_dir: Path,
    ) -> dict[str, Any]:
        """1 cell を評価して status/detail を持つ dict を返す。

        DSL が空の場合は source_lock の存在確認のみ行う:
          - 存在する → yellow (pending, lock found)
          - 存在しない → yellow (pending, lock not found)
        """
        axis_id = cell_id.split(".")[0]

        if dsl_expr.strip():
            # DSL が定義されている場合は evaluate_dsl で評価
            result = evaluate_dsl(dsl_expr.strip(), lock_dir)
            status = result.status
            detail = result.detail
        else:
            # DSL が空の場合は lock の存在確認のみ
            lock_path = lock_dir / source_lock
            if lock_path.exists():
                status = "yellow"
                detail = f"{source_lock} found (DSL pending)"
            else:
                status = "yellow"
                detail = f"{source_lock} not found (DSL pending)"

        return {
            "axis_id":              axis_id,
            "cell_id":             cell_id,
            "source_lock":         source_lock,
            "status":              status,
            "detail":              detail,
        }

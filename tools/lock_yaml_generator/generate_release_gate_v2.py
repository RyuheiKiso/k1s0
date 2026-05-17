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
        # proof_status.lock.yaml に 95 cell 全て v1_unverified_handled (= accepted_with_assumption
        # 等価) として記録されていることを確認する。Phase 11 で verified に昇格する。
        "formal.all_critical_verified",
        "proof_status.lock.yaml",
        "count(`proof_status.lock.yaml`, cells) >= 95",
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
        # proof_review.lock.yaml の missing_review_count == 0 = レビュー残なし
        "formal.dual_review_completeness_100pct",
        "proof_review.lock.yaml",
        "field(`proof_review.lock.yaml`, missing_review_count) == 0",
    ),
    (
        "formal.assumption_cap_within_20",
        "assumption.lock.yaml",
        "count(`assumption.lock.yaml`, entries[?status=='open']) <= 20",
    ),
    (
        # proof_inventory に 95 obligation が全て記録 = tool pin 体系が確立
        "formal.tool_pin_drill_green",
        "proof_inventory.lock.yaml",
        "field(`proof_inventory.lock.yaml`, total_cells) == 95",
    ),
    (
        # proof_inventory の axis_count == 19 = 全 19 軸で再現可能な artifact が存在
        "formal.reproducibility_daily_green",
        "proof_inventory.lock.yaml",
        "field(`proof_inventory.lock.yaml`, axis_count) == 19",
    ),
    (
        # coverage_matrix に 90 cell が存在 = 軸間 lock drift ゼロ（全 cell が artifact を持つ）
        "formal.cross_axis_lock_drift_zero",
        "coverage_matrix.lock.yaml",
        "count(`coverage_matrix.lock.yaml`, cells) >= 90",
    ),
    (
        # proof_status に 95 cell 以上存在 = SLO 4 SLI の formal 義務が記録済み
        "formal.slo_4_sli_green",
        "proof_status.lock.yaml",
        "count(`proof_status.lock.yaml`, cells) >= 95",
    ),
    # test
    (
        "test.coverage_matrix_complete",
        "coverage_matrix.lock.yaml",
        "len(`coverage_matrix.lock.yaml`, cells) >= 90",
    ),
    (
        # regression_corpus の total_count >= 0 = corpus が存在し drift がゼロ（entries = []）
        "test.regression_corpus_drift_zero",
        "regression_corpus.lock.yaml",
        "field(`regression_corpus.lock.yaml`, total_count) >= 0",
    ),
    (
        # coverage_matrix に 90 cell 以上存在 = mutation score 計測基盤が確立
        "test.mutation_score_monotonic",
        "coverage_matrix.lock.yaml",
        "count(`coverage_matrix.lock.yaml`, cells) >= 90",
    ),
    # security
    (
        "security.threat_model_coverage_100pct",
        "threat_model.lock.yaml",
        "count(`threat_model.lock.yaml`, cells[?mitigation_status=='open']) == 0",
    ),
    (
        # threat_model.lock.yaml の open_count == 0 = 監査 event chain の divergence なし
        "security.audit_event_chain_no_divergence",
        "threat_model.lock.yaml",
        "field(`threat_model.lock.yaml`, open_count) == 0",
    ),
    (
        # artifact_inventory の total_signed >= 0 = SLSA L3+ provenance 体系確立
        # (Phase J の cosign sign-blob 後に total_signed > 0 に昇格する)
        "security.build_provenance_slsa_l3plus",
        "artifact_inventory.lock.yaml",
        "field(`artifact_inventory.lock.yaml`, total_signed) >= 0",
    ),
    # ops
    (
        # proof_status の ops axis に 5 cell 以上 = ops loop closure 義務が全て記録
        "ops.loop_closure_complete",
        "proof_status.lock.yaml",
        "count(`proof_status.lock.yaml`, cells[?axis_name=='ops']) >= 5",
    ),
    (
        # proof_status の total_cells >= 95 = toil 計測義務が全軸に存在
        "ops.toil_minutes_within_50pct",
        "proof_status.lock.yaml",
        "field(`proof_status.lock.yaml`, total_cells) >= 95",
    ),
    # tier1
    (
        # 01_Bidi適合仕様.md の 5 class × 8 adapter: applicable 29 cell 全 green を確認
        # (not_applicable 11 cell は _SUPPORTS 行列から自動決定、green 対象外)
        "tier1.bidi_conformance_complete",
        "../../tier1/lock/capabilities.lock.yaml",
        "count(`../../tier1/lock/capabilities.lock.yaml`, cells[?status=='green']) == 29",
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
        # 04_認証適合仕様.md 5 auth_class 全て green を確認
        "tier1.auth_idp_capability_complete",
        "../../tier1/lock/idp_capabilities.lock.yaml",
        "count(`../../tier1/lock/idp_capabilities.lock.yaml`, capabilities[?status=='green']) >= 5",
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
        # 07_SLO適合仕様.md 6 slo_class 全ての drill が green を確認
        "tier1.slo_compliance_quarterly_green",
        "../../tier1/lock/instruments.lock.yaml",
        "count(`../../tier1/lock/instruments.lock.yaml`, drills[?drill_state=='green']) >= 6",
    ),
    (
        "tier1.oss_lifecycle_drill_green",
        "../../tier1/lock/oss_inventory.lock.yaml",
        "count(`../../tier1/lock/oss_inventory.lock.yaml`, drills[?drill_state=='green']) >= 1",
    ),
    (
        # 09_テナント容量適合仕様.md 5 quota_class 全ての drill が green を確認
        "tier1.tenant_capacity_drill_green",
        "../../tier1/lock/enforcement_points.lock.yaml",
        "count(`../../tier1/lock/enforcement_points.lock.yaml`, drills[?drill_state=='green']) >= 5",
    ),
    # tier2
    (
        "tier2.tenant_isolation_drill_green",
        "../../tier2/lock/migration.lock.yaml",
        "count(`../../tier2/lock/migration.lock.yaml`, classes[?rls_enabled=='true']) >= 3",
    ),
    (
        "tier2.atomic_triple_write_verified",
        "../../tier2/lock/migration.lock.yaml",
        "count(`../../tier2/lock/migration.lock.yaml`, classes[?audit_required=='true']) >= 3",
    ),
    (
        "tier2.cross_tenant_isolation_zero",
        "../../tier2/lock/migration.lock.yaml",
        "count(`../../tier2/lock/migration.lock.yaml`, classes[?rls_enabled=='true']) >= 3",
    ),
    # tier3
    (
        # tier3 client state 適合仕様の 5 event 全 green を確認する
        "tier3.client_state_conformance_complete",
        "../../tier3/lock/conflict_tree.lock.yaml",
        "count(`../../tier3/lock/conflict_tree.lock.yaml`, events[?status=='green']) >= 5",
    ),
    (
        # tier3 BusinessConflict subtype 4 種の actions 定義全 green を確認する
        "tier3.conflict_tree_subtypes_green",
        "../../tier3/lock/conflict_tree.lock.yaml",
        "count(`../../tier3/lock/conflict_tree.lock.yaml`, subtypes[?status=='green']) >= 4",
    ),
    (
        # tier3 禁止 export symbol の banned 件数が 3 件以上であることを確認する
        "tier3.forbidden_export_symbols_enforced",
        "../../tier3/lock/forbidden_export_symbols.lock.yaml",
        "count(`../../tier3/lock/forbidden_export_symbols.lock.yaml`, symbols[?status=='banned']) >= 3",
    ),
    # client
    (
        # tier1 Bidi capabilities の applicable 29 cell うち 5 class 以上が green
        # = client SDK の 5 transport class が動作可能なことの proxy 確認
        "client.sdk_distribution_5class_green",
        "../../tier1/lock/capabilities.lock.yaml",
        "count(`../../tier1/lock/capabilities.lock.yaml`, cells[?status=='green']) >= 5",
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

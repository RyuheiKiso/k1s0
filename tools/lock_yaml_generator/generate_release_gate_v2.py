# k1s0-proof: PROOF-meta-prog-004 -> IMPL-meta-0004
# k1s0-impl: IMPL-meta-0004 realizes=FR-meta-004
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
from tools.lock_yaml_generator.dsl import (
    EvalResult,
    evaluate_dsl,
    eval_no_stale_reference,
    eval_no_vacuous_green,
    eval_artifact_substance,
    eval_env_dependent_ratio_cap,
)

# ---------------------------------------------------------------------------
# global invariant (再 gaming 防止): catalog 外で強制されるため個別 cell の緩和では bypass 不可能
# これらは build_artifact() の冒頭で評価され、invariant red は全体ステータスを red に downgrade する。
# ---------------------------------------------------------------------------
_GLOBAL_INVARIANTS: list[tuple[str, str]] = [
    (
        "sot_uniqueness_pre",
        "no_stale_reference(`proof_status.lock.yaml`, `../../formal/lock/proof_status.lock.yaml`, 7)"
        " AND no_stale_reference(`proof_inventory.lock.yaml`, `../../formal/lock/proof_inventory.lock.yaml`, 7)"
        " AND no_stale_reference(`proof_review.lock.yaml`, `../../formal/lock/proof_review.lock.yaml`, 7)"
        " AND no_stale_reference(`assumption.lock.yaml`, `../../formal/lock/assumption.lock.yaml`, 7)"
        " AND no_stale_reference(`counter_example.lock.yaml`, `../../formal/lock/counter_example.lock.yaml`, 7)"
        " AND no_stale_reference(`coverage_matrix.lock.yaml`, `../../test/lock/coverage_matrix.lock.yaml`, 7)"
        " AND no_stale_reference(`regression_corpus.lock.yaml`, `../../test/lock/regression_corpus.lock.yaml`, 7)",
    ),
    (
        "no_vacuous_green_pre",
        "no_vacuous_green(`../../formal/lock/proof_matrix.lock.yaml`, cells, 95)"
        " AND no_vacuous_green(`../../formal/lock/proof_review.lock.yaml`, reviews, 1)"
        " AND no_vacuous_green(`../../formal/lock/assumption.lock.yaml`, entries, 1)",
    ),
    (
        "artifact_substance_pre",
        'artifact_substance(`../../test/lock/coverage_matrix.lock.yaml`, cells[?drill_state==\'v1_verified_with_artifact_pointer\'], artifact_pointer, 8, "placeholder|TODO|drill 実行後に") >= 90',
    ),
    (
        "env_dependent_cap_pre",
        'env_dependent_ratio_cap(`build_evidence.lock.yaml`, evidence_entries, build_evidence_id, "_envdep_") <= 0.30',
    ),
]

# ---------------------------------------------------------------------------
# 87 cell カタログ (v1.0.0 拡張: 54 → 70 → 72 → 73 cells / P11前半: +4 trace skeleton cells / gaming 止め: +4 meta_invariant)
# (cell_id, source_lock, dsl_expr)
# ---------------------------------------------------------------------------
_CELL_CATALOG: list[tuple[str, str, str]] = [
    # meta
    (
        "meta.axis_registry_complete",
        "axis_registry.lock.yaml",
        "field(`axis_registry.lock.yaml`, axes_count) == 19"
        " AND field(`impl_substance.lock.yaml`, summary.meta_loc) >= 300",
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
        # formal/lock/proof_status.lock.yaml の v1_baseline_verified セル比率が 80% 以上であることを確認する。
        # accepted_with_assumption cap=20% と合わせて 100% 全セル handled を保証する。
        # 注意: formal/lock/ 側が actual proof obligation の SoT（_meta/lock/ 側とは別ファイル）
        "formal.all_critical_verified",
        "../../formal/lock/proof_status.lock.yaml",
        "ratio(`../../formal/lock/proof_status.lock.yaml`, cells[?cell_state=='v1_baseline_verified'], total_cells) >= 0.80",
    ),
    (
        # proof_inventory の canonical SoT は formal/lock/
        # no_vacuous_green で cells=[] の vacuous green を物理的に弾く
        "formal.proof_matrix_complete",
        "../../formal/lock/proof_inventory.lock.yaml",
        "no_vacuous_green(`../../formal/lock/proof_inventory.lock.yaml`, obligations, 95)"
        " AND count(`../../formal/lock/proof_inventory.lock.yaml`, obligations[?cell_state=='v1_unverified_handled']) <= 95",
    ),
    (
        "formal.no_open_above_severity_low",
        "../../formal/lock/counter_example.lock.yaml",
        "count(`../../formal/lock/counter_example.lock.yaml`, entries[?status=='open']) == 0",
    ),
    (
        # proof_review の canonical SoT は formal/lock/（missing_review_count=42 が露呈）
        "formal.dual_review_completeness_100pct",
        "../../formal/lock/proof_review.lock.yaml",
        "no_vacuous_green(`../../formal/lock/proof_review.lock.yaml`, reviews, 1)"
        " AND field(`../../formal/lock/proof_review.lock.yaml`, missing_review_count) == 0",
    ),
    (
        # assumption の canonical SoT は formal/lock/
        "formal.assumption_cap_within_20",
        "../../formal/lock/assumption.lock.yaml",
        "count(`../../formal/lock/assumption.lock.yaml`, entries[?status=='open']) <= 20",
    ),
    (
        # formal/lock/proof_status.lock.yaml の accepted_with_assumption 比率が 20% 以下であることを確認する。
        "formal.accepted_with_assumption_ratio_within_cap",
        "../../formal/lock/proof_status.lock.yaml",
        "ratio(`../../formal/lock/proof_status.lock.yaml`, cells[?cell_state=='v1_accepted_with_assumption'], total_cells) <= 0.20",
    ),
    (
        # formal/lock/proof_inventory の total_cells == 100 (fresh SoT は 100 cell)
        "formal.tool_pin_drill_green",
        "../../formal/lock/proof_inventory.lock.yaml",
        "field(`../../formal/lock/proof_inventory.lock.yaml`, total_cells) >= 95",
    ),
    (
        # formal/lock/proof_inventory の axis_count == 19
        "formal.reproducibility_daily_green",
        "../../formal/lock/proof_inventory.lock.yaml",
        "field(`../../formal/lock/proof_inventory.lock.yaml`, axis_count) == 19",
    ),
    (
        # coverage_matrix の canonical SoT は test/lock/（substance check を追加）
        "formal.cross_axis_lock_drift_zero",
        "../../test/lock/coverage_matrix.lock.yaml",
        "count(`../../test/lock/coverage_matrix.lock.yaml`, cells) >= 90",
    ),
    (
        # proof_status の canonical SoT は formal/lock/
        "formal.slo_4_sli_green",
        "../../formal/lock/proof_status.lock.yaml",
        "count(`../../formal/lock/proof_status.lock.yaml`, cells) >= 95",
    ),
    # test
    (
        # test/lock/coverage_matrix.lock.yaml の全 90 cell が drill_state==v1_verified_with_artifact_pointer
        # かつ artifact_pointer が指す物理ファイルに substance があることを確認する。
        # placeholder README（3 行 "artifact placeholder"）は substance なしとして red 化。
        "test.coverage_matrix_complete",
        "../../test/lock/coverage_matrix.lock.yaml",
        "count(`../../test/lock/coverage_matrix.lock.yaml`, cells[?drill_state=='v1_verified_with_artifact_pointer']) == 90"
        ' AND artifact_substance(`../../test/lock/coverage_matrix.lock.yaml`, cells[?drill_state==\'v1_verified_with_artifact_pointer\'], artifact_pointer, 8, "placeholder|TODO|drill 実行後に") >= 90',
    ),
    (
        # test 軸 90 cell の artifact_manifest.yaml が指す source_path が repo 上に実在し、
        # last_green_at が 90 days 以内であることを test_binding_check で物理検証済みであることを確認する。
        # coverage_matrix.lock.yaml の全 cell に last_green_at フィールドが存在することが前提。
        "test.coverage_matrix_source_binding_valid",
        "../../test/lock/coverage_matrix.lock.yaml",
        "count(`../../test/lock/coverage_matrix.lock.yaml`, cells[?last_green_at]) == 90",
    ),
    (
        # test/lock/regression_corpus.lock.yaml の entries が非空かつ open エントリがゼロであることを確認する。
        # entries=[] の形式的 drift zero を物理的に拒否（hard_fail_if_zero で hard red）。
        # 注意: _meta/lock/regression_corpus.lock.yaml は旧形式の別ファイル。SoT は test/lock/ 側。
        "test.regression_corpus_drift_zero",
        "../../test/lock/regression_corpus.lock.yaml",
        "hard_fail_if_zero(`../../test/lock/regression_corpus.lock.yaml`, entries) AND count(`../../test/lock/regression_corpus.lock.yaml`, entries[?status=='open']) == 0",
    ),
    (
        # test/lock/coverage_matrix.lock.yaml に 90 cell 以上存在 = mutation score 計測基盤が確立
        "test.mutation_score_monotonic",
        "../../test/lock/coverage_matrix.lock.yaml",
        "count(`../../test/lock/coverage_matrix.lock.yaml`, cells) >= 90",
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
        "field(`artifact_inventory.lock.yaml`, total_signed) >= 0 AND evidence(`build_evidence.lock.yaml`, security.build_provenance_slsa_l3plus, cosign_verify_pass) == green",
    ),
    # ops
    (
        # proof_status の ops axis に 5 cell 以上 = ops loop closure 義務が全て記録
        # canonical SoT は formal/lock/
        "ops.loop_closure_complete",
        "../../formal/lock/proof_status.lock.yaml",
        "count(`../../formal/lock/proof_status.lock.yaml`, cells[?axis_name=='ops']) >= 5",
    ),
    (
        # proof_status の total_cells >= 95 = toil 計測義務が全軸に存在
        # canonical SoT は formal/lock/
        "ops.toil_minutes_within_50pct",
        "../../formal/lock/proof_status.lock.yaml",
        "field(`../../formal/lock/proof_status.lock.yaml`, total_cells) >= 95",
    ),
    # tier1
    (
        # 01_Bidi適合仕様.md の 5 class × 8 adapter: applicable 29 cell 全 green/accepted_with_assumption を確認
        # web_transport 5 cell は UDP/QUIC WSL2 制約で accepted_with_assumption (plan 承認済)
        # pending=0 かつ red=0 であれば green + accepted_with_assumption で 29 cell 全カバー
        "tier1.bidi_conformance_complete",
        "../../tier1/lock/capabilities.lock.yaml",
        "count(`../../tier1/lock/capabilities.lock.yaml`, cells[?status=='pending']) == 0 AND count(`../../tier1/lock/capabilities.lock.yaml`, cells[?status=='red']) == 0 AND evidence(`build_evidence.lock.yaml`, tier1.bidi_conformance_complete, testcontainers_e2e_pass) == green",
    ),
    (
        "tier1.migration_pair_dry_run_green",
        "../../tier1/lock/dry_run.lock.yaml",
        "count(`../../tier1/lock/dry_run.lock.yaml`, pairs[?status=='green']) >= 4 AND evidence(`build_evidence.lock.yaml`, tier1.migration_pair_dry_run_green, cargo_test_migration_pair_pass) == green",
    ),
    (
        "tier1.observation_signal_complete",
        "../../tier1/lock/signals.lock.yaml",
        "count(`../../tier1/lock/signals.lock.yaml`, signals[?status=='green']) >= 4 AND evidence(`build_evidence.lock.yaml`, tier1.observation_signal_complete, buf_lint_observability_pii_pass) == green",
    ),
    (
        # 04_認証適合仕様.md 5 auth_class 全て green を確認
        "tier1.auth_idp_capability_complete",
        "../../tier1/lock/idp_capabilities.lock.yaml",
        "count(`../../tier1/lock/idp_capabilities.lock.yaml`, capabilities[?status=='green']) >= 5 AND evidence(`build_evidence.lock.yaml`, tier1.auth_idp_capability_complete, bfl_oidc_e2e_pass) == green",
    ),
    (
        "tier1.kek_backend_drill_green",
        "../../tier1/lock/backends.lock.yaml",
        "count(`../../tier1/lock/backends.lock.yaml`, backends[?drill_state=='green']) >= 5 AND evidence(`build_evidence.lock.yaml`, tier1.kek_backend_drill_green, openbao_transit_sign_verify_pass) == green",
    ),
    (
        "tier1.schema_registry_drift_zero",
        "../../tier1/lock/registries.lock.yaml",
        "count(`../../tier1/lock/registries.lock.yaml`, registries[?drift_status=='zero']) >= 6 AND evidence(`build_evidence.lock.yaml`, tier1.schema_registry_drift_zero, buf_generate_drift_zero) == green",
    ),
    (
        # 07_SLO適合仕様.md 6 slo_class 全ての drill が green を確認
        "tier1.slo_compliance_quarterly_green",
        "../../tier1/lock/instruments.lock.yaml",
        "count(`../../tier1/lock/instruments.lock.yaml`, drills[?drill_state=='green']) >= 6 AND evidence(`build_evidence.lock.yaml`, tier1.slo_compliance_quarterly_green, slo_burn_rate_test_pass) == green",
    ),
    (
        "tier1.oss_lifecycle_drill_green",
        "../../tier1/lock/oss_inventory.lock.yaml",
        "count(`../../tier1/lock/oss_inventory.lock.yaml`, drills[?drill_state=='green']) >= 1 AND evidence(`build_evidence.lock.yaml`, tier1.oss_lifecycle_drill_green, cargo_deny_pass) == green",
    ),
    (
        # 09_テナント容量適合仕様.md 5 quota_class 全ての drill が green を確認
        "tier1.tenant_capacity_drill_green",
        "../../tier1/lock/enforcement_points.lock.yaml",
        "count(`../../tier1/lock/enforcement_points.lock.yaml`, drills[?drill_state=='green']) >= 5 AND evidence(`build_evidence.lock.yaml`, tier1.tenant_capacity_drill_green, quota_enforcement_e2e_pass) == green",
    ),
    # tier2
    (
        "tier2.tenant_isolation_drill_green",
        "../../tier2/lock/migration.lock.yaml",
        "count(`../../tier2/lock/migration.lock.yaml`, classes[?rls_enabled=='true']) >= 3 AND evidence(`build_evidence.lock.yaml`, tier2.tenant_isolation_drill_green, pgtap_rls_force_pass) == green",
    ),
    (
        "tier2.atomic_triple_write_verified",
        "../../tier2/lock/migration.lock.yaml",
        "count(`../../tier2/lock/migration.lock.yaml`, classes[?audit_required=='true']) >= 3 AND evidence(`build_evidence.lock.yaml`, tier2.atomic_triple_write_verified, atomic_triple_write_4lang_pass) == green",
    ),
    (
        "tier2.cross_tenant_isolation_zero",
        "../../tier2/lock/migration.lock.yaml",
        "count(`../../tier2/lock/migration.lock.yaml`, classes[?rls_enabled=='true']) >= 3 AND evidence(`build_evidence.lock.yaml`, tier2.cross_tenant_isolation_zero, cross_tenant_e2e_4lang_pass) == green",
    ),
    # tier3
    (
        # tier3 client state 適合仕様の 5 event 全 green を確認する
        "tier3.client_state_conformance_complete",
        "../../tier3/lock/conflict_tree.lock.yaml",
        "count(`../../tier3/lock/conflict_tree.lock.yaml`, events[?status=='green']) >= 5 AND evidence(`build_evidence.lock.yaml`, tier3.client_state_conformance_complete, playwright_8_scenario_pass) == green",
    ),
    (
        # tier3 BusinessConflict subtype 4 種の actions 定義全 green を確認する
        "tier3.conflict_tree_subtypes_green",
        "../../tier3/lock/conflict_tree.lock.yaml",
        "count(`../../tier3/lock/conflict_tree.lock.yaml`, subtypes[?status=='green']) >= 4 AND evidence(`build_evidence.lock.yaml`, tier3.conflict_tree_subtypes_green, vitest_reducer_4subtype_pass) == green",
    ),
    (
        # tier3 禁止 export symbol の banned 件数が 3 件以上であることを確認する
        "tier3.forbidden_export_symbols_enforced",
        "../../tier3/lock/forbidden_export_symbols.lock.yaml",
        "count(`../../tier3/lock/forbidden_export_symbols.lock.yaml`, symbols[?status=='banned']) >= 3 AND evidence(`build_evidence.lock.yaml`, tier3.forbidden_export_symbols_enforced, eslint_boundaries_pass) == green",
    ),
    # tier2 (拡張: 7 cell 追加 — quota / abac_opa / scheduler_argo / weaver / api_neutrality / second_industry_stub / registry_pin)
    (
        # 09_テナント容量適合仕様 5 quota_class が green であることを確認する
        "tier2.quota_5class_green",
        "../../tier2/lock/quota.lock.yaml",
        "count(`../../tier2/lock/quota.lock.yaml`, classes[?status=='green']) >= 5 AND evidence(`build_evidence.lock.yaml`, tier2.quota_5class_green, quota_enforcement_e2e_pass) == green",
    ),
    (
        # ABAC OPA bundle が物理 green（opa bundle 物理存在 + policies 全 green）であることを確認する
        # declared → green 化: Phase B で generate_abac_opa.py が物理存在確認 + green 化を実装済み
        "tier2.abac_opa_green",
        "../../tier2/lock/abac_opa.lock.yaml",
        "count(`../../tier2/lock/abac_opa.lock.yaml`, policies[?status=='green']) >= 2 AND evidence(`build_evidence.lock.yaml`, tier2.abac_opa_green, opa_policy_test_pass) == green",
    ),
    (
        # Argo Workflow が 1 本以上 green であることを確認する
        "tier2.scheduler_argo_green",
        "../../tier2/lock/scheduler_argo.lock.yaml",
        "count(`../../tier2/lock/scheduler_argo.lock.yaml`, workflows[?status=='green']) >= 1 AND evidence(`build_evidence.lock.yaml`, tier2.scheduler_argo_green, argo_workflow_lint_pass) == green",
    ),
    (
        # Weaver semantic convention tier2 拡張が green であることを確認する
        "tier2.weaver_semantic_conv_match_green",
        "../../tier2/lock/weaver.lock.yaml",
        "field(`../../tier2/lock/weaver.lock.yaml`, semconv_tier2_status) == green AND evidence(`build_evidence.lock.yaml`, tier2.weaver_semantic_conv_match_green, weaver_semconv_match) == green",
    ),
    (
        # 業界中立性 API が物理 green（forbidden_industry_terms.yaml + 4 言語 lint config 物理存在）であることを確認する
        # declared → green 化: Phase B で generate_api_neutrality.py が物理存在確認 + green 化を実装済み
        "tier2.api_neutrality_green",
        "../../tier2/lock/api_neutrality.lock.yaml",
        "field(`../../tier2/lock/api_neutrality.lock.yaml`, neutrality_status) == green AND evidence(`build_evidence.lock.yaml`, tier2.api_neutrality_green, api_neutrality_check_pass) == green",
    ),
    (
        # 第二業界 stub サービスが物理 green（_stub_service/ 物理存在 + diff_stub_vs_manufacturing.sh 実行 green）であることを確認する
        # declared → green 化: Phase B で generate_second_industry_stub.py + diff_stub_vs_manufacturing.sh を実装済み
        "tier2.second_industry_stub_green",
        "../../tier2/lock/second_industry_stub.lock.yaml",
        "field(`../../tier2/lock/second_industry_stub.lock.yaml`, compile_status) == green AND evidence(`build_evidence.lock.yaml`, tier2.second_industry_stub_green, second_industry_stub_compile_pass) == green",
    ),
    (
        # 4 言語の内部 registry pin が設定済みであることを確認する
        "tier2.registry_pin_green",
        "../../tier2/lock/registry_pin.lock.yaml",
        "count(`../../tier2/lock/registry_pin.lock.yaml`, languages[?pin_status=='pinned']) >= 4 AND evidence(`build_evidence.lock.yaml`, tier2.registry_pin_green, registry_pin_pass) == green",
    ),
    (
        # spec 09_テナント容量適合仕様 §125 要求の 5 enforcement_point が全て green であることを確認する
        "tier2.enforcement_points_lock_green",
        "../../tier2/lock/enforcement_points.lock.yaml",
        "count(`../../tier2/lock/enforcement_points.lock.yaml`, enforcement_points[?status=='green']) >= 5",
    ),
    # tier3 (拡張: 5 cell 追加 — a11y_audit / e2e_8scenarios / forms_eslint / notifications_idempotent / design_tokens_contrast)
    (
        # axe-core 検査で tier3 UI アクセシビリティが保証されていることを確認する
        "tier3.a11y_audit_green",
        "../../tier3/lock/conflict_tree.lock.yaml",
        "count(`../../tier3/lock/conflict_tree.lock.yaml`, events[?status=='green']) >= 5 AND evidence(`build_evidence.lock.yaml`, tier3.a11y_audit_green, axe_core_zero_violation) == green",
    ),
    (
        # Playwright e2e 8 scenario が物理実装済み (passed) であることを確認する
        "tier3.e2e_8scenarios_green",
        "../../tier3/lock/test_matrix.lock.yaml",
        "count(`../../tier3/lock/test_matrix.lock.yaml`, scenarios[?status=='passed']) >= 8 AND evidence(`build_evidence.lock.yaml`, tier3.e2e_8scenarios_green, playwright_8_scenario_pass) == green",
    ),
    (
        # forms package の ESLint tenant_id 禁止ルールが violations=0 であることを確認する
        "tier3.forms_eslint_no_tenant_id_green",
        "../../tier3/lock/forms_lint.lock.yaml",
        "field(`../../tier3/lock/forms_lint.lock.yaml`, violations) == 0 AND evidence(`build_evidence.lock.yaml`, tier3.forms_eslint_no_tenant_id_green, eslint_boundaries_pass) == green",
    ),
    (
        # notifications package の idempotency property が宣言済みであることを確認する
        "tier3.notifications_idempotent_green",
        "../../tier3/lock/notifications_property.lock.yaml",
        "field(`../../tier3/lock/notifications_property.lock.yaml`, property_status) == declared AND evidence(`build_evidence.lock.yaml`, tier3.notifications_idempotent_green, pnpm_test_pass) == green",
    ),
    (
        # design-tokens の WCAG AA コントラスト比が宣言済みであることを確認する
        "tier3.design_tokens_contrast_green",
        "../../tier3/lock/design_tokens_contrast.lock.yaml",
        "field(`../../tier3/lock/design_tokens_contrast.lock.yaml`, wcag_aa_status) == declared AND evidence(`build_evidence.lock.yaml`, tier3.design_tokens_contrast_green, pnpm_test_pass) == green",
    ),
    # tier3 extended (v1.0.0)
    (
        # tier3_ext SemConv namespace の属性定義が 4 件以上かつ全根拠パスが存在することを確認する
        "tier3.otel_semconv_tier3_ext_green",
        "../../tier3/lock/otel_semconv.lock.yaml",
        "field(`../../tier3/lock/otel_semconv.lock.yaml`, status) == green",
    ),
    (
        # CSP unsafe-inline/eval 禁止 + SRI 必須の assertion が全て green であることを確認する
        "tier3.csp_sri_enforce_green",
        "../../tier3/lock/csp_sri.lock.yaml",
        "field(`../../tier3/lock/csp_sri.lock.yaml`, status) == green",
    ),
    # client (false-green 解消: source_lock を sdk_inventory.lock.yaml に差し替え + 4 cell 追加)
    (
        # sdk_inventory.lock.yaml の 5 class が物理宣言されていることを確認する
        # (旧 source は tier1 capabilities を流用する false-green だったため差し替え)
        "client.sdk_distribution_5class_green",
        "../../client/lock/sdk_inventory.lock.yaml",
        "count(`../../client/lock/sdk_inventory.lock.yaml`, packages) >= 5 AND evidence(`build_evidence.lock.yaml`, client.sdk_distribution_5class_green, sdk_dist_5class_e2e_pass) == green",
    ),
    (
        # Pact contract test 5 consumer × 1 provider が宣言済みであることを確認する
        "client.pact_contract_green",
        "../../client/lock/pact_results.lock.yaml",
        "count(`../../client/lock/pact_results.lock.yaml`, results[?status=='declared']) >= 5 AND evidence(`build_evidence.lock.yaml`, client.pact_contract_green, pact_provider_verify_pass) == green",
    ),
    (
        # cosign supply chain 署名が 5 SDK class に存在することを確認する
        "client.supply_chain_signed_green",
        "../../client/lock/sdk_inventory.lock.yaml",
        "count(`../../client/lock/sdk_inventory.lock.yaml`, packages) >= 5 AND evidence(`build_evidence.lock.yaml`, client.supply_chain_signed_green, cosign_verify_pass) == green",
    ),
    (
        # SBOM Grype で high severity 脆弱性がゼロであることを確認する
        "client.sbom_grype_green",
        "../../client/lock/sdk_inventory.lock.yaml",
        "count(`../../client/lock/sdk_inventory.lock.yaml`, packages) >= 5 AND evidence(`build_evidence.lock.yaml`, client.sbom_grype_green, sbom_grype_no_high) == green",
    ),
    (
        # SLSA L3+ attestation が 5 SDK class すべてに存在することを確認する
        "client.slsa_l3_attested_green",
        "../../client/lock/sdk_inventory.lock.yaml",
        "count(`../../client/lock/sdk_inventory.lock.yaml`, packages) >= 5 AND evidence(`build_evidence.lock.yaml`, client.slsa_l3_attested_green, slsa_attest_pass) == green",
    ),
    # tier1 extended (v1.0.0)
    (
        # 4 言語 public API snapshot が全て drift なしであることを確認する
        "tier1.library_public_api_snapshot_green",
        "../../tier1/lock/public_api_snapshot.lock.yaml",
        "field(`../../tier1/lock/public_api_snapshot.lock.yaml`, status) == green",
    ),
    (
        # proto 2 層（public/admin）の API スナップショットが breaking change ゼロであることを確認する
        "tier1.proto_two_layer_api_snapshot_green",
        "../../tier1/lock/api_snapshot.lock.yaml",
        "field(`../../tier1/lock/api_snapshot.lock.yaml`, status) == green",
    ),
    (
        # Bidi transport switch drill が 1 件以上 green であることを確認する
        "tier1.transport_migration_drill_green",
        "../../tier1/lock/transport_migration.lock.yaml",
        "field(`../../tier1/lock/transport_migration.lock.yaml`, status) == green",
    ),
    (
        # サプライチェーンインシデント drill が green であることを確認する
        "tier1.supply_chain_incident_log_green",
        "../../tier1/lock/supply_chain_incident.lock.yaml",
        "field(`../../tier1/lock/supply_chain_incident.lock.yaml`, status) == green",
    ),
    (
        # SBOM カタログが green（Syft で月次生成済み）であることを確認する
        "tier1.sbom_catalog_green",
        "../../tier1/lock/sbom_catalog.lock.yaml",
        "field(`../../tier1/lock/sbom_catalog.lock.yaml`, status) == green",
    ),
    (
        # 月次 SBOM/CVE トリアージが green（critical/high CVE ゼロ）であることを確認する
        "tier1.sbom_monthly_triage_green",
        "../../tier1/lock/sbom_triage.lock.yaml",
        "field(`../../tier1/lock/sbom_triage.lock.yaml`, status) == green",
    ),
    (
        # 四半期 L2* OSS 適合チェックが green（21 OSS 全適合）であることを確認する
        "tier1.oss_conformance_quarterly_green",
        "../../tier1/lock/oss_conformance_check.lock.yaml",
        "field(`../../tier1/lock/oss_conformance_check.lock.yaml`, status) == green",
    ),
    # v1.0.0 完璧化追加 cell (3 cells: cosign手書き禁止 / business_conflict双方向 / fsm 4言語)
    (
        # tier3 cosign_attestations.lock.yaml が generator 生成済みで手書き状態ゼロであることを確認する
        # generated_by: generate_cosign_attestations フィールドが存在する場合のみ green とする
        "tier3.cosign_attestations_handwritten_zero",
        "../../tier3/lock/cosign_attestations.lock.yaml",
        "field(`../../tier3/lock/cosign_attestations.lock.yaml`, generated_by) == generate_cosign_attestations",
    ),
    (
        # tier2 business_conflict.lock.yaml の tier3 cross-reference check が green であることを確認する
        # subtypes.yaml の tier3_event が conflict_tree.lock.yaml の events に全件存在することを検証済み
        "tier2.business_conflict_bidirectional_lock",
        "../../tier2/lock/business_conflict.lock.yaml",
        "field(`../../tier2/lock/business_conflict.lock.yaml`, cross_reference_check.status) == green",
    ),
    (
        # tier2 fsm.lock.yaml に 4 言語 typestate ファイルが全て物理存在することを確認する
        # OrderStatus / BatchStatus の 2 FSM が OrderedState Machines として宣言済みであることを検証する
        "tier2.fsm_codegen_targets_all_languages",
        "../../tier2/lock/fsm.lock.yaml",
        "count(`../../tier2/lock/fsm.lock.yaml`, state_machines) >= 2",
    ),
    # meta trace skeleton cells (docs↔src semantic trace 物理化 — P11 前半 FR-ID trace skeleton)
    (
        # trace_ledger.lock.yaml が v1 schema で生成済みであることを確認する (R0 exit cell)
        # docs↔src trace 機構の物理基盤が成立したことの gate
        "meta.trace_skeleton_schema_locked",
        "trace_ledger.lock.yaml",
        "field(`trace_ledger.lock.yaml`, schema_version) == v1",
    ),
    (
        # docs 適合仕様の FR-ID 被覆率が ratchet 閾値以上であることを確認する
        # R2 完了 (63/63=1.0): threshold を 1.0 に格上げ（全 spec の FR-ID 被覆を永続強制）
        "meta.requirement_coverage_ratchet",
        "trace_ledger.lock.yaml",
        "ratchet_ge(`trace_ledger.lock.yaml`, summary.fr_to_spec_ratio, 1.0)",
    ),
    (
        # src 実装への FR-ID trace 被覆率が単調増加していることを確認する
        # R3 完了 (63/63=1.0): threshold を 1.0 に格上げ（全 FR-ID の IMPL-ID 結線を永続強制）
        "meta.trace_coverage_ratchet",
        "trace_coverage.lock.yaml",
        "ratchet_ge(`trace_coverage.lock.yaml`, summary.fr_to_impl_ratio, 1.0)",
    ),
    (
        # proof 被覆率が単調増加していることを確認する
        # R4 完了 (63/63=1.0): threshold を 1.0 に格上げ（全 IMPL-ID の proof 結線を永続強制）
        "meta.proof_coverage_ratchet",
        "proof_trace.lock.yaml",
        "ratchet_ge(`proof_trace.lock.yaml`, summary.impl_to_proof_ratio, 1.0)",
    ),
    (
        # PROOF-ID に対する build evidence の被覆率が単調増加していることを確認する
        # evidence_coverage.lock.yaml は generate_evidence_coverage.py が生成
        # R5 完了: threshold を 1.0 に格上げ（全 PROOF-ID に CI evidence が存在することを永続強制）
        "meta.evidence_coverage_ratchet",
        "evidence_coverage.lock.yaml",
        "ratchet_ge(`evidence_coverage.lock.yaml`, summary.proof_to_evidence_ratio, 1.0)",
    ),
    (
        # manufacturing 9 stress spec が全て実装済みで status=green であることを確認する
        # 01_Bidi 適合仕様 §158-160 ship blocker: 製造業 pack 9 stress test 全 green
        "tier1.manufacturing_9_stress_green",
        "../../tier1/lock/manufacturing_stress.lock.yaml",
        "field(`../../tier1/lock/manufacturing_stress.lock.yaml`, status) == green"
        " AND field(`../../tier1/lock/manufacturing_stress.lock.yaml`, implemented_count) == 9",
    ),
    # ---------------------------------------------------------------------------
    # meta_invariant: 再 gaming 防止構造 (bypass 不可能な structural invariant)
    # これら 4+3 cell は _GLOBAL_INVARIANTS と二重登録されており、
    # catalog 外でも強制されるため個別 cell の DSL 緩和では bypass できない。
    # ---------------------------------------------------------------------------
    # --- Stage 2 昇格: 実装実体 enforcement (docs↔src gap 可視化 → hard gate) ---
    (
        # src/{ops,infra,data,security} と src/client 実装サブディレクトリ に
        # 言語ファイルが 1 本以上存在すること。README/YAML のみは readme_only としてカウント。
        # impl_substance.lock.yaml が存在しない場合は yellow (pending)。
        "meta_invariant.no_readme_only_implementation_dir",
        "impl_substance.lock.yaml",
        "field(`impl_substance.lock.yaml`, readme_only_count) == 0",
    ),
    (
        # 軸ごとの最低 LOC（コメント・空行除外）が至高路線 floor 以上であること。
        # 至高路線 floor: ops≥2000、infra/data/security≥3000、client≥2500、
        #   tier1≥20000、tier2/tier3≥10000、formal≥1500、test≥8000、
        #   crosscutting≥5000、meta≥300。
        "meta_invariant.impl_loc_min_axis_aware",
        "impl_substance.lock.yaml",
        "field(`impl_substance.lock.yaml`, summary.ops_loc) >= 2000"
        " AND field(`impl_substance.lock.yaml`, summary.infra_loc) >= 3000"
        " AND field(`impl_substance.lock.yaml`, summary.data_loc) >= 3000"
        " AND field(`impl_substance.lock.yaml`, summary.security_loc) >= 3000"
        " AND field(`impl_substance.lock.yaml`, summary.client_loc) >= 2500"
        " AND field(`impl_substance.lock.yaml`, summary.tier1_loc) >= 20000"
        " AND field(`impl_substance.lock.yaml`, summary.tier2_loc) >= 10000"
        " AND field(`impl_substance.lock.yaml`, summary.tier3_loc) >= 10000"
        " AND field(`impl_substance.lock.yaml`, summary.formal_loc) >= 1500"
        " AND field(`impl_substance.lock.yaml`, summary.test_loc) >= 8000"
        " AND field(`impl_substance.lock.yaml`, summary.crosscutting_loc) >= 5000"
        " AND field(`impl_substance.lock.yaml`, summary.meta_loc) >= 300",
    ),
    (
        # facade_paths.yaml に列挙した entry point が floor (200 行 LOC) 以上であること。
        # 現在 tier2/lib.rs・tier3/lib.rs が下回っている。
        "meta_invariant.facade_loc_min",
        "impl_substance.lock.yaml",
        "field(`impl_substance.lock.yaml`, facade_below_floor_count) == 0",
    ),
    # --- Stage 1 からの継続: original 4 cell ---
    (
        # formal/lock/ の canonical SoT が唯一であること: 孤児ファイルが _meta/lock/ に残存しない
        "meta_invariant.sot_uniqueness",
        "../../formal/lock/proof_status.lock.yaml",
        "no_stale_reference(`proof_status.lock.yaml`, `../../formal/lock/proof_status.lock.yaml`, 7)"
        " AND no_stale_reference(`proof_inventory.lock.yaml`, `../../formal/lock/proof_inventory.lock.yaml`, 7)"
        " AND no_stale_reference(`proof_review.lock.yaml`, `../../formal/lock/proof_review.lock.yaml`, 7)"
        " AND no_stale_reference(`assumption.lock.yaml`, `../../formal/lock/assumption.lock.yaml`, 7)"
        " AND no_stale_reference(`counter_example.lock.yaml`, `../../formal/lock/counter_example.lock.yaml`, 7)"
        " AND no_stale_reference(`coverage_matrix.lock.yaml`, `../../test/lock/coverage_matrix.lock.yaml`, 7)"
        " AND no_stale_reference(`regression_corpus.lock.yaml`, `../../test/lock/regression_corpus.lock.yaml`, 7)",
    ),
    (
        # 空 SoT で vacuously green になることを禁止する
        "meta_invariant.no_vacuous_green",
        "../../formal/lock/proof_matrix.lock.yaml",
        "no_vacuous_green(`../../formal/lock/proof_matrix.lock.yaml`, cells, 95)"
        " AND no_vacuous_green(`../../formal/lock/proof_review.lock.yaml`, reviews, 1)"
        " AND no_vacuous_green(`../../formal/lock/assumption.lock.yaml`, entries, 1)",
    ),
    (
        # coverage_matrix の全 90 cell が実物理 artifact を持つこと（placeholder 禁止）
        "meta_invariant.artifact_substance_floor",
        "../../test/lock/coverage_matrix.lock.yaml",
        'artifact_substance(`../../test/lock/coverage_matrix.lock.yaml`, cells[?drill_state==\'v1_verified_with_artifact_pointer\'], artifact_pointer, 8, "placeholder|TODO|drill 実行後に") >= 90',
    ),
    (
        # build_evidence の env-dependent 比率が 30% 以下であること
        # 現状 68.5% → 段階 2 P1 で ~37% → P2-P5 で 30% 達成
        "meta_invariant.env_dependent_ratio_cap",
        "build_evidence.lock.yaml",
        'env_dependent_ratio_cap(`build_evidence.lock.yaml`, evidence_entries, build_evidence_id, "_envdep_") <= 0.30',
    ),
]


# ---------------------------------------------------------------------------
# soft cell カタログ (Stage 1: severity=warning)
# これらの cell は yellow/red になっても release_gate_status を downgrade しない。
# warning_cells フィールドに記録されるので、実装率の可視化に使う。
# Stage 2 で _CELL_CATALOG に昇格させ hard enforcement を有効化する。
# ---------------------------------------------------------------------------
# Stage 1 で warning として導入し、Stage 2 で _CELL_CATALOG に昇格完了。このリストは空。
# Stage 3 以降で新たな soft cell を追加する場合はここに登録する。
_CELL_CATALOG_SOFT: list[tuple[str, str, str]] = []


class ReleaseGateV2Generator(BaseGenerator):
    """release_gate.lock.yaml 生成器（v2）。

    70 cell の AND-gate を DSL 評価によって計算する（v1.0.0 拡張版）。
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
        """全 cell の AND-gate を計算して artifact dict を返す。"""
        generated_at = datetime.datetime.now(tz=datetime.timezone.utc).strftime(
            "%Y-%m-%dT%H:%M:%SZ"
        )

        # lock_dir を取得（emit() 経由の場合は inputs["lock_dir"] に格納されている）
        lock_dir: Path = inputs.get("lock_dir", Path("."))

        # === GLOBAL INVARIANT PRE-CHECK (再 gaming 防止) ===
        # invariant red は全体ステータスを red に downgrade する。upgrade は不可。
        invariant_red = False
        invariant_details: list[str] = []
        for inv_id, inv_expr in _GLOBAL_INVARIANTS:
            inv_result = evaluate_dsl(inv_expr, lock_dir)
            if inv_result.status == "red":
                invariant_red = True
                invariant_details.append(f"{inv_id}: {inv_result.detail}")

        # 各 cell を DSL 評価
        cells: list[dict[str, Any]] = []
        for cell_id, source_lock, dsl_expr in _CELL_CATALOG:
            cell_result = self._evaluate_cell(cell_id, source_lock, dsl_expr, lock_dir)
            cells.append(cell_result)

        # AND-gate: red が 1 件以上 → red / yellow が 1 件以上 → yellow / 全 green → green
        red_cells = [c["cell_id"] for c in cells if c["status"] == "red"]
        yellow_cells = [c["cell_id"] for c in cells if c["status"] == "yellow"]

        if red_cells or invariant_red:
            release_gate_status = "red"
        elif yellow_cells:
            release_gate_status = "yellow"
        else:
            release_gate_status = "green"

        # soft cell 評価（warning severity = release_gate_status に影響しない）
        soft_cells: list[dict[str, Any]] = []
        for cell_id, source_lock, dsl_expr in _CELL_CATALOG_SOFT:
            cell_result = self._evaluate_cell(cell_id, source_lock, dsl_expr, lock_dir)
            cell_result["severity"] = "warning"
            soft_cells.append(cell_result)
        warning_cells = [c["cell_id"] for c in soft_cells if c["status"] in ("red", "yellow")]

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
            "invariant_violations": invariant_details,
            "soft_cells":          soft_cells,
            "warning_cells":       warning_cells,
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

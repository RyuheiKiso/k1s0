---
id: plan.meta.release_gate_phase_mapping
axis: meta
phase: plan
kind: plan_doc
status: draft
depends_on:
  - plan.overview.implementation_phase_definition
  - detail.formal.release_gate_system
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# release_gate 対応表

## 一文方針
- release_gate.lock.yaml の 20 cell と 11 実装 Phase の 1:1 対応を単一の真として保持し、各 Phase の完了判定を機械評価可能にする。

## 20 cell ↔ Phase 対応表

| cell_id | source_lock_artifact | green 化 Phase | green 条件（要約） |
|---|---|---|---|
| `meta.docs_lint_green` | `docs_lint.lock.yaml` | P2 | docs lint 8 check 全 green |
| `meta.repository_layout_integrity` | `repository_layout.lock.yaml` | P2 | src/ 構造 allowlist 完全一致 |
| `meta.axis_registry_complete` | `axis_registry.lock.yaml` | P1 | 19 軸 entry / cap=20 / 残 1 |
| `meta.release_gate_dual_signoff_complete` | `release_gate.lock.yaml` | P11 | cosign dual reviewer 完備 |
| `infra.topology_class_drill_green` | `topology_class.lock.yaml` | P8 | 5 topology_class 全 drill green |
| `infra.clock_integrity_drill_green` | `clock_causality_proof.lock.yaml` | P8 | PTP/chrony/HLC drill cadence 内 |
| `data.preservation_class_drill_green` | `preservation_substrates.lock.yaml` | P9 | 5 preservation_class 全 drill green |
| `tier1.bidi_conformance_complete` | `capabilities.lock.yaml` | P10a | 20 軸 bidi 全 conformance class verified |
| `tier1.slo_compliance_quarterly_green` | `instruments.lock.yaml` | P10a | 13 SLO 全 burn rate 越えなし |
| `tier1.oss_lifecycle_drill_green` | `oss_inventory.lock.yaml` | P10a | 全 v1_l1plus_primary OSS drill green |
| `tier2.tenant_isolation_drill_green` | `migration.lock.yaml` | P10b | RLS / information flow drill green |
| `tier3.client_state_conformance_complete` | `conflict_tree.lock.yaml` | P10c | 4-state machine drill green |
| `security.threat_model_coverage_100pct` | `threat_model.lock.yaml` | P10d | 全 threat に mitigation bind 済み |
| `security.build_provenance_slsa_l3plus` | `artifact_inventory.lock.yaml` | P10d | SLSA L3+ chain 完備 |
| `ops.loop_closure_complete` | `ops_loop.lock.yaml` | P10e | 5 signal_class × 5 phase 全 closure |
| `client.sdk_distribution_5class_green` | `sdk_conformance.lock.yaml` | P10f | 5 distribution_class 全 pact green |
| `test.coverage_matrix_complete` | `coverage_matrix.lock.yaml` | P11 | 18 axis × 5 class = 90 cell 全 green |
| `formal.all_critical_verified` | `proof_status.lock.yaml` | P11 | 95 cell 全 verified or accepted |
| `formal.proof_matrix_complete` | `proof_matrix.lock.yaml` | P11 | 全 cell ∈ {verified, accepted, unverified_handled} |
| `formal.dual_review_completeness_100pct` | `proof_review.lock.yaml` | P11 | 全 obligation の dual_signoff_complete=true |

## Phase 別 green 化 cell 数（累積）

| Phase | green 化 cell | 累積 green | 残 red |
|---|---|---|---|
| P0 Tooling | 0（基盤整備） | 0 | 20 |
| P1 Meta | 1（axis_registry） | 1 | 19 |
| P2 B 層 lint | 2（docs_lint + layout） | 3 | 17 |
| P3 物理 root | 0（build 通過のみ） | 3 | 17 |
| P4 test scaffold | 0（placeholder） | 3 | 17 |
| P5 Formal 前置 | 0（proof_matrix は P11） | 3 | 17 |
| P6 物理 enforcement | 0（cell は評価器側） | 3 | 17 |
| P7 crosscutting | 0（enforcement.lock 各軸参照） | 3 | 17 |
| P8 infra | 2（topology + clock） | 5 | 15 |
| P9 data | 1（preservation） | 6 | 14 |
| P10a tier1 | 3（bidi + slo + oss） | 9 | 11 |
| P10b tier2 | 1（tenant isolation） | 10 | 10 |
| P10c tier3 | 1（client state） | 11 | 9 |
| P10d security | 2（threat + provenance） | 13 | 7 |
| P10e ops | 1（ops loop） | 14 | 6 |
| P10f client | 1（sdk 5class） | 15 | 5 |
| P11 quality | 5（coverage + formal 3 + meta signoff） | **20** | **0** |

## 関連参照
- [実装 Phase 定義](01_実装Phase定義.md)
- [release_gate 体系](../../04_詳細設計/05_lock_yaml体系/03_release_gate体系.md)
- [generate_release_gate.py](../../../tools/lock_yaml_generator/generate_release_gate.py)

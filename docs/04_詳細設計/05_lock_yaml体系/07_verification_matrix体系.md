---
id: detail.test.verification_matrix_system
axis: test
phase: detail
kind: detail
status: draft
depends_on:
  - detail.test.verification_conformance
  - detail.formal.proof_matrix_system
  - detail.meta.axis_registry_conformance
covered_by:
  defense_in_depth_layers: [A, B, C]
  proof_classes: []
lock_artifacts:
  - coverage_matrix.lock.yaml
  - regression_corpus.lock.yaml
---

# verification_matrix 体系

## 一文方針

18 軸（19 軸から formal 軸を除く）× 5 verification_class = 90 cell の coverage_matrix.lock.yaml を SoT として、全 drill の green/red/pending 状態を machine-readable に管理し、release_gate `test.coverage_matrix_complete` cell への input とする。

## cell 数

| 指標 | 値 | 説明 |
|---|---|---|
| matrix cell 数 | **90** | 18 axis × 5 verification_class |
| 対象軸 | 18 | 19 軸から formal 軸を除く（formal は proof_matrix が担当） |
| verification_class | 5 | v1_type_invariant / v1_property_axiom / v1_contract_pair / v1_scenario_replay / v1_fault_chaos |

## verification_class enum（test 軸専用）

| value | 意味 |
|---|---|
| `v1_type_invariant` | 型システムで静的に証明される不変量 |
| `v1_property_axiom` | corpus での確率的 property 検証（Hypothesis 等） |
| `v1_contract_pair` | 呼び出し元と呼び出し先のペア contract |
| `v1_scenario_replay` | シナリオ再生による regression |
| `v1_fault_chaos` | fault injection / chaos engineering |

**注意**: `v1_property_axiom` は test 軸の verification_class。formal 軸の proof_class `v1_program_correctness_proof` と混同しないこと。

## drill_state enum

| value | 意味 |
|---|---|
| `v1_baseline_green` | 基準通過（green） |
| `v1_drill_red_handled` | drill 失敗だが明示的に対処済み |
| `v1_pending_with_artifact` | drill 未実施だが artifact 生成済み |

## coverage_matrix.lock.yaml スキーマ

```yaml
schema_version: v1
axis_count: 18
verification_class_count: 5
total_cells: 90
cells:
  - axis: tier1
    verification_class: v1_type_invariant
    drill_state: v1_baseline_green | v1_drill_red_handled | v1_pending_with_artifact
    regression_corpus_ref: <regression_corpus.lock.yaml の entry id>
    last_drill_at: "ISO8601"
  # ... 90 rows
```

## regression_corpus.lock.yaml との bind

`regression_corpus.lock.yaml` は各 cell の drill 証拠（artifact path / sha256 / tool）を保持する。`coverage_matrix.lock.yaml` の `regression_corpus_ref` field が entry id で bind する。

## release_gate との接続

`test.coverage_matrix_complete` cell: `all(coverage_matrix.lock.yaml, cells[*], drill_state != v1_pending_without_artifact)`
`test.regression_corpus_drift_zero`: `fresh_within(regression_corpus.lock.yaml, last_updated, 7d)`

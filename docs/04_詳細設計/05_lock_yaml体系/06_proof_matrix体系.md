---
id: detail.formal.proof_matrix_system
axis: formal
phase: detail
kind: detail
status: draft
depends_on:
  - detail.formal.formal_conformance
  - detail.meta.axis_registry_conformance
covered_by:
  defense_in_depth_layers: [A, B, F]
  proof_classes:
    - v1_temporal_safety_proof
    - v1_temporal_liveness_proof
    - v1_refinement_proof
    - v1_program_correctness_proof
    - v1_runtime_modelcheck_proof
lock_artifacts:
  - proof_matrix.lock.yaml
  - proof_status.lock.yaml
  - proof_inventory.lock.yaml
  - proof_review.lock.yaml
trace:
  fr_ids:
  - FR-formal-006

---

# proof_matrix 体系

## 一文方針

19 軸 × 5 proof_class = 95 cell の proof_matrix.lock.yaml を SoT として、全 obligation の verified/accepted_with_assumption 状態を machine-readable に管理し、release_gate `formal.all_critical_verified` cell への input とする。

## cell 数

| 指標 | 値 | 説明 |
|---|---|---|
| matrix cell 数 | **95** | 19 axis × 5 proof_class（座標空間） |
| obligation 件数 | **99** | 各 cell に 1:N で obligation が集約される（1 cell に複数 obligation 許容） |
| P5 先行 2 cell | 2 | tier1 transport TLA+ + data Stainless invariant |
| P11 残余 cell | 93 | P5 2 cell を除く全 cell |

## cell_state enum

| value | 意味 |
|---|---|
| `v1_baseline_verified` | 正式に verified（proof certificate あり） |
| `v1_accepted_with_assumption` | assumption 付き受理（assumption.lock.yaml で TTL 管理） |
| `v1_unverified_handled` | 未証明だが明示的に対処済み（scope_narrowed 等） |
| `v1_unverified_unhandled` | 未証明・未処理（red 扱い） |

## close_kind enum（obligation 解消種別）

| value | 意味 |
|---|---|
| `fixed_in_code` | 実装修正で解消 |
| `fixed_in_spec` | spec 修正で解消（scope 縮小等） |
| `accepted_as_bug` | bug として受理（cap=10 内） |
| `scope_narrowed` | scope を明示的に縮小して解消 |

## proof_matrix.lock.yaml スキーマ

```yaml
schema_version: v1
axis_count: 19
proof_class_count: 5
total_cells: 95
cells:
  - axis: tier1
    proof_class: v1_temporal_safety_proof
    cell_state: v1_baseline_verified | v1_accepted_with_assumption | v1_unverified_handled | v1_unverified_unhandled
    obligation_ids: [<proof_inventory.lock.yaml の obligation id>]
    last_evaluated_at: "ISO8601"
  # ... 95 rows
```

## release_gate.formal.all_critical_verified との接続

`release_gate.lock.yaml` の `formal.all_critical_verified` cell は以下 DSL で評価:

```
all(proof_matrix.lock.yaml, cells[*], cell_state != v1_unverified_unhandled)
AND count(proof_matrix.lock.yaml, cells[?cell_state=='v1_accepted_with_assumption']) <= assumption_cap
AND assumption_ttl_valid(all_assumptions, today)
```

## 関連参照

- assumption 体系 → `docs/04_詳細設計/05_lock_yaml体系/08_assumption体系.md`
- verification_matrix (test 軸) → `docs/04_詳細設計/05_lock_yaml体系/07_verification_matrix体系.md`
- proof_artifact 体系 → `docs/04_詳細設計/05_lock_yaml体系/01_proof_artifact体系.md`

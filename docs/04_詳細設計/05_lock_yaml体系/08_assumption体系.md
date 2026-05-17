---
id: detail.formal.assumption_system
axis: formal
phase: detail
kind: detail
status: draft
depends_on:
  - detail.formal.proof_matrix_system
  - detail.formal.counter_example_system
covered_by:
  defense_in_depth_layers: [A, B, F]
  proof_classes: []
lock_artifacts:
  - assumption.lock.yaml
---

# assumption 体系

## 一文方針

proof_matrix の `v1_accepted_with_assumption` cell に紐付く assumption を TTL / severity / 軸別 cap で管理し、期限超過または cap 超過を release_gate の fail 条件として物理 enforce する。

## TTL by severity

| severity | revisit_at (assumption 提出から) | 意味 |
|---|---|---|
| `high` | 14 日 | 重大な未証明仮定（security / data 軸の invariant 等） |
| `medium` | 30 日 | 通常の仮定 |
| `low` | 90 日 | 低リスクの仮定（enum exhaustiveness 等） |

TTL 超過 assumption の cell は `cell_state: v1_unverified_unhandled` 扱いになり、release_gate が red になる。

## 軸別 cap

| axis | assumption cap |
|---|---|
| tier1 | 5 |
| tier2 | 3 |
| tier3 | 3 |
| infra | 5 |
| data | 3 |
| security | 2 |
| ops | 3 |
| client | 3 |
| test | 5 |
| formal | 3 |
| cross_* (各 cluster) | 2 |
| meta | 1 |
| 合計上限 | 20 |

## assumption.lock.yaml スキーマ

```yaml
schema_version: v1
total_cap: 20
entries:
  - assumption_id: "a.<axis>.<slug>"
    axis: tier1
    severity: high | medium | low
    submitted_at: "ISO8601"
    revisit_at: "ISO8601"
    proof_matrix_cell:
      axis: tier1
      proof_class: v1_temporal_safety_proof
    mitigation_pointer: "<doc path or PR url>"
    cosign_required_on_close: true | false
    status: open | closed
    close_kind: fixed_in_code | fixed_in_spec | scope_narrowed | null
```

## release_gate との接続

- `formal.assumption_cap_within_20`: `count(assumption.lock.yaml, entries[?status=='open']) <= 20`
- `formal.no_open_above_severity_low`: `all(assumption.lock.yaml, entries[?status=='open'], severity != 'high' AND severity != 'medium')` は黄 band で管理（今後厳格化）
- assumption TTL 越えは DSL `assumption_ttl_valid(id, today)` で評価し、期限切れ entry が 1 件でも → red

## close 時の dual signoff

- `cosign_required_on_close: true` の assumption を close する際、`dual_review.lock.yaml` での human + (ai_evidence | cosign_history) の dual sign が必要
- security / data 軸の high severity assumption は全て `cosign_required_on_close: true`

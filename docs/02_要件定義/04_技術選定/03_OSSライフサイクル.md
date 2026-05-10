---
id: req.tech.oss_lifecycle
axis: overview
phase: requirement
kind: requirement
status: draft
depends_on:
  - req.tech.tech_index
  - detail.tier1.oss_lifecycle_conformance
covered_by:
  defense_in_depth_layers: [A, B, C, D, E]
  proof_classes: []
---

# OSS ライフサイクル要件

## 一文方針
- 6 lifecycle_class × 4 dimension（adoption_status / health_check_cadence / migration_trigger_set / response_action）の bundle で全採用 OSS の lifecycle を機械可読に固定。8 signal で移行 trigger を発火、L1+ 4 primary pair の dry-run green を年次で維持する。

## 6 lifecycle_class
- `v1_l1plus_primary` / `v1_l1plus_pair_target` / `v1_l2star_member` / `v1_l3_runtime` / `v1_reserved_category` / `v1_inhouse_authoritative`

## 8 signal
- `license_change` / `eol_announced` / `cve_backlog_threshold` / `maintainer_turnover` / `fork_event` / `conformance_drift` / `major_up` / `spec_drift`

## 4 primary pair
- `relational_pg_pair`: CloudNativePG → StackGres
- `messaging_kafka_pair`: Apache Kafka → RedPanda
- `workflow_pair`: Temporal → Cadence
- `rule_engine_pair`: ZEN Engine → 自製 DSL backend

## 受入条件
- 6 lifecycle_class × 全 OSS instance の完全 conformance green
- `oss_inventory.lock.yaml` と 02_採用 OSS 一覧 の双方向整合
- 4 primary pair の `dry_run.lock.yaml` の `last_green_at` が 365 日以内
- v1_inhouse_authoritative 全 entry の inhouse_spec_memo 存在

## 関連参照
- [技術選定 index](README.md)
- [OSS 採用一覧](01_OSS採用一覧.md)
- [OSS ライフサイクル適合仕様](../../04_詳細設計/01_適合仕様/08_OSSライフサイクル適合仕様.md)
- [移行 Pair 適合仕様](../../04_詳細設計/01_適合仕様/02_移行Pair適合仕様.md)

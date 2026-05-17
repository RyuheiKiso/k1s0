---
id: detail.cross_slo.slo_protection_layers
axis: cross_slo
phase: cross_cutting
kind: cross_cut_spec
status: draft
depends_on:
  - arch.tier2.service_operation_policy
  - detail.tier2.tenant_isolation_conformance
  - detail.tier1.slo_conformance
  - detail.tier1.tenant_capacity_conformance
  - detail.ops.ops_loop_conformance
  - detail.meta.axis_registry_conformance
covered_by:
  defense_in_depth_layers: [B, C, D, E]
  proof_classes:
    - v1_temporal_safety_proof
    - v1_program_correctness_proof
related_axes:
  - tier1
  - tier2
  - ops
---

# 共有 Pod / DB の SLO 保護四層 + 自動昇格 trigger（v1）

## 位置づけ
- 07_Service 運用方針 / 27_信頼性運用準備 / 32_テナント分離適合仕様 / 11_ops/15_運用ループ適合仕様 と双方向 lock
- 共有 Pod / 共有 DB 上で multi-tenant を運用する際、テナント間の SLO 違反波及を「保証」ではなく「保護四層 + 自動昇格 trigger」で構造化

## 不可避性
- Linux cgroup / PostgreSQL shared buffer / Kafka broker IO は noisy neighbor を完全には isolate できない（page cache 競合 / WAL fsync 競合 / 同一 broker への書込集中）
- rate limiter は受付段の制御に過ぎず、tier1 backend 内の競合波及を完全停止できない。「波及なし担保」は技術的に強すぎる
- 「波及確率を SLO error-budget の閾値以下に抑える」+ 「閾値を超えたら専用 Pod / 専用 DB に自動昇格する」の二段で honest に表現

## 保護四層

### 層 SLO-A: per-tenant rate limiter（受付段）
- tier1 Gateway（Envoy Gateway）の Local Rate Limit filter で per-tenant RPS / 同時 connection / token bucket を制御
- 値は tier3 manifest の resource budget セクションから導出、Backstage Software Template が生成

### 層 SLO-B: cgroup quota（CPU / memory）
- 共有 Pod 上の per-tenant worker（spawn-per-request 不可、per-tenant goroutine pool / actor）に cgroup v2 `cpu.max` / `memory.max` を適用
- tier2 Service の per-tenant worker は systemd unit slice または k8s sidecar として cgroup を分離
- 値は tier3 manifest の `cpu_quota_ms` / `memory_mib` から導出

### 層 SLO-C: PG pool tenant 分離（pgbouncer multi-pool）
- PgBouncer を per-tenant pool で運用（pool 名 = `tenant_id`）。pool ごとに `max_client_conn` / `default_pool_size` を設定
- 1 tenant の connection burst が他 tenant の pool に波及しない構造（pool 物理分離）
- PgBouncer が tenant 数 N 個の pool を抱える際の connection 数爆発は、1 PgBouncer instance を tenant sharding（`hash(tenant_id) % shard`）で分散

### 層 SLO-D: Kafka quota（per-tenant produce/fetch quota）
- Kafka の Quota（`client_id` quota）を per-tenant に設定。`kafka_principal` を `tenant_id` で命名し、`producer_byte_rate` / `consumer_byte_rate` を tier3 manifest から導出
- quota violation は producer/consumer に backpressure として返却され、他 tenant の broker IO に波及しにくい

## 自動昇格 trigger
共有 Pod / DB で per-tenant SLO 違反が閾値を超えたら、当該 tenant を「専用 Pod / 専用 DB」に自動 promote。

### 検知指標
- `tenant_request_latency_p99_ms ÷ tier3_slo_p99 > 1.5` が連続 5 min
- `tenant_error_rate > tier3_slo_error_budget × 30% / day`
- `tenant_kafka_quota_violation_rate > 0`
- PG pool exhaustion event（per-tenant pool full）連続 3 回 / 30 min

### promote 経路
- Argo Workflow が新規 dedicated Pod / dedicated PG cluster を Provision（CloudNativePG Cluster CR / k8s Deployment YAML を Backstage template 経由で生成）
- tenant traffic を Envoy Gateway の routing で新 Pod に切替（gradual shift、Argo Rollouts）
- 旧共有 Pod 上の当該 tenant data は logical replication + dual-write で eventual consistency

### hysteresis
- promote 後 30 days は demote 不可（min-promotion-window）
- 同 tenant の promote / demote は 90 days で最大 2 回まで（flapping 抑止）

### blast radius
- promote/demote は当該 tenant のみに影響、他 tenant の SLO に影響しないことを Chaos drill で確認

## honest な SLO 表現
「SLO 違反波及なし担保」を以下に置換:
- 共有運用での「テナント間 SLO 違反波及確率を error-budget の 5% 以下に抑える」を SLO 化
- 抑制超過時は自動昇格 trigger が 5 min 以内に専用 Pod / 専用 DB へ promote する旨を SLA 文書に明記
- 「保証」「担保」表現は本ファイル + 関連箇所から削除し、「抑制 + 自動昇格」表現で統一

## 副作用
- 自動昇格 trigger の閾値振動（flapping）対策として hysteresis を必須化
- PG pool tenant 分離は connection 数が tenant 数 × pool size で爆発するため、PgBouncer multi-pool + tenant sharding を必須化。1 PgBouncer instance あたり tenant 数上限を 64 とする運用 prerequisite
- dedicated Pod / DB 数の増加で運用 toil が増える。Backstage + ArgoCD の自動化前提（手動 Provision 禁止）

## 整合
- 整合 1: 07_Service 運用方針 の「per-tenant rate limiter で抑制」は本仕様の保護四層 + 自動昇格に置換。「SLO 違反波及なし担保」は撤回
- 整合 2: 27_信頼性運用準備 の SLO 表現を「波及確率 5% 以下 + 自動昇格 5 min 以内」に変更
- 整合 3: 32_テナント分離適合仕様 の共有形態節は本仕様の保護四層を参照
- 整合 4: 11_ops/15_運用ループ適合仕様 SLO に `tenant_promotion_event_total` / `hysteresis_violation_total` を追加
- 整合 5: 19_検証規律適合仕様 の Chaos drill に「noisy-neighbor scenario で 5% 以下抑制 + 5 min 自動昇格」を追加

## 関連参照
- [Service 運用方針](../../03_概要設計/03_tier2設計方針/07_Service運用方針.md)
- [テナント分離適合仕様](../01_適合仕様/10_テナント分離適合仕様.md)
- [SLO 適合仕様](../01_適合仕様/07_SLO適合仕様.md)
- [テナント容量適合仕様](../01_適合仕様/09_テナント容量適合仕様.md)
- [運用ループ適合仕様](../01_適合仕様/17_運用ループ適合仕様.md)

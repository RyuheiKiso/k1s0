---
id: plan.data.scenario_dr_cross_region_failover
axis: data
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.data.data_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [C, D]
  proof_classes: []
---

# DR cross-region 実 failover

## 一文方針

region 喪失インシデント発生時に、drill（シナリオ 03）とは異なる非計画的 DR failover を data 担当者が主導して実行し、`v1_cross_region_replicated` の restore_window（5 分）以内に secondary region でのサービス再開と RTO 計測・audit 記録を完結させる。

## Trigger（発火条件）

primary region の可用性が失われ、ops 担当者からの DR failover 宣言が発令された時（CloudNativePG の primary cluster が 5 分以上到達不能 / Kafka primary broker が quorum 喪失 / Istio の region ヘルスチェックが primary 全インスタンスで fail）。

## 想定頻度 / 典型きっかけ

想定頻度: 非計画的（年間 0-1 件。DR drill は定期実施だがこれは実インシデント対応）。典型きっかけ: 「primary region のデータセンター電源障害により PostgreSQL primary cluster と Kafka broker quorum が同時に喪失。`#infra-alert` で ops 担当者が DR 宣言を発令し、secondary region への failover を data 担当者が主導することになった」

## 主役 / 関与者

- 主役: data 担当者（シニア級、on-call 担当者）
- 関与: ops 担当者（DR 宣言の発令 / Mattermost `#dr-failover` の incident channel 管理）
- 関与: infra 担当者（Kubernetes / Istio / DNS の traffic rerouting）
- 関与: tier2 担当者（failover 後の business tier サービス再起動確認）
- 関与: security 担当者（failover 後の OpenBao availability 確認）
- 承認: ops 担当者（RTO 計測の sign-off）

## 前提

- `v1_cross_region_replicated` preservation_class で管理されているデータは secondary region に最新の replication 状態で存在している（restore_drill で確認済み）
- CloudNativePG のレプリカ（secondary region に `replica cluster` が deploy 済み）が最新 WAL を受信済み
- Strimzi Kafka の MirrorMaker 2 が secondary region で稼働しており、topic offset が同期済み
- OpenBao が secondary region でも可用（Ha mode / raft レプリカ）
- Argo Rollouts / Istio の failover policy が `traffic-split` manifest として GitOps 管理済み
- `restore_drill.lock.yaml` の直近エントリが green（drill が green でない場合は failover 手順の信頼性が担保されない）

## 流れ

**T+0（DR 宣言受領）**
1. ops 担当者から Mattermost `#dr-failover` で DR 宣言を受領する。即座に on-call data 担当者 2 名が集合する
2. `recovery_start_time` を記録する（RTO 計測の基点）

**T+1〜2 分（PostgreSQL failover）**
3. secondary region の CloudNativePG replica cluster を promoted cluster に昇格させる（`kubectl cnpg promote <cluster-name>`）
   - WAL 受信の完了を確認する（`pg_last_wal_receive_lsn() = pg_last_wal_replay_lsn()`）
   - 昇格完了を Perses の `cnpg-primary-role` パネルで確認する

**T+2〜3 分（Kafka failover）**
4. secondary region の Strimzi Kafka を active として設定する（MirrorMaker 2 の active-passive 切替）
   - `kafka_consumer_group_offsets` が secondary に同期済みであることを確認する
   - Apicurio Registry が secondary region で可用であることを確認する

**T+3〜4 分（traffic rerouting）**
5. infra 担当者と協力して Istio の traffic routing を primary → secondary region に切り替える（GitOps で `VirtualService` を更新）
   - DNS の secondary region エンドポイントへの切替を確認する（Argo Rollouts の failover manifest apply）

**T+4〜5 分（サービス再開確認）**
6. tier2 担当者に secondary region でのサービス起動を確認してもらう（Backstage health check）
7. security 担当者に OpenBao（secondary region）が応答していることを確認してもらう

**T+5 分以内（RTO 計測）**
8. `recovery_complete_time` を記録し、RTO = `recovery_complete_time - recovery_start_time` を算出する
   - RTO が 5 分を超えた場合は即時 ship blocker 認定（`v1_cross_region_replicated` の restore_window 違反）
9. DR failover の操作全体が audit hash chain に記録されていることを確認する
10. `restore_drill.lock.yaml` に実 failover 結果を追記する（failover 時刻 / RTO / 手順逸脱の有無 / 参加者）
11. **postmortem を 3 営業日以内に実施する**（DR 宣言から failover 完了までの timeline を詳細記録）

**primary region 復旧後（failback）**
12. primary region が復旧したら、infra 担当者主導で failback の可否を判断する
    - failback は DR failover の逆操作だが慎重に実施する（secondary で発生した write が primary と diverge していないことを確認）
    - failback は別の PR / sign-off プロセスで管理する

## 関連適合仕様 / 関連 OSS

- データ保全適合仕様: [../../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md](../../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md)
- クラスタ位相適合仕様: [../../../04_詳細設計/01_適合仕様/12_クラスタ位相適合仕様.md](../../../04_詳細設計/01_適合仕様/12_クラスタ位相適合仕様.md)
- 関連 OSS: CloudNativePG（PostgreSQL HA）/ Strimzi Kafka + MirrorMaker 2 / Istio（traffic routing）/ Argo Rollouts（failover policy）/ OpenBao（secret availability）

## 期待結果 / 観測指標

- rto: failover 完了まで ≤ 5 分（`v1_cross_region_replicated` restore_window 以内。Mattermost タイムスタンプで計測）
- artifact: `restore_drill.lock.yaml` に実 failover エントリが追記済み（RTO / 手順逸脱の有無 / 参加者）
- audit: audit hash chain に failover 操作全体が emit 済み（security 担当者確認）
- runtime: secondary region の CloudNativePG primary role が `primary` に昇格（Perses `cnpg-primary-role` パネル）
- runtime: secondary region の Kafka broker が quorum を持つ（Perses `kafka-broker-count` パネル）
- postmortem: 3 営業日以内に postmortem が Backstage TechDocs に公開済み

## 失敗時の挙動 / escalation

- **RTO が 5 分を超過**: 即時 ship blocker 認定。ops 担当者に `#dr-failover` で通報し、1.0.0 ship blocker ticket を Backstage に起票する（**SLA: 超過確認後 30 分以内**）。drill（シナリオ 03）の手順を見直し、次回 drill までに RTO を 5 分以内に改善する。
- **CloudNativePG の promotion が失敗する**: WAL replay が完了していない場合（`pg_last_wal_receive_lsn() != pg_last_wal_replay_lsn()`）は replay 完了まで待機（最大 3 分）。3 分経過しても完了しない場合は infra 担当者と共同で CloudNativePG Operator の状態を確認し、強制 promotion（`--force`）の是非を判断する（**SLA: 5 分以内に判断**）。
- **Kafka MirrorMaker 2 の offset sync が不完全**: Consumer Group のリプレイが必要になる場合がある。tier2 担当者に Mattermost `#dr-failover` で通知し、影響を受ける Domain Event の消費者を確認する（**SLA: 10 分以内**）。冪等性保証（Idempotency-Key）により重複 event は tier2 側で排除される。
- **audit hash chain への emit 失敗**: security / ops 担当者に即時 escalate（**SLA: 1h 以内**）。Backstage runbook `audit-chain-integrity-check` を参照。failover 完了の宣言はaudit emit が保証されるまで保留する。
- **failback 判断での data diverge 検出**: infra 担当者 + data 担当者 + tier2 担当者でアーキテクト / tier1 担当者に設計レビューを escalate（**SLA: 48h 以内**）。Mattermost `#dr-arch-review` で相談する。

## 関連参照

- [data 担当者シナリオ index](./README.md) — data 担当者シナリオ全体の構成と 5 preservation_class 一覧
- [restore drill](./03_restore_drill.md) — 定期実施の DR drill シナリオ（本シナリオの訓練版）
- [replication lag / split-brain 対応](./06_replication_lag_split_brain対応.md) — failover 前の replication 状態確認と split-brain 検出手順
- [cluster topology drill（infra-02）](../04_infra担当者シナリオ/02_topology_drill.md) — infra 側からの DR topology 検証（data failover と組み合わせる）
- [データ保全適合仕様](../../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md) — `v1_cross_region_replicated` の restore_window と drill_cadence の正典定義

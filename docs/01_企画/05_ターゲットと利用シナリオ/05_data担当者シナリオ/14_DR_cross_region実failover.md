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

> 深夜 1 時、自宅 on-call 中の data 担当者（シニア級）が PagerDuty のアラートで起床し、Mattermost `#dr-failover` で ops 担当者の DR 宣言メッセージを確認する。CloudNativePG primary cluster が 5 分以上到達不能になっていることを Perses の `cnpg-primary-role` パネルで確認する。手元にはノート PC の `kubectl` 端末と Perses、Mattermost 越しに ops 担当者・infra 担当者・tier2 担当者・security 担当者がいる。

## ペルソナ要約

主役: data 担当者（シニア級）、目的: DR cross-region 実 failover を RTO 5 分以内で完結させ audit trail を完全記録する

## 現状業務での痛み

- DR failover を実際に試していないため、手順の格差が実 failover 時に長時間停止を引き起こす
- failover 後のデータ整合性確認手順が不明確で、復旧後にデータ不整合が発覚するリスクがある
- failover の audit trail が残らず、規制当局への証跡提出が困難

## k1s0 でこう変わる

- restore_drill.lock.yaml が DR failover 手順を管理し、実 failover と drill の手順が同一品質に維持される
- CloudNativePG の promote と Kafka MirrorMaker 2 切替が手順化され、5 分以内の RTO が継続的に確認される
- failover 操作全体が audit hash chain に記録され、規制当局への証跡提出が即時に可能になる

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

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（data）| シニア | 自宅 on-call | Perses（`cnpg-primary-role`）/ PagerDuty / Mattermost `#dr-failover` | PostgreSQL / Kafka failover 主導・RTO 計測・audit 記録・restore_drill.lock.yaml 更新 |
| 関与（ops）| シニア | 自宅 on-call / 本社 | Mattermost `#dr-failover` | DR 宣言発令・incident channel 管理・RTO sign-off |
| 関与（infra）| シニア | 自宅 on-call / 本社 IT 室 | Argo CD / Istio / Kyverno | Kubernetes / Istio / DNS の traffic rerouting |
| 関与（tier2）| ミドル〜シニア | リモート | Backstage TechDocs | failover 後のサービス再起動確認 |
| 関与（security）| シニア | リモート | OpenBao 管理画面 | failover 後の OpenBao availability 確認 |

## 個人 KPI / 達成感

- RTO ≤ 5 分の達成を restore_drill.lock.yaml で定量確認でき、DR 対応能力の達成感を得られる
- failover 後の audit trail 完全性を確認でき、compliance 品質の向上を実感できる

## 工数 / 関与人数 / コスト感

- 工数: 1〜2 時間/failover（on-call 対応のため工数は最小化が必須）
- 関与人数: 5〜6 名（data・ops・infra・tier2・security 担当者 + dual reviewer）
- コスト感: 中〜高（実インシデント対応のため工数は変動）。drill により継続的に削減される

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

## 業界 9 業務との紐付け

全 9 業務に共通基盤として影響（data は全業務の PostgreSQL / Kafka / ClickHouse の永続化基盤を担うため）。特に影響度が高い 2 業務:

- **警報配信**: RTO 5 分以内の failover 完了が製造ライン安全管理の警報配信継続に直結し、primary region 喪失時でも secondary region での警報 emit が途切れないことが最重要要件となる。
- **FA（ファクトリーオートメーション）**: FA 制御データの write 経路が failover によって secondary に切り替わる際、WAL replay 完了確認と PostgreSQL promotion の RTO が FA の稼働継続に直接影響する。

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

## Timeline

| T+ | actor | action | Mattermost 投稿例 |
|---|---|---|---|
| 0 | data 担当者 | PagerDuty で起床。Mattermost `#dr-failover` で DR 宣言を確認し `recovery_start_time` を記録 | `@data-oncall DR 宣言受領 / 対象: primary cluster 全停 / T+0 01:03` |
| 2 分 | data 担当者 | secondary region の CloudNativePG replica cluster を `kubectl cnpg promote` で昇格。WAL replay 完了を確認 | `cnpg promote 完了 / pg_last_wal_receive_lsn = pg_last_wal_replay_lsn 確認済` |
| 3 分 | data 担当者 | Strimzi Kafka の MirrorMaker 2 を active-passive 切替。consumer_group_offsets sync を確認 | `Kafka failover 完了 / MirrorMaker2 active: secondary / offset sync 確認済` |
| 4 分 | infra 担当者 | Istio VirtualService を GitOps で更新し traffic を secondary region へ rerouting。DNS 切替完了を確認 | `pod 状態確認完了 / VirtualService apply 済 / DNS secondary 切替確認` |
| 5 分 | data 担当者 | `recovery_complete_time` を記録し RTO を算出。audit hash chain への failover 操作 emit を確認 | `RTO = X 分 / v1_cross_region_replicated restore_window 以内 / audit emit 確認済` |
| 1 営業日 | data 担当者 | postmortem 着手（DR 宣言から failover 完了までの timeline を詳細記録） | `#postmortem postmortem PR 作成済 / Backstage TechDocs 公開予定: 3 営業日以内` |

## 失敗パターン (anti-pattern)

- drill なし実 failover: restore_drill.lock.yaml が green でない状態での failover は手順の欠陥が実インシデントで発覚する
- audit trail なしの failover 完了宣言: audit emit 確認前の完了宣言は compliance 違反になる

## 関連参照

- [data 担当者シナリオ index](./README.md) — data 担当者シナリオ全体の構成と 5 preservation_class 一覧
- [restore drill](./03_restore_drill.md) — 定期実施の DR drill シナリオ（本シナリオの訓練版）
- [replication lag / split-brain 対応](./06_replication_lag_split_brain対応.md) — failover 前の replication 状態確認と split-brain 検出手順
- [cluster topology drill（infra-02）](../04_infra担当者シナリオ/02_topology_drill_failover.md) — infra 側からの DR topology 検証（data failover と組み合わせる）
- [データ保全適合仕様](../../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md) — `v1_cross_region_replicated` の restore_window と drill_cadence の正典定義

---
id: plan.data.scenario_replication_lag_split_brain
axis: data
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.data.data_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# replication lag / split-brain 対応

## 一文方針

CloudNativePG / Kafka / Valkey の replication lag アラートまたは split-brain 疑いを症状別に分類し、data 整合性確認と恒久対処 PR まで完結させる。

## Trigger（発火条件）

CloudNativePG / Kafka / Valkey の replication lag SLI アラートが発火した時、またはネットワーク分断で split-brain の疑いが生じた時。

## 想定頻度 / 典型きっかけ

想定頻度: イベント駆動（Perses alert 発火時）。典型きっかけ: 「CloudNativePG secondary の replication lag が 30 秒を超え SLI alert が発火した。ネットワーク帯域の突然の逼迫が原因と疑われた」

## 主役 / 関与者

- 主役: data 担当者（シニア級）
- 関与: infra 担当者（network 分断・CRUSH map・IaC 修正）
- 関与: tier2 担当者（Outbox relay の二重送信確認 / integration test 実行）
- 関与: dual reviewer（恒久対処 PR の sign-off）

## 前提

- CloudNativePG / Strimzi（Kafka）/ Valkey の replication lag SLI がモニタリングダッシュボードで可視化済み
- etcd quorum が CloudNativePG の leader election の基盤として稼働済み
- KEDA による自動スケールが設定済み
- [Outbox relay](../../../03_概要設計/02_tier1設計方針/01_Server系.md)（tier1 Sidecar コンポーネント）が Kafka に配送する構成が稼働済み

## 流れ

1. 症状を分類する: **replication lag**（遅延が閾値を超えている）/ **split-brain**（両系統が primary と認識している）/ **leader election 失敗**（primary が選出されない）
2. **replication lag の場合**: 原因を特定する（network 帯域不足 / secondary の I/O ボトルネック / バックプレッシャー）→ KEDA でスケールするか、I/O 設定を調整する
3. **CloudNativePG split-brain の場合**: pod label を確認し、複数 pod が `primary=true` を持っていないかチェックする → 異常な primary を demote する（etcd quorum の状態を先に確認する）
4. **Kafka split-brain の場合**: Strimzi の ZooKeeper / KRaft quorum を確認し、outlier broker を fence する
5. 復旧後、data 整合性を tier2 integration test で確認する。特に Outbox relay が二重送信していないかを確認する
6. 根本原因を特定し、IaC / network 設定 / CRUSH map を修正した恒久対処 PR を作成する
7. 可用性 SLO 違反が発生した場合は **postmortem 期限: 2 営業日以内**（Backstage runbook `data-postmortem-template`）。dual reviewer sign-off を得る

## 関連適合仕様 / 関連 OSS

- データ保全適合仕様: [../../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md](../../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md)
- 関連 OSS: CloudNativePG / Strimzi / Valkey / etcd / KEDA

## 期待結果 / 観測指標

- replication lag が SLI 閾値内に収まっている
- split-brain が解消し、primary が 1 系統のみであることが pod label / quorum ログで確認できる
- Outbox relay の二重送信が発生していないことが consumer の de-dup ログで確認できる
- 恒久対処 PR がマージ済み
- dual reviewer 2 名の sign-off が記録済み

## 失敗時の挙動 / escalation

- **split-brain 確認後**: data 担当者が単独で demote 操作を行ってはならない。必ず ops 担当者と dual review した上で Backstage runbook `pg-split-brain-recovery` に従う（SLA: 操作開始 30 分以内に dual confirmation）。
- **split-brain 解消不能**: infra 担当者に escalate し、network レベルの強制分断を実施する。data 損失が発生した場合は最新の backup から restore を検討する
- **etcd quorum 喪失**: infra 担当者が etcd cluster を復旧するまで CloudNativePG の leader election は機能しない。read-only モードで業務継続を検討する
- **Outbox 二重送信が確認された場合**: consumer の Idempotency-Key による de-dup が機能しているかを確認し、機能していない場合は tier2 担当者に escalate する

## 関連参照

- [data 設計方針](../../../03_概要設計/06_data設計方針/README.md) — CloudNativePG / Kafka / Valkey の replication 設計思想と split-brain 防止原則
- [Outbox / atomic 三表書込障害対応シナリオ](08_Outbox_atomic三表書込障害対応.md) — split-brain 復旧後の Outbox 二重送信確認と補償トランザクション手順
- [シナリオ index](README.md) — data 担当者シナリオ全体の構成と preservation_class 一覧

---
id: plan.data.scenario_outbox_atomic_failure
axis: data
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.data.data_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [C]
  proof_classes: []
---

# Outbox / atomic 三表書込障害対応

## 一文方針

[Outbox relay](../../../03_概要設計/02_tier1設計方針/01_Server系.md)（tier1 Sidecar コンポーネント）の障害または [atomic 三表書込](../../../03_概要設計/03_tier2設計方針/13_状態遷移パターン.md)（state / outbox / audit を同一 DB トランザクションで書く）の integrity 違反を症状別に分類し、audit hash chain を証跡として補償トランザクションで整合を回復し、audit 欠落があれば compliance incident として即時 escalate する。

## Trigger（発火条件）

Outbox relay の障害、または atomic 三表書込（state / outbox / audit を同一 DB トランザクション）の integrity 違反が検出された時。

## 想定頻度 / 典型きっかけ

想定頻度: イベント駆動（障害発生時）。典型きっかけ: 「Kafka connectivity が 15 分間断絶し Outbox table に 500 件の未送信 record が滞留。復旧後の replay と audit chain 整合確認が必要になった」

## 主役 / 関与者

- 主役: data 担当者（シニア級）
- 関与: tier2 担当者（Outbox relay の Sidecar 実装確認 / 補償トランザクションの実装）
- 関与: security / ops 担当者（audit 欠落時の compliance incident 対応）
- 関与: dual reviewer（恒久対処 PR の sign-off）

## 前提

- [atomic 三表書込](../../../03_概要設計/03_tier2設計方針/13_状態遷移パターン.md)（`state` / `outbox` / `audit` を同一 DB トランザクション）が tier2 の設計原則として確立済み
- [Outbox relay](../../../03_概要設計/02_tier1設計方針/01_Server系.md)（tier1 Sidecar コンポーネント）が Kafka に配送する構成が稼働済み
- audit は audit hash chain に記録される
- Kafka consumer が Idempotency-Key で de-dup する at-least-once 保証が実装済み

## 流れ

1. 症状を分類する: **Outbox table に未送信 record が滞留** / **audit table に対応 record がない** / **state table と outbox table の整合が取れていない**
2. **Outbox 滞留の場合**: Sidecar の relay process を確認する（Kafka への connectivity / credential の期限切れ）→ relay を復旧し再送する
3. **三表整合違反の場合**: どのトランザクションで atomic 書込が分割されたかを audit hash chain から追跡する
4. 分割が見つかった場合: 補償トランザクション（compensating transaction）で `state` / `audit` を一致させる。tier2 担当者が実装する
5. **Kafka 重複配送の場合**: Idempotency-Key で consumer 側が de-dup できていることを確認する。at-least-once 保証の範囲内であれば正常動作とみなす
6. 根本原因を特定し、tier2 のコード / infra 設定の恒久修正 PR を作成する
7. audit chain に欠落があった場合は compliance incident として扱い、**postmortem 期限: 2 営業日以内**（compliance incident の場合は 1 営業日以内）。dual reviewer sign-off を得る

## 関連適合仕様 / 関連 OSS

- tier2 設計方針 08_業務エラー監査コンプライアンス: [../../../03_概要設計/03_tier2設計方針/08_業務エラー監査コンプライアンス.md](../../../03_概要設計/03_tier2設計方針/08_業務エラー監査コンプライアンス.md)
- データ保全適合仕様: [../../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md](../../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md)
- 関連 OSS: CloudNativePG / Kafka（Strimzi）/ Argo Workflows

## 期待結果 / 観測指標

- Outbox table の滞留 record が 0 件であることを確認できる
- `state` / `outbox` / `audit` の三表が整合していることを確認できる
- Kafka consumer の de-dup ログで二重処理が発生していないことを確認できる
- audit hash chain に欠落がないことを確認できる
- 恒久対処 PR がマージ済み
- dual reviewer 2 名の sign-off が記録済み

## 失敗時の挙動 / escalation

- **audit chain 欠落**: compliance incident として扱い、security 担当者 + ops 担当者に Mattermost `#compliance-incident` で即時通報（SLA: 1h 以内）。postmortem は 2 営業日以内。欠落が確認された時点で当該機能を停止し、原因特定まで再開しない。
- **補償トランザクション後も整合が取れない**: data 担当者と tier2 担当者が postmortem を実施し、atomic 三表書込の実装を根本から見直す。
- **relay 復旧後に重複配送が多発する**: consumer の Idempotency-Key の実装を tier2 担当者が確認し、de-dup が機能していない場合は consumer を一時停止して修正する。

## 関連参照

- [data 設計方針](../../../03_概要設計/06_data設計方針/README.md) — atomic 三表書込（state / outbox / audit）の設計原則と Outbox relay の全体方針
- [replication lag / split-brain 対応シナリオ](06_replication_lag_split_brain対応.md) — Outbox 障害と連動する replication 障害の対応手順
- [シナリオ index](README.md) — data 担当者シナリオ全体の構成と preservation_class 一覧

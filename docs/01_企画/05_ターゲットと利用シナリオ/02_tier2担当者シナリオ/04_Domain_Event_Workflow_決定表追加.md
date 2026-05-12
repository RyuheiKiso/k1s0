---
id: plan.tier2.scenario_domain_event_workflow_addition
axis: tier2
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.tier2.tier2_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# Domain Event / Workflow / 決定表追加

## 一文方針

業務フロー変更に伴い新規 Domain Event / Workflow / 決定表を追加する際、Apicurio FULL_TRANSITIVE 互換検査・FSM 状態遷移追加・atomic 三表書込・Outbox relay E2E 検証を全て通過させてから merge する。

> 朝 9 時半、本社 IT 室の tier2 担当者（中堅級）が GitHub PR レビュー画面で `MachineOperationStarted` イベントの追加要求を見つける。手元には Apicurio Registry UI・Temporal Workflow コード、Mattermost 越しに tier1 担当者がいる。

## Trigger（発火条件）

業務フロー変更により新規 Domain Event / Workflow / 決定表の追加が必要になった時。

## 想定頻度 / 典型きっかけ

- 想定頻度: 月次
- 典型きっかけ: 「新 FA 生産指示フローで `MachineOperationStarted` Domain Event が必要になった」
- 典型きっかけ 2: 「`MachineHaltRequested` ドメインイベント + `EmergencyStopWorkflow` + `HaltReason` 決定表の 3 点セットが FA 業務に追加された」

## 主役 / 関与者

- 主役: tier2 担当者（中堅級）
- 関与: tier1 担当者（Outbox relay / Kafka 設定の整合確認）
- 承認: dual reviewer（tier2 担当者 2 名、変更 PR の author 不可）

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（tier2）| 中堅 | 本社 IT 室 | GitHub PR / Apicurio Registry UI | Domain Event schema 登録 / FSM 追加 / Workflow 実装 |
| 関与（tier1）| シニア | 本社 / リモート | GitHub PR | Outbox relay / Kafka 設定確認 / E2E test 協働 |
| 承認（dual reviewer）| 中堅〜シニア | 本社 / リモート | GitHub PR | PR レビュー / sign-off（author 不可） |

## 前提

- [atomic 三表書込](../../../03_概要設計/03_tier2設計方針/13_状態遷移パターン.md)（state / outbox / audit を同一 DB トランザクション）が全 aggregate に適用済みであること
- Apicurio Registry が稼働し FULL_TRANSITIVE compatibility check が CI に組み込まれていること
- Temporal Workflow 基盤が稼働済みであること

## 流れ

1. 新 Domain Event の schema を Apicurio Registry に登録し、FULL_TRANSITIVE compatibility check を通過させる
2. aggregate の状態遷移 FSM（[protoc-gen-go FSM](../../../04_詳細設計/03_クロスカッティング適合仕様/04_protoc_gen_go_fsm.md) 方式 / 4 言語等価強度）に新イベントを追加する
3. Workflow（Temporal Workflow）か Saga（choreography / orchestration）かを業務要件に基づいて判定し実装する
4. 決定表は DSL または codegen 形式で定義し [表現手段.md](../../../03_概要設計/03_tier2設計方針/10_表現手段.md) 規約に従う
5. [atomic 三表書込](../../../03_概要設計/03_tier2設計方針/13_状態遷移パターン.md) の確認: state table + outbox + audit table が必ず同一 tx で書かれることを integration test で確認する
   - Testcontainers で PostgreSQL + Kafka 環境を起動
   - state table / outbox table / audit table の 3 テーブルが同一 tx で書かれることをアサート
   - outbox が空 or 1 件のみであることを確認（二重書込みのない状態）
6. Outbox relay（Sidecar）が新イベントを Kafka に正しく配送することを E2E test で確認する
7. dual reviewer sign-off + Apicurio compatibility green を確認してから merge する

## 業界 9 業務との紐付け

- **FA 生産指示・設備操作**: `MachineOperationStarted` / `EmergencyStopWorkflow` 等のイベントが FA 生産指示フローに直接組み込まれ、設備操作のトレーサビリティが向上する。
- **ライン稼働監視**: 新 Domain Event が Outbox relay 経由で監視 Projector にリアルタイム配信され、ライン稼働状態が即座に更新される。
- **警報配信**: `MachineHaltRequested` 等の緊急イベントが警報配信 Workflow のトリガーとなる。

## 関連適合仕様 / 関連 OSS

- [protoc_gen_go_fsm 適合仕様](../../../04_詳細設計/03_クロスカッティング適合仕様/04_protoc_gen_go_fsm.md)
- [スキーマ進化適合仕様](../../../04_詳細設計/01_適合仕様/06_スキーマ進化適合仕様.md)
- Apicurio Registry（Domain Event schema 登録・FULL_TRANSITIVE 互換検査）
- Temporal（Workflow / Saga 実装）
- Kafka + Strimzi（Outbox relay による Event 配送）
- CloudNativePG（atomic 三表書込の DB トランザクション）

## 期待結果 / 観測指標

- Apicurio FULL_TRANSITIVE compatibility check が green になること
- 全 aggregate で atomic 三表書込が integration test により確認されること
- Outbox relay の E2E test が green になること
- 4 言語 FSM が等価強度で新イベントを処理できること

## 失敗時の挙動 / escalation

- **Apicurio compatibility check fail**: schema 変更が後方互換性を破壊していると判断し設計を見直す。escalate 先: Mattermost `#tier2-ci-alert`（SLA: 24h 以内に是正 PR 提出）。runbook: Backstage `domain-event-addition-procedure`
- **atomic 三表書込 integration test fail**: DB トランザクション設計の不備として tier2 担当者が修正する。escalate 先: Mattermost `#tier2-ci-alert`（SLA: 24h 以内に是正 PR 提出）。runbook: Backstage `domain-event-addition-procedure`
- **Outbox relay E2E test fail**: Sidecar 設定または Kafka topic 設定を確認し tier1 担当者と連携して対処する。escalate 先: Mattermost `#tier2-ci-alert`（SLA: 24h 以内に是正 PR 提出、tier1 担当者と協働）。runbook: Backstage `domain-event-addition-procedure`

## 関連参照

- [tier2 担当者シナリオ index](README.md)
- [tier2 設計方針](../../../03_概要設計/03_tier2設計方針/README.md)
- [ドメイン分割 BoundedContext](02_ドメイン分割_BoundedContext.md)
- [通知 UI 4 種実装（tier3-13）](../03_tier3担当者シナリオ/13_通知UI_4種実装.md) — Domain Event 追加後に tier3 担当者が通知 UI を実装するシナリオ

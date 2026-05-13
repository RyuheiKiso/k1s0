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
  defense_in_depth_layers: [A, B, C, D]
  proof_classes: []
---

# Domain Event / Workflow / 決定表追加

## 一文方針

業務フロー変更に伴い新規 Domain Event / Workflow / 決定表を追加する際、Apicurio FULL_TRANSITIVE 互換検査・FSM 状態遷移追加・atomic 三表書込・Outbox relay E2E 検証を全て通過させてから merge する。

> 朝 9 時半、本社 IT 室の tier2 担当者（中堅級）が GitHub PR レビュー画面で `MachineOperationStarted` イベントの追加要求を見つける。手元には Apicurio Registry UI・Temporal Workflow コード、Mattermost 越しに tier1 担当者がいる。

## ペルソナ要約

主役: tier2 担当者（中堅級）、目的: 新 Domain Event / Workflow / 決定表を Apicurio 互換検査・FSM 追加・atomic 三表書込・Outbox relay E2E 検証を全て通過させて merge する

## 現状業務での痛み

- Domain Event の schema 変更が後方互換性を破壊しているかどうかを手動確認のみに依存し、consumer 障害で気付く
- atomic 三表書込の適用が漏れ、Outbox への二重書込みや audit 欠落が本番データの不整合として顕在化する
- Workflow と Saga の選択基準が明確でなく、担当者ごとに実装方針が変わる
- Outbox relay の E2E 検証がなく、Kafka topic への配送が実際に動いているか確認されないまま merge される

## k1s0 でこう変わる

- Apicurio FULL_TRANSITIVE compatibility check が CI gate となり、後方互換性の破壊を merge 前に物理検出する
- atomic 三表書込の integration test（state / outbox / audit が同一 tx）が全 aggregate の merge 条件となる
- Workflow / Saga の選択基準（LRO 要否 / choreography vs orchestration）が設計方針に明文化される
- Outbox relay の E2E test（Sidecar → Kafka 配送）が green を merge 条件として強制する

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

## 個人 KPI / 達成感

- Apicurio FULL_TRANSITIVE compatibility check green
- atomic 三表書込 integration test（3 テーブル同一 tx）green 率
- Outbox relay E2E test green（Kafka 配送確認）
- dual reviewer 応答時間 ≤ 24h

## 工数 / 関与人数 / コスト感

- 初回（Domain Event + Workflow + 決定表 3 点セット）: 3〜5 日、関与 4〜5 名（主役 + tier1 担当者 + dual reviewer 2 名 + tier3 担当者 1 名）
- 平常（Domain Event のみ追加）: 1 日、関与 3〜4 名
- 失敗時（Apicurio fail / Outbox relay fail）: +1〜2 日、関与 4 名

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

## Timeline

| T+ | actor | action | 通知例 |
|---|---|---|---|
| 0 | tier2 担当者 | schema draft・Apicurio push・FULL_TRANSITIVE check | `MachineOperationStarted schema push / compatibility check: green` |
| 1 日 | tier2 担当者 | FSM 追加・Workflow/Saga 実装・決定表定義 | `FSM 追加完了 / Temporal Workflow 実装中` |
| 2 日 | tier2 担当者 | atomic 三表書込 integration test・Outbox relay E2E test | `3 テーブル同一 tx: green / Outbox → Kafka 配送: green` |
| 3 日 | tier1 担当者 | Outbox relay / Kafka 設定整合確認 | `tier1 整合確認完了` |
| 4 日 | dual reviewer | sign-off | `dual sign-off 完了` |

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

## 失敗パターン (anti-pattern)

- **Apicurio compatibility check なしで schema を直接更新**: 既存 consumer が新スキーマを受信できない状態になり、全 consumer での緊急対応が必要になる。schema 変更は必ず Apicurio に push して FULL_TRANSITIVE check を経由することを CI で強制する。
- **atomic 三表書込を「後で適用」として省略**: state だけ書いて outbox を書かない実装が先行し、Outbox relay がイベントを取得できない状態が発生する。atomic 三表書込の integration test を aggregate 追加 PR の merge 条件として先行して要求する。
- **Workflow と Saga の選択を ad hoc に決定**: 後から変更が難しい実装アーキテクチャが混在し、整合性が損なわれる。Workflow（LRO 要否）/ Saga（choreography vs orchestration）の選択基準を設計方針に明文化し、PR で選択理由の明示を必須とする。

## 関連参照

- [tier2 担当者シナリオ index](README.md)
- [tier2 設計方針](../../../03_概要設計/03_tier2設計方針/README.md)
- [ドメイン分割 BoundedContext](02_ドメイン分割_BoundedContext.md)
- [通知 UI 4 種実装（tier3-13）](../03_tier3担当者シナリオ/13_通知UI_4種実装.md) — Domain Event 追加後に tier3 担当者が通知 UI を実装するシナリオ

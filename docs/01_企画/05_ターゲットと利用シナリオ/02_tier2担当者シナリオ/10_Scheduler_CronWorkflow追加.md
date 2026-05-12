---
id: plan.tier2.scenario_scheduler_cronworkflow_addition
axis: tier2
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.tier2.tier2_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [C]
---

# Scheduler / CronWorkflow 追加

## 一文方針

tier2 担当者が業務要件に基づいて Argo CronWorkflow + Temporal Workflow による日次 / 月次バッチ処理を新規追加し、at-least-once 保証 + 冪等性 + テナント分離を維持する。

> 朝 9 時半、本社 IT 室の tier2 担当者（中堅級）が Argo Workflows UI で「在庫集計バッチ」の新規スケジュール要件チケットを確認する。手元には `batch_schedule.lock.yaml`・Temporal Workflow コード、Mattermost 越しに data 担当者と ops 担当者がいる。

## Trigger（発火条件）

業務担当者から「毎晩 23:00 に在庫集計を更新してほしい」等の定期バッチ要件が tier2 に起票された時。

## 想定頻度 / 典型きっかけ

- 想定頻度: 月次〜四半期（新規バッチ要件は随時）
- 典型きっかけ: 「製造業 pack の FA 設備 KPI（稼働率 / 不良率）を日次で集計して ClickHouse read model に投影するバッチが必要になった」
- 典型きっかけ 2: 「月次の受注確定処理（`OrderCloseWorkflow`）を Temporal で定期実行する要件が業務担当から来た」

## 主役 / 関与者

- 主役: tier2 担当者（中堅級）
- 関与: data 担当者（ClickHouse / PostgreSQL アクセス）/ ops 担当者（SLO 設定）
- 承認: dual reviewer（tier2 担当者 2 名、変更 PR の author 不可）

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（tier2）| 中堅 | 本社 IT 室 | Argo Workflows UI / Mattermost `#tier2-ops` | CronWorkflow / Temporal Workflow 実装 / atomic 三表書込適用 |
| 関与（data）| シニア | 本社 / リモート | GitHub PR | ClickHouse スキーマ設計 / PostgreSQL アクセス確認 |
| 関与（ops）| 中堅 | 本社 / リモート | Perses dashboard | SLO 定義登録 / バッチ完了時刻監視設定 |
| 承認（dual reviewer）| 中堅〜シニア | 本社 / リモート | GitHub PR | PR レビュー / sign-off（author 不可） |

## 前提

- [Scheduler / バッチ方針](../../../03_概要設計/03_tier2設計方針/25_スケジューラ・バッチ.md)（Argo CronWorkflow + Temporal Workflow + KEDA）が確立済みであること
- [atomic 三表書込](../../../03_概要設計/03_tier2設計方針/13_状態遷移パターン.md)（state / outbox / audit を同一 DB トランザクションで書く仕組み）が前提として整備済みであること
- バッチ処理の Idempotency-Key 設計が確定済みであること

## 流れ

1. バッチ処理の要件を確認する（対象データ / 処理内容 / 実行スケジュール / テナントスコープ）
2. 処理種別を判定する: Argo CronWorkflow（単純な定期 Job）か Temporal Workflow（LRO・Saga が必要な複雑処理）か
   - 冪等性の確保: 同一 job が 2 回実行されても同じ結果になることを設計段階で確認
   - テナント分離: 各バッチが特定 `tenant_id` スコープで動作し cross-tenant アクセスが不可能であることを確認
3. Argo CronWorkflow の場合: `manifests/workflows/<batch-name>-cron.yaml` に CronWorkflow を定義し GitOps（Argo CD）で deploy
4. Temporal Workflow の場合: Workflow コード（Go）を実装し、`temporal_client` を通じてスケジュール登録する
5. atomic 三表書込をバッチ処理に適用する: バッチの実行状態（`batch_state`）+ Outbox + audit を同一 tx で書く
6. SLO を定義する: 「23:30 までに完了する」等の時間制約を `slo_catalog.lock.yaml` に追記し、Perses で監視する
7. Testcontainers + Argo Workflows local runner で integration test を実施する（at-least-once + 冪等性を確認）
8. dual reviewer sign-off を取得し、GitOps 経由で staging に deploy して動作確認する
9. 本番 deploy 後 1 週間は完了時刻とエラー率を Perses dashboard で監視する

## 業界 9 業務との紐付け

- **在庫**: 日次在庫集計バッチが ClickHouse read model に投影されることで、在庫一覧画面のリアルタイム性が確保される。
- **受注**: 月次受注確定処理（`OrderCloseWorkflow`）が Temporal で定期実行されることで、受注業務のバッチ集計が自動化される。
- **ライン稼働監視**: FA 設備 KPI（稼働率 / 不良率）の日次バッチがライン稼働監視 read model を更新する。

## 関連適合仕様 / 関連 OSS

- [テナント分離適合仕様](../../../04_詳細設計/01_適合仕様/10_テナント分離適合仕様.md)
- [SLO 適合仕様](../../../04_詳細設計/01_適合仕様/07_SLO適合仕様.md)
- [tier2 強制機構](../../../04_詳細設計/02_強制機構/02_tier2強制機構.md)
- Argo Workflows（CronWorkflow）
- Temporal（複雑 Workflow）
- KEDA（event-driven scaling）
- Perses（SLO monitoring）

## 期待結果 / 観測指標

- artifact: `batch_schedule.lock.yaml` に新 CronWorkflow / Workflow エントリが記録済み
- ci: Testcontainers integration test（at-least-once + 冪等性）all green
- slo: SLO 完了時刻内に全テナントのバッチが完了（計測: Perses `batch-completion-time` パネル）
- sign-off: dual reviewer（tier2 担当者 2 名、変更 PR の author 不可）sign-off 完了
- 観測: 本番 deploy 後 1 週間、Perses `batch-slo-breach` アラートが 0 件

## 失敗時の挙動 / escalation

- **バッチが SLO 時間（完了時刻）を超過**: ops 担当者に Mattermost `#tier2-slo-breach` で自動 alert（**SLA: alert から 30 分以内**に ops 担当者が状況確認）。Backstage runbook `batch-slo-breach` を参照。
- **冪等性違反（2 回実行で結果が異なる）**: バッチを一時停止し tier2 担当者が原因調査（**SLA: 24h 以内**に修正 PR）。**postmortem 期限: 3 営業日以内**。
- **cross-tenant data アクセス検出**: バッチを即時停止。security 担当者に Mattermost `#security-incident` で即時通報（**SLA: 15 分以内**）。**postmortem 期限: 2 営業日以内**。

## 関連参照

- [tier2 担当者シナリオ index](./README.md) — tier2 担当者シナリオ全体の構成と dual reviewer 規約
- [Domain Event / Workflow / 決定表追加](./04_Domain_Event_Workflow_決定表追加.md) — バッチ処理で発火する Domain Event の追加シナリオ
- [tier2 設計方針 スケジューラ・バッチ](../../../03_概要設計/03_tier2設計方針/25_スケジューラ・バッチ.md) — Argo CronWorkflow vs Temporal の選択基準

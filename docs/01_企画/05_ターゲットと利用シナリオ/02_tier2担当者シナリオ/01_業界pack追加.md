---
id: plan.tier2.scenario_industry_pack_addition
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

# 業界 pack 追加

## 一文方針

新業界（例: サービス業 / 医療業）の正式 pack を stub から昇格させる際、4 抽象化レベルの分類・依存方向検証・atomic 三表書込・テナント識別子強制注入・8 層強制機構への組込みを一貫して通過させる。

> 朝 10 時、本社 IT 室の tier2 担当者（中堅級）が Backstage Software Catalog で「medical-pack stub」の昇格タスクに気付く。手元には GitHub PR・`industry-neutrality-linter` レポート、Mattermost 越しに tier1 担当者がいる。

## ペルソナ要約

主役: tier2 担当者（中堅級）、目的: 新業界 pack を 4 抽象化レベルに分類し 8 層強制機構・atomic 三表書込・テナント識別子注入を全て通過させて昇格する

## 現状業務での痛み

- 業界固有概念が業界横断層に混入していても CI なしでは発覚が遅く、後から大規模リファクタが発生する
- atomic 三表書込の適用が属人的で一部 aggregate で outbox が二重書込みになっていても気付かない
- テナント識別子強制注入の漏れが新 pack 追加時に発生し、cross-tenant データ漏洩リスクが生まれる
- 8 層強制機構への組み込みが後工程になり、tier3 が tier2 を迂回する経路が先に生まれてしまう

## k1s0 でこう変わる

- 業界中立性 lint が CI で自動実行され、業界固有概念の業界横断層への混入を merge 前に物理拒否する
- atomic 三表書込の Testcontainers integration test が全 aggregate を網羅し、二重書込みゼロを担保する
- テナント識別子注入経路 lint が全 API に対して実行され、注入漏れを merge 前に検出する
- 8 層強制機構への組み込みを新 pack の merge 条件として CI gate に組み込む

## Trigger（発火条件）

新業界（例: サービス業 / 医療業）の正式 pack 追加要求が来た時（v1.0.0 は製造業のみだが v2 候補として stub から昇格させる場合）。

## 想定頻度 / 典型きっかけ

- 想定頻度: 年次〜バージョン毎
- 典型きっかけ: 「v2.0 で医療業 pack の追加要求が来た際に、1.0.0 の製造業 pack と CI 専用サービス業 stub を参考に昇格作業を開始した」
- 典型きっかけ 2: 「物流業 pack の荷物追跡機能（`Shipment` Aggregate）のサービス業 stub からの昇格を実施した」

## 主役 / 関与者

- 主役: tier2 担当者（中堅級）
- 関与: tier1 担当者（API 整合確認）
- 承認: dual reviewer（tier2 担当者 2 名、変更 PR の author 不可）

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（tier2）| 中堅 | 本社 IT 室 | Backstage Catalog / Argo Workflows | 業界 pack 昇格 / 4 抽象化レベル分類 / 8 層強制機構組込み |
| 関与（tier1）| シニア | 本社 / リモート | GitHub PR | API 整合確認 / stub conformance レビュー |
| 承認（dual reviewer）| 中堅〜シニア | 本社 / リモート | GitHub PR | PR レビュー / sign-off（author 不可） |

## 個人 KPI / 達成感

- 業界中立性 lint green（`forbidden_cross_dependency` カウント = 0）
- 全 aggregate での atomic 三表書込 integration test green 率
- テナント識別子注入経路 lint green（全 API）
- 8 層強制機構への組み込み完了・CI 全件 green

## 工数 / 関与人数 / コスト感

- 初回（新業界 pack 昇格）: 1〜2 週間（業務語彙分類・atomic 三表書込・8 層組み込み・CI green）、関与 4〜5 名（主役 + tier1 担当者 + dual reviewer 2 名 + ops 担当者）
- 平常（既存 pack への aggregate 追加）: 1〜3 日、関与 3〜4 名
- 失敗時（中立性 lint fail・conformance fail）: +1〜3 日、関与 4 名

## 前提

- 業界横断層（Cross-industry Lv）が確立済みであること
- [第二業界 stub conformance](../../../03_概要設計/03_tier2設計方針/02_業界拡張モデル.md) CI が green であること（arch.tier2.tier2_index の「第二業界 stub conformance」）
- [atomic 三表書込](../../../03_概要設計/03_tier2設計方針/13_状態遷移パターン.md)（state / outbox / audit を同一 DB トランザクションで書く仕組み）が既存 pack の全 aggregate に適用済みであること

## 流れ

1. 新業界の業務語彙（例: サービス業なら「サービス品目」「予約」「顧客 SLA」等）を業界横断 / 業界共通 / 業界固有の 3 Lv に分類する
2. 業界横断層 → 新業界 pack の単方向依存であることを CI で検証する（逆方向依存 = 中立性違反として reject）
   - 業界中立性 lint（`industry-neutrality-linter`）を CI で実行し、新 pack → 業界横断層の依存が無いことを確認
   - 第二業界 stub（サービス業 stub）の API から新 pack の業界横断 API が消費可能かを Testcontainers で確認
   - 受入基準: `forbidden_cross_dependency` カウント = 0
3. 第二業界 stub（現在はサービス業 stub が CI 専用で存在）と新 pack の API が同型であることを確認する
4. 新 pack 固有の Domain Event schema / Workflow / 決定表 / 業務マスタ / DB schema を作成する
5. atomic 三表書込（state / outbox / audit）を新 pack の全 aggregate に適用する
6. テナント識別子強制注入経路が全 API に存在することを lint で確認する
7. tier3 側が tier2 を迂回しないよう 8 層強制機構に新 pack を追加する
   - tier3 リポジトリ側 lint の allowlist に新 pack の public API を追加（`tier3_allowed_imports.yaml`）
   - tier2 公開 API 表面 lint の新 pack 用 policy を宣言（`policies/tier2-pack-<name>.rego`）
   - 8 層が全件 green になることを CI で確認してから merge
8. dual reviewer sign-off + Testcontainers green（新 pack 全 API）を確認してから merge する

## Timeline

| T+ | actor | action | 通知例 |
|---|---|---|---|
| 0 | tier2 担当者 | 業務語彙分類（4 Lv）・中立性 lint 実行 | `medical-pack 昇格開始 / 業務語彙分類中 / lint 実行中` |
| 2〜3 日 | tier2 担当者 | Domain Event/Workflow/決定表/DB schema 作成 | `atomic 三表書込 integration test: green / 8 層組み込み中` |
| 4〜5 日 | tier1 担当者 | API 整合確認・stub conformance レビュー | `stub conformance CI: green / tier1 整合確認完了` |
| 1 週間 | dual reviewer | sign-off・CI 全件 green 確認 | `dual sign-off 完了 / 業界中立性 lint green / 8 層全件 green` |

## 業界 9 業務との紐付け

- **受注**: 新業界 pack の受注エンティティが業界横断層 API 経由で受注業務に統合され、製造業 pack との共存が確立される。
- **SCADA テレメトリ**: 業界横断層の SCADA インタフェースが新 pack でも利用可能か stub conformance CI で確認する。
- **FA 生産指示・設備操作**: 新 pack 固有の生産指示フローが業界横断 Workflow 基盤に正しく登録されることを確認する。
- **在庫**: 新業界 pack の品目マスタ / 在庫アグリゲートが atomic 三表書込で整合する。

## 関連適合仕様 / 関連 OSS

- [テナント分離適合仕様](../../../04_詳細設計/01_適合仕様/10_テナント分離適合仕様.md)
- [tier2 強制機構](../../../04_詳細設計/02_強制機構/02_tier2強制機構.md)
- Testcontainers（新 pack 全 API の integration test）
- Apicurio Registry（Domain Event schema 登録・互換性検査）

## 期待結果 / 観測指標

- 新 pack が 8 層強制機構に組み込まれ、全 CI が green になること
- 業界横断層 API が stub からも消費可能であること
- 業界中立性 lint が green を維持すること
- 全 aggregate で atomic 三表書込が integration test により確認されること

## 失敗時の挙動 / escalation

- **業界中立性 lint fail**: 業界固有概念が業界横断層に混入。CI が merge を阻止。escalate 先: Mattermost `#tier2-ci-alert` に自動通知（SLA: 24h 以内に是正 PR を提出）。runbook: Backstage `industry-pack-promotion-procedure`
- **逆方向依存 fail（新 pack → 業界横断層）**: 依存方向検証 CI が fail し merge を阻止。escalate 先: Mattermost `#tier2-ci-alert`（SLA: 24h 以内に是正 PR 提出）。runbook: Backstage `industry-pack-promotion-procedure`
- **第二業界 stub conformance fail**: API 同型性が未達と判断し tier2 担当者が API 設計を見直す。escalate 先: Mattermost `#tier2-ci-alert`（SLA: 48h 以内）。runbook: Backstage `industry-pack-promotion-procedure`
- **Testcontainers fail**: atomic 三表書込または tenant_id 注入の実装不備。escalate 先: Mattermost `#tier2-ci-alert`（SLA: 24h 以内に是正 PR 提出）。runbook: Backstage `industry-pack-promotion-procedure`

## 失敗パターン (anti-pattern)

- **業務語彙の分類を曖昧にしたまま昇格を進める**: 業界横断層と業界固有層の境界が不明確なまま実装が始まり、後から大規模な分類修正が発生する。4 抽象化レベルへの分類をコードを書く前に完了させ、チーム間での合意を取得してから実装に進む。
- **atomic 三表書込を一部 aggregate で省略**: state / outbox / audit の 3 テーブルを別 tx で書くことで、イベント欠落や audit chain の穴が生まれる。integration test で「3 テーブルが必ず同一 tx で書かれること」を全 aggregate に対して assert する。
- **8 層強制機構への組み込みを後から追加**: tier3 が tier2 を迂回する経路が先に生まれ、後から組み込もうとすると既存コードへの影響が広がる。新 pack の merge 条件として 8 層強制機構 CI 全件 green を先行して要求する。

## 関連参照

- [tier2 担当者シナリオ index](README.md)
- [tier2 設計方針](../../../03_概要設計/03_tier2設計方針/README.md)
- [業務資産所有権](../../../03_概要設計/03_tier2設計方針/06_業務資産所有権.md)

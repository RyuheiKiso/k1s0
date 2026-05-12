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
  defense_in_depth_layers: []
  proof_classes: []
---

# 業界 pack 追加

## 一文方針

新業界（例: サービス業 / 医療業）の正式 pack を stub から昇格させる際、4 抽象化レベルの分類・依存方向検証・atomic 三表書込・テナント識別子強制注入・8 層強制機構への組込みを一貫して通過させる。

> 朝 10 時、本社 IT 室の tier2 担当者（中堅級）が Backstage Software Catalog で「medical-pack stub」の昇格タスクに気付く。手元には GitHub PR・`industry-neutrality-linter` レポート、Mattermost 越しに tier1 担当者がいる。

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

## 関連参照

- [tier2 担当者シナリオ index](README.md)
- [tier2 設計方針](../../../03_概要設計/03_tier2設計方針/README.md)
- [業務資産所有権](../../../03_概要設計/03_tier2設計方針/06_業務資産所有権.md)

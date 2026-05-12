---
id: plan.tier2.scenario_domain_split_bounded_context
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

# ドメイン分割 BoundedContext

## 一文方針

ドメイン肥大化により単一 Bounded Context に複数の業務語彙が混在していると判明した際、クロスドメイン参照を Domain Event 経由に限定しながら expand-contract pattern で安全に境界を再設定する。

> 朝 9 時、本社 IT 室の tier2 担当者（中堅級）が Argo Workflows UI で CI 依存グラフ分析の結果レポートを開き、`PurchaseOrder` と `Inspection` の混在を示す警告に気付く。手元には GitHub PR と CloudNativePG migration ツール、Mattermost 越しに tier1 担当者がいる。

## Trigger（発火条件）

ドメインが肥大化し単一 Bounded Context に複数の業務語彙が混在していることが判明した時（レビューや CI 依存グラフ分析により発覚する場合を含む）。

## 想定頻度 / 典型きっかけ

- 想定頻度: 四半期〜年次
- 典型きっかけ: 「調達 Context と検査 Context が同一 Aggregate に混在し、調達 Entity の変更が検査テストに波及するようになった」
- 典型きっかけ 2: 「`PurchaseOrder` Aggregate と `Inspection` Aggregate が同一 Context に混在し、発注変更が検査テストに波及するようになった」

## 主役 / 関与者

- 主役: tier2 担当者（中堅級）
- 関与: tier1 担当者（contract test の型変更影響確認）
- 承認: dual reviewer（tier2 担当者 2 名、変更 PR の author 不可）

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（tier2）| 中堅 | 本社 IT 室 | Argo Workflows UI / GitHub PR | 境界再設定 / Domain Event 移動 / DB schema expand-contract |
| 関与（tier1）| シニア | 本社 / リモート | GitHub PR | contract test 型変更影響確認 / sign-off |
| 承認（dual reviewer）| 中堅〜シニア | 本社 / リモート | GitHub PR | PR レビュー / sign-off（author 不可） |

## 前提

- 現行 Bounded Context の Domain Event 一覧とアグリゲートが把握されていること
- [Outbox relay](../../../03_概要設計/02_tier1設計方針/01_Server系.md)（Sidecar コンポーネント）が稼働済みであること
- CloudNativePG で [expand-contract pattern](../../../03_概要設計/06_data設計方針/08_マイグレーション方針.md) に基づく forward-only migration が運用ポリシーとして確立済みであること

## 流れ

1. 現行 Bounded Context の Domain Event 一覧とアグリゲートを棚卸しし、境界を可視化する
2. クロスドメイン参照が Direct ではなく Domain Event 経由になっているか確認する（直接参照は禁止）
3. 新 Bounded Context の境界（ubiquitous language）を決定し、チーム間で合意を得る
4. Domain Event を新 Bounded Context に移動または複製する（consumer 側は Outbox relay 経由で受信）
5. DB schema（CloudNativePG）を分割する: [expand-contract pattern](../../../03_概要設計/06_data設計方針/08_マイグレーション方針.md) で forward-only migration を実施する
   - tier2 担当者が sqlx-cli で migration ファイルを作成（`migrations/` ディレクトリ）
   - `up` のみ作成し `down` は不要（forward-only）
   - 適用前に staging DB で dry-run 確認
6. tier3 が参照している型の変更影響を contract test で検出し、breaking change を事前に排除する
7. 旧コンテキストの API を deprecated 宣言し、移行期間後に SemVer に従い廃止する
8. dual reviewer sign-off を得てから merge する

## 業界 9 業務との紐付け

- **受注**: 調達 Context と受注 Context の境界再設定により、受注エンティティへの意図しない変更伝播を防止する。
- **品質検査結果**: 検査 Context を独立させることで、品質検査結果の更新が発注フローに波及しなくなる。
- **在庫**: Context 分割後の Domain Event 経由連携により、在庫集計の整合性が Outbox relay によって保証される。

## 関連適合仕様 / 関連 OSS

- [スキーマ進化適合仕様](../../../04_詳細設計/01_適合仕様/06_スキーマ進化適合仕様.md)
- [tier2 強制機構](../../../04_詳細設計/02_強制機構/02_tier2強制機構.md)
- CloudNativePG（DB schema 分割 / expand-contract）
- Apicurio Registry（Domain Event schema の互換性管理）
- Kafka + Strimzi（Outbox relay による Domain Event 配送）

## 期待結果 / 観測指標

- 新 Bounded Context が明確な ubiquitous language で境界付けられること
- クロスドメイン参照が Domain Event 経由のみになり直接参照が CI で検出されないこと
- forward-only migration が適用され、ロールバック不要の schema 変更が成立すること
- 旧 API の deprecated 宣言が全 consumer に周知されること

## 失敗時の挙動 / escalation

- **クロスドメイン直接参照 CI fail**: 依存グラフ検査 CI が fail し merge を阻止。escalate 先: Mattermost `#tier2-ci-alert`（SLA: 24h 以内に是正 PR 提出）。runbook: Backstage `domain-split-procedure`
- **DB schema expand-contract fail**: migration rollback 要求 → data 担当者に即時連絡。escalate 先: Mattermost `#data-incident`（SLA: 30 分以内に data 担当者へ連絡）。runbook: Backstage `schema-rollback-procedure`
- **contract test breaking change 検出**: tier3 担当者と連携し型変更の影響範囲を確定してから再提出。escalate 先: Mattermost `#tier2-ci-alert`（SLA: 48h 以内に是正 PR 提出）。runbook: Backstage `domain-split-procedure`

## 関連参照

- [tier2 担当者シナリオ index](README.md)
- [tier2 設計方針](../../../03_概要設計/03_tier2設計方針/README.md)
- [Domain Event / Workflow / 決定表追加](04_Domain_Event_Workflow_決定表追加.md)

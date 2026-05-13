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
  defense_in_depth_layers: [A, B, C, D]
  proof_classes: []
---

# ドメイン分割 BoundedContext

## 一文方針

ドメイン肥大化により単一 Bounded Context に複数の業務語彙が混在していると判明した際、クロスドメイン参照を Domain Event 経由に限定しながら expand-contract pattern で安全に境界を再設定する。

> 朝 9 時、本社 IT 室の tier2 担当者（中堅級）が Argo Workflows UI で CI 依存グラフ分析の結果レポートを開き、`PurchaseOrder` と `Inspection` の混在を示す警告に気付く。手元には GitHub PR と CloudNativePG migration ツール、Mattermost 越しに tier1 担当者がいる。

## ペルソナ要約

主役: tier2 担当者（中堅級）、目的: 単一 Bounded Context に混在した業務語彙をクロスドメイン参照を Domain Event 経由に限定しながら expand-contract pattern で安全に分割する

## 現状業務での痛み

- ドメインが肥大化していても CI による検出手段がなく、発覚が設計レビューでの人手確認か本番障害後になる
- DB schema の直接分割を試みると rollback が必要になる forward-only 違反が発生し、データ不整合を招く
- クロスドメイン直接参照が残っていても contract test を実行するまで気付かず、breaking change が下流に波及する
- 旧 API の deprecation 宣言が徹底されず、廃止後も consumer が旧 API を使い続けてしまう

## k1s0 でこう変わる

- 依存グラフ検査 CI がクロスドメイン直接参照を merge 前に物理拒否し、Domain Event 経由への移行を強制する
- CloudNativePG の expand-contract pattern + forward-only migration で rollback なしの schema 分割を標準化する
- contract test が型変更の breaking impact を事前検出し、tier3 への影響を PR 段階で把握できる
- Outbox relay による Domain Event 経由連携が新 Bounded Context 間の唯一の接続方式として確立される

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

## 個人 KPI / 達成感

- クロスドメイン直接参照 CI fail ゼロ（Domain Event 経由のみ）
- forward-only migration 適用完了（rollback なし）
- 旧 API deprecated 宣言が全 consumer に周知
- dual reviewer 応答時間 ≤ 48h

## 工数 / 関与人数 / コスト感

- 初回（大規模分割）: 1〜2 週間（棚卸し・境界決定・Domain Event 移動・DB schema 分割・contract test）、関与 4〜5 名
- 平常（小規模分割）: 3〜5 日、関与 3〜4 名（主役 + tier1 担当者 + dual reviewer 2 名）
- 失敗時（DB schema expand-contract fail）: +1〜2 日、関与 5 名（+ data 担当者）

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

## Timeline

| T+ | actor | action | 通知例 |
|---|---|---|---|
| 0 | tier2 担当者 | 現行 Context 棚卸し・境界可視化・新境界合意 | `PurchaseOrder/Inspection 混在確認 / 分割方針: 2 Context に分離` |
| 1〜2 日 | tier2 担当者 | Domain Event 移動・DB schema expand-contract migration 作成 | `migrate up 適用 / staging dry-run green` |
| 3 日 | tier1 担当者 | contract test 型変更影響確認 | `contract test: 変更なし / breaking change なし` |
| 4〜5 日 | tier2 担当者 | 旧 API deprecated 宣言・Outbox relay 動作確認 | `旧 Context API: deprecated 宣言済 / Outbox relay green` |
| 1 週間 | dual reviewer | sign-off | `dual sign-off 完了 / クロスドメイン直接参照 CI: green` |

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

## 失敗パターン (anti-pattern)

- **境界合意なしにコードリファクタから始める**: 実装が先行して境界の ubiquitous language が曖昧なまま進み、後から設計方針の不一致が判明して大規模なやり直しが発生する。境界の合意と ubiquitous language の決定を実装の絶対的な先行条件とする。
- **DB schema を rollback 前提で分割**: 下 migration（down）を作成して rollback 可能な設計にしようとし、データ分割の論理的一貫性が保てなくなる。forward-only migration のみを作成し、rollback が必要な場合は新たな up migration で対処する方針を徹底する。
- **旧 Context API の deprecation 宣言を後回し**: 移行期限が曖昧なまま consumer が旧 API を使い続け、廃止が永遠に先送りされる。新 Context の API が stable になった時点で旧 API に deprecation 宣言と廃止期限を明記し、Backstage でトラッキングする。

## 関連参照

- [tier2 担当者シナリオ index](README.md)
- [tier2 設計方針](../../../03_概要設計/03_tier2設計方針/README.md)
- [Domain Event / Workflow / 決定表追加](04_Domain_Event_Workflow_決定表追加.md)

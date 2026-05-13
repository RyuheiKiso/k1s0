---
id: plan.tier2.scenario_read_model_projector
axis: tier2
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.tier2.tier2_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers:
    - C
  proof_classes: []
---

# 読取モデル Projector 追加

## 一文方針

tier2 担当者が CQRS の Read model projector を新規追加し、Domain Event → ClickHouse / read-replica / cache への投影を設定することで、tier3 の検索 / 集計 / 一覧画面のデータ要件を充足する。

> 朝 10 時、本社 IT 室の tier2 担当者（中堅級）が Mattermost `#tier2-ops` で tier3 担当者からの「在庫一覧 cursor pagination 対応の read model 欲しい」という要求に気付き、GitHub PR と ClickHouse スキーマ設計を開始する。手元には Apicurio Registry UI・Testcontainers、Mattermost 越しに data 担当者と tier3 担当者がいる。

## ペルソナ要約

主役: tier2 担当者（中堅級）、目的: CQRS の Read model projector を新規追加して Domain Event → ClickHouse への投影を設定し tier3 の検索 / 集計 / 一覧画面のデータ要件を充足する

## 現状業務での痛み

- tier3 からの read model 要求があるたびに RDB 直接クエリで対応し、1 秒以上かかるクエリが本番に混入する
- read model の SLO（p99 latency）が定義されておらず、応答時間の悪化が発覚するのが本番障害後になる
- cross-tenant 投影のチェックが手動で属人的になり、他テナントのデータが混在するリスクがある
- tier3 との API 仕様が口頭合意のみで contract test がなく、破壊的変更が UI を壊してから気付く

## k1s0 でこう変わる

- ClickHouse / read-replica / Valkey キャッシュへの投影スキーマを data 担当者と協議して設計し、パフォーマンス要件を事前担保する
- `slo_catalog.lock.yaml` に p99 latency ≤ 100ms を登録し、SLO 未達を release 保留条件として明文化する
- projector の tenant_id フィルタと Testcontainers integration test で cross-tenant 投影 0 件を物理証明する
- Pact contract test で tier3 からの read model API 消費を自動検証し、破壊的変更を merge 前に検出する

## Trigger（発火条件）

tier3 担当者から「検索一覧 UX（cursor pagination + virtual scroll）に必要な read model がない」または「集計値（在庫総数 / 発注金額合計 等）を表示するための専用 API が必要」という要求が来た時

## 想定頻度 / 典型きっかけ

- 想定頻度: 月次〜四半期
- 典型きっかけ: 「製造業 pack 在庫一覧画面で「品目コード / 拠点 / 状態」の 3 軸フィルタリング + cursor pagination が必要だが、現行 RDB 直接クエリでは 1 秒以上かかる。ClickHouse に投影する read model を新設することになった」「月次在庫集計バッチが tier3 の月次レポート画面に必要で、Temporal WorkflowResult から ClickHouse の集計テーブルに投影する projector を追加することになった」

## 主役 / 関与者

- 主役: tier2 担当者（中堅級）
- 関与: data 担当者（ClickHouse スキーマ設計）/ tier3 担当者（read model の要件定義）
- 承認: dual reviewer（tier2 担当者 2 名、変更 PR の author 不可）

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（tier2）| 中堅 | 本社 IT 室 | Mattermost `#tier2-ops` / GitHub PR | projector 実装 / Read model API 追加 / Testcontainers integration test |
| 関与（data）| シニア | 本社 / リモート | GitHub PR | ClickHouse スキーマ設計 / index・projection 最適化協議 |
| 関与（tier3）| 中堅 | 本社 / リモート | GitHub PR / Mattermost `#tier2-ops` | read model 要件定義 / Pact contract test 作成 |
| 承認（dual reviewer）| 中堅〜シニア | 本社 / リモート | GitHub PR | PR レビュー / sign-off（author 不可） |

## 個人 KPI / 達成感

- Testcontainers integration test（投影正確性 + テナント分離）all green
- read model API p99 latency ≤ 100ms（`slo_catalog.lock.yaml` 登録済み）
- tier3 担当者の Pact contract test all green
- dual reviewer sign-off 完了

## 工数 / 関与人数 / コスト感

- 初回（ClickHouse projector 新設）: 3〜5 日（要件定義・スキーマ設計・projector 実装・API 追加・contract test）、関与 5 名（主役 + data 担当者 + tier3 担当者 + dual reviewer 2 名）
- 平常（既存 projector への新フィールド追加）: 1〜2 日、関与 3〜4 名
- 失敗時（cross-tenant 投影検出・SLO 未達）: 即時 API 停止 + 再実装、関与 4〜5 名（cross-tenant 時は + security 担当者）

## 前提

- [読み取りモデル方針](../../../03_概要設計/03_tier2設計方針/15_読み取りモデル方針.md)（CQRS）が確立済み
- ClickHouse が data 軸で管理済み（[データストア構成方針](../../../03_概要設計/06_data設計方針/01_データストア構成方針.md)）
- 投影先の Domain Event が Apicurio Registry に登録済み

## 流れ

1. tier3 担当者から read model の要件を受領する（表示フィールド / フィルタ軸 / ソート軸 / pagination の必要性）
2. 投影元の Domain Event を特定する（どのイベントを subscribe して read model を更新するか）
3. read model のスキーマを設計する: data 担当者と協議して ClickHouse テーブル / PostgreSQL read-replica view / Valkey キャッシュのどれに投影するかを決定する
4. projector を実装する: Domain Event を subscribe → read model テーブルに upsert する Go or Rust のコンポーネントを実装
   - テナント分離: projector が必ず `tenant_id` でフィルタして投影すること（cross-tenant 投影禁止）
   - at-least-once: Outbox relay 経由で Domain Event が届くことを前提に、幂等な upsert を実装
5. Read model API を tier2 に追加する: `GET /v1/{tenant_id}/read-models/{model_name}?cursor=...&filter=...` の形式で提供
6. Testcontainers で integration test を実施する（投影の正確性 + テナント分離 + cursor pagination の動作）
7. tier3 担当者に API を共有し、contract test を作成してもらう（tier3 側から tier2 の read model API を Pact でテスト）
8. dual reviewer sign-off を取得する

## Timeline

| T+ | actor | action | 通知例 |
|---|---|---|---|
| 0 | tier2 担当者 / tier3 担当者 | read model 要件定義（フィルタ軸 / pagination 要件）| `在庫 read model: 3 軸フィルタ + cursor pagination / ClickHouse 採用` |
| 1 日 | tier2 担当者 / data 担当者 | ClickHouse スキーマ設計・projector 実装 | `ClickHouse テーブル設計完了 / projector 実装中` |
| 2 日 | tier2 担当者 | Read model API 追加・tenant_id フィルタ・幂等 upsert 実装 | `API 追加完了 / tenant_id フィルタ: 適用済` |
| 3 日 | tier2 担当者 | Testcontainers integration test・SLO 登録 | `cross-tenant: 0 件 / p99: 45ms ≤ 100ms / slo_catalog 更新` |
| 4 日 | tier3 担当者 | Pact contract test 作成・green 確認 | `Pact contract test: green` |
| 5 日 | dual reviewer | sign-off | `dual sign-off 完了` |

## 業界 9 業務との紐付け

- **在庫**: 在庫一覧 read model（品目コード / 拠点 / 状態の 3 軸フィルタ + cursor pagination）が ClickHouse に投影され、在庫一覧画面の応答時間が 1 秒以内に収まる。
- **受注**: 月次受注集計 projector が Temporal WorkflowResult から ClickHouse 集計テーブルに投影され、月次レポート画面のデータが自動更新される。
- **ライン稼働監視**: ライン稼働状態の read model が Domain Event から ClickHouse に継続投影され、監視画面がリアルタイムで更新される。

## 関連適合仕様 / 関連 OSS

- 関連適合仕様: [テナント分離適合仕様](../../../04_詳細設計/01_適合仕様/10_テナント分離適合仕様.md) / [SLO 適合仕様](../../../04_詳細設計/01_適合仕様/07_SLO適合仕様.md) / [tier2 強制機構](../../../04_詳細設計/02_強制機構/02_tier2強制機構.md)
- 関連 OSS: ClickHouse（分析 read model）/ Valkey（キャッシュ）/ Testcontainers（integration test）/ Pact（contract test）

## 期待結果 / 観測指標

- ci: Testcontainers integration test（投影正確性 + テナント分離）all green
- slo: read model API の p99 latency ≤ 100ms（`slo_catalog.lock.yaml` に登録済み）
- contract: tier3 担当者の Pact contract test all green
- sign-off: dual reviewer（tier2 担当者 2 名）sign-off 完了

## 失敗時の挙動 / escalation

- **cross-tenant 投影検出（他テナントのデータが混在）**: API を即時停止し security 担当者に Mattermost `#security-incident` で即時通報（**SLA: 15 分以内**）。**postmortem 期限: 2 営業日以内**。
- **SLO 未達（p99 > 100ms）**: ClickHouse のクエリ最適化（index / projection の追加）を data 担当者と協議し再実装（**SLO 達成まで本番 release 保留**）。

## 失敗パターン (anti-pattern)

- **RDB 直接クエリで read model を代替**: 複雑なフィルタ・ソート・集計クエリが 1 秒以上かかる状態で本番稼働が始まり、tier3 の UX が劣化する。ClickHouse / read-replica / Valkey キャッシュへの専用投影を read model の前提条件とし、RDB 直接クエリでの代替を設計段階で禁止する。
- **cross-tenant 投影チェックを省略**: projector が tenant_id フィルタなしで全テナントのデータを混在投影し、security incident になる。projector 実装時に tenant_id フィルタの適用と Testcontainers cross-tenant leak test の green を merge 必須条件とする。
- **Pact contract test なしで tier3 に API を提供**: tier2 が read model API を変更した際に tier3 の UI が壊れても merge 後に発覚する。Pact contract test を tier3 担当者が作成することを read model API 追加の完了条件として明文化する。

## 関連参照

- [tier2 担当者シナリオ index](./README.md) — tier2 担当者シナリオ全体の構成と dual reviewer 規約
- [Domain Event / Workflow / 決定表追加](./04_Domain_Event_Workflow_決定表追加.md) — projector の投影元 Domain Event 追加シナリオ
- [tier3 シナリオ: 検索一覧 UX](../03_tier3担当者シナリオ/12_検索一覧UX実装.md) — 本シナリオで追加した read model を消費する tier3 側実装
- [tier2 設計方針 読み取りモデル](../../../03_概要設計/03_tier2設計方針/15_読み取りモデル方針.md) — CQRS / Read model projector の設計指針
- [Kafka topic・partition 変更（data-11）](../05_data担当者シナリオ/11_Kafka_Strimzi_topic_partition変更.md) — Projector が消費する Kafka topic の追加・partition 変更は data 担当者が実施する

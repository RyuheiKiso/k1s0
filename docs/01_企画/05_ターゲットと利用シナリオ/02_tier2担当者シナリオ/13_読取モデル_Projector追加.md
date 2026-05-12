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

## 関連参照

- [tier2 担当者シナリオ index](./README.md) — tier2 担当者シナリオ全体の構成と dual reviewer 規約
- [Domain Event / Workflow / 決定表追加](./04_Domain_Event_Workflow_決定表追加.md) — projector の投影元 Domain Event 追加シナリオ
- [tier3 シナリオ: 検索一覧 UX](../03_tier3担当者シナリオ/12_検索一覧UX実装.md) — 本シナリオで追加した read model を消費する tier3 側実装
- [tier2 設計方針 読み取りモデル](../../../03_概要設計/03_tier2設計方針/15_読み取りモデル方針.md) — CQRS / Read model projector の設計指針
- [Kafka topic・partition 変更（data-11）](../05_data担当者シナリオ/11_Kafka_Strimzi_topic_partition変更.md) — Projector が消費する Kafka topic の追加・partition 変更は data 担当者が実施する

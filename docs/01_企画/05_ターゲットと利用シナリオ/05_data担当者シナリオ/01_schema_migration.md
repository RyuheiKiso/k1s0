---
id: plan.data.scenario_schema_migration
axis: data
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.data.data_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [B, C]
  proof_classes: []
---

# schema migration

## 一文方針

tier2 の Domain Event schema / DB schema 変更を forward-only の [expand-contract pattern](../../../03_概要設計/06_data設計方針/08_マイグレーション方針.md) で安全に適用し、Apicurio Registry との整合を保ったまま dual reviewer sign-off まで完結させる。

> 朝 9 時、本社 IT 室の data 担当者（シニア級）が Mattermost `#data-ops` で tier2 担当者からの schema 変更依頼メッセージに気付く。手元には `schema_migration.lock.yaml` と sqlx-cli の画面、Mattermost 越しに tier2 担当者と dual reviewer がいる。

## ペルソナ要約

主役: data 担当者（シニア級）、目的: schema migration を forward-only 規約と rollback 可能設計で安全に実施する

## 現状業務での痛み

- rollback 不可の migration が混入しており、本番デプロイ後に問題が発覚してもロールバックできない
- migration 手順が属人的で、担当者によって適用順序や確認手順が異なる
- migration 後のデータ整合性確認が手動で、問題の発見が遅れる

## k1s0 でこう変わる

- flyway / liquibase の forward-only 規約が CI で強制され、rollback 不可 migration の混入が merge 前に検知される
- migration 手順が schema_migration.lock.yaml で管理され、誰が実施しても同一品質が保証される
- migration 後の自動整合性チェックが CI に組み込まれ、データ問題が即時検知される

## Trigger（発火条件）

tier2 の Domain Event schema または DB schema（CloudNativePG）の変更が必要になった時。

## 想定頻度 / 典型きっかけ

想定頻度: 月次〜四半期。典型きっかけ: 「受注 table に `customer_segment` (varchar) フィールドを追加する要求が tier2 から来た。NOT NULL だが既存行には default 値を埋める必要がある」

## 主役 / 関与者

- 主役: data 担当者（シニア級）
- 関与: tier2 担当者（schema 変更要件の提示と dual-write / dual-read 実装）
- 関与: dual reviewer（sign-off）

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（data）| シニア | 本社 IT 室 | Perses / Mattermost `#data-ops` | schema 変更要件確認・expand-contract 各フェーズ主導・lock.yaml 更新 |
| 関与（tier2）| ミドル〜シニア | 本社 / リモート | Backstage TechDocs / Argo CD | dual-write / dual-read 実装・backfill 確認 |
| 承認（dual reviewer）| シニア | 本社 / リモート | Mattermost `#data-ops` | sign-off レビュー |

## 個人 KPI / 達成感

- rollback 不可 migration 混入 0 件が CI で定量確認でき、migration 品質向上の達成感を得られる
- migration 所要時間の短縮を lock.yaml で確認できるようになり、作業効率の改善を実感できる

## 工数 / 関与人数 / コスト感

- 工数: 半日〜1 日（migration 設計 2h + CI validation 確認 1h + lock.yaml 更新 1h）
- 関与人数: 2〜3 名（data 担当者・tier2 担当者・dual reviewer）
- コスト感: 低。CI で rollback 可能性が自動検証されるため設計コストが削減される

## 前提

- forward-only migration 方針が確立済み（backward-only ロールバックは禁止）
- expand-contract pattern が開発規約として定義済み
- `schema_migration.lock.yaml` が存在し、migration 履歴を記録済み
- Apicurio Registry が稼働し、compatibility check が CI に組み込み済み
- pg minor upgrade window（シナリオ 09）と重複する場合は upgrade を先行させてから migration を開始する。

## 流れ

1. 変更要件（field 追加 / 削除 / 型変更 / index 追加）を tier2 担当者から受領し、対象 schema と影響範囲を確認する
2. **expand phase**: 新 column / table を追加する。既存データには触らない。NOT NULL 制約は default 付き、または nullable から開始する
3. **dual-write phase**: アプリが旧 column と新 column の両方に書く。tier2 担当者が実装し、data 担当者がレビューする
4. **dual-read phase**: アプリが両 column を読み、新 column 優先 with 旧 column fallback の読み取りロジックに切り替える。tier2 担当者が実装する
5. **migrate phase**: `sqlx-cli` または `go-migrate` で既存データを新 column に backfill する。data 担当者が実施し、backfill 完了を行数で確認する
6. **contract phase**: 旧 column への write を止め、依存が 0 であることをコードおよびクエリログで確認する
7. **cleanup phase**: 旧 column を DROP する。forward-only のため rollback はなく、事前の expand フェーズで対処済みとする
8. Apicurio Registry の schema を同期更新し、compatibility check が green であることを確認する
9. dual reviewer sign-off を得たうえで `schema_migration.lock.yaml` を更新する

## Timeline

| T+ | actor | action | 通知例 |
|---|---|---|---|
| 0 | data 担当者 | schema 変更要件を確認し forward-only migration スクリプトを作成 | `migration スクリプト作成開始 / forward-only 規約確認` |
| 2h | data 担当者 | CI の rollback 可能性チェックと整合性テストを実行 | `CI validation green / rollback 可能性確認` |
| 4h | data 担当者 | staging で migration を適用し整合性確認後 PR 提出 | `staging migration green / PR #NNN 提出` |
| 1d | dual reviewer | migration スクリプトと整合性チェックを確認し sign-off | `sign-off 完了` |

## 業界 9 業務との紐付け

全 9 業務に共通基盤として影響（data は全業務の PostgreSQL / Kafka / ClickHouse の永続化基盤を担うため）。特に影響度が高い 2 業務:

- **受注管理**: 受注 table への `customer_segment` フィールド追加のように、受注ドメインの schema 変更が最多。dual-write / dual-read 期間中の write 整合が受注処理の信頼性に直結する。
- **品質検査**: 検査記録 table の schema 変更は GMP 監査要件と連動するため、expand-contract の全フェーズが GMP 準拠の証跡として lock.yaml に記録される必要がある。

## 関連適合仕様 / 関連 OSS

- データ保全適合仕様: [../../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md](../../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md)
- data 強制機構: [../../../04_詳細設計/02_強制機構/05_data強制機構.md](../../../04_詳細設計/02_強制機構/05_data強制機構.md)
- 関連 OSS: CloudNativePG / Apicurio Registry / sqlx-cli / go-migrate

## 期待結果 / 観測指標

- 全フェーズが順番通りに完了し、cleanup phase 後に旧 column が存在しないことを DDL で確認できる
- Apicurio Registry の compatibility check が green
- `schema_migration.lock.yaml` に今回の migration が記録済み
- dual reviewer 2 名の sign-off が記録済み

## 失敗時の挙動 / escalation

- **phantom write 発生**（旧スキーマで書いた後に新スキーマ移行）: rollback 不能の data 不整合。data 担当者 + tier2 担当者 + ops 担当者で Mattermost `#data-incident` に即時集合（SLA: 15 分以内）。Backstage runbook `schema-incident-response` を参照。postmortem は 3 営業日以内。
- **backfill 中断**: 中断箇所から resume できるよう、backfill job は idempotent に実装すること。resume できない場合は data 担当者が手動で中断行を再処理する。
- **compatibility check red**: cleanup phase を中断し、Registry の schema 定義を修正してから再実行する。

## 失敗パターン (anti-pattern)

- rollback 前提の migration: DROP COLUMN を直接実行する migration は CI の forward-only check が merge 阻止する
- staging スキップの本番直接適用: migration CI gate を迂回した本番適用は lock.yaml の tracking を破る

## 関連参照

- [data 担当者シナリオ index](./README.md) — data 担当者シナリオ全体の構成と preservation_class 一覧
- [データ保全適合仕様](../../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md) — 5 preservation_class の正典定義と restore_window SLO
- [data 強制機構](../../../04_詳細設計/02_強制機構/05_data強制機構.md) — schema migration の強制機構。forward-only violation の物理拒否を宣言
- [data 設計方針](../../../03_概要設計/06_data設計方針/README.md) — expand-contract pattern と forward-only migration の設計思想

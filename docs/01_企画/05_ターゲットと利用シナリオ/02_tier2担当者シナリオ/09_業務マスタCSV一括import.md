---
id: plan.tier2.scenario_master_csv_bulk_import
axis: tier2
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.tier2.tier2_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [B, C]
---

# 業務マスタ CSV 一括 import

## 一文方針

tier2 担当者が新規テナント onboarding 時または業界 pack 更新時に、業務マスタデータ（設備マスタ / 品目マスタ / 拠点マスタ / BOM 等）を CSV / Excel から bulk import し、テナント別 RLS FORCE と整合を維持する。

## Trigger（発火条件）

新テナント追加時 / 業界 pack の製品カタログ更新 / マスタデータの大規模改訂が必要になった時。

## 想定頻度 / 典型きっかけ

- 想定頻度: 月次（テナント onboarding 時）/ 不定期（大規模マスタ更新時）
- 典型きっかけ: 「`mfg-acme-jp` テナントが新工場を開設し、200 設備の設備マスタを CSV から一括 import する必要が生じた」
- 典型きっかけ 2: 「製造業 pack の品目マスタスキーマに新フィールド `hazmat_class` が追加され、既存全テナントに空値でデータを充填する必要が生じた」

## 主役 / 関与者

- 主役: tier2 担当者（中堅級）
- 関与: data 担当者（DB 操作）/ 業務管理者（マスタ内容の確認）
- 承認: dual reviewer（tier2 担当者 2 名、変更 PR の author 不可）

## 前提

- [テナント分離適合仕様](../../../04_詳細設計/01_適合仕様/10_テナント分離適合仕様.md)（RLS FORCE）が CloudNativePG に設定済みであること
- 業務マスタのスキーマが `schema_catalog.lock.yaml` に定義済みであること
- バリデーション規則（必須フィールド / 型 / 一意制約）が tier2 API に宣言済みであること

## 流れ

1. import 対象の CSV / Excel を受領し、スキーマ定義と照合してバリデーションを実施する
   - data-10（tenant_onboarding）が完了済みであること（RLS FORCE と DEK が設定済みの状態）を事前確認してから本 import を開始する。
   - 必須フィールドの欠落チェック
   - 型変換（文字列 → enum / 日付 / 数値）を確認
   - 重複チェック（一意キーの重複 row を事前排除）
2. import 用の tier2 Admin API エンドポイント（`POST /admin/v1/master/bulk-import`）を使用する（直接 SQL INSERT は禁止）
   - `tenant_id` を JWT claim から取得して API ヘッダに付与（成りすまし不可）
   - バッチサイズを 1000 件に分割して送信
3. 各バッチの import 結果を確認する（成功件数 / エラー件数 / エラー詳細）
4. エラー row は CSV に書き出し、業務管理者に確認依頼を送る（Mattermost `#master-import-<tenant>` チャンネル）
5. import 完了後に CloudNativePG で RLS FORCE が有効であることを確認する（`EXPLAIN ANALYZE SELECT * FROM equipment WHERE tenant_id = '...'` で RLS filter が付いていることを確認）
6. integration test（Testcontainers）で cross-tenant data leak がないことを確認する
7. 業務管理者の内容確認 + dual reviewer sign-off を取得して import を確定する

## 関連適合仕様 / 関連 OSS

- [テナント分離適合仕様](../../../04_詳細設計/01_適合仕様/10_テナント分離適合仕様.md)
- [tier2 強制機構](../../../04_詳細設計/02_強制機構/02_tier2強制機構.md)
- CloudNativePG（PostgreSQL RLS）
- Testcontainers（integration test）
- Backstage（Admin API portal）

## 期待結果 / 観測指標

- count: import 行数 = CSV 総行数（エラー行を除く）。import API レスポンスの `success_count` で確認
- rls: `EXPLAIN ANALYZE SELECT * FROM equipment WHERE tenant_id = '<tenant_id>'` で RLS filter が付いていること（`Filter: (tenant_id = current_setting('app.current_tenant')::uuid)` が出力される）
- ci: Testcontainers cross-tenant leak test all green（0 件の cross-tenant data アクセス）
- sign-off: 業務管理者の内容確認 + dual reviewer（tier2 担当者 2 名）sign-off 完了

## 失敗時の挙動 / escalation

- **RLS FORCE が効いておらず cross-tenant data が見えた**: import を即時中断。data 担当者 + security 担当者に Mattermost `#security-incident` で即時通報（**SLA: 15 分以内**）。Backstage runbook `rls-breach-response` を起動。**postmortem 期限: 2 営業日以内**。
- **バッチ途中で API エラー（500 系）**: import を中断し部分的に import 済みのデータをロールバック API（`DELETE /admin/v1/master/bulk-import/{import_id}`）で削除してから原因調査。data 担当者に Mattermost `#tier2-incident` で連絡（**SLA: 30 分以内**）。
- **バリデーションエラーが全体の 10% を超える**: import 全体を中断し業務管理者に CSV の修正を依頼（**SLA: 業務管理者が 2 営業日以内に修正版を提出**）。

## 関連参照

- [tier2 担当者シナリオ index](./README.md) — tier2 担当者シナリオ全体の構成と dual reviewer 規約
- [テナント別 override 拡張点](./03_テナント別override拡張点.md) — テナント onboarding 時に並行して実施する拡張点設定シナリオ
- [tenant_id 強制注入追加](./06_tenant_id強制注入追加.md) — RLS FORCE の設定手順との接続
- [tier2 設計方針](../../../03_概要設計/03_tier2設計方針/README.md) — 業務マスタの所有権と override 拡張点の設計指針

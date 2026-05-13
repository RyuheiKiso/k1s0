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
  proof_classes: []
---

# 業務マスタ CSV 一括 import

## 一文方針

tier2 担当者が新規テナント onboarding 時または業界 pack 更新時に、業務マスタデータ（設備マスタ / 品目マスタ / 拠点マスタ / BOM 等）を CSV / Excel から bulk import し、テナント別 RLS FORCE と整合を維持する。

> 朝 9 時、本社 IT 室の tier2 担当者（中堅級）が Mattermost `#master-import-mfg-acme-jp` で新工場の設備マスタ 200 件 CSV を受け取り、Backstage Admin API portal での import 作業を開始する。手元には import CSV・Testcontainers ローカル環境、Mattermost 越しに data 担当者と業務管理者がいる。

## ペルソナ要約

主役: tier2 担当者（中堅級）、目的: 新テナント onboarding 時や大規模マスタ更新時に業務マスタ CSV を一括 import しテナント別 RLS FORCE との整合を維持する

## 現状業務での痛み

- バリデーションエラーの多い CSV を受け取ってから確認が始まり、業務管理者との往復で import 完了まで数日かかる
- 直接 SQL INSERT が使われ RLS FORCE の設定確認が後回しになり、cross-tenant データアクセスが発生する
- バッチ途中でエラーが発生しても部分 import のロールバック手段が不明確で、データが不整合状態になる
- 業務管理者の確認プロセスが不透明で、import 確定のサインオフが取れないまま本番データが確定してしまう

## k1s0 でこう変わる

- Admin API エンドポイントを通じた import で RLS FORCE が常に有効な状態が保たれ、直接 SQL Insert を構造的に防止する
- Testcontainers cross-tenant leak test が import 完了後の green を確認し、RLS の有効性を物理証明する
- バリデーションエラーが 10% を超えた場合に import を自動中断し、業務管理者への修正依頼を標準フローとして定義する
- dual reviewer sign-off と業務管理者の内容確認を import 確定の必須条件として明文化する

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

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（tier2）| 中堅 | 本社 IT 室 | Mattermost `#tier2-ops` / Backstage Admin API portal | CSV バリデーション / bulk import 実行 / RLS 確認 |
| 関与（data）| シニア | 本社 / リモート | GitHub PR | DB 操作支援 / CloudNativePG RLS 設定確認 |
| 関与（業務管理者）| — | 本社 | Mattermost `#master-import-<tenant>` | マスタ内容確認 / 修正版 CSV 提出 |
| 承認（dual reviewer）| 中堅〜シニア | 本社 / リモート | GitHub PR | import 確定前レビュー / sign-off（author 不可） |

## 個人 KPI / 達成感

- import 行数 = CSV 総行数（エラー行を除く）の達成率
- Testcontainers cross-tenant leak test green（0 件）
- RLS FORCE 有効確認（EXPLAIN ANALYZE で RLS filter 付き）
- 業務管理者確認 + dual reviewer sign-off 取得

## 工数 / 関与人数 / コスト感

- 初回（200 件程度の設備マスタ import）: 半日〜1 日（バリデーション・バッチ import・RLS 確認・Testcontainers）、関与 4 名（主役 + data 担当者 + 業務管理者 + dual reviewer 2 名）
- 平常（100 件以下の小規模更新）: 2〜3h、関与 3 名
- 失敗時（RLS 不整合・バリデーションエラー多数・バッチ中断）: +半日〜1 日、関与 4〜5 名（+ security 担当者）

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

## Timeline

| T+ | actor | action | 通知例 |
|---|---|---|---|
| 0 | tier2 担当者 | CSV 受領・スキーマ照合バリデーション | `設備マスタ CSV 200 件 受領 / バリデーション開始 / エラー: 3 件` |
| 1h | 業務管理者 | バリデーションエラー確認・修正版 CSV 提出 | `修正版 CSV 提出 / エラー解消確認` |
| 2h | tier2 担当者 | Admin API バッチ import 実行（1000 件単位）| `バッチ import 完了 / success_count: 197 / エラー: 0` |
| 3h | tier2 担当者 / data 担当者 | RLS FORCE 確認・Testcontainers cross-tenant leak test | `RLS filter 確認: OK / Testcontainers: green (0 件)` |
| 1 日 | 業務管理者 + dual reviewer | 内容確認 + sign-off | `業務管理者確認済 / dual sign-off 完了` |

## 業界 9 業務との紐付け

- **在庫**: 品目マスタ・BOM の import 完了後、在庫 aggregate が正しい品目コードで初期化される。
- **受注**: 製品マスタの import により、受注 API で有効な品目コードとして受け付けられるようになる。
- **FA 生産指示・設備操作**: 設備マスタの import により、生産指示で参照可能な設備 ID が確定する。
- **計量装置・出荷指示**: 拠点マスタ・出荷先マスタの import により、出荷指示の宛先バリデーションが機能する。

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

## 失敗パターン (anti-pattern)

- **直接 SQL INSERT で import**: RLS FORCE が適用されない経路でデータが書き込まれ、cross-tenant アクセスが発生する。Admin API エンドポイント経由の import を唯一の正規経路として強制し、直接 SQL を構造的に禁止する。
- **バリデーションエラーを無視して強行 import**: 不正データが本番マスタに混入し、downstream の受注 / 生産指示が誤ったデータを参照する。バリデーションエラーが 10% を超えた時点で import を自動中断し、業務管理者に修正依頼を送るフローを標準化する。
- **業務管理者の確認なしに import を確定**: 担当者間の確認ミスで意図と異なるマスタが確定してしまい、修正コストが発生する。業務管理者の内容確認と dual reviewer sign-off を import 確定の物理前提条件として明文化する。

## 関連参照

- [tier2 担当者シナリオ index](./README.md) — tier2 担当者シナリオ全体の構成と dual reviewer 規約
- [テナント別 override 拡張点](./03_テナント別override拡張点.md) — テナント onboarding 時に並行して実施する拡張点設定シナリオ
- [tenant_id 強制注入追加](./06_tenant_id強制注入追加.md) — RLS FORCE の設定手順との接続
- [tier2 設計方針](../../../03_概要設計/03_tier2設計方針/README.md) — 業務マスタの所有権と override 拡張点の設計指針

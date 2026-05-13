---
id: plan.overview.scenario_business_admin_index
axis: overview
phase: plan
kind: index
status: draft
depends_on:
  - plan.target_use_case
  - plan.development_team_structure
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [A, B, C, D, E]
  proof_classes: []
---

# 業務管理者シナリオ INDEX

## 一文方針

業務管理者（シニア級）が Backstage プラグインと tier2 admin API を通じて踏む 12 シナリオを 1 ファイル 1 シナリオで列挙する。テナント別マスタ管理 / decision table override / 監査検索 / 緊急対応 / partner 連携設定が本軸の核心であり、schema migration 以上の変更は data 担当者へ escalation する判断境界を明確に持つ。

## 担当者プロフィール

| 属性 | 内容 |
|------|------|
| 職種 | 業務管理者（シニア級） |
| IT スキル | 業界知識 + tier2 admin API (Backstage プラグイン) |
| 業務知識 | マスタ管理 / 監査検索 / 決定表編集 / 緊急対応 / partner 連携設定 |
| 想定人数 | テナント単位（1 事業部 × 数名） |
| 主要デバイス | Windows PC / MacBook |
| UI 接点 | Backstage プラグイン（tier2 admin UI） |
| 認証手段 | パスワード + WebAuthn（FIDO2 セキュリティキー / Touch ID） |
| 責務 | テナント別マスタ / decision table override / 監査検索 / 緊急対応 / partner 連携 |

## 本シナリオ群の特徴と注意点

- **業務管理者は tier2 担当者（開発者）とは別人格。** 業務管理者が直接触れるのは Backstage プラグインで提供される管理 UI である。tier2 API の実装詳細は tier2 担当者シナリオへ委任する。
- 本シナリオは「業務管理者の目線」で記述する。tier2 内部実装の記述は最小限にとどめ、操作フロー・判断境界・エラー対応手順に集中する。
- schema migration を伴う変更は必ず data 担当者へ escalation する。この境界は業務管理者の決定権限外である。
- シナリオ間の依存関係は後述の「シナリオ間の依存関係」セクションで管理する。

## シナリオ一覧

| # | シナリオ名 | trigger | 想定頻度 | 主たる関連適合仕様 | 種別 |
|---|-----------|---------|---------|-----------------|------|
| 01 | [マスタ更新_品目仕入先BOM](01_マスタ更新_品目仕入先BOM.md) | マスタ変更申請が承認された時 | 週次 | スキーマ進化適合仕様 / テナント分離適合仕様 | [計画] |
| 02 | [監査ログ検索_PII_DSAR](02_監査ログ検索_PII_DSAR.md) | DSAR 申請 / 内部調査 / 規制照会が来た時 | 月次〜イベント駆動 | データ保全適合仕様 / audit_ingest_gap_monitor | [計画]+[緊急] |
| 03 | [緊急対応_設備停止承認](03_緊急対応_設備停止承認.md) | 現場担当者から設備緊急停止の承認依頼が来た時 | 月次〜イベント駆動 | 認証適合仕様 / security 強制機構 | [緊急] |
| 04 | [partner連携設定_IdP_federation](04_partner連携設定_IdP_federation.md) | 新規 partner との連携設定が必要になった時 | 四半期〜年次 | 認証適合仕様 / BFF_auth_edge | [計画] |
| 05 | [schedule編集_業務workflow](05_schedule編集_業務workflow.md) | 生産計画変更 / 季節変動 / 設備メンテナンスで schedule 変更が必要な時 | 月次 | テナント容量適合仕様 | [計画] |
| 06 | [decision_table_override編集](06_decision_table_override編集.md) | 業務ルール変更 / 季節閾値調整 / 規制対応で判定ルール変更が必要な時 | 月次〜四半期 | スキーマ進化適合仕様 / テナント分離適合仕様 | [計画] |
| 07 | [テナント_onboarding_offboarding](07_テナント_onboarding_offboarding.md) | 事業部追加 / 組織再編 / 撤退が発生した時 | 四半期〜年次 | テナント分離適合仕様 / テナント容量適合仕様 | [計画] |
| 08 | [警報escalation_政策変更](08_警報escalation_政策変更.md) | 担当者異動 / 夜間体制変更 / 新警報種別追加時 | 月次 | SLO 適合仕様 / ops 強制機構 | [計画] |
| 09 | [業務エラー解析集計](09_業務エラー解析集計.md) | 月次 review 前 / 業務エラー頻度急増通知受信時 | 月次 | クライアント状態適合仕様 / 観測適合仕様 | [周期] |
| 10 | [業界規制_自主点検](10_業界規制_自主点検.md) | 定期内部監査 / 外部監査準備 / 規制更新対応時 | 四半期〜年次 | データ保全適合仕様 / 検証規律適合仕様 | [周期] |
| 11 | [帳票レイアウト_業務UI変更要求](11_帳票レイアウト_業務UI変更要求.md) | 規制改正通知受信 / 業務部門から UI 改善要求発生時 | 四半期〜年次 | スキーマ進化適合仕様 / 検証規律適合仕様 | [計画] |
| 12 | [v2業界pack追加_要件取りまとめ](12_v2業界pack追加_要件取りまとめ.md) | 経営判断で v2 業界 pack 検討が始まった時 | 年次〜不定期 | (業界横断) | [計画] |

## 製造業 9 業務との対応

| 業務名 | 関連シナリオ |
|--------|------------|
| FA 生産指示 | 01, 05, 06 |
| ライン稼働監視 | 03, 08 |
| 品質検査結果配信 | 01, 06, 10 |
| SCADA テレメトリ収集 | 03, 08 |
| 図面 collaborative review | 11 |
| 在庫最新値 | 01, 05, 06 |
| 受注 sub | 01, 04, 05 |
| 警報配信 | 03, 08 |
| 計量装置連続データ | 06, 09 |

## 業務管理者の決定権限境界

業務管理者が **単独で実施できる** 操作と **escalation が必要な** 操作を明確に区別する。

| 操作 | 業務管理者の権限 | escalation 先 |
|------|----------------|--------------|
| マスタレコード値の更新（schema 変更なし） | 実施可 | — |
| decision table override 値の変更 | 実施可 | — |
| 業務 workflow schedule の変更 | 実施可 | — |
| 警報 escalation 政策の変更 | 実施可 | — |
| partner IdP federation 設定 | 実施可（tier2 担当者と協働） | — |
| テナント onboarding / offboarding | 実施可（tier2 担当者と協働） | — |
| **schema migration（カラム追加 / 型変更）** | **実施不可** | data 担当者 |
| **RLS policy 変更** | **実施不可** | data 担当者 + security 担当者 |
| **新規 API エンドポイント追加** | **実施不可** | tier2 担当者 |
| **crypto-shred 実行（offboarding 最終確認）** | **実施不可** | data 担当者 + security 担当者 |

## 新規参画者向けオンボーディング

1. まず 01 (マスタ更新) を読み、業務管理者が日常的に行う最頻操作を把握する。
2. 次に 06 (decision_table_override 編集) で判定ルール変更の判断軸を確認する。
3. 監査対応の核心は 02 (DSAR 監査ログ検索) と 10 (業界規制_自主点検) で確認する。
4. 緊急対応フローは 03 (設備停止承認) で WebAuthn step_up と audit emit の流れを把握する。
5. partner 連携は 04 (IdP federation) で claim mapping の判断ポイントを確認する。
6. テナント lifecycle 全体は 07 (onboarding_offboarding) で確認する。
7. 技術実装の詳細は tier2 担当者シナリオへ進む。

## シナリオ間の依存関係

```
01 (マスタ更新)
  └─► 06 (decision_table_override 編集) — マスタ変更が判定閾値に影響する場合

02 (監査ログ検索_DSAR)
  └─► 10 (業界規制_自主点検) — 外部監査準備で監査ログ抽出が先行する

03 (緊急対応_設備停止承認)
  └─► 08 (警報 escalation 政策変更) — 緊急対応後に政策見直しが発生することがある

07 (テナント onboarding_offboarding)
  ├─► 01 (マスタ更新) — onboarding 後に初期マスタ投入が発生
  └─► 04 (partner 連携設定) — onboarding 時に partner IdP federation を設定

09 (業務エラー解析集計)
  └─► 11 (帳票レイアウト_業務 UI 変更要求) — エラー頻度分析から UI 改善要求を起票

10 (業界規制_自主点検)
  └─► 11 (帳票レイアウト_業務 UI 変更要求) — 規制対応で帳票変更要求が発生

12 (v2 業界 pack 追加_要件取りまとめ)
  └─► (全シナリオ参照) — 業界横断視点で全 11 シナリオの gap を整理
```

## 関連参照

- `docs/01_企画/05_ターゲットと利用シナリオ/02_tier2担当者シナリオ/` — 業務管理者が依頼する先の tier2 担当者シナリオ
- `docs/01_企画/05_ターゲットと利用シナリオ/README.md` — ターゲット・利用シナリオ全体 index
- `arch.tier2.tier2_index` — tier2 アーキテクチャ index
- `req.team.tier_engineer_requirement` — tier 担当者要件

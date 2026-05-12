---
id: plan.overview.scenario_business_operator_index
axis: overview
phase: plan
kind: index
status: draft
depends_on:
  - plan.target_use_case
  - arch.tier3.tier3_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [A, B, C, D]
  proof_classes: []
---

# 業務担当者シナリオ INDEX

## 一文方針

業務担当者 (工場現場 / 事務所 / 倉庫の現場スタッフ) が 1.0.0 ship 対象の製造業 9 業務を通じて踏む 13 シナリオを 1 ファイル 1 シナリオで列挙する。25h オフライン / BusinessConflict UX / Companion 端末 / WebAuthn step_up / レガシー ERP 並行入力が本軸の核心である。

## 担当者プロフィール

| 属性 | 内容 |
|------|------|
| 職種 | 工場現場スタッフ / 事務所担当 / 倉庫作業員 |
| IT スキル | プログラミング不要。業務 UI の操作のみ |
| 業務知識 | 担当ライン・工程の現場業務知識を保有 |
| 主要デバイス | iPad / Windows tablet / Windows PC (WinForms ERP 端末含む) |
| UI 接点 | tier3 担当者が開発した業務 SPA / Companion / WinForms ERP |
| 認証手段 | パスワード + WebAuthn (Touch ID / セキュリティキー + PIN) |
| 勤務環境 | 工場フロア (騒音・防塵) / 事務所 / 倉庫 |

## 本シナリオ群の特徴と注意点

- **業務担当者は tier3 担当者 (開発者) とは別人格。** 業務担当者が直接触れるのは tier3 が開発した業務 UI である。技術的な詳細 (API 設計 / proto スキーマ / bidi 実装) は tier3 担当者シナリオへ委任する。
- 本シナリオは「業務担当者の目線」で記述する。tier3 内部実装の記述は最小限にとどめ、UX・操作フロー・エラー対応手順に集中する。
- 25h オフライン / BusinessConflict UX / Companion / WebAuthn step_up / レガシー ERP 並行入力は業務担当者が直面する固有課題であり、本シナリオ群で優先的に扱う。
- シナリオ間の依存関係は後述の「シナリオ間の依存関係」セクションで管理する。

## シナリオ一覧

| # | シナリオ名 | trigger | 想定頻度 | 主たる関連適合仕様 | 種別 |
|---|-----------|---------|---------|-----------------|------|
| 01 | 出社_朝礼_ライン稼働確認 | 毎朝の出社時 | 日次 | Bidi 適合仕様 / SLO 適合仕様 | [周期] |
| 02 | 検査結果入力_オンライン | 検査工程完了時 | 日次 (多数回) | クライアント状態適合仕様 / 認証適合仕様 | [周期] |
| 03 | 25h_オフライン業務記録 | 25h 超オフライン業務発生時 | 月次〜四半期 | クライアント状態適合仕様 / Bidi 適合仕様 | [周期]+[緊急] |
| 04 | 業務エラー対応_BusinessConflict4種 | BusinessConflict が SPA に表示された時 | 週次〜日次 | クライアント状態適合仕様 / スキーマ進化適合仕様 | [緊急] |
| 05 | 発注承認_WebAuthn_step_up | 発注承認 workflow 到達時 | 日次〜週次 | 認証適合仕様 / BFF_auth_edge | [周期] |
| 06 | 承認回付_業務workflow | 検査不良 / 出荷保留 / 品質逸脱記録時 | 週次 | テナント分離適合仕様 / SLO 適合仕様 | [周期] |
| 07 | 設備リモート操作_v1_interactive | 設備の遠隔操作指示が必要な時 | 日次 | Bidi 適合仕様 / SLO 適合仕様 | [周期]+[緊急] |
| 08 | 計量装置連続データ送信_Companion | 計量ライン稼働開始時 | 日次 | Tauri_companion_sidecar / Bidi 適合仕様 | [周期] |
| 09 | 警報受信_アラート対応 | 警報配信受信時 | 週次〜月次 | Bidi 適合仕様 / SLO 適合仕様 | [緊急] |
| 10 | 図面collaborative_review | 図面設計変更 / 承認依頼到着時 | 週次〜月次 | Bidi 適合仕様 / クライアント状態適合仕様 | [計画] |
| 11 | 帰宅前_未送信件数確認 | 業務終了時 | 日次 | クライアント状態適合仕様 / 観測適合仕様 | [周期] |
| 12 | legacy_NetFx_ERP_並行入力 | ERP 側への入力が必要な業務時 | 日次 | dotnet8_connect_inhouse / HTTP2_enforcement | [周期] |
| 13 | break_glass発火時の現場対応 | break-glass 発火時 | 半年〜年次 | 認証適合仕様 / security 強制機構 | [緊急] |

## 製造業 9 業務との対応

| 業務名 | 関連シナリオ |
|--------|------------|
| FA 生産指示 | 01, 07 |
| ライン稼働監視 | 01, 07, 09 |
| 品質検査結果配信 | 02, 03, 04, 06 |
| SCADA テレメトリ収集 | 07, 08, 09 |
| 図面 collaborative review | 10 |
| 在庫最新値 | 05, 06 |
| 受注 sub | 05, 06 |
| 警報配信 | 09, 13 |
| 計量装置連続データ | 08 |

## 新規参画者向けオンボーディング

1. まず 01 (出社_朝礼_ライン稼働確認) を読み、業務担当者が毎日触れる基本 UX を把握する。
2. 次に 02 (検査結果入力_オンライン) で標準的な入力フローを確認する。
3. オフライン対応の核心は 03 (25h_オフライン業務記録)。IndexedDB encrypted + Idempotency-Key TTL の UX を理解する。
4. 04 (BusinessConflict4種) で競合解消 UX の 4 パターンを把握する。
5. 認証強化は 05 (WebAuthn step_up) と 13 (break_glass) で確認する。
6. レガシー ERP との共存は 12 (legacy_NetFx_ERP_並行入力) で確認する。
7. 技術実装の詳細は tier3 担当者シナリオへ進む。

## シナリオ間の依存関係

```
01 (朝礼稼働確認)
  └─► 07 (設備リモート操作) — ライン稼働確認後に操作が発生

02 (検査結果入力_オンライン)
  ├─► 03 (25h_オフライン業務記録) — オフライン時の代替フロー
  ├─► 04 (BusinessConflict4種) — 競合発生時の分岐
  └─► 06 (承認回付_業務workflow) — 検査不良時の後続フロー

05 (発注承認_WebAuthn_step_up)
  └─► 13 (break_glass発火時の現場対応) — 緊急時の認証強化

08 (計量装置連続データ送信_Companion)
  └─► 09 (警報受信_アラート対応) — 異常値検出時の警報発火

11 (帰宅前_未送信件数確認)
  └─► 03 (25h_オフライン業務記録) — 翌日 sync 計画に接続
```

## 関連参照

- `docs/01_企画/05_ターゲットと利用シナリオ/03_tier3担当者シナリオ/` — 業務 UI 実装側のシナリオ
- `docs/01_企画/05_ターゲットと利用シナリオ/README.md` — ターゲット・利用シナリオ全体 index
- `arch.tier3.tier3_index` — tier3 アーキテクチャ index
- `req.team.tier_engineer_requirement` — tier 担当者要件

---
id: plan.target_use_case
axis: overview
phase: plan
kind: index
status: draft
depends_on:
  - plan.background_purpose
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# ターゲットと利用シナリオ

## 一文方針
- 1.0.0 ship のターゲットは製造業 pack を消費する中堅以上の製造業企業（多拠点 / 多テナント / 多業務領域）。利用シナリオは 9 業務（FA 生産指示 / 進捗実績 / 検査結果 / SCADA / 図面 review / 在庫 / 受注 / 警報配信 / 計量装置）+ 25h オフライン現場業務 + レガシー .NET Framework 並行運用。

## ターゲット企業像
- 中堅以上の製造業（従業員 500 人以上、複数工場 / 複数事業部）
- 既存基幹（ERP / MES / SCADA / 検査機器）との並行運用が必要
- 多拠点 / 多テナント（テナント = 事業部 / 工場 / 子会社）
- 規制対応（ISO 9001 / 医薬品 GMP / 食品 HACCP / 環境 ISO 14001）
- IT 部門 + 業務部門 + 現場担当者の 3 階層運用

## 利用ペルソナ
- **業務担当者（tier3 業務 UI 利用）**: 工場現場 / 事務所 / 倉庫の業務操作
- **業務管理者（Backstage プラグイン利用）**: マスタ管理 / 監査検索 / 緊急対応 / schedule 編集
- **業務エンジニア（tier3 開発）**: ジュニア級、業務 UI 開発に集中
- **プラットフォーム運営者（infra / data / security / ops 軸運用）**: シニア級、cluster 運用 / KEK shamir custodian
- **外部監査人**: audit hash chain + 外部公証 attestation を介した独立検証

## 製造業 9 業務シナリオ（v1.0.0 stress test 対象）
1. 設備リモート操作（v1_interactive、bidi 双方向）
2. ライン稼働監視 live tile（v1_live_snapshot、latest-wins）
3. 品質検査結果配信（v1_event_feed、順序 + replay）
4. SCADA テレメトリ収集（v1_bulk_upload、at-least-once）
5. 図面 collaborative review（v1_interactive、双方向 presence）
6. 在庫最新値表示（v1_live_snapshot）
7. 受注 sub（基幹 → 製造管理）（v1_event_feed）
8. 警報配信（v1_alert、低 lag + replay）
9. 計量装置連続データ（v1_bulk_upload、continuous push）
10. 出荷指示双方向確認（v1_interactive）

## オフライン業務シナリオ（25h オフライン best-effort）
- 工場現場の検査担当が React SPA で 5 件オフライン記録 → 復帰
- 25h オフライン → Idempotency-Key TTL 超過 → user 確認 → 新 key で resend
- 4 layer state + IndexedDB encrypted で透過

## レガシー資産統合シナリオ
- レガシー .NET Framework 4.8 ERP が NuGet で `k1s0.Library.NetFx` を組込み、CLR Profiler で観測 attach、Companion 経由で送信（[レガシー資産統合](../../03_概要設計/04_tier3設計方針/09_レガシー資産統合.md)）
- HTTP/1.1 + SSE（v1_legacy_http11 専用 listener、別ポート 8443-legacy）
- per-tab 6 subscription で縮退

## 認証 / 認可シナリオ
- 業務担当 SPA login + WebAuthn step_up → 発注承認（v1_human_session + step_up）
- 親会社 IdP federation で取引先パートナーアクセス（v1_federated_exchange）
- 障害対応 break-glass で本番 DB cluster-admin 取得（v1_emergency_step_up）+ 強制 audit emit

## 並行編集シナリオ（BusinessConflict subtype）
- stale_write: 並行編集（actor A: 数量 / actor B: 納期）→ field-level rebase + auto resend
- lost_update: 並行編集（同 field）→ 3-way merge UI
- supersede: 同 actor 後続 op 検出 → silent toast
- concurrent_edit: presence indicator → user choice

## v1.0.0 出荷後の拡張（v2 候補）
- 業界 pack 追加（金融業 / サービス業 / 医療業）
- iOS / Android ネイティブ（Swift / Kotlin の tier1 / tier2 言語追加が先行）
- HTTP/3 + WebTransport の standard 化追従
- Continuous Profiling client-side（Parca v2）
- post-quantum 暗号

## 担当者別シナリオ
- [tier1 担当者シナリオ](01_tier1担当者シナリオ/README.md)
- [tier2 担当者シナリオ](02_tier2担当者シナリオ/README.md)
- [tier3 担当者シナリオ](03_tier3担当者シナリオ/README.md)
- [infra 担当者シナリオ](04_infra担当者シナリオ/README.md)
- [data 担当者シナリオ](05_data担当者シナリオ/README.md)
- [security 担当者シナリオ](06_security担当者シナリオ/README.md)

## 関連参照
- [背景と目的](../01_背景と目的/README.md)
- [提供する価値や体験](../02_提供する価値や体験/README.md)
- [業界 pack 戦略](../07_業界pack戦略/README.md)

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

1.0.0 ship のターゲットは製造業 pack を消費する中堅以上の製造業企業（多拠点 / 多テナント / 多業務領域）。利用シナリオは 9 業務（FA 生産指示 / 進捗実績 / 検査結果 / SCADA / 図面 review / 在庫 / 受注 / 警報配信 / 計量装置）+ 25h オフライン現場業務 + レガシー .NET Framework 並行運用。至高路線として、運用コスト度外視・規律最大化の判断を全 11 ペルソナの要求水準に反映する。

## ターゲット企業像

- 中堅以上の製造業（従業員 500 人以上、複数工場 / 複数事業部）
- 既存基幹（ERP / MES / SCADA / 検査機器）との並行運用が必要
- 多拠点 / 多テナント（テナント = 事業部 / 工場 / 子会社）
- 規制対応（ISO 9001 / 医薬品 GMP / 食品 HACCP / 環境 ISO 14001）
- IT 部門 + 業務部門 + 現場担当者の 3 階層運用

## 利用ペルソナ（11 ペルソナ詳細表）

| # | ペルソナ | 級 | 想定人数 | 必須スキル概要 | 担当軸 |
|---|---------|----|---------|--------------|----|
| 1 | 業務担当者（工場 / 倉庫 / 事務所） | — | 多数（現場比例） | プログラミング不要、業務操作のみ | [07_業務担当者シナリオ](07_業務担当者シナリオ/README.md) |
| 2 | 業務管理者（Backstage プラグイン利用） | — | テナント単位 | 業界知識 + tier2 admin API + マスタ管理 / 監査検索 | [08_業務管理者シナリオ](08_業務管理者シナリオ/README.md) |
| 3 | 外部監査人 | — | 年次 1-2 回 | 公認会計士 / 監査法人 + 業界規制監査経験 + audit hash chain / RFC 3161 / Sigstore Rekor | [09_外部監査人シナリオ](09_外部監査人シナリオ/README.md) |
| 4 | ops 担当者（SRE） | シニア | 3-5 名 | SRE 50% rule / SLO / MWMBR / Argo Rollouts / Backstage TechDocs / Tekton | [10_ops担当者シナリオ](10_ops担当者シナリオ/README.md) |
| 5 | 法務 / DPO 担当者 | — | 2-3 名 | GDPR / 個人情報保護法 / 業界規制 / OSS ライセンス | [11_法務DPO担当者シナリオ](11_法務DPO担当者シナリオ/README.md) |
| 6 | tier1 担当者 | シニア | 5-8 名 | Rust / Go / C# / TypeScript / proto / Buf / OSS lifecycle | [01_tier1担当者シナリオ](01_tier1担当者シナリオ/README.md) |
| 7 | tier2 担当者 | 中堅 | 5-10 名 | 4 言語 + ドメイン業務 / 業界 pack | [02_tier2担当者シナリオ](02_tier2担当者シナリオ/README.md) |
| 8 | tier3 担当者 | ジュニア | 業界 pack × 業務領域 × テナント数で増減 | TypeScript + React / C# WPF / .NET Framework + WinForms / Tauri | [03_tier3担当者シナリオ](03_tier3担当者シナリオ/README.md) |
| 9 | infra 担当者 | シニア | 3-5 名 | Kubernetes / Argo CD / Istio / Calico BGP / Envoy / OpenBao / Cosign / Litmus | [04_infra担当者シナリオ](04_infra担当者シナリオ/README.md) |
| 10 | data 担当者 | シニア | 3-5 名 | PostgreSQL / CloudNativePG / Kafka / Strimzi / ClickHouse / Apicurio / Valkey / Rook+Ceph | [05_data担当者シナリオ](05_data担当者シナリオ/README.md) |
| 11 | security 担当者 | シニア | 2-3 名 | 5⁴ cell threat model / KEK shamir / OAuth 2.1 + DPoP / OWASP | [06_security担当者シナリオ](06_security担当者シナリオ/README.md) |

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

## 担当者別シナリオ（11 軸ナビゲーション）

### 既存 6 軸

- [tier1 担当者シナリオ](01_tier1担当者シナリオ/README.md) — Rust / Go / C# / TypeScript, OSS lifecycle, Server/Library/Companion
- [tier2 担当者シナリオ](02_tier2担当者シナリオ/README.md) — ドメイン業務, 業界 pack, BusinessConflict, テナント別 override
- [tier3 担当者シナリオ](03_tier3担当者シナリオ/README.md) — 業務 UI, オフライン UX, レガシー統合, WebAuthn step_up
- [infra 担当者シナリオ](04_infra担当者シナリオ/README.md) — cluster 構築/運用, topology_class, clock_integrity, GitOps
- [data 担当者シナリオ](05_data担当者シナリオ/README.md) — preservation_class, restore_drill, DR, PII/DSAR
- [security 担当者シナリオ](06_security担当者シナリオ/README.md) — 5⁴ threat model, KEK ceremony, supply chain, audit chain

### 新規 5 軸（Phase 1 追加）

- [業務担当者シナリオ](07_業務担当者シナリオ/README.md) — オフライン記録, BusinessConflict UI, 警報対応, WebAuthn 承認
- [業務管理者シナリオ](08_業務管理者シナリオ/README.md) — Backstage マスタ更新, 監査検索, partner federation, 規制証跡エクスポート
- [外部監査人シナリオ](09_外部監査人シナリオ/README.md) — Sigstore Rekor 検証, WORM 改竄不可確認, SLSA L3+ provenance, shamir ceremony 観察
- [ops 担当者シナリオ](10_ops担当者シナリオ/README.md) — MWMBR alerting, on-call rotation, postmortem PR, runbook 統合, toil 削減
- [法務 DPO 担当者シナリオ](11_法務DPO担当者シナリオ/README.md) — GDPR 72h 通知, crypto-shred 証跡, OSS ライセンス CI enforce, 業界規制証跡

## 横断メタドキュメント

- [00_ペルソナ詳細カード集](00_ペルソナ詳細カード集.md) — 11 ペルソナ × 痛み / After / KPI / タイムライン

## v1.0.0 出荷後の拡張（v2 候補）

- 業界 pack 追加（金融業 / サービス業 / 医療業）
- iOS / Android ネイティブ（Swift / Kotlin の tier1 / tier2 言語追加が先行）
- HTTP/3 + WebTransport の standard 化追従
- Continuous Profiling client-side（Parca v2）
- post-quantum 暗号

## 関連参照

- [背景と目的](../01_背景と目的/README.md)
- [提供する価値や体験](../02_提供する価値や体験/README.md)
- [業界 pack 戦略](../07_業界pack戦略/README.md)
- [開発体制](../08_開発体制/README.md)

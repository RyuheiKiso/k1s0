---
id: req.functional.tier2_functional
axis: tier2
phase: requirement
kind: requirement
status: draft
depends_on:
  - req.functional.functional_index
  - arch.tier2.tier2_index
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# tier2 機能カテゴリ要件

## 一文方針
- tier2 はドメイン業務共通化を 4 抽象化レベル（業界横断 / 業界共通 / 業界固有 / テナント固有）+ bounded context 分割 + 業務資産所有 + atomic 三表書込 + BusinessConflict subtype 4 種で機能要件として宣言する。

## 主要機能カテゴリ
- **業界 pack 並立**: 製造業 pack 1.0.0 ship、業界中立性 3 種機械的担保 CI green
- **bounded context（製造業）**: FA / 調達 / 検査 + 共通モジュール（設備 / ロット / 品目 / 拠点 / BOM）
- **マルチテナント**: 共有 Pod / 共有 DB default、行レベル `tenant_id` 列分離、専用 Service への自動昇格
- **atomic 三表書込**: State change + Outbox + Audit を同一 DB トランザクション
- **業務資産所有 + override 拡張点**: tier3 が override で業務固有差分を表現
- **BusinessConflict subtype 4 種**: stale_write / lost_update / supersede / concurrent_edit
- **権限モデル**: Keycloak ABAC + Delegation + Emergency Override
- **業務エラー監査コンプライアンス**: hash chain + 外部公証 attestation + audit_local schema PII 不在
- **データライフサイクル**: onboarding / export / offboarding / bulk / 品質
- **外部システム統合**: External Proto + Companion + Webhook
- **状態遷移パターン**: 4 言語等価強度の compile-time 不正遷移防止

## 詳細設計参照
- [tier2 設計方針](../../03_概要設計/03_tier2設計方針/README.md)（20 方針）
- [テナント分離適合仕様](../../04_詳細設計/01_適合仕様/10_テナント分離適合仕様.md)
- [tier2 強制機構](../../04_詳細設計/02_強制機構/02_tier2強制機構.md)（8 層）

## 受入条件
- 製造業 pack の FA / 調達 / 検査 全業務 production ready
- 第二業界 stub conformance test green
- atomic 三表書込 P1〜P4 invariant property test green
- BusinessConflict subtype 4 種の UI 分岐 1:1 固定が CI green

## 関連参照
- [機能要件 index](README.md)
- [tier2 設計方針 index](../../03_概要設計/03_tier2設計方針/README.md)

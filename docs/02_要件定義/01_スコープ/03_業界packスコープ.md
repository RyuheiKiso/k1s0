---
id: req.scope.industry_pack_scope
axis: overview
phase: requirement
kind: requirement
status: draft
depends_on:
  - req.overview.provided_scope
  - plan.industry_pack_strategy
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# 業界 pack スコープ

## 一文方針
- 1.0.0 ship では製造業 pack のみを出荷する。業界並立構造（[業界拡張モデル](../../03_概要設計/03_tier2設計方針/02_業界拡張モデル.md)）は day-1 から有効、業界中立性 3 種機械的担保（命名禁則 / 依存方向 / 第二業界 stub conformance）を CI 必須化する。

## 1.0.0 出荷業界 pack
- **製造業 pack**: FA / 調達 / 検査 ドメイン + 共通モジュール（設備 / ロット / 品目 / 拠点 / BOM）
- 業界規制対応: ISO 9001 / 医薬品 GMP / 食品 HACCP / 環境 ISO 14001（pack 単位で宣言）

## 業界 pack 構造（要件レベル）
- 業界横断層: 通知 / 監査 / 承認 workflow / 帳票 / マスタ管理 / ユーザ・組織管理 / テナント識別子伝播
- 業界 pack: 業界横断層を消費し、業界固有の業務資産を提供
- 単方向依存: 業界 pack → 業界横断層（CI 物理 enforce）

## 業界中立性 3 種機械的担保（必須要件）
1. 命名禁則: 業界横断層に業界固有語ゼロ（CI で禁止語リスト照合）
2. 依存方向: 業界横断層 → 業界 pack の依存ゼロ（言語別 lint）
3. 第二業界 stub conformance: サービス業 stub pack で業界横断層 API consumption が green

## v2 候補業界 pack
- 金融業 / 医療業 / サービス業 / 物流業

## 受入条件（1.0.0 ship blocker）
- 製造業 pack の FA / 調達 / 検査 全業務が完成
- 業界中立性 3 種機械的担保 CI green
- 業界横断 / 業界共通 / 業界固有 / テナント固有 の 4 抽象化レベル分離が CI 検査で green
- 第二業界 stub での conformance test green

## 採用しない設計
- 業界横断層に業界固有概念を置く
- 業界 pack 間の相互依存
- 1.0.0 で複数業界 pack 同時 ship（業界横断層検証深度確保のため製造業のみ）

## 関連参照
- [提供スコープ](01_提供スコープ.md)
- [非提供スコープ](02_非提供スコープ.md)
- [tier2 業界拡張モデル](../../03_概要設計/03_tier2設計方針/02_業界拡張モデル.md)
- [業界 pack 戦略](../../01_企画/07_業界pack戦略/README.md)

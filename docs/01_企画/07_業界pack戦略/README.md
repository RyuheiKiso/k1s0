---
id: plan.industry_pack_strategy
axis: overview
phase: plan
kind: index
status: draft
depends_on:
  - plan.background_purpose
  - plan.target_use_case
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# 業界 pack 戦略

## 一文方針
- 1.0.0 で製造業 pack のみを ship するが、業界拡張モデル（[tier2 業界拡張モデル](../../03_概要設計/03_tier2設計方針/02_業界拡張モデル.md)）は day-1 から有効、製造業 pack 自身も本モデルの規約に従って実装する。新業界 pack の追加は purely additive、業界横断層は単方向依存 + 第二業界 stub conformance で構造的に enforce。

## 1.0.0 ship 業界 pack
- **製造業 pack**: FA / 調達 / 検査 ドメイン + 共通モジュール（設備 / ロット / 品目 / 拠点 / BOM）
- 業務語彙: 設備 / ロット / 品目 / 拠点 / 検査結果 / 不良票 / 生産指示 / 進捗実績 / 発注 / 検収 / 仕入先 / 図面 / SCADA / 計量装置
- 業界規制: ISO 9001 / 医薬品 GMP / 食品 HACCP / 環境 ISO 14001（pack 単位で宣言）

## 業界 pack 並立構造
- 業界横断層: 通知 / 監査 / 承認 workflow / 帳票 / マスタ管理 / ユーザ・組織管理 / テナント識別子伝播
- 業界 pack: 業界横断層を消費し、業界固有の業務資産を提供
- 分離方向: 業界 pack → 業界横断層の単方向（CI で依存方向検査）

## 業界中立性の機械的担保（3 種すべて採用、day-1 から有効）
1. **命名禁則の機械検査**: 業界横断層に業界固有語（Manufacturing / Factory / Lot / Inspection 等）が出現したら CI fail
2. **依存方向の機械検査**: 業界横断層 → 業界 pack の依存ゼロを CI で物理 enforce
3. **第二業界 stub conformance**: サービス業を想定した最小 stub pack を CI 専用で保持、業界横断層 API が stub からも消費できることを merge 条件

## 新業界 pack 追加手順
1. bounded context を [ドメイン分割方針](../../03_概要設計/03_tier2設計方針/04_ドメイン分割方針.md) に従って設計
2. 業界横断層に追加が必要な API があれば、業界中立な形で追加（命名禁則・依存方向・stub 検査を満たす）
3. 業界 pack を独立 module として実装（他業界 pack への依存ゼロ）
4. tier3 への提供形態を [Service 運用方針](../../03_概要設計/03_tier2設計方針/07_Service運用方針.md) に従って決定
5. tier3 移行支援（既存 tier3 から新業界 pack への移行）を互換性ポリシーに従って整備

## v2 候補業界 pack
- **金融業 pack**: 勘定 / 取引 / 決済 / 与信 / 規制対応（SOX / Basel III 等）
- **医療業 pack**: 患者 / 診療 / 処方 / 検査 / HIPAA 対応
- **サービス業 pack**: 顧客 / 契約 / 請求 / 業務 workflow / SLA
- **物流業 pack**: 配送 / 在庫 / 倉庫 / 配車 / 追跡

## 業界 pack の owned by
- 業界 pack の業務資産（Domain Event / Workflow / 決定表 / 業務マスタ / DB schema）は tier2 が所有
- テナント別差分は override 拡張点 + decision table override layer で表現
- 業界 pack 単位の version は tier2 互換性ポリシーに従う

## 1.0.0 出荷時の業界 pack 1 つだけ ship する理由
- 業界並立構造を day-1 で固めるため（出荷物が 1 業界の時こそ業界固有概念が業界横断層に漏れる圧力が最大）
- 製造業に特化することで深い業界知識を投入可能
- v2 以降、第二業界 stub の知見を活かして他業界 pack を追加

## 業界 pack 追加の経済合理性
- 新業界 pack 追加は tier1 / tier2 業界横断層への影響最小化（CI で物理 enforce）
- tier3 業務 UI は業界 pack 単位、別 tier3 として開発
- 既存 tier3 への影響は extension point + 第二業界 stub conformance で吸収

## 採用しない戦略
- 業界横断層に業界固有概念を置く（CI で物理拒否）
- 業界 pack 間の相互依存
- 1.0.0 で複数業界 pack 同時 ship（業界横断層の検証深度を確保するため、製造業のみで day-1 構造を固める）

## 関連参照
- [背景と目的](../01_背景と目的/README.md)
- [ターゲットと利用シナリオ](../05_ターゲットと利用シナリオ/README.md)
- [tier2 業界拡張モデル](../../03_概要設計/03_tier2設計方針/02_業界拡張モデル.md)
- [tier2 抽象化レベル](../../03_概要設計/03_tier2設計方針/03_抽象化レベル.md)
- [tier2 ドメイン分割方針](../../03_概要設計/03_tier2設計方針/04_ドメイン分割方針.md)

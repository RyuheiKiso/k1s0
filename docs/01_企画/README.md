---
id: plan.plan_index
axis: overview
phase: plan
kind: index
status: draft
depends_on: []
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# 01_企画

## 一文方針
- 本フェーズは企画レベルの 9 ドキュメントで構成。背景と目的 / 提供する価値や体験 / 競合との差別化 / 法務確認 / ターゲットと利用シナリオ / OSS 公開戦略 / 業界 pack 戦略 / 開発体制 / 用語集 を一覧化する。

## 9 ドキュメント

| # | ドキュメント | 役割 |
|---|---|---|
| 01 | [背景と目的](01_背景と目的/README.md) | 課題認識 / 19 軸同型構造 / 至高路線 |
| 02 | [提供する価値や体験](02_提供する価値や体験/README.md) | 業務エンジニア / 運用エンジニア / プラットフォーム所有者の 3 ペルソナへの価値 |
| 03 | [競合との差別化](03_競合との差別化/README.md) | 商用 BaaS / OSS-first low-code との差別化、L1+ 単一深耕の哲学 |
| 04 | [法務確認](04_法務確認/README.md) | OSS ライセンス階層 / linkage_model / GDPR / 各国規制 |
| 05 | [ターゲットと利用シナリオ](05_ターゲットと利用シナリオ/README.md) | 中堅以上製造業 / 9 業務シナリオ / 25h オフライン / レガシー統合 |
| 06 | [OSS 公開戦略](06_OSS公開戦略/README.md) | `v1_inhouse_authoritative` 区画 OSS の Apache 2.0 公開、CLA + dual sign-off |
| 07 | [業界 pack 戦略](07_業界pack戦略/README.md) | 1.0.0 製造業のみ ship、業界並立構造 day-1 有効、業界中立性 3 種機械的担保 |
| 08 | [開発体制](08_開発体制/README.md) | 5 階層エンジニア + 4 横断軸専任 + プラットフォーム運営者 + 業務管理者 |
| 09 | [用語集](09_用語集/README.md) | 19 軸 / 5 階層論 / L1+ / 業界 pack / 4 layer state 等の主要術語 |

## 1.0.0 ship スコープ（再掲）
- 業界 pack: 製造業のみ（業界並立構造は day-1 から有効）
- アプリケーション形態: Web SPA / デスクトップ exe / レガシー .NET Framework の 3 形態
- 19 軸全てが完成、release_gate.lock.yaml の全 cell green、cosign signed tag が物理 prerequisite

## 関連参照
- [02_要件定義](../02_要件定義/README.md)
- [03_概要設計](../03_概要設計/README.md)
- [04_詳細設計](../04_詳細設計/README.md)

---
id: req.functional.functional_index
axis: overview
phase: requirement
kind: index
status: draft
depends_on:
  - req.overview.provided_scope
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# 機能要件 index

## 一文方針
- 本ディレクトリは機能要件を tier1 / tier2 / tier3 / クライアント SDK / 運用 UI の 5 機能カテゴリで集約する。各カテゴリの機能は提供スコープ（[01_提供スコープ](../01_スコープ/01_提供スコープ.md)）と各軸の概要設計に集約済の機能を要件 view として再定義する。

## 5 機能カテゴリ
- [01_tier1 機能カテゴリ](01_tier1機能カテゴリ.md) — 17 機能カテゴリ × 3 抽象化レベル（L3 / L2\* / L1+）
- [02_tier2 機能カテゴリ](02_tier2機能カテゴリ.md) — 業界 pack + ドメイン分割 + 業務資産所有
- [03_tier3 機能カテゴリ](03_tier3機能カテゴリ.md) — 3 アプリケーション形態 × 業務シナリオ
- [04_クライアント SDK 機能](04_クライアントSDK機能.md) — 5 distribution_class × 9 言語 SDK
- [05_運用 UI 機能](05_運用UI機能.md) — 8 軸別運用 UI + 開発者体験統合

## 1.0.0 ship blocker（機能要件）
- 5 機能カテゴリの全 cell が green
- 製造業 pack 9 業務シナリオ stress test 全 green
- `release_gate.lock.yaml` の機能要件 cells 全 green

## 関連参照
- [提供スコープ](../01_スコープ/01_提供スコープ.md)
- [非提供スコープ](../01_スコープ/02_非提供スコープ.md)
- [03_概要設計](../../03_概要設計/README.md)

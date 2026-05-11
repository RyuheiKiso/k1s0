---
id: req.constraint.constraint_index
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

# 制約と前提 index

## 一文方針
- 本ディレクトリは 1.0.0 ship の制約と前提を OS 固有依存例外 / レガシー資産統合制約 / クラウド非依存制約 の 3 カテゴリで集約する。

## 3 ドキュメント
- [01_OS 固有依存例外](01_OS固有依存例外.md) — APNs / FCM / DPAPI / Keychain / Secret Service の OS native API 例外
- [02_レガシー資産統合制約](02_レガシー資産統合制約.md) — .NET Framework 4.6.2+ Companion 経路、`v1_legacy_http11` 別ポート listener
- [03_クラウド非依存制約](03_クラウド非依存制約.md) — vendor lock-in 回避、cloud managed service 不採用

## 関連参照
- [提供スコープ](../01_スコープ/01_提供スコープ.md)
- [非提供スコープ](../01_スコープ/02_非提供スコープ.md)

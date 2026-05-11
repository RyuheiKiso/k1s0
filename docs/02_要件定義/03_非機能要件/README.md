---
id: req.non_functional.non_functional_index
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

# 非機能要件 index

## 一文方針
- 本ディレクトリは非機能要件を SLO / 可用性災害対策 / セキュリティ / 性能容量 / 互換性 / 観測可能性 / 監査コンプライアンス / アクセシビリティ国際化 / 検証規律 + 運用要件 の 9 + 1 カテゴリで集約する。

## 9 + 1 非機能要件
- [01_SLO 要件](01_SLO要件.md) — 6 slo_class × instance × MWMBR alert
- [02_可用性災害対策](02_可用性災害対策.md) — 5 topology_class × 5 preservation_class × restore_drill
- [03_セキュリティ要件](03_セキュリティ要件.md) — 5⁴ cell threat model + 8 secret class + 5 build_provenance_class
- [04_運用要件](04_運用要件.md) — 5 signal_class × 5 phase ops loop + on-call + postmortem
- [05_互換性ポリシー](05_互換性ポリシー.md) — SemVer + 並行バージョン提供 + 移行コミットメント
- [06_観測可能性要件](06_観測可能性要件.md) — 5 signal class + 4 dimension layer + trace_id 統一
- [07_監査コンプライアンス](07_監査コンプライアンス.md) — hash chain + 外部公証 + WORM + 7 年〜10 年 retention
- [08_アクセシビリティ国際化](08_アクセシビリティ国際化.md) — WCAG 2.1 AA + i18n + Web Vitals
- [09_検証規律要件](09_検証規律要件.md) — 18 軸 × 5 verification_class = 90 cell coverage
- [10_性能容量要件](10_性能容量要件.md) — 5 quota_class × 4 階層 enforcement + noisy neighbor isolation

## 1.0.0 ship blocker（非機能要件）
- 全 SLO の MWMBR alert 配備 + error budget 物理 enforcement
- 5 preservation_class × 4 種 restore_drill green
- 製造業 pack 9 stress test 全 green
- WCAG 2.1 AA / Web Vitals 閾値内
- 90 cell coverage matrix green

## 関連参照
- [提供スコープ](../01_スコープ/01_提供スコープ.md)
- [03_概要設計](../../03_概要設計/README.md)

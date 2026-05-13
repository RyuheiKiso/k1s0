---
id: plan.overview.implementation_roadmap_index
axis: overview
phase: plan
kind: index
status: draft
depends_on:
  - plan.plan_index
  - plan.background_purpose
  - plan.target_use_case
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# 10_実装ロードマップ

## 一文方針
- 本セクションは k1s0 を 1.0.0 ship まで導く 11 実装 Phase の定義・release_gate 対応・軸内依存順序・cross-cutting blocking 関係を単一の真として保持する。

## 4 ドキュメント

| # | ドキュメント | 役割 |
|---|---|---|
| 01 | [実装 Phase 定義](01_実装Phase定義.md) | 11 Phase の成果物・完了判定・至高路線根拠 |
| 02 | [release_gate 対応表](02_release_gate対応表.md) | release_gate 20 cell と Phase の 1:1 対応 |
| 03 | [軸内実装順序](03_軸内実装順序.md) | tier1 sub-component・infra class・data class の依存順序 |
| 04 | [crosscutting blocking 表](04_crosscutting_blocking表.md) | 13 cross-cutting 適合仕様の blocking 軸・難度・P7 compile 強制技術 |

## 関連参照
- [01_企画 index](../README.md)
- [19 軸論](../../03_概要設計/01_アーキテクチャ概観/02_19軸論.md)
- [軸間依存図](../../03_概要設計/01_アーキテクチャ概観/05_軸間依存図.md)
- [release_gate 体系](../../04_詳細設計/05_lock_yaml体系/03_release_gate体系.md)

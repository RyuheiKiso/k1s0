---
id: arch.overview.architecture_index
axis: overview
phase: architecture
kind: index
status: draft
depends_on:
  - plan.background_purpose
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# アーキテクチャ概観

## 一文方針
- 本企画は 5 階層論 + 19 軸同型構造 + defense-in-depth 6 層 + 5 proof_class + 軸間依存図 の 5 視点でアーキテクチャ全体を表現する。各軸は同型構造（dimension override 禁止 / dead spec 殺し / build artifact 化 / 5 層 defense-in-depth）を持ち、軸間接合は 19 軸間依存マップで機械的に検証可能。

## 5 視点
- [01_5 階層論](01_5階層論.md) — infra / data / tier1 / tier2 / tier3 の 5 階層構造、責務 / 隠蔽境界 / 依存方向
- [02_19 軸論](02_19軸論.md) — 10 主要軸 + 9 cross-cutting / meta 軸 / 適合仕様軸の 19 軸同型構造
- [03_defense_in_depth](03_defense_in_depth.md) — 6 層（A: compile / B: lint / C: integration test / D: runtime / E: 物理 / F: 数学的）の cross-axis 横断
- [04_5 proof_class 論](04_5proof_class論.md) — temporal_safety / liveness / refinement / program_correctness / runtime_modelcheck の 5 proof_class
- [05_軸間依存図](05_軸間依存図.md) — 19 軸間の depends_on / cross-cut / proof_obligation の依存マップ

## アーキテクチャ哲学
- **L1+ 単一深耕 + 移行コミットメント**: 複数 OSS 同時サポートを諦める代わりに、単一 OSS の全機能 + 移行 toolchain
- **dimension override 禁止**: 各軸の class bundle で dimension を一意に導出、override 経路なし
- **dead spec を CI で殺す**: 参照消失で CI fail
- **build artifact 化**: lock.yaml / 機械可読 spec を build script で生成、手書き drift 禁止
- **物理 enforcement**: 文章運用ではなく CI / lint / Kyverno / HSM / 外部公証で物理 enforce

## 軸の同型構造
全 19 軸が次の同型構造を持つ:
1. **位置づけ**: 他軸との接合関係を明示
2. **設計原則**: bundle / dimension override 禁止 / dead spec 殺し / defense-in-depth
3. **v1 class セット（5-6 class）**: 軸 enum を YAML 宣言
4. **dimension の根拠**: 各 dimension が独立次元である理由
5. **derived dimension を独立次元にしない理由**: class bundle で吸収する dimension の説明
6. **単一の真（3-5 ファイル構成）**: classes.yaml / scenarios.yaml / *.lock.yaml
7. **5 層 defense-in-depth**: A〜E（formal は + F）
8. **CI 不変条件**: 整合 1〜N、merge 不可
9. **1.0.0 ship blocker**: release_gate.lock.yaml AND-gate
10. **製造業 pack stress test**: 各軸の正当性証跡

## 1.0.0 ship blocker
- 全 19 軸の release_gate.lock.yaml の cell が green
- cosign signed tag が物理 prerequisite
- 4 primary pair の dry_run.lock.yaml の last_green_at が 365 日以内
- 製造業 pack 9 stress test 全 green
- formal proof 95 cell（proof_matrix 基底）の verified or accepted_with_assumption

## 関連参照
- [背景と目的](../../01_企画/01_背景と目的/README.md)
- [提供スコープ](../../02_要件定義/01_スコープ/01_提供スコープ.md)
- [tier1 設計方針](../02_tier1設計方針/README.md)
- [12_クロスカッティング設計](../12_クロスカッティング設計/README.md)
- [05_lock_yaml 体系](../../04_詳細設計/05_lock_yaml体系/README.md)

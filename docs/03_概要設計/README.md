---
id: arch.architecture_index
axis: overview
phase: architecture
kind: index
status: draft
depends_on:
  - req.requirement_index
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# 03_概要設計

## 一文方針
- 本フェーズは概要設計レベルの 12 ディレクトリで構成。アーキテクチャ概観（5 視点）+ 10 軸別設計方針 + クロスカッティング設計（8 機構）の cross-product で構成、全軸が同型構造（class bundle / dimension override 禁止 / dead spec 殺し / build artifact 化 / 5 層 defense-in-depth）を持つ。

## 12 ディレクトリ

### [01_アーキテクチャ概観](01_アーキテクチャ概観/README.md)
- [01_5 階層論](01_アーキテクチャ概観/01_5階層論.md) — infra / data / tier1 / tier2 / tier3 の 5 階層責務 / 隠蔽境界 / 依存方向
- [02_19 軸論](01_アーキテクチャ概観/02_19軸論.md) — 10 主要軸 + 9 cross-cutting / meta 軸 / 適合仕様軸
- [03_defense_in_depth](01_アーキテクチャ概観/03_defense_in_depth.md) — 6 層（A〜F）の cross-axis 横断
- [04_5 proof_class 論](01_アーキテクチャ概観/04_5proof_class論.md) — temporal_safety / liveness / refinement / program_correctness / runtime_modelcheck
- [05_軸間依存図](01_アーキテクチャ概観/05_軸間依存図.md) — 19 軸間の depends_on / cross-cut / proof_obligation

### 10 軸別設計方針
| # | 軸 | ディレクトリ |
|---|---|---|
| 02 | tier1 | [02_tier1設計方針](02_tier1設計方針/README.md) |
| 03 | tier2 | [03_tier2設計方針](03_tier2設計方針/README.md) |
| 04 | tier3 | [04_tier3設計方針](04_tier3設計方針/README.md) |
| 05 | infra | [05_infra設計方針](05_infra設計方針/README.md) |
| 06 | data | [06_data設計方針](06_data設計方針/README.md) |
| 07 | security | [07_security設計方針](07_security設計方針/README.md) |
| 08 | ops | [08_ops設計方針](08_ops設計方針/README.md) |
| 09 | client | [09_client設計方針](09_client設計方針/README.md) |
| 10 | test | [10_test設計方針](10_test設計方針/README.md) |
| 11 | formal | [11_formal設計方針](11_formal設計方針/README.md) |

### [12_クロスカッティング設計](12_クロスカッティング設計/README.md)
- [01_認証コンテキスト伝播](12_クロスカッティング設計/01_認証コンテキスト伝播.md)
- [02_観測コンテキスト伝播](12_クロスカッティング設計/02_観測コンテキスト伝播.md)
- [03_スキーマ進化](12_クロスカッティング設計/03_スキーマ進化.md)
- [04_鍵管理](12_クロスカッティング設計/04_鍵管理.md)
- [05_OSS ライフサイクル](12_クロスカッティング設計/05_OSSライフサイクル.md)
- [06_Bidi 適応経路](12_クロスカッティング設計/06_Bidi適応経路.md)
- [07_時刻整合 HLC](12_クロスカッティング設計/07_時刻整合HLC.md)
- [08_数学的 enforcement](12_クロスカッティング設計/08_数学的enforcement.md)

## 設計原則（全軸共通）
- **L1+ 単一深耕 + 移行コミットメント**: 複数 OSS 同時サポートを諦める代わりに、単一 OSS の全機能 + 移行 toolchain
- **dimension override 禁止**: class bundle で他 dimension を一意に導出
- **dead spec を CI で殺す**: 参照消失で CI fail
- **build artifact 化**: lock.yaml は build script で生成、手書き drift 禁止
- **defense-in-depth 5 層（A〜E、formal は + F）**

## 関連参照
- [01_企画](../01_企画/README.md)
- [02_要件定義](../02_要件定義/README.md)
- [04_詳細設計](../04_詳細設計/README.md)

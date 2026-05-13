---
id: arch.formal.formal_index
axis: formal
phase: architecture
kind: index
status: draft
depends_on: []
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# formal 設計方針 index

## 一文方針
- 本フォルダは formal 軸（19 軸目 + meta-meta-layer）の概要設計を集約する。formal は tier1 / tier2 / tier3 / infra / data / security / ops / test の 8 階層 × 19 軸 grid を直交に貫く `proof_class ⊗ axis ⊗ phase ⊗ 層` invariant の管掌 layer であり、本フォルダの 8 方針メモが 5 proof_class の各方針 + proof artifact / counter-example / proof review の運用方針を物理 enforce 可能な形で宣言する。

## 至高路線における立ち位置
- formal は 5 層 defense-in-depth の最上層に「層 F: 数学的（proof certificate）」を追加する。
    - 層 A: compile（type check）
    - 層 B: lint（policy check）
    - 層 C: integration test（実行検証）
    - 層 D: runtime（admission webhook + drill）
    - 層 E: 物理（cosign / Kyverno / Object Lock）
    - 層 F: 数学的（proof certificate）← formal が担当
- 「実行で検出されない潜在反例」を数学的に impossible にする。Amazon S3 / DynamoDB / TigerBeetle / CockroachDB が運用している路線の OSS 投影。
- 「proof は完成が難しいから後回し」「critical invariant の証明は v2 で良い」を全て禁止し、`proof_status.lock.yaml` の `all_critical_verified` を 1.0.0 ship blocker とする（CLAUDE.md「段階的 release 禁止」「機能削減なし」）。

## test 軸との同型直交
- test 軸: 「実行 artifact」（pact / Playwright trace / chaos workflow result / mutation report）を 18 axis × 5 verification_class = 90 cell に詰める。
- formal 軸: 「machine-checkable certificate」（TLA+ / Apalache / P / Stainless / Dafny / Lean / Kani / CBMC）を 19 axis × 5 proof_class = 95 cell に詰める。
- 両者は同型だが直交。bind: formal の `v1_temporal_safety_proof` / `v1_program_correctness_proof` で verified された invariant は test 軸の `v1_property_axiom` の seed pool に物理転写される。drift は CI fail。

## 配下ドキュメント

| 番号 | ドキュメント | proof_class | tool |
|---|---|---|---|
| 01 | [時相安全性方針](01_時相安全性方針.md) | v1_temporal_safety_proof | TLA+ + Apalache |
| 02 | [時相活性方針](02_時相活性方針.md) | v1_temporal_liveness_proof | TLA+ + Apalache |
| 03 | [精緻化方針](03_精緻化方針.md) | v1_refinement_proof | TLA+ + Apalache / Stainless |
| 04 | [プログラム正当性方針](04_プログラム正当性方針.md) | v1_program_correctness_proof | Stainless / Dafny / Lean 4 |
| 05 | [実行時モデル検査方針](05_実行時モデル検査方針.md) | v1_runtime_modelcheck_proof | Kani / CBMC |
| 06 | [proof_artifact 方針](06_proof_artifact方針.md) | -（横断） | - |
| 07 | [counter_example 方針](07_counter_example方針.md) | -（横断） | - |
| 08 | [proof_review 方針](08_proof_review方針.md) | -（横断） | - |

## 5 proof_class × 19 軸 = 95 cell の構造
- 全 cell が `proof_inventory.lock.yaml` に entry を持ち、`proof_status.lock.yaml` の `cell_state` が `v1_baseline_verified` または `v1_accepted_with_assumption` であることを 1.0.0 ship blocker として CI 不変条件化。
- cell 詳細は [形式検証適合仕様](../../04_詳細設計/01_適合仕様/20_形式検証適合仕様.md) で structural spec として宣言。

## 採用 OSS（v1_l1plus_primary 単一深耕）

| 機能カテゴリ | OSS | proof_class 主担当 |
|---|---|---|
| temporal logic（safety + liveness） | TLA+ + Apalache | v1_temporal_safety_proof / v1_temporal_liveness_proof / v1_refinement_proof |
| state machine code-gen | P language | v1_temporal_safety_proof（補助）/ v1_refinement_proof（補助） |
| program logic (Scala) | Stainless | v1_program_correctness_proof / v1_refinement_proof |
| program logic (multi-lang) | Dafny | v1_program_correctness_proof |
| proof assistant (math + crypto) | Lean 4 + mathlib | v1_program_correctness_proof |
| runtime model check (Rust) | Kani | v1_runtime_modelcheck_proof |
| runtime model check (C / C++) | CBMC | v1_runtime_modelcheck_proof |

詳細: [採用 OSS 一覧](../../02_要件定義/04_技術選定/01_OSS採用一覧.md) の formal 軸セクション。

## 上位フェーズへの依存
- [検証規律要件](../../02_要件定義/03_非機能要件/09_検証規律要件.md): formal 軸の要件側 view
- [OSS 採用一覧](../../02_要件定義/04_技術選定/01_OSS採用一覧.md): 採用 OSS の根拠

## 下位フェーズへの委譲
- [形式検証適合仕様](../../04_詳細設計/01_適合仕様/20_形式検証適合仕様.md): structural spec、build artifact 化
- [formal 強制機構](../../04_詳細設計/02_強制機構/10_formal強制機構.md): 5 層 defense-in-depth + Kyverno admission policy
- [formal 運用 UI](../../04_詳細設計/04_運用UI開発者体験/07_formal運用UI.md): Perses dashboard + ChatOps + IDE plugin
- [proof_artifact 体系](../../04_詳細設計/05_lock_yaml体系/01_proof_artifact体系.md): lock.yaml catalog
- [counter_example 体系](../../04_詳細設計/05_lock_yaml体系/02_counter_example体系.md)
- [release_gate 体系](../../04_詳細設計/05_lock_yaml体系/03_release_gate体系.md)

## 横断軸との bind
- [test 設計方針](../10_test設計方針/README.md): 双方向 lock（property test seed ↔ proof obligation）
- [security 設計方針](../07_security設計方針/README.md): cryptographic assumption の整合
- [ops 設計方針](../08_ops設計方針/README.md): operational loop の formal proof obligation

## 読み筋
- 初読: README.md → 01_時相安全性方針 → 04_プログラム正当性方針 → 06_proof_artifact 方針
- 実装時参照: 各 proof_class 方針（01〜05）→ [形式検証適合仕様](../../04_詳細設計/01_適合仕様/20_形式検証適合仕様.md)
- 障害対応時参照: 07_counter_example 方針 → [formal 強制機構](../../04_詳細設計/02_強制機構/10_formal強制機構.md) の break-glass 経路

## 関連参照
- [親フォルダ index](../README.md)
- [規約層 index](../../00_format/README.md)

---
id: arch.cross_cutting.mathematical_enforcement
axis: overview
phase: architecture
kind: policy
status: draft
depends_on:
  - arch.cross_cutting.cross_cutting_index
  - arch.overview.five_proof_class_theory
covered_by:
  defense_in_depth_layers: [F]
  proof_classes:
    - v1_temporal_safety_proof
    - v1_temporal_liveness_proof
    - v1_refinement_proof
    - v1_program_correctness_proof
    - v1_runtime_modelcheck_proof
---

# 数学的 enforcement

## 一文方針
- formal 軸が 5 proof_class（temporal_safety / liveness / refinement / program_correctness / runtime_modelcheck）で他 18 軸の不変条件を数学的 proof として転写し、95 cell coverage + counter-example 物理 closure + reviewer dual sign-off + cosign signature + 外部公証 attestation の 6 機構で物理 enforce する。defense-in-depth 層 F として全軸の最終 safety net。

## 5 proof_class の対象軸

| proof_class | OSS | 主要 obligation 元 |
|---|---|---|
| temporal_safety_proof | TLA+ + Apalache | tier2 atomic 三表書込 P1〜P4 / tier1 Bidi conformance / 鍵管理 KEK shamir threshold |
| temporal_liveness_proof | TLA+ + Apalache | tier3 pending queue resume completion / data restore_drill RTO / Workflow completion |
| refinement_proof | TLA+ / Stainless | tier1 移行 pair 旧→新 / Connect-RPC ↔ paired_post_sse 等価性 |
| program_correctness_proof | Stainless / Dafny / Lean 4 + mathlib | tier1 Library API contract / KEK shamir threshold algebra / HLC happens-before |
| runtime_modelcheck_proof | Kani / CBMC | infra eBPF / HSM driver / PTP daemon / Rust 実装 memory safety |

## 95 cell coverage
- temporal_safety_proof: 19 cell
- temporal_liveness_proof: 12 cell
- refinement_proof: 35 pair
- program_correctness_proof: 19 cell
- runtime_modelcheck_proof: 14 cell
- 合計: 99 cell（予備 4）

## counter-example の物理 closure
- 4 close_kind: `fixed_in_code` / `fixed_in_spec` / `accepted_as_bug` / `scope_narrowed`
- severity 別 close_due_at（high=14d / medium=30d / low=90d）
- `regression_corpus.lock.yaml`（test 軸）への双方向 lock
- accepted_as_bug cap ≤ 10 件、cap 越えは破壊的変更扱い

## reviewer dual sign-off
- reviewer pool: core team 4 名 + domain expert 各 2 名
- cosign signature 物理 enforce
- LLM 補助 reviewer は最大 1 名（人間 1 名以上必須）
- LLM 単独 sign-off 禁止

## assumption 管理
- `assumption.lock.yaml` に明示登録された assumption（DDH / RSA hardness / discrete log 等）の下で proof 成立
- assumption cap ≤ 20 件
- 文献 pointer（DOI 含む）必須

## 5 enforcement orchestrator
- `tlaplus_apalache_pipeline`
- `p_pgo_pipeline`
- `stainless_dafny_pipeline`
- `lean_mathlib_pipeline`
- `kani_cbmc_pipeline`
- 共通: `proof_gate_kyverno`

## proof_event subject の hash chain
- cryptographic chaining + WORM retention 10〜15 year
- audit hash chain と同型の物理機構

## 採用しない proof
- 「文章のみで証明済み」（natural language proof / 論文引用のみ）
- 商用 proof assistant（Coq Inria / Isabelle UK 商用 / F\* / SPARK Pro）
- SMT solver の単独運用（Z3 / CVC5 を直接 driver で叩く）
- LLM 単独 sign-off
- 単一 reviewer sign-off
- counter-example の close_due_at 延長

## 形式検証の長期収斂
- 19 軸の不変条件 を 5 proof_class で完全 cover
- 95 cell の 1.0.0 ship 後の維持は cadence（mathlib monthly / TLA+ / Apalache 6 month / Lean 4 6 month）
- v2 候補: post-quantum 暗号 proof / ZK proof 連携 / WASM verifier 統合

## 関連参照
- [5 proof_class 論](../01_アーキテクチャ概観/04_5proof_class論.md)
- [defense_in_depth](../01_アーキテクチャ概観/03_defense_in_depth.md)
- [formal 設計方針](../11_formal設計方針/README.md)
- [形式検証適合仕様](../../04_詳細設計/01_適合仕様/20_形式検証適合仕様.md)
- [proof_artifact 体系](../../04_詳細設計/05_lock_yaml体系/01_proof_artifact体系.md)
- [counter_example 体系](../../04_詳細設計/05_lock_yaml体系/02_counter_example体系.md)

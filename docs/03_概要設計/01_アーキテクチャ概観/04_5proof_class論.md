---
id: arch.overview.five_proof_class_theory
axis: overview
phase: architecture
kind: policy
status: draft
depends_on:
  - arch.overview.architecture_index
  - arch.overview.defense_in_depth
covered_by:
  defense_in_depth_layers: [F]
  proof_classes:
    - v1_temporal_safety_proof
    - v1_temporal_liveness_proof
    - v1_refinement_proof
    - v1_program_correctness_proof
    - v1_runtime_modelcheck_proof
---

# 5 proof_class 論

## 一文方針
- formal 軸は 5 proof_class（temporal_safety / temporal_liveness / refinement / program_correctness / runtime_modelcheck）で他軸の不変条件を数学的 proof として転写する。95 cell coverage（19 + 12 + 35 + 19 + 14）で 1.0.0 ship 時に verified または accepted_with_assumption green、counter-example は close_due_at 内に物理 closure。

## 5 proof_class

### v1_temporal_safety_proof
- **OSS**: TLA+ + Apalache（symbolic model checker）
- **対象**: 時相論理上の safety property（「悪いことが起きない」）
- **典型 obligation**: tenant 越境ゼロ / atomic 三表書込 P1〜P4 / KEK shamir M-of-N threshold / Bidi conformance class invariant
- **cell 数**: 19 cell

### v1_temporal_liveness_proof
- **OSS**: TLA+ + Apalache
- **対象**: 時相論理上の liveness property（「良いことが必ず起きる」）
- **典型 obligation**: pending queue resume completion / restore_drill RTO 内収束 / failover 完了 / Workflow 完了率
- **cell 数**: 12 cell

### v1_refinement_proof
- **OSS**: TLA+ / Stainless（refinement type）
- **対象**: refinement relation（spec → impl の意味論的等価性）
- **典型 obligation**: dual-write / migration pair の旧 backend ↔ 新 backend / Connect-RPC ↔ paired_post_sse 等価性
- **cell 数**: 35 pair

### v1_program_correctness_proof
- **OSS**: Stainless（Scala 系）/ Dafny（multi-lang）/ Lean 4 + mathlib（math + crypto）
- **対象**: program logic 正当性（Hoare logic、refinement type）
- **典型 obligation**: KEK shamir threshold algebra / HLC happens-before / Library API contract
- **cell 数**: 19 cell

### v1_runtime_modelcheck_proof
- **OSS**: Kani（Rust）/ CBMC（C / C++）
- **対象**: bounded model checking（runtime 実装の bounded depth 検証）
- **典型 obligation**: Rust 実装の memory safety / concurrency safety / overflow / kernel module / eBPF / HSM driver / PTP daemon
- **cell 数**: 14 cell

## 95 cell coverage
95 cell = **19 軸 × 5 proof_class の matrix cell 数**（19 × 5 = 95）。

各 proof_class の対象 obligation 件数（軸横断合計）は次の通り:
- temporal_safety_proof: 19 件
- temporal_liveness_proof: 12 件
- refinement_proof: 35 件（refinement pair の数が多い）
- program_correctness_proof: 19 件
- runtime_modelcheck_proof: 14 件
合計 obligation 件数: 99 件（複数 obligation が 1 cell に束ねられる場合あり）

これに cross-cutting / meta の追加 cell が加わり、`proof_inventory.lock.yaml` で全 coverage を固定する。各 cell は次のいずれかの状態:
- **verified**: proof artifact が CI で green、reviewer dual sign-off 済
- **accepted_with_assumption**: assumption.lock.yaml に明示登録された assumption の下で verified
- **counter_example_open**: counter-example が close_due_at 内（high=14d / medium=30d / low=90d）

## counter-example の物理 closure
- 4 close_kind: `fixed_in_code` / `fixed_in_spec` / `accepted_as_bug` / `scope_narrowed`
- severity 別 close_due_at（high=14d / medium=30d / low=90d）
- `regression_corpus.lock.yaml`（test 軸）への双方向 lock

## proof reviewer dual sign-off
- reviewer pool: core team 4 名 + domain expert 各 2 名
- cosign signature 物理 enforce
- LLM 補助 reviewer は最大 1 名（人間 1 名以上必須）

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

## 1.0.0 ship blocker
- 95 cell の verified or accepted_with_assumption（counter_example_open は close_due_at 内）
- proof reviewer dual sign-off 完備
- mathlib_pin / tla_apalache_pin / kani_cbmc_pin の lock artifact green
- accepted_as_bug cap ≤ 10 件
- assumption cap ≤ 20 件
- termination_exempt annotation cap ≤ 10 件

## 採用しない設計
- 文章のみで証明済み（natural language proof / 論文引用のみ）
- LLM 単独 sign-off
- 単一 reviewer sign-off
- counter-example の close_due_at 延長
- assumption / accepted_as_bug の cap 越え

## 関連参照
- [アーキテクチャ概観 README](README.md)
- [defense_in_depth](03_defense_in_depth.md)
- [formal 設計方針](../11_formal設計方針/README.md)
- [形式検証適合仕様](../../04_詳細設計/01_適合仕様/20_形式検証適合仕様.md)
- [proof_artifact 体系](../../04_詳細設計/05_lock_yaml体系/01_proof_artifact体系.md)
- [counter_example 体系](../../04_詳細設計/05_lock_yaml体系/02_counter_example体系.md)

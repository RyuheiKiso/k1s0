---
id: detail.formal.proof_artifact_system
axis: formal
phase: detail
kind: detail
status: draft
depends_on:
  - arch.formal.proof_artifact_policy
covered_by:
  defense_in_depth_layers: [A, B, D, E, F]
  proof_classes:
    - v1_temporal_safety_proof
    - v1_temporal_liveness_proof
    - v1_refinement_proof
    - v1_program_correctness_proof
    - v1_runtime_modelcheck_proof
lock_artifacts:
  - proof_inventory.lock.yaml
  - proof_status.lock.yaml
  - proof_matrix.lock.yaml
  - assumption.lock.yaml
  - mathlib_pin.lock.yaml
  - tla_apalache_pin.lock.yaml
  - kani_cbmc_pin.lock.yaml
  - ownership_table.lock.yaml
trace:
  fr_ids:
  - FR-formal-004

---

# proof_artifact 体系

## 一文方針
- 全 proof artifact（TLA+ `.tla` / Apalache `.json` / P `.p` / Stainless `.stainless` / Dafny `.doo` / Lean `.olean` / Kani `.json` / CBMC `.json`）は build artifact として generator から生成、cosign signed、Ceph RGW Object Lock Compliance mode で retention 物理 delete 不可、`proof_status.lock.yaml` に fingerprint されることを物理 enforce する。

## 設計の物理転写
- 本詳細設計は [proof_artifact 方針](../../03_概要設計/11_formal設計方針/06_proof_artifact方針.md) を物理 artifact として完全転写する。
- 文章 only の合意は禁止。全規律が `*.lock.yaml` + Tekton Pipeline + cosign signature + Ceph RGW Object Lock の 4 物理層に落とし込まれる。

## artifact / lock.yaml 体系

### `proof_inventory.lock.yaml`
- 生成元: 各軸 spec の `formal_obligations` 段落から generator が生成
- 入力:
    - 全軸 spec の `formal_obligations` 段落
    - `formal_classes.yaml`
    - `axis_registry.lock.yaml`
- 出力 schema（各 entry）:
    - `obligation_id`: 軸内一意な ID（例: `InvBidiHandshake`）
    - `axis_id`: 19 軸の ID（例: `07_transport`）
    - `proof_class`: 5 class enum
    - `tool_kind`: `["tlaplus_apalache" | "p_language" | "stainless" | "dafny" | "lean" | "kani" | "cbmc"]`
    - `statement_hash`: obligation 文（自然言語 + formal）の SHA-256 hash
    - `tool_version`: 採用 tool の version pin（pin file への参照）
    - `expected_certificate_hash`: 検証通過時の certificate の expected hash（再現性検証用）
    - `artifact_pointer`: proof artifact 本体の path / git ref / cosign signature pointer
    - `cross_axis_links`: 関連する他軸の `obligation_id` list
    - `assumption_refs`: `assumption.lock.yaml` の `assumption_id` list
    - `fairness_assumption`（liveness のみ）: 自然言語 + TLA+ 表現
    - `pair_id` / `S_high_pointer` / `S_low_pointer` / `mapping_sketch`（refinement のみ）
    - `function_name` / `pre` / `post` / `invariant` / `termination_metric`（program correctness のみ）
    - `module_name` / `entry_point` / `bound_parameter`（runtime modelcheck のみ）
- enforce 経路:
    - 層 A: jsonschema / CUE による schema 検証
    - 層 B: Conftest による entry 完備 check
    - 層 D: Kyverno `require-proof-cell-verified` admission policy
    - 層 E: cosign signed + Ceph RGW Object Lock retention

### `proof_status.lock.yaml`
- 生成元: Tekton Pipeline の proof execution result から generator が生成
- 入力:
    - `proof_inventory.lock.yaml`
    - 各 proof tool の execution log
    - `proof_review.lock.yaml`
- 出力 schema（各 entry）:
    - `obligation_id`: `proof_inventory` との bind
    - `status`: `["verified" | "in_progress" | "failed" | "accepted_with_assumption"]`
    - `last_verified_at`: ISO8601 timestamp
    - `certificate_hash`: 検証通過時の certificate hash
    - `tool_version_used`: 実際に使用した tool version
    - `reviewer_sigs`: cosign signature 2 名分
    - `cadence_days`: cadence_class から導出
    - `cell_state`: `["v1_baseline_verified" | "v1_accepted_with_assumption" | "v1_unverified_handled" | "v1_unverified_unhandled"]`
- enforce 経路:
    - 層 A: jsonschema 検証
    - 層 B: Conftest による cadence check / cell_state check
    - 層 D: Kyverno `require-reproducibility-passed` admission policy
    - 層 E: cosign signed + Ceph RGW Object Lock retention（`certificate_hash` の reproducibility 物理保証）

### `proof_matrix.lock.yaml`
- 生成元: `proof_inventory.lock.yaml` + `proof_status.lock.yaml` から generator が集約
- 入力:
    - `proof_inventory.lock.yaml`
    - `proof_status.lock.yaml`
    - `axis_registry.lock.yaml`
- 出力 schema:
    - 19 axis × 5 proof_class = 95 cell（proof_matrix 基底）の matrix
    - 各 cell に `cell_state` + `last_verified_at` + `coverage_weight`
- enforce 経路:
    - 層 A: schema 検証
    - 層 B: Conftest による 95 cell（proof_matrix 基底）完備 check
    - 層 D: `require-proof-cell-verified` の入力
    - 層 E: cosign signed + retention

### `assumption.lock.yaml`
- 生成元: 手書き＋ generator validate
- cap: v1=20 件、超過は CI fail
- 各 entry:
    - `assumption_id`
    - `description`: 自然言語
    - `literature_pointer`: 文献 pointer + DOI
    - `mitigation_pointer`: 軽減策の path
    - `revisit_at`: ISO8601 期限
    - `cryptographic` / `mathematical` / `fairness` / `bound` の分類
- enforce 経路:
    - 層 A: schema 検証
    - 層 B: Conftest による cap=20 check / 完備 check
    - 層 D: `require-assumption-cap` admission policy
    - 層 E: cosign signed + retention

### `mathlib_pin.lock.yaml`
- 生成元: 手書き＋ generator validate
- 内容: Lean mathlib の specific revision を git submodule + cosign 署名で固定
- 各 entry:
    - `mathlib_revision`: git ref
    - `lean_version`: Lean 4 version
    - `pin_signed_at`: ISO8601
    - `cosign_signature_pointer`
- enforce 経路:
    - 層 A: schema 検証
    - 層 B: Conftest による pin 完備 check
    - 層 D: `require-mathlib-pinned` admission policy
    - 層 E: git ref + signature 不一致で物理検出

### `tla_apalache_pin.lock.yaml`
- 生成元: 手書き＋ generator validate
- 内容: TLA+ tools + Apalache の version + bound parameter を pin
- 各 entry:
    - `tla_tools_version`
    - `apalache_version`
    - `tlc_version`
    - `bound_parameter`: state space depth / SMT timeout 等
    - `pin_signed_at`
- enforce 経路:
    - 層 A: schema 検証
    - 層 B: Conftest による pin 完備 check
    - 層 D: `require-tool-versions-pinned` admission policy

### `kani_cbmc_pin.lock.yaml`
- 生成元: 手書き＋ generator validate
- 内容: Kani / CBMC の version + bound parameter を pin
- 各 entry:
    - `kani_version`
    - `cbmc_version`
    - `bound_parameter`: unwind / object-bits / loop iteration count / SMT timeout
    - `pin_signed_at`
- enforce 経路:
    - 層 A: schema 検証
    - 層 B: Conftest による pin 完備 check + bound 引き下げ dual review check
    - 層 D: `require-tool-versions-pinned` `require-bound-parameter-review` admission policy

### `proof_review_assignment.yaml`
- 生成元: 手書き
- 内容: reviewer pool 管理
- 各 entry:
    - `reviewer_id`: PIN
    - `kind`: `["human" | "llm"]`
    - `domain_expertise`: `["crypto" | "distributed_system" | "proof_assistant" | "rust" | "c_cpp"]`
    - `cosign_keypair_id`
    - `cooldown_until`（過去 90 day 以内 review 履歴から導出）
- enforce 経路:
    - 層 B: Conftest による pool 完備 check
    - 層 D: `require-cooldown-respected` `require-no-self-review` admission policy

### `proof_minutes_budget.yaml`
- 生成元: 手書き
- 内容: 四半期 proof budget
- 各 entry:
    - `quarter`: 例 `2026-Q2`
    - `proof_minutes_target`
    - `verified_theorem_count_target`
    - `counter_example_open_count_target`
    - `accepted_with_assumption_count_target`
- enforce 経路:
    - 層 B: 四半期で（verified_theorem_count 単調増加 + counter_example_open_count 単調減少 + accepted_with_assumption_count 単調減少）の check
    - 層 D: 13 SLO 4 SLI への登録

### `ownership_table.lock.yaml`
- 生成元: 手書き＋ generator validate
- 共有: test / formal / ops / security の 4 軸で共有
- 各 entry:
    - `service_id`
    - `formal_owner`: PIN
    - `test_owner`: PIN
    - `ops_owner`: PIN
    - `security_owner`: PIN
- enforce 経路:
    - 層 B: Conftest による全 service 4 軸 owner 完備 check
    - 層 D: ownership 未宣言 service の deploy deny by default

## reproducibility（再現性）
- 全 proof artifact は reproducible build:
    - tool version + bound parameter + mathlib revision + statement_hash の同一性で certificate hash が同一であること（hermetic build）
    - 非 deterministic な proof tool（SMT timeout の flake、Z3 random seed）は seed 固定 + retry policy で deterministic 化
- reproducibility violation（同一 input で certificate hash が異なる）は CI fail、reviewer alert
- daily reproducibility checker: 全 verified proof を再 run、certificate hash 一致確認

## retention 戦略
- cap 10 year を default、regulatory requirement に応じて引き上げ:
    - **cryptographic proof**（11 key、Lean Shamir 等）: assumption の lifetime（NIST PQC migration horizon の 2030+ を想定して 15 year）
    - **SLSA chain proof**（security17）: build provenance の SLSA L4 retention 規定に従う
    - その他: 10 year
- retention 期限切れ proof artifact は cold archive tier（[data 設計方針](../../../03_概要設計/06_data設計方針/) の archive lifecycle）に移行、object lock は継続

## 移行（major version migration）
- tool major version migration（TLA+ tools / Apalache / Stainless / Dafny / Lean 4 / Kani / CBMC）は v1_refinement_proof drill として演習:
    - 旧 version で verified だった全 proof を新 version で再 verify
    - 再 verify が通らない proof は close_kind=fixed_in_spec として spec 修正、close_due_at = 30 day
- mathlib upstream pull は monthly drill:
    - 旧 mathlib pin で verified だった全 Lean proof を新 mathlib pin で再 verify
    - 失敗 proof は break-glass token + dual review で spec 修正

## CI / CD 経路
- Tekton Pipeline:
    - `proof_inventory-generate`
    - `proof-run`
    - `proof-review-assign`
    - `mathlib-pin-update`
    - `tool-version-migrate`
    - `bound-parameter-review`
    - `reproducibility-check`
- Argo CD ApplicationSet:
    - sync wave 1: schema validators
    - sync wave 2: proof tools deployment
    - sync wave 3: admission policies
- `release_gate.lock.yaml`:
    - cell `proof_artifact_complete` → 1.0.0 ship blocker

## audit / immutability
- audit_event subject: `v1_formal_proof_artifact_<action>`
- WORM 経路: Object Lock retention 10〜15 year
- cryptographic chaining: proof_event hash chain で divergence 物理特定

## 例外 / break-glass
- break-glass 経路:
    - 発行: OpenBao response wrapping (TTL = 1 hour)
    - audit: `v1_formal_break_glass` fact emit
    - 事後: 24 hour 内 retro review 必須
- mathlib pull / tool version migration / bound parameter 引き下げの break-glass は二人承認必須

## 関連参照
- [proof_artifact 方針](../../03_概要設計/11_formal設計方針/06_proof_artifact方針.md)
- [形式検証適合仕様](../01_適合仕様/20_形式検証適合仕様.md)
- [formal 強制機構](../02_強制機構/10_formal強制機構.md)
- [counter_example 体系](02_counter_example体系.md)
- [release_gate 体系](03_release_gate体系.md)

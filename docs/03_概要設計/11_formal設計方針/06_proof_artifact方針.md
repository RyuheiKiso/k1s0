---
id: arch.formal.proof_artifact_policy
axis: formal
phase: architecture
kind: policy
status: draft
depends_on:
  - arch.formal.formal_index
covered_by:
  defense_in_depth_layers: [A, B, C, D, E, F]
  proof_classes:
    - v1_temporal_safety_proof
    - v1_temporal_liveness_proof
    - v1_refinement_proof
    - v1_program_correctness_proof
    - v1_runtime_modelcheck_proof
lock_artifacts:
  - proof_inventory.lock.yaml
  - proof_status.lock.yaml
  - counter_example.lock.yaml
  - proof_review.lock.yaml
  - assumption.lock.yaml
  - mathlib_pin.lock.yaml
  - tla_apalache_pin.lock.yaml
  - kani_cbmc_pin.lock.yaml
---

# formal proof_artifact 方針

## 一文方針
- 全 proof artifact（TLA+ `.tla` / Apalache `.json` / P `.p` / Stainless `.stainless` / Dafny `.doo` / Lean `.olean` / Kani `.json` / CBMC `.json`）は build artifact として generator から生成、cosign signed、Ceph RGW Object Lock Compliance mode で retention 物理 delete 不可、`proof_status.lock.yaml` に fingerprint されることを物理 enforce する。

## 至高路線における立ち位置
- 手書き lock.yaml を全面禁止する。`proof_inventory` / `proof_status` / `counter_example` / `proof_review` / `coverage_matrix` の手書きは CI fail。
- 「文章のみで証明済み」「論文引用のみ」を全面禁止する。assumption.lock.yaml に明示登録された cryptographic assumption（DDH / RSA hardness / discrete log 等）のみ文献 pointer 可、cap=20 件。
- reproducibility violation の「軽微だから無視」を禁止する。certificate hash 不一致は必ず CI fail。

## artifact lifecycle

### 生成（generate）
- 各軸 spec の `formal_obligations` 段落から generator が `proof_inventory.lock.yaml` を build artifact として生成、手書き禁止。
- `proof_inventory.lock.yaml` は obligation の宣言的 catalog（theorem 名 + tool + statement_hash + expected_certificate_hash）。
- proof artifact 本体（`.tla` / `.scala` / `.dfy` / `.lean` / `.rs` / `.c`）は実装言語に応じて軸 owner が書く。ただし定型 boilerplate（init / next / harness）は generator が生成。

### 検証（verify）
- Tekton Pipeline で TLA+ + Apalache / P / Stainless / Dafny / Lean / Kani / CBMC を起動、verification obligation を SMT solver で discharge。
- tool 起動 log（`.json` / `.xml` / `.stdout`）を artifact として保存。
- certificate（verified state の fingerprint）を `proof_status.lock.yaml` に登録。

### 署名（sign）
- 全 proof artifact は cosign で署名。署名鍵は OpenBao Transit + sign-on-demand、CI runner に key material は物理に存在しない（[security 強制機構](../../04_詳細設計/02_強制機構/06_security強制機構.md) と整合）。
- 署名対象: `proof_inventory.lock.yaml` / `proof_status.lock.yaml` / `counter_example.lock.yaml` / `proof_review.lock.yaml` / 各 proof artifact 本体。

### 保存（archive）
- Ceph RGW Object Lock Compliance mode で retention、cap 10 year（cryptographic assumption の lifetime / regulatory requirement に応じて）。
- retention 期限内 物理 delete 不可（[security 強制機構](../../04_詳細設計/02_強制機構/06_security強制機構.md) / [data 強制機構](../../04_詳細設計/02_強制機構/05_data強制機構.md) と共有）。

### 検索（query）
- `proof_status.lock.yaml` + `proof_event` subject（ClickHouse SoR）が cross-axis ad-hoc 検索の primary 経路。
- Perses で proof matrix UI（19 axis × 5 proof_class の grid）を表示（[formal 運用 UI](../../04_詳細設計/04_運用UI開発者体験/07_formal運用UI.md) 参照）。

## proof_inventory.lock.yaml schema
- 各 entry の field:
    - `obligation_id`: 軸内一意な ID（例: `InvBidiHandshake`）
    - `axis_id`: 19 軸の ID（例: `07_transport`）
    - `proof_class`: 5 class enum
    - `tool_kind`: `["tlaplus_apalache" | "p_language" | "stainless" | "dafny" | "lean" | "kani" | "cbmc"]`
    - `statement_hash`: obligation 文（自然言語 + formal）の SHA-256 hash
    - `tool_version`: 採用 tool の version pin（`tla_apalache_pin.lock.yaml` / `kani_cbmc_pin.lock.yaml` / `mathlib_pin.lock.yaml` の参照）
    - `expected_certificate_hash`: 検証通過時の certificate の expected hash（再現性検証用）
    - `artifact_pointer`: proof artifact 本体の path / git ref / cosign signature pointer
    - `cross_axis_links`: 関連する他軸の `obligation_id` list
    - `assumption_refs`: `assumption.lock.yaml` の `assumption_id` list（参照する cryptographic / mathematical assumption）

## proof_status.lock.yaml schema
- build artifact、generator が proof execution の result から生成。
- 各 entry の field:
    - `obligation_id`: `proof_inventory` との bind
    - `status`: `["verified" | "in_progress" | "failed" | "accepted_with_assumption"]`
    - `last_verified_at`: ISO8601 timestamp
    - `certificate_hash`: 検証通過時の certificate hash（expected と一致しない場合 CI fail、再現性 violation）
    - `tool_version_used`: 実際に使用した tool version
    - `reviewer_sigs`: cosign signature 2 名分（dual review、`proof_review.lock.yaml` と bind）
    - `cadence_days`: cadence_class から導出
    - `cell_state`: `["v1_baseline_verified" | "v1_accepted_with_assumption" | "v1_unverified_handled" | "v1_unverified_unhandled"]`

## build artifact 化の規律
- 手書き `proof_inventory` / `proof_status` / `counter_example` / `proof_review` は禁止。
- 全 `.lock.yaml` は generator output、generator の input は軸 spec の `formal_obligations` 段落 + tool 実行 log + reviewer cosign signature。
- 手書き drift = 0 を CI で物理 enforce（generator 再実行で diff 0 を要求）。

## 再現性（reproducibility）
- 全 proof artifact は reproducible build:
    - tool version + bound parameter + mathlib revision + statement_hash の同一性で certificate hash が同一であること（hermetic build）。
    - 非 deterministic な proof tool（SMT timeout の flake、Z3 random seed）は seed 固定 + retry policy で deterministic 化。
- reproducibility violation（同一 input で certificate hash が異なる）は CI fail、reviewer alert。

## retention 戦略
- cap 10 year を default、regulatory requirement に応じて引き上げ:
    - **cryptographic proof**（11 key、Lean Shamir 等）: assumption の lifetime（NIST PQC migration horizon の 2030+ を想定して 15 year）
    - **SLSA chain proof**（security17）: build provenance の SLSA L4 retention 規定に従う
    - その他: 10 year
- retention 期限切れ proof artifact は cold archive tier（[data 設計方針](../06_data設計方針/) の archive lifecycle）に移行、object lock は継続。

## 移行（major version migration）
- tool major version migration（TLA+ tools / Apalache / Stainless / Dafny / Lean 4 / Kani / CBMC）は v1_refinement_proof drill として演習:
    - 旧 version で verified だった全 proof を新 version で再 verify。
    - 再 verify が通らない proof は close_kind=fixed_in_spec として spec 修正、close_due_at = 30 day。
- mathlib upstream pull は monthly drill:
    - 旧 mathlib pin で verified だった全 Lean proof を新 mathlib pin で再 verify。
    - 失敗 proof は break-glass token + dual review で spec 修正。

## 採用しない設計
- 手書き lock.yaml: 禁止。
- cosign 署名なし artifact: 禁止、CI / admission policy で reject。
- Object Lock retention 解除: 禁止、retention 期限内は物理 delete 不可。
- 非 deterministic proof tool の adhoc seed: 禁止、seed 固定 + retry policy で deterministic 化。
- reproducibility violation の「軽微だから無視」: 禁止、必ず CI fail + reviewer alert。

## 提供する artifact
- `proof_inventory.lock.yaml`（build artifact）
- `proof_status.lock.yaml`（build artifact）
- `counter_example.lock.yaml`（build artifact）
- `proof_review.lock.yaml`（build artifact）
- `assumption.lock.yaml`（手書き＋ generator validate、cap 20 件）
- `mathlib_pin.lock.yaml`（手書き＋ generator validate）
- `tla_apalache_pin.lock.yaml`（手書き＋ generator validate）
- `kani_cbmc_pin.lock.yaml`（手書き＋ generator validate）
- `proof_review_assignment.yaml`（手書き、reviewer pool 管理）
- `proof_minutes_budget.yaml`（手書き、四半期 budget）
- `proof_event` subject schema（09 観測 SoR、test_event と同列）
- Perses proof matrix dashboard JSON（build artifact）

## 強制機構との bind
- 本方針は [formal 強制機構](../../04_詳細設計/02_強制機構/10_formal強制機構.md) の以下経路で物理 enforce される:
    - 層 A: jsonschema / CUE による lock.yaml schema 検証
    - 層 B: Conftest による drift = 0 / cap 系 check
    - 層 D: Kyverno `require-cosign-signed-proof-artifact` `require-mathlib-pinned` admission policy
    - 層 E: cosign signature 物理検証 + Ceph RGW Object Lock retention
    - 層 F: 全 proof_class artifact の machine-checkable certificate

## 関連参照
- [formal 設計方針 index](README.md)
- [counter_example 方針](07_counter_example方針.md)
- [proof_review 方針](08_proof_review方針.md)
- [proof_artifact 体系](../../04_詳細設計/05_lock_yaml体系/01_proof_artifact体系.md)

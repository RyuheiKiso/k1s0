---
id: detail.formal.formal_enforcement
axis: formal
phase: detail
kind: enforcement
status: draft
depends_on:
  - arch.formal.formal_index
  - arch.formal.proof_artifact_policy
  - arch.formal.counter_example_policy
  - arch.formal.proof_review_policy
  - detail.formal.proof_artifact_system
  - detail.formal.counter_example_system
covered_by:
  defense_in_depth_layers: [A, B, C, D, E, F]
  proof_classes:
    - v1_temporal_safety_proof
    - v1_temporal_liveness_proof
    - v1_refinement_proof
    - v1_program_correctness_proof
    - v1_runtime_modelcheck_proof
lock_artifacts:
  - formal_enforcement.lock.yaml
trace:
  fr_ids:
  - FR-formal-002

---

# formal 強制機構

## 一文方針
- formal 層の規律は Kyverno admission policy + Tekton Pipeline + Argo CD sync wave + cosign signed proof artifact の AND-gate + Ceph RGW Object Lock Compliance mode + proof_event の hash chain 物理 enforce + dual reviewer cosign signature の AND-gate の 7 種で enforce、手書き `proof_inventory` / `proof_status` / `counter_example` / `proof_review` は CI fail、最終 safety net は cosign signature の物理的真正性 + Kyverno admission webhook の deploy block + Object Lock retention 物理 delete 不可 + dual reviewer signature の cryptographic impossibility of single-actor forgery とする。

## 5 層 defense-in-depth（formal 規律全体）

### 層 A: compile（catalog / schema / type の type check）
- `proof_inventory.lock.yaml` schema 検証（jsonschema / CUE）
- `formal_classes.yaml` / `proof_status.lock.yaml` / `counter_example.lock.yaml` / `proof_review.lock.yaml` / `assumption.lock.yaml` / `mathlib_pin.lock.yaml` / `tla_apalache_pin.lock.yaml` / `kani_cbmc_pin.lock.yaml` / `proof_review_assignment.yaml` / `proof_minutes_budget.yaml` の field 完備 check
- artifact_pointer の参照解決 check（path 物理存在）
- cross_axis_links / assumption_refs の参照整合 check
- proof_class / tool_kind / close_kind / severity の enum 整合
- cell_state enum 整合
- 19 軸 × 5 proof_class = 95 cell の `class_axis_cadence_table.yaml` の cadence_days enum 範囲内

### 層 B: lint（policy / 規約 check）

#### Conftest custom rule（Rego）
- `proof_status.lock.yaml` の 95 cell 完備、欠損 cell ゼロ
- cell の last_verified_at が cadence_days 以内
- proof_inventory entry の statement_hash + expected_certificate_hash 完備
- `counter_example.lock.yaml` の close_due_at 越え entry ゼロ（high=14d / medium=30d / low=90d）
- high severity の open counter-example ゼロ
- accepted_as_bug 数 cap=10 件以内
- `assumption.lock.yaml` の cap=20 件以内、文献 pointer + DOI + 軽減策 + revisit 期限完備
- mathlib_pin / tla_apalache_pin / kani_cbmc_pin の version + bound_parameter pin 済み
- `proof_review.lock.yaml` の dual_signoff_complete=true 完備
- reviewer のうち 1 名以上が human kind
- cryptographic proof の reviewer に crypto-domain expert が含まれる
- PR author 自身が reviewer に含まれない
- cooldown 違反（90 day 内連続 review）ゼロ
- termination_exempt annotation 数 cap=10 件以内
- certificate_hash が expected と一致（reproducibility）
- proof_event hash chain divergence ゼロ

#### Semgrep custom rule
- proof artifact 中の skip 注釈が新 statement_hash 切替 / artifact justification を伴う
- LLM-generated proof の `ai_generator_id` / `model_version` / `prompt_hash` 注釈完備
- SMT solver の random seed 固定
- Kani / CBMC の bound 引き下げ PR が dual review 済み

#### generator output と手書き drift = 0
- `proof_inventory` / `proof_status` / `counter_example` / `proof_review` / `coverage_matrix` 等の lock.yaml は generator output 完全一致

### 層 C: integration test（chainsaw / kuttl / Testcontainers）
- 全 formal admission policy が intended state を block / pass する test（後述一覧）
- Tekton Pipeline → Apalache / TLC / Stainless / Dafny / Lean / Kani / CBMC 起動の e2e 試験
- reviewer round-robin 割当 → cosign signature → `proof_review.lock.yaml` 更新の e2e 試験
- counter-example detect → triage → close → regression corpus 追加の e2e 試験
- mathlib upstream pull drill: monthly cron で mathlib new revision を candidate に試行、全 proof 再 verify
- tool major version migration drill: 6 month cron で tool new version を candidate に試行
- bound parameter review drill: quarterly cron
- reproducibility check: 同一 input で certificate hash が一致することを daily で検証

### 層 D: runtime（cluster 上の actual enforcement）
- Kyverno admission webhook: 本層担当 policy（後述一覧）の deploy / migration / DDL 全 block
- Argo CD: sync wave による proof artifact deploy 順序、drift detect で self-heal
- Tekton Pipeline:
    - `proof_inventory-generate`
    - `proof-run`
    - `proof-review-assign`
    - `counter-example-triage`
    - `mathlib-pin-update`
    - `tool-version-migrate`
    - `bound-parameter-review`
    - `reproducibility-check`
- Argo Workflows: long-running proof（Apalache large sweep / Lean 重い tactic）の orchestration
- cron-based audit chain verifier: proof_event hash chain の online verification
- daily reproducibility checker: 全 verified proof を再 run、certificate hash 一致確認

### 層 E: 物理 enforcement（OS / engine / 暗号 / 物理 layer の物理機構）
- cosign signature の cryptographic verification（proof artifact が真正であることを公開鍵で verify、Sigstore 標準）
- Kyverno admission webhook の物理拒否（deploy API request を webhook が deny で返却、kube-apiserver が物理 block）
- Ceph RGW Object Lock Compliance mode: proof artifact / counter-example trace / reviewer signature の retention 期限内 物理 delete 不可（[security 強制機構](06_security強制機構.md) / [data 強制機構](05_data強制機構.md) と共有）
- proof_event hash chain: cryptographic chain が divergence 時点を物理特定（暗号学的 impossibility of forgery）
- dual reviewer cosign signature: reviewer 2 名の private key を物理に必要、single-actor forgery が cryptographic に impossible
- mathlib revision pin: mathlib の specific revision を git submodule + cosign 署名で固定、改ざんは git ref + signature 不一致で物理検出

### 層 F: 数学的 enforcement（本軸自身が提供）
- TLA+ + Apalache の symbolic model check による safety / liveness invariant の証明
- Stainless / Dafny / Lean による program correctness の機械検証
- Kani / CBMC による runtime model check
- counter-example の machine-readable trace + reproducer による反例構造の物理特定

## Kyverno admission policy 一覧（formal 担当部分）

| policy 名 | 内容 |
|---|---|
| `require-proof-cell-verified` | deploy 前に対象 service の関連 proof cell が verified or accepted_with_assumption、cadence 内 |
| `require-counter-example-closed-in-due` | `counter_example.lock.yaml` の close_due_at 越え entry ゼロ |
| `require-high-severity-zero` | high severity の open counter-example が 1 件でもあれば全 service deploy block（global freeze）|
| `require-dual-reviewer-signoff` | `proof_review.lock.yaml` の dual_signoff_complete=true 完備 |
| `require-human-reviewer-included` | reviewer のうち 1 名以上が human kind |
| `require-crypto-expert-for-crypto-proof` | cryptographic proof の reviewer に crypto-domain expert が含まれる |
| `require-no-self-review` | PR author 自身が reviewer に含まれない |
| `require-cooldown-respected` | cooldown 違反（90 day 内連続 review）ゼロ |
| `require-assumption-cap` | `assumption.lock.yaml` の cap=20 件以内 |
| `require-accepted-as-bug-cap` | accepted_as_bug 数 cap=10 件以内 |
| `require-termination-exempt-cap` | termination_exempt annotation 数 cap=10 件以内 |
| `require-mathlib-pinned` | `mathlib_pin.lock.yaml` が pin 済み |
| `require-tool-versions-pinned` | `tla_apalache_pin` / `kani_cbmc_pin` が pin 済み |
| `require-bound-parameter-review` | bound 引き下げ PR が dual review 済み |
| `require-cosign-signed-proof-artifact` | proof artifact が cosign signed |
| `require-mathlib-revision-monthly-pull` | mathlib upstream pull cadence 越えなし（monthly）|
| `require-tool-version-migration-drill` | tool major version migration drill cadence 越えなし（6 month）|
| `require-reproducibility-passed` | daily reproducibility check が green |
| `block-on-llm-only-signoff` | LLM 単独 sign-off の artifact deploy block |
| `block-during-active-incident-proof-rewrite` | active incident 中の proof rewrite を block（incident commander ack 必須）|
| `block-during-global-freeze-tool-version-migration` | global freeze 内の tool version migration を block |
| `block-on-proof-event-chain-divergence` | proof_event hash chain divergence detect 時に全 service deploy block |
| `block-on-non-cosign-proof-artifact` | proof artifact が cosign signed でない場合 block |
| `audit-formal-chatops` | 全 ChatOps slash command 実行で audit_event emit |
| `audit-break-glass-issuance` | break-glass token 発行を全件 audit_event emit |
| `audit-counter-example-closure` | counter-example closure を全件 audit_event emit |
| `audit-mathlib-pull` | mathlib upstream pull を全件 audit_event emit |
| `audit-bound-parameter-change` | bound parameter change を全件 audit_event emit |

## admission policy のライフサイクル
- 全 admission policy は build artifact（`formal/policy/kyverno-formal-policies.yaml`）から生成、手書き禁止。
- generator の input:
    - `proof_inventory.lock.yaml`
    - `proof_status.lock.yaml`
    - `counter_example.lock.yaml`
    - `proof_review.lock.yaml`
    - `assumption.lock.yaml`
    - `mathlib_pin.lock.yaml`
    - `tla_apalache_pin.lock.yaml`
    - `kani_cbmc_pin.lock.yaml`
    - `proof_review_assignment.yaml`
    - `proof_minutes_budget.yaml`
    - `registry.yaml`（00_軸登録）
- drift: runtime 上の admission policy が build artifact と乖離した場合、Argo CD が drift として検出 + self-heal。

## break-glass
- 緊急時に formal admission policy を bypass する経路:
    1. OpenBao response wrapping で短期 cluster-admin 相当 token + Argo CD admin token + Argo Rollouts promote token + cosign signing temporary key を発行
    2. 発行操作は `audit_event` subject に `v1_formal_break_glass` fact emit（必須 column: `actor` / `reason` / `ticket_ref` / `expected_duration` / `incident_ref`）
    3. 使用後 24 hour 以内に retro review 完了必須
    4. break-glass 使用は 13 SLO error budget consumption として記録（`v1_formal_break_glass_count` の増分、ops 14 と整合）
    5. break-glass 経由の proof rewrite / counter-example 強制 close / mathlib pull / tool version migration / bound parameter 引き下げ も後追いで artifact 化（再現性確保）
    6. break-glass session 中も proof_event の hash chain は継続、chain divergence 不可
    7. proof rewrite / counter-example 強制 close / mathlib pull の break-glass は二人承認必須（ChatOps 上で primary + secondary on-call の双方 ack）

## CI 不変条件（formal 全体、20 項）

[形式検証適合仕様](../01_適合仕様/20_形式検証適合仕様.md) の「CI 不変条件（20 項）」を物理 enforce する。違反は merge 不可。

## 採用しない強制機構
- 商用 admission engine（OPA Gatekeeper / Styra DAS）: 採用しない、Kyverno L1+ 深耕（infra 14 / data 14 / security 14 / ops 14 / test 14 と同じ）
- 商用 proof tooling: 採用しない（[非提供スコープ](../../02_要件定義/01_スコープ/02_非提供スコープ.md) 参照）
- 文章 only の formal 規律: 禁止、全規律は機械検証可能な形で codify
- 「便利だから break-glass を常用」: 禁止、break-glass は incident IR + audit_event + retro review 必須の例外経路のみ
- 「business 都合で proof drill を一時 off」: 禁止、cadence 越えは admission block で物理 block
- assumption の cap 越え: 禁止、cap=20 件以内
- accepted_as_bug の cap 越え: 禁止、cap=10 件以内
- LLM 単独 sign-off: 禁止、必ず人間 1 名以上
- 単一 reviewer sign-off: 禁止、dual 必須
- PR author の self-review: 禁止
- cosign signing key を CI runner に長期保持: 禁止、OpenBao Transit + sign-on-demand のみ、key material は CI runner に物理に存在しない（[security 強制機構](06_security強制機構.md) / [ops 強制機構](07_ops強制機構.md) と整合）
- reproducibility violation の「軽微だから無視」: 禁止、certificate hash 不一致は CI fail
- mathlib pin なし運用: 禁止、必ず revision pin
- bound parameter 引き下げ without review: 禁止、dual review 必須

## 関連参照
- [formal 設計方針 index](../../03_概要設計/11_formal設計方針/README.md)
- [形式検証適合仕様](../01_適合仕様/20_形式検証適合仕様.md)
- [proof_artifact 体系](../05_lock_yaml体系/01_proof_artifact体系.md)
- [counter_example 体系](../05_lock_yaml体系/02_counter_example体系.md)
- [release_gate 体系](../05_lock_yaml体系/03_release_gate体系.md)
- [formal 運用 UI](../04_運用UI開発者体験/07_formal運用UI.md)

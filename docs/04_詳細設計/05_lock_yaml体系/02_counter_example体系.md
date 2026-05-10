---
id: detail.formal.counter_example_system
axis: formal
phase: detail
kind: detail
status: draft
depends_on:
  - arch.formal.counter_example_policy
  - detail.formal.formal_conformance
  - detail.formal.formal_enforcement
  - detail.formal.proof_artifact_system
covered_by:
  defense_in_depth_layers: [A, B, D, E]
  proof_classes:
    - v1_temporal_safety_proof
    - v1_temporal_liveness_proof
    - v1_refinement_proof
    - v1_program_correctness_proof
    - v1_runtime_modelcheck_proof
lock_artifacts:
  - counter_example.lock.yaml
  - proof_review.lock.yaml
---

# counter_example 体系

## 一文方針
- model checker（Apalache / TLC / P / Kani / CBMC）が検出した counter-example は必ず `counter_example.lock.yaml` に entry 化、close_kind ∈ {fixed_in_code, fixed_in_spec, accepted_as_bug, scope_narrowed} のいずれかで severity に応じた close_due_at 以内に closure、closure 状況は admission policy で物理 enforce する。

## 設計の物理転写
- 本詳細設計は [counter_example 方針](../../03_概要設計/11_formal設計方針/07_counter_example方針.md) を物理 artifact として完全転写する。
- counter-example の trace は machine-readable 形式（Apalache `.json` / P `.json` / Kani `.json` / CBMC `.xml`）で artifact 化、Ceph RGW Object Lock Compliance mode で retention。

## artifact / lock.yaml 体系

### `counter_example.lock.yaml`
- 生成元: tool log から generator が抽出（detect 時に open entry 自動追加）
- 入力:
    - 各 proof tool（Apalache / TLC / P / Kani / CBMC）の execution log
    - drill_owner の triage 結果（severity 判定 + close_kind）
    - close_actor の cosign signature
- 出力 schema（各 entry）:
    - `counter_example_id`: 一意 ID（軸 + obligation + discovery_at の triple から導出）
    - `axis_id`: 19 軸の ID
    - `obligation_id`: `proof_inventory` との bind
    - `proof_class`: 5 class enum
    - `discovery_at`: ISO8601 timestamp
    - `tool_kind` / `tool_version`: 検出した tool
    - `severity`: `["high" | "medium" | "low"]`
    - `trace_pointer`: trace artifact の path / git ref
    - `reproducer_pointer`: 再現可能な spec / harness の path
    - `close_kind`: `["fixed_in_code" | "fixed_in_spec" | "accepted_as_bug" | "scope_narrowed" | "open"]`
    - `close_due_at`: ISO8601 timestamp（high=14d / medium=30d / low=90d）
    - `close_at`: ISO8601 timestamp（closed のみ）
    - `close_actor`: cosign signature
    - `mitigation_pointer`: 軽減策（runbook / monitoring / alert）の path
    - `revisit_at`: accepted_as_bug の場合の revisit 期限
    - `cross_axis_links`: 関連する他軸の `counter_example_id` list
- enforce 経路:
    - 層 A: jsonschema 検証
    - 層 B: Conftest による close_due_at 越え check / cap check
    - 層 D: Kyverno `require-counter-example-closed-in-due` `require-high-severity-zero` `require-accepted-as-bug-cap` admission policy
    - 層 E: counter-example trace の Ceph RGW Object Lock Compliance mode 物理 retention

## severity の判定基準

### high
- safety property の違反（"bad thing happens"）で、production で実際に起きうる（reachable state、bound 内）
- cryptographic invariant の違反（11 key、security15 関連）
- tenant 越境（32 RLS、information flow）
- data loss / corruption（data01）
- close_due_at = 14 day

### medium
- safety / liveness の違反だが、bound parameter を引き上げないと到達しない
- 補助 invariant の違反（critical 軸の non-critical obligation）
- close_due_at = 30 day

### low
- liveness の特殊 fairness 仮定下でのみ violation
- 仕様の vacuous truth による誤検出に近い
- close_due_at = 90 day

## close_kind の物理転写

### fixed_in_code
- 実装側を修正、新 statement_hash で再 verify
- counter-example が再現しないことを確認
- PR に `regression_corpus_added` 注釈必須（test/15 軸の `regression_corpus.lock.yaml` に転写）
- physical artifact:
    - 修正 PR の git commit hash
    - 再 verify の新 certificate hash
    - test/15 の regression corpus entry pointer

### fixed_in_spec
- spec / model 側を修正、新 statement_hash で再 verify
- spec 変更は dual review 必須
- physical artifact:
    - spec 修正 PR の git commit hash
    - 再 verify の新 certificate hash
    - dual reviewer の cosign signature 2 名分

### accepted_as_bug
- known issue として登録、mitigation を別軸（runbook / monitoring / alert）で吸収
- revisit 期限を artifact_lock（最大 1 year）
- revisit 期限到達時に再評価、close_kind downgrade（fixed_in_code / fixed_in_spec へ）
- cap: v1=10 件以内、超過は CI fail
- physical artifact:
    - mitigation pointer（runbook / monitoring / alert の path）
    - revisit_at（ISO8601）
    - 認可者 cosign signature

### scope_narrowed
- obligation の scope を絞り、絞った範囲で再 verify
- scope_narrow の log を artifact_lock、軸 spec の `formal_obligations` 段落に scope 縮小理由を記載
- physical artifact:
    - scope_narrow log の path
    - 軸 spec の `formal_obligations` 段落更新 commit hash
    - 再 verify の新 certificate hash

## closure 物理 enforce

### Kyverno admission policy
- `require-counter-example-closed-in-due`: `counter_example.lock.yaml` の close_due_at 越え entry が 1 件でもあれば関連 service deploy block
- `require-high-severity-zero`: high severity の open entry が 1 件でもあれば全 service deploy block（global freeze）
- `require-accepted-as-bug-cap`: accepted_as_bug 数 cap=10 件以内

### cron-based audit
- daily で `counter_example.lock.yaml` の状態を `audit_event` subject に emit
- close_due_at breach の事前 alert（7 day before）

## counter-example trace の物理保存

### machine-readable 形式
- Apalache `.json` trace
- P `.json` trace
- Kani `.json` report
- CBMC `.xml` + `.json` report

### human-readable 形式（dashboard 表示用）
- trace を可視化した sequence diagram
- variable timeline
- Perses dashboard で表示

### reproducer
- trace から bounded harness を自動生成
- close 後の regression check に利用

## test 軸との bind（regression_corpus.lock.yaml と双方向 lock）
- counter-example で発見された input は test/15 軸の `regression_corpus.lock.yaml` に物理転写
- property test の seed pool に追加:
    - close_kind=fixed_in_code: 修正後の実装が counter-example input で必ず正常動作することを property test で継続 check（regression corpus として永久保存）
    - close_kind=fixed_in_spec: spec 修正後の formal proof で counter-example が解消されることを確認、property test seed として追加
- test/15 の v1_property_axiom cell の seed pool drift は CI fail（双方向 lock check）

## workflow の物理転写

### detect
- tool 起動で counter-example 検出
- generator が tool log から抽出 → `counter_example.lock.yaml` に open entry 自動追加
- audit_event subject に `v1_formal_counter_example_detect` fact emit

### triage
- drill_owner が severity 判定 + close_kind 候補判断
- 24 hour 以内に severity 決定
- high severity は ChatOps + page

### close
- close_kind に応じた action:
    - fixed_in_code: 実装修正 PR + 再 verify + test/15 regression corpus 追加
    - fixed_in_spec: spec 修正 PR + dual review + 再 verify
    - accepted_as_bug: mitigation pointer + revisit 期限の artifact_lock
    - scope_narrowed: scope_narrow log + 軸 spec の `formal_obligations` 段落更新
- audit_event subject に `v1_formal_counter_example_close` fact emit

### verify_close
- closure 後に再 verify
- certificate hash が新 expected と一致することを CI で確認
- 不一致は close fail として再 open

### archive
- closed entry を `proof_event` subject（ClickHouse SoR）に emit
- counter-example trace を Object Lock で retention（10 year）

## CI / CD 経路
- Tekton Pipeline:
    - `counter-example-detect`
    - `counter-example-triage`
    - `counter-example-close`
    - `counter-example-verify-close`
- Argo Workflows:
    - `counter-example-storm-handler`（複数軸同時 detect 時）
- `release_gate.lock.yaml`:
    - cell `counter_example_no_open_above_severity_low` → 1.0.0 ship blocker

## audit / immutability
- audit_event subject: `v1_formal_counter_example_<action>`
- WORM 経路: Object Lock retention 10 year
- cryptographic chaining: proof_event hash chain

## 例外 / break-glass
- break-glass 経路:
    - 発行: OpenBao response wrapping (TTL = 1 hour)
    - audit: `v1_formal_break_glass` fact emit
    - 事後: 24 hour 内 retro review 必須
- counter-example 強制 close の break-glass は二人承認必須

## 関連参照
- [counter_example 方針](../../03_概要設計/11_formal設計方針/07_counter_example方針.md)
- [形式検証適合仕様](../01_適合仕様/20_形式検証適合仕様.md)
- [formal 強制機構](../02_強制機構/10_formal強制機構.md)
- [proof_artifact 体系](01_proof_artifact体系.md)
- [release_gate 体系](03_release_gate体系.md)

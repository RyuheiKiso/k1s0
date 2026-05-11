---
id: detail.<axis>.enforcement
axis: <axis>
phase: detail
kind: enforcement
status: draft
depends_on: []
covered_by:
  defense_in_depth_layers: [A, B, C, D, E]
  proof_classes: []
lock_artifacts:
  - <axis>_enforcement.lock.yaml
---

# <axis> 強制機構

## 一文方針
- <axis> 層の規律は <主機構列挙: Kyverno admission policy + Tekton Pipeline + Argo CD sync wave + cosign signed artifact + Object Lock + ...> の N 種で enforce、手書き <lock.yaml 列挙> は CI fail、最終 safety net は <最終 enforcement: cosign 真正性 + Kyverno admission deploy block + Object Lock 物理 delete 不可 + dual reviewer signature> とする。

## 5 層 defense-in-depth（<axis> 規律全体）

### 層 A: compile（catalog / schema / type の type check）
- <axis>_<lock>.lock.yaml schema 検証（jsonschema / CUE）
- <field 完備 check>
- <enum 整合 check>
- <参照解決 check>

### 層 B: lint（policy / 規約 check）
- Conftest custom rule（Rego）:
    - <規律 1>
    - <規律 2>
- Semgrep custom rule:
    - <パターン規律>
- generator output と手書き drift = 0:
    - <lock.yaml の generator output 完全一致>

### 層 C: integration test（実行検証）
- <test scenario>
- <pact contract test>
- <chaos drill>

### 層 D: runtime（admission webhook / drill / circuit breaker）
- Kyverno ClusterPolicy:
    - <admission rule 1>
    - <admission rule 2>
- Argo CD sync wave:
    - <wave 順>
- runtime drill:
    - <drill 内容 + cadence>

### 層 E: 物理（cosign / Object Lock / 物理隔離）
- cosign signed artifact:
    - <artifact 名>: cosign verify 必須、署名なしは admission deny
- Ceph RGW Object Lock Compliance mode:
    - <bucket 名>: retention=N year、物理 delete 不可
- 物理隔離 / 専用 cluster:
    - <該当時のみ記述>

### 層 F: 数学的（proof certificate）
- formal 関連時のみ:
    - <proof obligation>
    - <tool: TLA+ / Lean / Stainless / Dafny / Kani / CBMC>
    - <statement_hash + tool_version pin>

## audit / immutability
- 全 enforcement event は audit_event subject に `v1_<axis>_<action>` fact emit
- audit_event の append-only / WORM / cryptographic chaining は [audit_ingest_gap_monitor](../../03_クロスカッティング適合仕様/09_audit_ingest_gap_monitor.md) で物理 enforce

## CI / CD 経路
- Tekton Pipeline:
    - stage: <stage 名> → <task 列挙>
- Argo CD ApplicationSet:
    - <sync wave 順>
- release_gate.lock.yaml:
    - cell: `<axis>_enforcement` → 1.0.0 ship blocker

## 例外 / break-glass
- break-glass 経路:
    - 発行: OpenBao response wrapping (TTL = N hour)
    - audit: `v1_<axis>_break_glass` fact emit
    - 事後: 24 hour 内 retro review 必須
- 文章 only の例外承認は禁止。全例外は cosign signed approval + audit_event 物理 emit を伴う。

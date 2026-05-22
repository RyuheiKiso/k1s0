---
id: detail.test.test_enforcement
axis: test
phase: detail
kind: enforcement
status: draft
depends_on:
  - arch.test.test_index
  - arch.test.coverage_matrix_policy
  - arch.test.property_based_policy
  - arch.test.contract_pair_policy
  - arch.test.scenario_replay_policy
  - arch.test.fault_chaos_policy
  - arch.test.mutation_testing_policy
  - arch.test.regression_policy
  - arch.test.performance_policy
  - detail.test.verification_conformance
covered_by:
  defense_in_depth_layers: [A, B, C, D, E]
  proof_classes: []
lock_artifacts:
  - test_enforcement.lock.yaml
trace:
  fr_ids:
  - FR-test-002

---

# test 強制機構

## 一文方針
- test 層の規律は Kyverno admission policy + Tekton Pipeline + Argo CD sync wave + Pact Broker webhook + Litmus workflow CR + cosign signed build artifact の AND-gate + test_event の hash chain 物理 enforce の 7 種で enforce、手書き coverage matrix / mutation report / chaos blueprint / pact 注釈は CI fail、最終 safety net は cosign signature の物理的真正性 + Kyverno admission webhook の deploy block + Ceph RGW Object Lock Compliance mode の retention 物理 delete 不可 とする。

## 5 層 defense-in-depth

### 層 A: compile（catalog / schema / type の type check）
- `coverage_matrix.lock.yaml` schema 検証（jsonschema / CUE）
- properties.yaml / scenarios.yaml / pacts.yaml / chaos_blueprints.lock.yaml / mutation_modules.yaml / performance_scenarios.lock.yaml / flaky_quarantine.lock.yaml / snapshot_masks.lock.yaml の field 完備 check
- artifact_pointer の参照解決 check
- regression_corpus_id の参照整合 check
- mutation_operator_class / fault_class / performance_class / journey_class / declaration_class の enum 整合
- performance_baseline.lock.yaml の SLI 列挙が 13 SLO 登録 SLI と整合
- class_axis_cadence_table.yaml の 18 × 5 = 90 cell が完備、cadence_days enum 範囲内
- ownership_table.lock.yaml の test ownership 4 軸完備（cross-axis）

### 層 B: lint（policy / 規約 check）

#### Conftest custom rule（Rego）
- coverage_matrix の 90 cell 完備、欠損 cell ゼロ
- cell の last_green_at が cadence_days 以内
- cell の owner_4axis 4 軸完備
- properties.yaml の全 property_id が test code に annotated
- scenarios.yaml の全 scenario_id が Playwright / Apache JMeter / chainsaw 実装に bind
- pacts.yaml の全 pair_id が Pact runtime output に bind
- chaos_blueprints.lock.yaml の全 blueprint_id が Litmus workflow CR に bind
- mutation_modules.yaml の全 module_id が mutation_score.lock.yaml に bind
- flaky_quarantine の全 entry が quarantine_due_at 14 day 以内
- mutation_score の quarter monotonic increase
- snapshot_masks の cap 20% 以下
- blast_radius v1_prod_gameday の chaos experiment は break-glass token 注釈必須
- regression_corpus_added 注釈なしの defect 修正 PR ゼロ
- performance_baseline 引き下げ PR ゼロ
- shadow cluster 注釈なしの v1_scenario_replay / v1_fault_chaos / 性能 test ゼロ

#### Semgrep custom rule
- test code 中の skip 注釈が新 verification_class 切替 / artifact justification を伴う
- LLM-generated test の `ai_generator_id` / `model_version` / `prompt_hash` 注釈完備
- random seed の test runner ローカル random 化使用ゼロ

#### generator output と手書き drift = 0
- coverage_matrix / mutation_score / performance_baseline 等の lock.yaml は generator output 完全一致

### 層 C: integration test（chainsaw / kuttl / Testcontainers）
- 全 test admission policy が intended state を block / pass する test
- Pact Broker webhook → Tekton Pipeline → producer verify の e2e 試験
- Litmus workflow の dry-run 実行（全 chaos blueprint 対象、blast_radius v1_shadow_only のみ）
- mutation testing の per_module 実行 + score baseline 比較の試験
- flaky_classifier の機械判定 pipeline 試験
- snapshot 更新 PR の drill_owner ack 経路の e2e 試験
- performance regression 検出 → PR merge block の e2e 試験
- shadow cluster 上の scenario_replay e2e 試験

### 層 D: runtime
- Kyverno admission webhook: 本層担当 policy の deploy / migration / DDL 全 block
- Argo CD: sync wave による test artifact deploy 順序、drift detect で self-heal
- Tekton Pipeline:
    - `coverage-matrix-generate`
    - `mutation-run`
    - `pact-publish`
    - `pact-verify`
    - `chaos-drill`
    - `scenario-replay`
    - `performance-baseline-update`
    - `flaky-classifier`
    - `regression-corpus-replay`
- Pact Broker（self-host）: can-i-deploy result を Kyverno admission policy 入力に bind
- Litmus workflow controller: chaos blueprint の物理発火、blast_radius の物理制限
- Argo Workflows: scenario_replay の長時間 run orchestration
- cron-based audit chain verifier: test_event hash chain の online verification
- flaky_classifier daemon: test_event SoR を毎日集計

### 層 E: 物理 enforcement
- cosign signature の cryptographic verification
- Kyverno admission webhook の物理拒否
- Ceph RGW Object Lock Compliance mode: security incident reproducer / mutation equivalent mutant input / chaos drill failure log の retention 期限内 物理 delete 不可
- test_event hash chain: cryptographic chain divergence の物理特定
- Pact pact / mutation report / Playwright trace / chaos workflow 結果は cosign signed artifact
- chaos blueprint の v1_prod_gameday は OpenBao response wrapping の二人承認 break-glass token 物理発行

## Kyverno admission policy 一覧（test 担当部分）

| policy 名 | 内容 |
|---|---|
| `require-coverage-cell-green` | deploy 前に対象 service の関連 coverage cell が green、cadence 内 |
| `require-mutation-score-monotonic` | mutation_score.lock.yaml の quarter monotonic increase |
| `require-flaky-quarantine-cap` | flaky_quarantine の cap 越えなし、quarantine_due_at 14 day 以内 |
| `require-pact-can-i-deploy` | Pact Broker の can-i-deploy green |
| `require-chaos-drill-cadence` | chaos blueprint の cadence 越えなし |
| `require-regression-corpus-added` | defect 修正 PR に regression_corpus_added 注釈 |
| `require-snapshot-signed` | snapshot 更新 PR が drill_owner ack 済み |
| `require-blast-radius-annotation` | chaos experiment CR に blast_radius 注釈 |
| `block-prod-gameday-without-break-glass` | v1_prod_gameday は break-glass token 必須 |
| `block-on-equivalent-mutant-without-proof` | v1_strict policy module の equivalent mutant に symbolic_proof_pointer 必須 |
| `block-on-non-cosign-test-artifact` | test artifact が cosign signed でない場合 block |
| `block-during-active-incident-chaos` | active incident 中の chaos experiment block（incident commander ack 必須）|
| `block-during-global-freeze-chaos-prod-gameday` | global freeze 内の v1_prod_gameday を block |
| `block-on-performance-regression` | perf regression red の関連 service deploy block |
| `block-on-flaky-cap-exceeded` | flaky 認定数 / total test 数 cap 越え時 service deploy block |
| `block-on-shadow-only-violation` | v1_scenario_replay が production cluster で起動した場合 block |
| `block-on-snapshot-mask-overflow` | snapshot mask cap 20% 越え時 block |
| `block-on-deterministic-violation` | v1_full_deterministic 注釈 scenario が determinism_violation として alert された場合 deploy block |
| `audit-test-chatops` | 全 ChatOps slash command 実行で audit_event emit |
| `audit-snapshot-update` | snapshot 更新 PR を全件 audit_event emit |
| `audit-mutation-equivalent-decision` | equivalent mutant 認定 PR を全件 audit_event emit |
| `audit-corpus-write` | property_corpus / chaos_blueprints / scenarios の write 操作を全件 audit |
| `require-test-ownership-completeness` | service deploy 前に test ownership 4 軸完備 check |

## admission policy のライフサイクル
- 全 admission policy は build artifact（`test/policy/kyverno-test-policies.yaml`）から生成、手書き禁止
- generator の input:
    - `coverage_matrix.lock.yaml`
    - `mutation_score.lock.yaml` / `mutation_history.lock.yaml`
    - `pacts.yaml` + Pact Broker can-i-deploy state
    - `chaos_blueprints.lock.yaml`
    - `scenarios.yaml` / `properties.yaml`
    - `flaky_quarantine.lock.yaml`
    - `performance_baseline.lock.yaml`
    - `ownership_table.lock.yaml`
- drift: runtime 上の admission policy が build artifact と乖離した場合、Argo CD が drift として検出 + self-heal

## break-glass
- 緊急時に admission policy を bypass する経路:
    1. OpenBao response wrapping で短期 token を発行
    2. 発行操作は audit_event subject に `v1_test_break_glass` fact emit
    3. 使用後 1 時間以内に postmortem PR 起票必須
    4. break-glass 使用は 13 SLO error budget consumption として記録
    5. severity_freeze / security_bridge の break-glass は二人承認必須

## CI 不変条件（18 項）
- [検証規律適合仕様](../01_適合仕様/19_検証規律適合仕様.md) の「CI 不変条件（18 項）」を物理 enforce する。違反は merge 不可。

## 採用しない強制機構
- OPA Gatekeeper 単独運用: 採用しない、Kyverno L1+ 深耕
- 商用 admission engine: 採用しない
- 文章 only の運用ルール: 禁止
- 「便利だから break-glass を常用」: 禁止
- mutation testing skip: 禁止
- LLM-only mutation 認定: 禁止
- production cluster 上の chaos / scenario / load test: 禁止
- cosign signing key を CI runner に長期保持: 禁止

## 関連参照
- [test 設計方針 index](../../03_概要設計/10_test設計方針/README.md)
- [検証規律適合仕様](../01_適合仕様/19_検証規律適合仕様.md)
- [test 運用 UI](../04_運用UI開発者体験/06_test運用UI.md)

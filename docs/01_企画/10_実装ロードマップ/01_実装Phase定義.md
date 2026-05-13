---
id: plan.overview.implementation_phase_definition
axis: overview
phase: plan
kind: plan_doc
status: draft
depends_on:
  - plan.overview.implementation_roadmap_index
  - arch.overview.nineteen_axis_theory
  - arch.overview.axis_dependency_diagram
  - detail.formal.release_gate_system
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# 実装 Phase 定義

## 一文方針
- k1s0 の src/ 実装は「enforcement first, code second」の至高路線に従い P0 Tooling から P11 release_gate closure まで 11 Phase strict 直列で進む。各 Phase の完了は release_gate 20 cell の機械評価で判定する。

## 前提条件

| 項目 | 現状 |
|---|---|
| src/ 実装 | 全 12 dir が README 1 枚のみ。コード・マニフェスト 0 件 |
| docs/ 整備 | 01_企画〜05_環境構築 完備（20 適合仕様 md・13 crosscutting 仕様・100+ 方針 md）|
| release_gate generator | skeleton（evaluate_cell が全 cell red を返す stub）|
| axis_registry.lock.yaml | 未生成 |
| CI | docs_lint.yml 1 件のみ。コード系 CI ゼロ |

## 11 Phase 定義

### P0 — Tooling bedrock

**一文**: release_gate AND-gate が機械評価可能な状態を確立する。

**成果物**:
- `tools/lock_yaml_generator/generate_release_gate.py` の `evaluate_cell` 本実装（source_lock_artifact を読み source_field_path を解決）
- `generate_docs_lint.py` — docs lint JSON 出力を `docs_lint.lock.yaml` に変換
- `generate_repository_layout.py` — src/docs/tools の存在検査
- `generate_axis_registry.py` — 19 軸 + 13 crosscutting を `axis_registry.lock.yaml` に生成
- 残 13 generator の skeleton（`capabilities / instruments / oss_inventory / migration / conflict_tree / sdk_conformance / topology_class / clock_causality_proof / preservation_substrates / threat_model / artifact_inventory / ops_loop / coverage_matrix / proof_matrix / proof_status / proof_review`）
- `.github/workflows/release_gate.yml`

**参照**:
- [generate_release_gate.py](../../../tools/lock_yaml_generator/generate_release_gate.py)（20 cell カタログ L28-76）
- [release_gate 体系](../../04_詳細設計/05_lock_yaml体系/03_release_gate体系.md)
- [artifact_lock 命名規約](../../04_詳細設計/05_lock_yaml体系/04_artifact_lock命名規約.md)

**完了判定**: `python3 tools/lock_yaml_generator/generate_release_gate.py` が 20 cell 評価して `release_gate_status: red`（red=20/20 が期待動作）を出力し、`evaluate_cell` が stub から本実装に移行した状態。

---

### P1 — Meta 軸

**一文**: 19 軸（cap 20、残 1）の単一 registry を build artifact 化し、他全 cell の入力基盤を確立する。

**成果物**:
- `axis_registry.lock.yaml`（19 軸 entry、cap=20 / 残 1 / 13 crosscutting 列挙）
- `docs/04_詳細設計/01_適合仕様/*.md` frontmatter ⇄ axis_registry の双方向 lock

**参照**:
- [19 軸論](../../03_概要設計/01_アーキテクチャ概観/02_19軸論.md)
- [run_lint.py](../../../tools/docs_lint/run_lint.py)（SRC_ALLOWED_AXES L236-241）

**完了判定 (release_gate cell)**: `meta.axis_registry_complete` = green

---

### P2 — B 層 lint 完備

**一文**: src/ 構造逸脱・lock.yaml 手書きが merge される経路を物理拒否する lint check を追加する。

**成果物**:
- `tools/docs_lint/run_lint.py` に 3 check 追加:
  - `check_src_layout()` — src/ 直下許可サブ dir の allowlist 検査
  - `check_crosscutting_slug()` — `_crosscutting/NN_<slug>` パターン強制
  - `check_lock_yaml_handwriting()` — `*.lock.yaml` の手書き diff 検出
- `.github/workflows/docs_lint.yml` に新 check を job 追加

**完了判定 (cell)**: `meta.repository_layout_integrity` + `meta.docs_lint_green` = green

---

### P3 — 物理 root 設立

**一文**: 多言語 workspace root（Cargo / Go / npm / Buf / TLA+ / Lean）を軸実装前に確立し、後続 PR での import path drift を物理排除する。

**成果物**（root 直下）:
- `/Cargo.toml`（Cargo workspace root）
- `/go.work`（Go workspace）
- `/package.json`（npm workspaces: src/tier3, src/client/ts, src/test）
- `/buf.yaml` + `/buf.gen.yaml`
- `/src/formal/tlaplus/`（TLA+ spec root）
- `/src/formal/lean/lakefile.lean`（Lean 4 toolchain pin）

各軸 dir の空 manifest:
- `src/tier1/Cargo.toml`、`src/infra/go.mod`、`src/data/Cargo.toml`、`src/security/Cargo.toml`
- `src/client/{rust,go,ts,cs,java,python,ruby,dotnet_fw,swift}/` 各 manifest
- `src/test/package.json`

**参照**:
- [ARCHITECTURE.md](../../../ARCHITECTURE.md)（src/ 直下許可構造 L97-112）
- [tier1 設計方針 README](../../03_概要設計/02_tier1設計方針/README.md)

**完了判定**: 各 manifest で `cargo check` / `go build ./...` / `dotnet build` が空 lib で green。

---

### P4 — test framework scaffold

**一文**: 18 軸 × 5 verification_class = 90 cell の scaffold と `coverage_matrix.lock.yaml` 生成器を tier1 着手前に確立し、cell の循環依存を排除する。

**成果物**:
- `src/test/` 配下 90 cell placeholder（初期値 `v1_pending_with_artifact`）
- Pact Broker / Litmus / pitest / Playwright / k6 / Testcontainers pipeline skeleton
- `tools/lock_yaml_generator/generate_coverage_matrix.py` 本実装

**参照**: [検証規律適合仕様](../../04_詳細設計/01_適合仕様/19_検証規律適合仕様.md)

**完了判定**: `coverage_matrix.lock.yaml` 生成 green、全 90 cell が `v1_pending_with_artifact`。

---

### P5 — Formal 部分前置

**一文**: tier1 transport の deadlock-freedom（TLA+）と data preservation の atomic 三表書込 invariant（Stainless）の 2 件を tier1 着手前に証明し、spec drift を数学的に物理防止する。

**成果物**:
- `src/formal/tlaplus/tier1_transport_safety.tla` — Connect-RPC over HTTP/2 の deadlock-freedom safety proof（Apalache で検証）
- `src/formal/stainless/data_preservation_invariant.scala` — data 軸 5 preservation_class の atomic 三表書込 invariant
- `proof_inventory.lock.yaml` partial 生成（2 件 `v1_baseline_verified`、残 93 件 `v1_pending`）

**参照**:
- [形式検証適合仕様](../../04_詳細設計/01_適合仕様/20_形式検証適合仕様.md)（5 proof_class）
- [軸間依存図](../../03_概要設計/01_アーキテクチャ概観/05_軸間依存図.md)（proof_obligation 転写表 L50-60）

**完了判定**: Apalache `No error found`、Stainless `All functions verified`。

---

### P6 — 物理 enforcement 層

**一文**: release_gate が red の間 production deploy を Kyverno admission policy で物理 block し、cosign dual signoff と Argo CD ApplicationSet を配線する。

**成果物**:
- `tools/k8s/policies/require-release-gate-green.yaml`
- `tools/k8s/policies/require-release-gate-cosign-signed.yaml`
- `tools/k8s/policies/require-release-gate-dual-signoff.yaml`
- `tools/cosign/` 鍵管理 SOP + dual signoff workflow
- `tools/argocd/` ApplicationSet（production sync wave = release_gate green 前提）

**参照**: [release_gate 体系](../../04_詳細設計/05_lock_yaml体系/03_release_gate体系.md)（enforce 経路 L130-145）

**完了判定**: kind cluster で unsigned image deploy が物理 reject（Kyverno E2E test）。

---

### P7 — 13 crosscutting compile 強制

**一文**: 13 cross-cutting 適合仕様を tier1 着手前に lint rule + Kyverno policy として物理化し、tier1 コードが制約に事後違反する経路を排除する。

**成果物**: `src/_crosscutting/01_*〜13_*` 各サブ dir に:
- Buf custom lint plugin（proto-level 制約）
- Kyverno cross-axis admission policy
- `enforcement.lock.yaml`（各仕様の enforcement 状態）

難度高の 5 件（02_KEK_shamir / 05_SLO_four_layers / 08_PII_cluster / 10_ops_edge / 13_dotnet8_connect）を優先着手。

**参照**: [crosscutting blocking 表](04_crosscutting_blocking表.md)、[クロスカッティング適合仕様](../../04_詳細設計/03_クロスカッティング適合仕様/)

**完了判定**: 13 件の lint rule が CI 稼働、違反 PR が物理 reject。

---

### P8 — infra 軸実装

**一文**: tier1 が依存するクラスタ位相（5 topology_class）と時刻整合（PTP/NTP/HLC）を実装し、failover drill と clock causality proof を lock artifact 化する。

**成果物**:
- `src/infra/topology/`（Go）— 5 topology_class の IaC manifest + CRD
- `src/infra/clock/`（Go）— PTP/chrony/HLC 監視 + `clock_causality_proof.lock.yaml`
- `src/infra/topology_drill/` — Litmus chaos runner（failover drill）
- `src/_crosscutting/10_ops_edge_cluster/` infra 側

**参照**:
- [クラスタ位相適合仕様](../../04_詳細設計/01_適合仕様/12_クラスタ位相適合仕様.md)
- [時刻整合適合仕様](../../04_詳細設計/01_適合仕様/13_時刻整合適合仕様.md)

**完了判定 (cell)**: `infra.topology_class_drill_green` + `infra.clock_integrity_drill_green` = green

---

### P9 — data 軸実装

**一文**: 5 preservation_class と WAL/snapshot/replication と restore_drill を実装し、preservation_substrates.lock.yaml を green にする。

**成果物**:
- `src/data/preservation/`（Rust）— 5 class registry
- `src/data/wal/` — WAL + snapshot replication
- `src/data/migration/` — Pair 定義 + apicurio schema sync
- `src/_crosscutting/03_apicurio_gitops_sot/` data 側
- `src/_crosscutting/08_pii_dedicated_cluster/`

**参照**: [データ保全適合仕様](../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md)

**完了判定 (cell)**: `data.preservation_class_drill_green` = green

---

### P10 — tier1 → tier2 → tier3 → security → ops → client（順次）

**一文**: 単一実装者 strict 直列で 6 サブ Phase を順に完成させ、6 軸の release_gate cell を順次 green にする。

**P10a tier1**（Rust server + Go Operator + Rust/C#/Go/TS Library）:

| 手順 | 成果物 | green 化 lock |
|---|---|---|
| 1 | Cargo workspace + Connect-RPC echo | — |
| 2 | `SessionContext` 抽象 + tracing/otel 配線 | — |
| 3 | Bidi streaming（8 adapter 同型 API） | `capabilities.lock.yaml`（前提） |
| 4 | OIDC + mTLS 認証層（AuthContext 必須引数化） | — |
| 5 | KEK/DEK 鍵管理 + Shamir 統合 | — |
| 6 | apicurio schema registry client | — |
| 7 | conformance harness（20 軸 bidi pass） | `capabilities.lock.yaml` |
| 8 | SLO meter（RED/USE 四層） | `instruments.lock.yaml` |
| 9 | OSS inventory generator + lifecycle drill | `oss_inventory.lock.yaml` |

完了 cell: `tier1.bidi_conformance_complete` + `tier1.slo_compliance_quarterly_green` + `tier1.oss_lifecycle_drill_green`

**P10b tier2**（Go Operator + Rust）:

| 手順 | 成果物 |
|---|---|
| 1 | Operator CRD 定義 |
| 2 | Tenant 分離 reconciler（RLS） |
| 3 | Migration Pair runner + drill harness → `migration.lock.yaml` |

完了 cell: `tier2.tenant_isolation_drill_green`

**P10c tier3**（TS + Tauri + .NET Framework）:

| 手順 | 成果物 |
|---|---|
| 1 | Tauri shell + TS state model（4-state machine） |
| 2 | Conflict tree resolution algo |
| 3 | Client state conformance harness → `conflict_tree.lock.yaml` |
| 4 | crosscutting 06_BFF_auth_edge / 07_tauri_companion_sidecar / 12_UA_aware_adapter |

完了 cell: `tier3.client_state_conformance_complete`

**P10d security**:

| 手順 | 成果物 |
|---|---|
| 1 | STRIDE 5⁴ catalog → `threat_model.lock.yaml` |
| 2 | SLSA L3+ generator + cosign 統合 → `artifact_inventory.lock.yaml` |

完了 cell: `security.threat_model_coverage_100pct` + `security.build_provenance_slsa_l3plus`

**P10e ops**:

| 手順 | 成果物 |
|---|---|
| 1 | PDCA runbook + dashboard（5 signal_class × 5 phase） |
| 2 | 全 closure → `ops_loop.lock.yaml` |
| 3 | crosscutting 05_SLO_protection / 09_audit_gap_monitor / 10_ops_edge |

完了 cell: `ops.loop_closure_complete`

**P10f client**（9 言語 SDK — 1 言語ずつ strict 直列）:

| 順序 | 言語 |
|---|---|
| 1 | Rust（native server と同 proto、L1+） |
| 2 | Go |
| 3 | TypeScript（browser SPA） |
| 4 | C#（.NET 8） |
| 5 | Java 21 |
| 6 | Python |
| 7 | Ruby |
| 8 | .NET Framework（legacy） |
| 9 | Swift |

各言語 pact conformance 後 → `sdk_conformance.lock.yaml`。crosscutting 13_dotnet8_connect / 11_companion_otel を client 側で完成。

完了 cell: `client.sdk_distribution_5class_green`

---

### P11 — 残 proof matrix 完成 → release_gate 全 cell green

**一文**: P5 で前置済 2 件以外の 93 cell proof_matrix と test 90 cell coverage を完成させ、release_gate.lock.yaml の全 20 cell を green にして 1.0.0 git tag を発行する。

**成果物**:
- TLA+（Apalache）: 残 temporal safety/liveness proof
- Stainless / Dafny: program correctness
- Lean 4 + mathlib: cryptographic theorem
- Kani / CBMC: Rust memory safety + crypto invariant
- `proof_matrix.lock.yaml`（95 cell 完成）
- `proof_status.lock.yaml`（全 cell `v1_baseline_verified or v1_accepted_with_assumption`）
- `proof_review.lock.yaml`（dual reviewer cosign signoff 100%）
- `coverage_matrix.lock.yaml`（90 cell 全 green）
- `assumption.lock.yaml`（cap=20 件以内、全 entry に軽減策）
- release_gate.lock.yaml dual signoff（cosign 二重署名）

**完了 (release_gate 全 20 cell green)**:
- `formal.all_critical_verified` / `formal.proof_matrix_complete` / `formal.dual_review_completeness_100pct`
- `test.coverage_matrix_complete`
- `meta.release_gate_dual_signoff_complete`

**1.0.0 git tag 発行 → cosign signed tag が物理 prerequisite → Argo CD production sync。**

---

## Phase 間依存図（DAG）

```
P0 → P1 → P2 → P3 → P4 → P5 → P6 → P7 → P8 → P9 → P10a → P10b → P10c → P10d → P10e → P10f → P11
```

**逆方向依存禁止**（CI で物理拒否）:
- tier3 → tier2 → tier1 → {data, infra}（5 階層論の単方向）

## 至高路線における判断

- **enforcement first, code second**: P0〜P7 の 8 Phase は動くコードゼロ。spec drift を物理的に発生させない構造を先に確立することが、後続の tier1〜client 実装の品質保証の前提。
- **formal 部分前置**: tier1 transport の TLA+ safety proof と data atomic 三表書込 Stainless invariant の 2 件のみ P5 で前置。proof 失敗時の手戻りを最小化しつつ、deadlock-freedom の数学的保証を実装着手前に取得する。
- **単一実装者 strict 直列**: tier1 着手まで 7 Phase 凍結を受け入れ、並列化による spec drift リスクを排除する。
- **段階的 release 禁止**: P11 で全 20 cell が green になるまで 1.0.0 tag は発行しない。partial ship 経路は持たない。

## 関連参照
- [release_gate 対応表](02_release_gate対応表.md)
- [軸内実装順序](03_軸内実装順序.md)
- [crosscutting blocking 表](04_crosscutting_blocking表.md)
- [release_gate 体系](../../04_詳細設計/05_lock_yaml体系/03_release_gate体系.md)
- [generate_release_gate.py](../../../tools/lock_yaml_generator/generate_release_gate.py)

---
id: detail.meta.lock_yaml_index
axis: meta
phase: detail
kind: index
status: draft
depends_on:
  - detail.formal.proof_artifact_system
  - detail.formal.counter_example_system
  - detail.formal.release_gate_system
covered_by:
  defense_in_depth_layers: [A, B, D, E]
  proof_classes: []
---

# lock_yaml 体系 index

## 一文方針
- 本フォルダは全 `*.lock.yaml` 系 build artifact の catalog を集約する。lock.yaml は機械可読な単一の真であり、手書き禁止 + generator output + cosign signed + Object Lock retention の 4 規律で物理 enforce される。

## 至高路線における立ち位置
- 「文章で運用」を全面禁止する。全規律は lock.yaml に物理転写され、機械検証可能な状態で 1.0.0 ship blocker の AND-gate に入力される。
- lock.yaml は generator output として手書き drift = 0 を CI で物理 enforce する。
- `release_gate.lock.yaml` は全 19 軸の lock.yaml 集約 cell を AND-gate で受け取り、1.0.0 ship 可否を決定する。

## lock.yaml catalog（軸別）

### formal 軸
| ファイル | 形式 | 内容 |
|---|---|---|
| `formal_classes.yaml` | 手書き | 5 proof_class 定義 |
| `proof_inventory.lock.yaml` | build artifact | 95 cell の obligation catalog |
| `proof_status.lock.yaml` | build artifact | 各 cell の verified state |
| `counter_example.lock.yaml` | build artifact | 反例 catalog + close_kind |
| `proof_review.lock.yaml` | build artifact | dual signoff |
| `assumption.lock.yaml` | 手書き＋ validate | cryptographic / mathematical assumption（cap 20 件）|
| `mathlib_pin.lock.yaml` | 手書き＋ validate | Lean mathlib revision pin |
| `tla_apalache_pin.lock.yaml` | 手書き＋ validate | TLA+ tools / Apalache version pin |
| `kani_cbmc_pin.lock.yaml` | 手書き＋ validate | Kani / CBMC version + bound parameter pin |
| `proof_review_assignment.yaml` | 手書き | reviewer pool 管理 |
| `proof_minutes_budget.yaml` | 手書き | 四半期 budget |
| `proof_matrix.lock.yaml` | build artifact | 19 axis × 5 proof_class = 95 cell の cell_state matrix |
| `formal_enforcement.lock.yaml` | build artifact | formal 強制機構の admission policy 集約 |

詳細: [proof_artifact 体系](01_proof_artifact体系.md), [counter_example 体系](02_counter_example体系.md)

### test 軸（参考、他軸 spec で詳述）
| ファイル | 形式 | 内容 |
|---|---|---|
| `coverage_matrix.lock.yaml` | build artifact | 18 axis × 5 verification_class = 90 cell |
| `test_classes.yaml` | 手書き | verification_class 定義 |
| `test_status.lock.yaml` | build artifact | 各 cell の verified state |
| `regression_corpus.lock.yaml` | build artifact | property test seed pool（formal counter-example と双方向 lock）|

### ops 軸（参考、他軸 spec で詳述）
| ファイル | 形式 | 内容 |
|---|---|---|
| `ops_loop.lock.yaml` | build artifact | 5 signal_class × 5 phase の operational loop 状態 |

### security 軸（参考、他軸 spec で詳述）
| ファイル | 形式 | 内容 |
|---|---|---|
| `threat_model.lock.yaml` | build artifact | threat catalog + mitigation bind |
| `audit_event.lock.yaml` | append-only | WORM + cryptographic chaining |

### infra / data 軸（参考、他軸 spec で詳述）
| ファイル | 形式 | 内容 |
|---|---|---|
| `topology_class.lock.yaml` | build artifact | 5 topology_class の drill green state |
| `preservation_class.lock.yaml` | build artifact | 5 preservation_class の drill green state |

### meta 軸
| ファイル | 形式 | 内容 |
|---|---|---|
| `axis_registry.lock.yaml` | 手書き＋ validate | 19 軸 + meta-registry（cap v1=20 中 19 / 20）|
| `release_gate.lock.yaml` | build artifact | 全 19 軸の ship blocker cell を AND-gate で集約 |
| `ownership_table.lock.yaml` | 手書き＋ validate | 4 軸 ownership（test / formal / ops / security 共有）|

詳細: [release_gate 体系](03_release_gate体系.md)

## 配下ドキュメント

| 番号 | ドキュメント | 主題 |
|---|---|---|
| 01 | [proof_artifact 体系](01_proof_artifact体系.md) | formal 軸の proof artifact lifecycle 全体 |
| 02 | [counter_example 体系](02_counter_example体系.md) | counter-example の triage / closure / regression bind |
| 03 | [release_gate 体系](03_release_gate体系.md) | 1.0.0 ship blocker の AND-gate 構造 |
| 04 | [artifact_lock 命名規約](04_artifact_lock命名規約.md) | lock.yaml ファイル命名・バージョニング規約 |
| 05 | [immutable archive 体系](05_immutable_archive体系.md) | Object Lock retention / hash chain / cosign 規律 |
| 06 | [dual signoff 体系](05_dual_signoff体系.md) | dual_review.lock.yaml のスキーマと dual signoff CI 8 cell |
| 07 | [proof_matrix 体系](06_proof_matrix体系.md) | proof_matrix.lock.yaml 95 cell スキーマ、cell_state enum、close_kind enum |
| 08 | [verification_matrix 体系](07_verification_matrix体系.md) | coverage_matrix.lock.yaml 90 cell スキーマ、drill_state enum |
| 09 | [assumption 体系](08_assumption体系.md) | assumption.lock.yaml TTL/cap 体系、軸別 cap 表 |

## 共通規律

### 手書き禁止 lock.yaml
以下は全て build artifact、手書きは CI fail:
- `proof_inventory.lock.yaml`
- `proof_status.lock.yaml`
- `counter_example.lock.yaml`
- `proof_review.lock.yaml`
- `proof_matrix.lock.yaml`
- `coverage_matrix.lock.yaml`
- `release_gate.lock.yaml`
- `formal_enforcement.lock.yaml`

### 手書き＋ validate lock.yaml
以下は手書き許容だが generator validate を要する:
- `assumption.lock.yaml`（cap 20 件）
- `mathlib_pin.lock.yaml`
- `tla_apalache_pin.lock.yaml`
- `kani_cbmc_pin.lock.yaml`
- `axis_registry.lock.yaml`
- `ownership_table.lock.yaml`

### 共通 cosign signing
- 全 `*.lock.yaml` は cosign で署名、署名鍵は OpenBao Transit + sign-on-demand
- CI runner に key material は物理に存在しない
- 署名なし artifact は admission policy で reject

### 共通 Object Lock retention
- 全 `*.lock.yaml` は Ceph RGW Object Lock Compliance mode で retention
- retention cap: 10 year（cryptographic proof は 15 year）
- retention 期限内 物理 delete 不可

### 共通 hash chain
- proof_event / audit_event subject は cryptographic hash chain で前後関係 tamper-evidence
- chain divergence detect 時は全 service deploy block（global freeze）

## 関連参照
- [親フォルダ index](../README.md)
- [形式検証適合仕様](../01_適合仕様/20_形式検証適合仕様.md)
- [formal 強制機構](../02_強制機構/10_formal強制機構.md)
- [規約層 index](../../00_format/README.md)

---
id: detail.formal.release_gate_system
axis: meta
phase: detail
kind: detail
status: draft
depends_on:
  - detail.formal.formal_conformance
  - detail.formal.formal_enforcement
  - detail.formal.proof_artifact_system
  - detail.formal.counter_example_system
covered_by:
  defense_in_depth_layers: [A, B, D, E]
  proof_classes: []
lock_artifacts:
  - release_gate.lock.yaml
  - axis_registry.lock.yaml
trace:
  fr_ids:
  - FR-meta-004

---

# release_gate 体系

## 一文方針
- `release_gate.lock.yaml` は全 19 軸の ship blocker cell を AND-gate で集約する build artifact であり、1.0.0 ship 可否は本 lock.yaml の `release_gate_status` field が `green` であることを唯一の真として 00 meta 軸が物理判定する。「段階的 release 禁止」「機能削減なし」「至高路線」の物理転写。

## 設計の物理転写
- 本詳細設計は CLAUDE.md「段階的 release 禁止」を `release_gate.lock.yaml` の AND-gate として物理化する。
- 全軸の ship blocker は本 lock.yaml の cell として登録され、cell が 1 つでも red ならば全 service の deploy が Kyverno admission policy で物理 block される。
- 文章 only の「ship 可」判断は禁止、必ず lock.yaml + cosign signature + dual reviewer signoff を要する。

## artifact / lock.yaml 体系

### `release_gate.lock.yaml`
- 生成元: 各軸 ship blocker lock.yaml の集約 + generator が生成
- 入力:
    - `proof_status.lock.yaml`（formal）
    - `counter_example.lock.yaml`（formal）
    - `proof_review.lock.yaml`（formal）
    - `proof_matrix.lock.yaml`（formal）
    - `coverage_matrix.lock.yaml`（test）
    - `ops_loop.lock.yaml`（ops）
    - `threat_model.lock.yaml`（security）
    - `topology_class.lock.yaml`（infra）
    - `preservation_class.lock.yaml`（data）
    - 各軸の `<axis>_enforcement.lock.yaml`
    - `axis_registry.lock.yaml`
    - `cross_cutting_registry.lock.yaml`
    - `ownership_table.lock.yaml`
- 出力 schema:
    - `release_version`: semver（1.0.0）
    - `release_gate_status`: `["green" | "red"]`（AND-gate 結果）
    - `cells`: array of `{ axis_id, cell_id, status, last_evaluated_at, source_lock_artifact, source_field_path }`
    - `red_cells`: status=red の cell 一覧（detail）
    - `cosign_signature_pointers`: dual reviewer の cosign signature
- enforce 経路:
    - 層 A: jsonschema 検証
    - 層 B: Conftest による全軸 cell 完備 check
    - 層 D: Kyverno `require-release-gate-green` admission policy（本 lock.yaml が red の間 release branch への merge を物理 block）
    - 層 E: cosign signed + Ceph RGW Object Lock retention

## release_gate cell catalog

### formal 軸 cells
| cell_id | 入力 lock.yaml | 条件 |
|---|---|---|
| `formal.all_critical_verified` | `proof_status.lock.yaml` | `count(cells[?cell_state=='v1_baseline_verified']) >= 95`（length check ではなく verified 件数で判定） |
| `formal.proof_matrix_complete` | `proof_matrix.lock.yaml` | 全 cell `cell_state` ∈ {v1_baseline_verified, v1_accepted_with_assumption, v1_unverified_handled} |
| `formal.no_open_above_severity_low` | `counter_example.lock.yaml` | high severity open ゼロ + medium severity decreasing monotonic |
| `formal.dual_review_completeness_100pct` | `proof_review.lock.yaml` | 全 obligation の dual_signoff_complete=true |
| `formal.assumption_cap_within_20` | `assumption.lock.yaml` | cap=20 件以内 + 全 entry に軽減策 + revisit 期限完備 |
| `formal.accepted_with_assumption_ratio_within_cap` | `proof_status.lock.yaml` | `ratio(cells[?cell_state=='v1_accepted_with_assumption'], total_cells) <= 20%`（61% は red、41 cell を v1 必達に巻き戻す義務） |
| `formal.tool_pin_drill_green` | `tla_apalache_pin.lock.yaml` `kani_cbmc_pin.lock.yaml` `mathlib_pin.lock.yaml` | major version migration drill green 維持 |
| `formal.reproducibility_daily_green` | reproducibility check log | 日次 green |
| `formal.cross_axis_lock_drift_zero` | 18 軸との bidirectional lock check | drift ゼロ |
| `formal.slo_4_sli_green` | 13 SLO の v1_proof_* SLI | burn rate 越えなし |

### test 軸 cells（参考、他軸 spec で詳述）
| cell_id | 内容 |
|---|---|
| `test.coverage_matrix_complete` | `count(cells[?drill_state=='verified']) == 90`（pending_with_artifact は品質不足として red） |
| `test.regression_corpus_drift_zero` | `hard_fail_if_zero(entries) AND count(entries[?status=='open']) == 0`（entries=[] の形式的 drift zero を物理的に拒否） |
| `test.mutation_score_monotonic` | quarter で monotonic increase |

### ops 軸 cells（参考、他軸 spec で詳述）
| cell_id | 内容 |
|---|---|
| `ops.loop_closure_complete` | 5 signal_class × 5 phase 全 closure |
| `ops.toil_minutes_within_50pct` | SRE 50% rule 物理 enforcement |

### security 軸 cells（参考、他軸 spec で詳述）
| cell_id | 内容 |
|---|---|
| `security.threat_model_coverage_100pct` | 全 threat に mitigation bind 済み |
| `security.audit_event_chain_no_divergence` | hash chain divergence ゼロ |
| `security.build_provenance_slsa_l3plus` | SLSA L3+ chain 完備 |

### infra 軸 cells（参考、他軸 spec で詳述）
| cell_id | 内容 |
|---|---|
| `infra.topology_class_drill_green` | 5 topology_class 全 drill green |
| `infra.clock_integrity_drill_green` | PTP / chrony / HLC drill green |

### data 軸 cells（参考、他軸 spec で詳述）
| cell_id | 内容 |
|---|---|
| `data.preservation_class_drill_green` | 5 preservation_class 全 drill green |
| `data.restore_drill_quarterly_green` | 四半期 restore drill green |

### tier1 / tier2 / tier3 / client 軸 cells（参考、他軸 spec で詳述）
| cell_id | 内容 |
|---|---|
| `tier1.bidi_conformance_complete` | 5 conformance class 全 verified |
| `tier1.slo_compliance_quarterly_green` | 13 SLO 全 burn rate 越えなし |
| `tier1.oss_lifecycle_drill_green` | major version migration drill 完了 |
| `tier1.tenant_capacity_drill_green` | quota cap 物理 enforcement 確認 |
| `tier2.tenant_isolation_drill_green` | 32 RLS / information flow drill green |
| `tier3.client_state_conformance_complete` | 4-state machine drill green |
| `client.sdk_distribution_5class_green` | 5 distribution_class 全 conformance test green |

### meta 軸 cells
| cell_id | 内容 |
|---|---|
| `meta.axis_registry_complete` | 19 軸全 entry + meta-registry cap=20 中 19 / 20 |
| `meta.ownership_table_complete` | 全 service 4 軸 owner 完備 |
| `meta.docs_lint_green` | docs lint（frontmatter / id / depends_on / forbidden 表現）green |
| `meta.release_gate_dual_signoff_complete` | release_gate.lock.yaml の dual reviewer cosign signature 完備 |

### cross-cutting cluster cells
| cell_id | 入力 lock.yaml | 条件 |
|---|---|---|
| `cross_http2.enforcement_complete` | `cross_cutting_registry.lock.yaml` | HTTP/2 強制 enforcement 全 UA 経路で完全（Envoy h2 only + Companion ALPN h2 必須 + v1_legacy_http11 listener 分離）|
| `cross_kek.shamir_threshold_drill_green` | `cross_cutting_registry.lock.yaml` | M-of-N Shamir threshold drill（property p1〜p5）quarterly green |
| `cross_schema.apicurio_sot_drift_zero` | `cross_cutting_registry.lock.yaml` | git SoT と全 cluster Apicurio instance の drift ゼロ（cross-cluster byte-equal property test green）|
| `cross_fsm.protoc_gen_go_codegen_drift_zero` | `cross_cutting_registry.lock.yaml` | protoc-gen-k1s0-go-fsm 生成コードと proto annotation の drift ゼロ（4 言語 typestate enforcement CI green）|
| `cross_slo.protection_layers_4tier_green` | `cross_cutting_registry.lock.yaml` | SLO 保護四層（rate limiter / cgroup / PG pool / Kafka quota）+ 自動昇格 trigger の全 drill green |
| `cross_bff.auth_edge_isolation_complete` | `cross_cutting_registry.lock.yaml` | BFF auth-edge（06/07 aggregate）: httpOnly cookie 分離 + CSRF/CORS + back-channel logout + Tauri sidecar PSK 非露出の全 conformance green |
| `cross_bff.tauri_sidecar_distribution_green` | `cross_cutting_registry.lock.yaml` | Tauri sidecar cosign 署名 MDM 配布パイプライン green（全 OS variant、v1_no_sidecar degradation path 確認）|
| `cross_pii.dedicated_cluster_drill_green` | `cross_cutting_registry.lock.yaml` | PII 専用 cluster drill（08/09 aggregate）: PII 物理分離 + dual-write atomicity + reconciliation job green |
| `cross_pii.audit_ingest_gap_zero` | `cross_cutting_registry.lock.yaml` | audit ingest gap モニター: 全 source heartbeat 到達 + gap > 0 で 90 sec 以内 page（Chaos drill green）|
| `cross_edge.ops_edge_cluster_independent_green` | `cross_cutting_registry.lock.yaml` | ops-edge cluster（10-13 aggregate）: target cluster kill → ops-edge から page 5 min 以内 drill green |
| `cross_edge.companion_otel_4stack_green` | `cross_cutting_registry.lock.yaml` | .NET Framework Companion OTel 4 stack（WCF / HttpWebRequest / HttpClient / WebClient）JWT claim 注入 E2E green |
| `cross_edge.ua_aware_adapter_capability_matrix_complete` | `cross_cutting_registry.lock.yaml` | UA-aware adapter 5 ua_subclass 全 capability cell 完備（四軸 entry 要件 CI green）|
| `cross_edge.dotnet8_connect_conformance_green` | `cross_cutting_registry.lock.yaml` | .NET 8 Connect-RPC Conformance Suite 全 case green（bidi / server-streaming / unary / client-streaming）|

## AND-gate の意味論
- 全 cell が `status=green` でなければ `release_gate_status=red`
- 1 cell でも `red` ならば 1.0.0 ship 不可
- 「機能削減なし」: cell を削除して red を回避することは禁止（cell 削除は規約変更を要する）
- 「段階的 release 禁止」: 一部 cell が red のまま 1.0.0 ship する経路は持たない

## ship blocker 物理 enforce

### Kyverno admission policy
- `require-release-gate-green`: `release_gate.lock.yaml` の `release_gate_status` が green でない間、release branch（`main`）への merge / production cluster への deploy を物理 block
- `require-release-gate-cosign-signed`: `release_gate.lock.yaml` 自体が cosign signed
- `require-release-gate-dual-signoff`: dual reviewer の cosign signature 完備

### Argo CD ApplicationSet
- production sync wave は `release_gate_status=green` を前提に sync 開始
- staging / preview env は cell 単位の partial green を許容（cell 別 deploy 可）

### CI gate
- PR merge 時に `release_gate.lock.yaml` を再生成、red cells を PR description に列挙
- red cells は PR merge を block するわけではない（development branch では許容）が、release branch への cherry-pick は block

## 1.0.0 ship blocker（formal 軸サマリ）
- 5 proof_class 全てに対し全 19 軸の cross-axis proof closure（5 phase 全 verified）
- `proof_matrix.lock.yaml` の全 cell が `cell_state` ∈ {v1_baseline_verified, v1_accepted_with_assumption, v1_unverified_handled} かつ unverified_unhandled が ゼロ
- counter-example quarterly closure（high severity 0、medium severity decreasing monotonic）
- proof reviewer dual sign-off 100%
- `assumption.lock.yaml` の cap=20 件以内 + 全 entry に軽減策 + revisit 期限完備
- mathlib_pin / tool version pin の major version migration drill green 維持
- reproducibility check 日次 green
- 18 軸との双方向 lock 整合（drift ゼロ）
- 13 SLO 4 SLI green 維持

## release_gate 自体の規律
- `release_gate.lock.yaml` は build artifact、手書き禁止
- generator は各軸 lock.yaml を入力として AND-gate を計算
- generator output と手書き drift は CI fail
- release_gate 変更は dual reviewer signoff 必須

## v2 拡張ルール
- v2 で軸数が +5 されても release_gate cell の追加で対応
- AND-gate の意味論は不変（全 cell green）
- 既存 cell の status 変更は新 cell として表現

## 採用しない設計
- 「ship 緊急」を理由とする cell の skip: 禁止
- cell の手動 green 化: 禁止、必ず lock.yaml の生成を経由
- partial release: 禁止、1.0.0 で全 cell green
- override 経路: 持たない、break-glass は cell 評価には影響しない（緊急 deploy は preview env のみ）

## 関連参照
- [proof_artifact 体系](01_proof_artifact体系.md)
- [counter_example 体系](02_counter_example体系.md)
- [形式検証適合仕様](../01_適合仕様/20_形式検証適合仕様.md)
- [formal 強制機構](../02_強制機構/10_formal強制機構.md)
- [リポジトリ root CLAUDE.md](../../../CLAUDE.md)

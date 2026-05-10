---
id: detail.detail_index
axis: overview
phase: detail
kind: index
status: draft
depends_on:
  - arch.architecture_index
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# 04_詳細設計

## 一文方針
- 本フェーズは詳細設計レベルの 5 ディレクトリで構成。20 適合仕様 + 10 強制機構 + 13 cross-cutting 適合仕様 + 8 運用 UI + 5 lock.yaml 体系 = 全 19 軸の機械可読な単一の真を一箇所に固定。

## 5 ディレクトリ

### [01_適合仕様](01_適合仕様/) — 20 適合仕様
| # | 適合仕様 | 軸 |
|---|---|---|
| 01 | [Bidi 適合仕様](01_適合仕様/01_Bidi適合仕様.md) | tier1 |
| 02 | [移行 Pair 適合仕様](01_適合仕様/02_移行Pair適合仕様.md) | tier1 |
| 03 | [観測適合仕様](01_適合仕様/03_観測適合仕様.md) | tier1 |
| 04 | [認証適合仕様](01_適合仕様/04_認証適合仕様.md) | tier1 |
| 05 | [鍵管理適合仕様](01_適合仕様/05_鍵管理適合仕様.md) | tier1 |
| 06 | [スキーマ進化適合仕様](01_適合仕様/06_スキーマ進化適合仕様.md) | tier1 |
| 07 | [SLO 適合仕様](01_適合仕様/07_SLO適合仕様.md) | tier1 |
| 08 | [OSS ライフサイクル適合仕様](01_適合仕様/08_OSSライフサイクル適合仕様.md) | tier1 |
| 09 | [テナント容量適合仕様](01_適合仕様/09_テナント容量適合仕様.md) | tier1 |
| 10 | [テナント分離適合仕様](01_適合仕様/10_テナント分離適合仕様.md) | tier2 |
| 11 | [クライアント状態適合仕様](01_適合仕様/11_クライアント状態適合仕様.md) | tier3 |
| 12 | [クラスタ位相適合仕様](01_適合仕様/12_クラスタ位相適合仕様.md) | infra |
| 13 | [時刻整合適合仕様](01_適合仕様/13_時刻整合適合仕様.md) | infra |
| 14 | [データ保全適合仕様](01_適合仕様/14_データ保全適合仕様.md) | data |
| 15 | [脅威モデル適合仕様](01_適合仕様/15_脅威モデル適合仕様.md) | security |
| 16 | [build_provenance 適合仕様](01_適合仕様/16_build_provenance適合仕様.md) | security |
| 17 | [運用ループ適合仕様](01_適合仕様/17_運用ループ適合仕様.md) | ops |
| 18 | [クライアント SDK 配布適合仕様](01_適合仕様/18_クライアントSDK配布適合仕様.md) | client |
| 19 | [検証規律適合仕様](01_適合仕様/19_検証規律適合仕様.md) | test |
| 20 | [形式検証適合仕様](01_適合仕様/20_形式検証適合仕様.md) | formal |

### [02_強制機構](02_強制機構/) — 10 強制機構
| # | 強制機構 | 軸 |
|---|---|---|
| 01 | [tier1 強制機構](02_強制機構/01_tier1強制機構.md) | tier1 |
| 02 | [tier2 強制機構](02_強制機構/02_tier2強制機構.md) | tier2 |
| 03 | [tier3 強制機構](02_強制機構/03_tier3強制機構.md) | tier3 |
| 04 | [infra 強制機構](02_強制機構/04_infra強制機構.md) | infra |
| 05 | [data 強制機構](02_強制機構/05_data強制機構.md) | data |
| 06 | [security 強制機構](02_強制機構/06_security強制機構.md) | security |
| 07 | [ops 強制機構](02_強制機構/07_ops強制機構.md) | ops |
| 08 | [client 強制機構](02_強制機構/08_client強制機構.md) | client |
| 09 | [test 強制機構](02_強制機構/09_test強制機構.md) | test |
| 10 | [formal 強制機構](02_強制機構/10_formal強制機構.md) | formal |

### [03_クロスカッティング適合仕様](03_クロスカッティング適合仕様/) — 13 cross-cutting
| # | cross-cutting |
|---|---|
| 01 | [HTTP/2 enforcement](03_クロスカッティング適合仕様/01_HTTP2_enforcement.md) |
| 02 | [KEK Shamir 分散](03_クロスカッティング適合仕様/02_KEK_shamir_distribution.md) |
| 03 | [apicurio GitOps SoT](03_クロスカッティング適合仕様/03_apicurio_gitops_sot.md) |
| 04 | [protoc-gen-go FSM](03_クロスカッティング適合仕様/04_protoc_gen_go_fsm.md) |
| 05 | [SLO protection layers](03_クロスカッティング適合仕様/05_SLO_protection_layers.md) |
| 06 | [BFF auth-edge](03_クロスカッティング適合仕様/06_BFF_auth_edge.md) |
| 07 | [Tauri Companion sidecar](03_クロスカッティング適合仕様/07_Tauri_companion_sidecar.md) |
| 08 | [PII 専用クラスタ](03_クロスカッティング適合仕様/08_PII_dedicated_cluster.md) |
| 09 | [audit_ingest_gap_monitor](03_クロスカッティング適合仕様/09_audit_ingest_gap_monitor.md) |
| 10 | [ops_edge_cluster](03_クロスカッティング適合仕様/10_ops_edge_cluster.md) |
| 11 | [Companion OTel 拡張](03_クロスカッティング適合仕様/11_companion_otel_extension.md) |
| 12 | [Connect-RPC UA-aware adapter](03_クロスカッティング適合仕様/12_UA_aware_adapter.md) |
| 13 | [.NET 8 LTS Connect-RPC 自製実装](03_クロスカッティング適合仕様/13_dotnet8_connect_inhouse.md) |

### [04_運用 UI 開発者体験](04_運用UI開発者体験/) — 8 運用 UI
| # | 運用 UI | 軸 |
|---|---|---|
| 01 | [infra 運用 UI](04_運用UI開発者体験/01_infra運用UI.md) | infra |
| 02 | [data 運用 UI](04_運用UI開発者体験/02_data運用UI.md) | data |
| 03 | [security 運用 UI](04_運用UI開発者体験/03_security運用UI.md) | security |
| 04 | [ops 運用 UI](04_運用UI開発者体験/04_ops運用UI.md) | ops |
| 05 | [client 運用 UI](04_運用UI開発者体験/05_client運用UI.md) | client |
| 06 | [test 運用 UI](04_運用UI開発者体験/06_test運用UI.md) | test |
| 07 | [formal 運用 UI](04_運用UI開発者体験/07_formal運用UI.md) | formal |
| 08 | [開発者体験統合](04_運用UI開発者体験/08_開発者体験統合.md) | overview |

### [05_lock_yaml 体系](05_lock_yaml体系/) — 5 lock.yaml 体系
| # | lock.yaml 体系 |
|---|---|
| 01 | [proof_artifact 体系](05_lock_yaml体系/01_proof_artifact体系.md) |
| 02 | [counter_example 体系](05_lock_yaml体系/02_counter_example体系.md) |
| 03 | [release_gate 体系](05_lock_yaml体系/03_release_gate体系.md) |
| 04 | [artifact_lock 命名規約](05_lock_yaml体系/04_artifact_lock命名規約.md) |
| 05 | [immutable_archive 体系](05_lock_yaml体系/05_immutable_archive体系.md) |

## 全 lock.yaml catalog
- `capabilities.lock.yaml` / `dry_run.lock.yaml` / `idp_capabilities.lock.yaml` / `backends.lock.yaml` / `registries.lock.yaml` / `instruments.lock.yaml` / `oss_inventory.lock.yaml` / `enforcement_points.lock.yaml`（tier1）
- `migration.lock.yaml`（tier2）
- `conflict_tree.lock.yaml`（tier3）
- `sdk_inventory.lock.yaml` / `capability_matrix.lock.yaml` / `sdk_conformance.lock.yaml`（client）
- `proof_inventory.lock.yaml` / `proof_status.lock.yaml` / `counter_example.lock.yaml` / `proof_review.lock.yaml` / `proof_matrix.lock.yaml` / `assumption.lock.yaml` / `mathlib_pin.lock.yaml` / `tla_apalache_pin.lock.yaml` / `kani_cbmc_pin.lock.yaml`（formal）
- `release_gate.lock.yaml`（meta-axis）

## 関連参照
- [01_企画](../01_企画/README.md)
- [02_要件定義](../02_要件定義/README.md)
- [03_概要設計](../03_概要設計/README.md)

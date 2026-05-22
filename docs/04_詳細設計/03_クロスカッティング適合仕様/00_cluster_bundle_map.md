---
id: detail.meta.cross_cluster_bundle_map
axis: meta
phase: cross_cutting
kind: cross_cut_spec
status: published
version: 1.0.0
depends_on:
  - detail.meta.axis_registry_conformance
covered_by:
  defense_in_depth_layers: [A, B]
  proof_classes: []
lock_artifacts:
  - cross_cutting_registry.lock.yaml
trace:
  fr_ids:
  - FR-meta-002

---

# cross-cutting cluster bundle map

## 一文方針

8 cross-cutting cluster (canonical axis 11-18) と 13 cross-cutting 適合仕様の N:M bundle を単一 SoT として管理し、cluster ↔ spec の単一所有規定を enforce する。

## 束ね規約

- 各 spec はちょうど 1 cluster に bind する (multi-cluster bind disallow、CI fail)
- 各 cluster は 1 件以上の spec を所有する (cluster orphan = CI fail)
- spec 追加時は本ファイルの bundle 表を先に更新してから spec frontmatter を書く

## 8 cluster × 13 spec bundle 表

| canonical axis | cluster | spec NN | spec ファイル | related_axes |
|---|---|---|---|---|
| 11 | cross_http2 | c01 | 01_HTTP2_enforcement.md | tier1, tier3, client, infra |
| 12 | cross_kek | c02 | 02_KEK_shamir_distribution.md | tier1, data, security, infra |
| 13 | cross_schema | c03 | 03_apicurio_gitops_sot.md | tier1, data, client, infra |
| 14 | cross_fsm | c04 | 04_protoc_gen_go_fsm.md | tier2, client |
| 15 | cross_slo | c05 | 05_SLO_protection_layers.md | tier1, tier2, ops |
| 16 | cross_bff | c06 | 06_BFF_auth_edge.md | tier3, tier1, tier2 |
| 16 | cross_bff | c07 | 07_Tauri_companion_sidecar.md | tier3, client, infra |
| 17 | cross_pii | c08 | 08_PII_dedicated_cluster.md | data, security, infra |
| 17 | cross_pii | c09 | 09_audit_ingest_gap_monitor.md | security, ops |
| 18 | cross_edge | c10 | 10_ops_edge_cluster.md | ops, infra, security |
| 18 | cross_edge | c11 | 11_companion_otel_extension.md | client, infra |
| 18 | cross_edge | c12 | 12_UA_aware_adapter.md | client, tier1 |
| 18 | cross_edge | c13 | 13_dotnet8_connect_inhouse.md | client, tier1 |

## 概要設計 8 機構 ↔ 詳細 8 cluster 対応表

| 概要設計 8 機構 (03_概要設計/12_クロスカッティング設計) | 詳細 8 cluster |
|---|---|
| 認証コンテキスト伝播 | cross_bff |
| 観測コンテキスト伝播 | cross_edge (companion_otel) |
| スキーマ進化 | cross_schema |
| 鍵管理 (KEK/DEK) | cross_kek |
| OSS lifecycle | (tier1 軸で管理、cross_* 独立 cluster なし) |
| Bidi 適応経路 | cross_http2 |
| 時刻整合 HLC | (infra 軸で管理、cross_* 独立 cluster なし) |
| 数学的 enforcement | cross_fsm |

注: OSS lifecycle と時刻整合 HLC は概要設計 8 機構に含まれるが、詳細設計では独立 cross_* cluster を持たず、cross_slo / cross_pii / cross_edge が cross_* cluster 独自の PII/SLO/edge 領域を扱う。

## CI 不変条件

1. `cross_cutting_registry.lock.yaml` は本表から自動生成し、手書き不可
2. `docs/04_詳細設計/03_クロスカッティング適合仕様/` 直下の各 spec frontmatter `axis:` が本表 `cluster` 列と一致すること
3. cluster orphan（spec を持たない cluster）は 0 件であること
4. spec の multi-cluster bind（1 spec が 2 cluster に assign）は 0 件であること

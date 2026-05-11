---
id: arch.cross_cutting.cross_cutting_index
axis: overview
phase: architecture
kind: index
status: draft
depends_on:
  - arch.overview.architecture_index
  - arch.overview.axis_dependency_diagram
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# クロスカッティング設計 index

## 一文方針
- 19 軸を跨ぐクロスカッティング機構（認証コンテキスト伝播 / 観測コンテキスト伝播 / スキーマ進化 / 鍵管理 / OSS lifecycle / Bidi 適応経路 / 時刻整合 HLC / 数学的 enforcement）の概要を、04_詳細設計/03_クロスカッティング適合仕様/ への入口として一覧化する。

## クロスカッティング機構 8 種

- [01_認証コンテキスト伝播](01_認証コンテキスト伝播.md) — Keycloak OIDC + DPoP + AuthContext + SessionContext + tenant_id 強制注入
- [02_観測コンテキスト伝播](02_観測コンテキスト伝播.md) — W3C Trace Context + Baggage + 5 signal class + 4 dimension layer
- [03_スキーマ進化](03_スキーマ進化.md) — proto + Avro + DDL + yaml + SemConv の 5 軸 aggregate_qualified_name 統合
- [04_鍵管理](04_鍵管理.md) — KeyHandle + KEK shamir M-of-N + envelope encryption + crypto-shred
- [05_OSSライフサイクル](05_OSSライフサイクル.md) — 6 lifecycle_class + 8 signal + 移行 toolchain + L1+ 単一深耕
- [06_Bidi適応経路](06_Bidi適応経路.md) — 5 conformance_class + 8 adapter + UA-aware + Capability Negotiation + Resume
- [07_時刻整合HLC](07_時刻整合HLC.md) — PTP + chrony + HLC（Hybrid Logical Clock）+ wall-clock TTL 禁止
- [08_数学的enforcement](08_数学的enforcement.md) — 5 proof_class + 95 cell coverage + counter-example closure + reviewer dual sign-off

## 詳細設計（cross-cutting 適合仕様）への参照
本 index は概要、機械可読な単一の真は [04_詳細設計/03_クロスカッティング適合仕様/](../../04_詳細設計/03_クロスカッティング適合仕様/) を参照:
- 01_HTTP2_enforcement / 02_KEK_shamir_distribution / 03_apicurio_gitops_sot / 04_protoc_gen_go_fsm / 05_SLO_protection_layers / 06_BFF_auth_edge / 07_Tauri_companion_sidecar / 08_PII_dedicated_cluster / 09_audit_ingest_gap_monitor / 10_ops_edge_cluster / 11_companion_otel_extension / 12_UA_aware_adapter / 13_dotnet8_connect_inhouse

## 関連参照
- [アーキテクチャ概観 README](../01_アーキテクチャ概観/README.md)
- [軸間依存図](../01_アーキテクチャ概観/05_軸間依存図.md)
- [04_詳細設計 README](../../04_詳細設計/README.md)

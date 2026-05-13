---
id: plan.overview.crosscutting_blocking_table
axis: overview
phase: plan
kind: plan_doc
status: draft
depends_on:
  - plan.overview.implementation_phase_definition
  - arch.overview.axis_dependency_diagram
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# crosscutting blocking 表

## 一文方針
- 13 cross-cutting 適合仕様の blocking 軸・P7 実装難度・compile 強制技術を単一の真として保持し、tier1 着手前に全 13 件の enforcement が稼働した状態を確立する根拠とする。

## 13 crosscutting blocking 表

| # | 名称 | 接合軸 | P7 難度 | compile 強制技術 | blocking 理由 |
|---|---|---|---|---|---|
| 01 | HTTP2_enforcement | tier1 / tier3 / client | 中 | Buf lint + Envoy Gateway policy | tier1 Transport が HTTP/2 以外で通信する経路を compile 時に排除 |
| 02 | KEK_shamir_distribution | tier1 / data / security / infra | 高 | HSM 連携 + OpenBao Shamir API | KEK を Shamir 分配以外で生成する経路を禁止。infra + data 先行なくして組立不可 |
| 03 | apicurio_gitops_sot | tier1 / data / client / infra / security | 中 | Argo CD Kyverno admission + Buf codegen | schema の GitOps 以外の変更経路を admission policy で物理 reject |
| 04 | protoc_gen_go_fsm | tier2 / client / test | 中 | buf generate + FSM codegen CI | tier2 状態遷移を手書き実装する経路を codegen 強制で排除 |
| 05 | SLO_protection_layers | tier1 / tier2 / ops | 高 | cgroup v2 + Envoy rate_limit filter + PgBouncer | SLO 四層を全て揃えないと tier2 業務 SLO が空振りする |
| 06 | BFF_auth_edge | tier3 / tier1 / tier2 / client | 中 | Keycloak DPoP/mTLS policy + Kyverno | BFF を経由しない tier3 → tier1 直接呼び出しを admission policy で物理拒否 |
| 07 | Tauri_companion_sidecar | tier3 / client / infra | 低 | Tauri build script + companion API schema | sidecar 未起動での offline state 遷移を compile エラーに |
| 08 | PII_dedicated_cluster | data / security | 高 | LUKS + column envelope + Kyverno namespace policy | PII データを汎用 cluster に混在させる経路を namespace policy で物理拒否 |
| 09 | audit_ingest_gap_monitor | security / data / ops | 中 | Vector + ClickHouse + Falco alert rule | 監査 log の欠損を runtime で自動検出し、gap 発生時に ops loop signal を発報 |
| 10 | ops_edge_cluster | ops / infra | 高 | 独立 k8s cluster + Mattermost webhook | 本番 cluster と同一 k8s に ops コンソールを置く経路を infra topology policy で物理拒否 |
| 11 | companion_otel_extension | client / tier1 / infra | 中 | CLR Profiler + 4-stack IL rewrite | .NET FW client の OTel 計装を IL rewrite 以外の経路で行う実装を lint で警告 |
| 12 | UA_aware_adapter | client / tier1 | 低 | fetch stream polyfill + Buf plugin | UA 判定なしの tier1 直接呼び出しを TypeScript 型制約で compile 時に排除 |
| 13 | dotnet8_connect_inhouse | client / tier1 | 高 | HTTP/3 QUIC system.net policy + Buf codegen | .NET 8 client が inhouse 実装以外の Connect-RPC transport を使う経路を型制約で排除 |

## P7 優先着手順序

難度高の 5 件を先行し、残 8 件を後続で完成させる。

**先行（難度高）**:

| 着手順 | crosscutting | 先行理由 |
|---|---|---|
| 1 | 02_KEK_shamir | infra + security の物理層依存が最大。OpenBao HSM が P8 infra 前提 |
| 2 | 05_SLO_protection_layers | tier1 / tier2 の SLO 計装前提。cgroup v2 の設定が infra topology に依存 |
| 3 | 08_PII_dedicated_cluster | data 軸の cluster 分離が前提。P9 data 着手前に namespace policy 確立が必須 |
| 4 | 10_ops_edge_cluster | infra topology class 前提。`v1_multi_zone` 以上の topology なくして ops-edge cluster 分離不可 |
| 5 | 13_dotnet8_connect | tier1 transport の inhouse 実装前提。P10a tier1 Transport Adapter 完了後に最終化 |

**後続（難度中/低、接合軸の依存方向に従う）**:

| 着手順 | crosscutting |
|---|---|
| 6 | 03_apicurio_gitops_sot（tier1 Schema Registry 前提） |
| 7 | 01_HTTP2_enforcement（tier1 Transport Adapter 前提） |
| 8 | 04_protoc_gen_go_fsm（tier2 CRD 前提） |
| 9 | 06_BFF_auth_edge（tier1 AuthContext + tier2 RLS 前提） |
| 10 | 09_audit_gap_monitor（security threat_model + data WAL 前提） |
| 11 | 07_Tauri_companion_sidecar（tier3 Tauri shell 前提） |
| 12 | 11_companion_otel_extension（tier1 tracing/otel 前提） |
| 13 | 12_UA_aware_adapter（tier1 Bidi streaming 前提） |

## enforcement.lock.yaml 生成条件

`enforcement_status: green` = 当該 crosscutting の Buf lint / Kyverno policy / Falco rule が CI で稼働し、違反 PR が物理 reject される状態。

`enforcement_status` が red の crosscutting を跨ぐ tier1 / tier2 / tier3 実装は P10 で block される（Kyverno admission policy の AND 条件）。

## 至高路線における判断
- 13 crosscutting の enforcement を tier1 着手前に確立することで、「後から制約を追加して既存コードを修正する」という spec drift の最大原因を排除する。
- 難度高 5 件のうち 02 / 05 / 08 / 10 は infra / data Phase の物理層（P8 / P9）に依存するが、compile 強制の lint rule と Kyverno policy 自体は P7 で先に配備する。実際に物理 block が効くのは P8 / P9 以降。

## 関連参照
- [実装 Phase 定義](01_実装Phase定義.md)
- [クロスカッティング適合仕様](../../04_詳細設計/03_クロスカッティング適合仕様/)
- [軸間依存図](../../03_概要設計/01_アーキテクチャ概観/05_軸間依存図.md)

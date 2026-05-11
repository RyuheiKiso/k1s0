---
id: plan.glossary
axis: overview
phase: plan
kind: index
status: draft
depends_on:
  - plan.background_purpose
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# 用語集

## 一文方針
- 本企画で使用する 19 軸 / 5 階層論 / L1+ / 業界 pack / 4 layer state / BusinessConflict subtype / KEK shamir / atomic 三表書込 / クライアント状態 lineage / formal proof_class 等の主要術語を、企画 / 要件 / 概要設計 / 詳細設計 文書全体から参照可能な単一の真として固定する。

## 5 階層論
- **infra 階層**: Kubernetes cluster / OSS / ミドルウェア / 物理 storage / network。tier1 が隠蔽
- **data 階層**: PostgreSQL / Kafka / ClickHouse / Valkey / Apicurio。tier1 Library 経由でアクセス
- **tier1 階層**: Server 系（動かす成果物、Rust 主体）+ Library（配る成果物、4 言語）+ Companion（レガシー言語向け）
- **tier2 階層**: ドメイン業務共通化（業界横断 / 業界 pack / ドメイン）。tier1 経由で infra にアクセス
- **tier3 階層**: 個別業務 UI（Web SPA / exe / レガシー）。tier2 経由で業務資産にアクセス

## 19 軸
- tier1 / tier2 / tier3 / infra / data / security / ops / client / test / formal の 10 主要軸
- + meta 軸（軸自体の追加削除を物理 enforce）+ 8 cross-cutting 軸（HTTP/2 / KEK_shamir / apicurio_gitops / protoc_gen_go_fsm / SLO_protection_layers / BFF_auth_edge / Tauri_companion_sidecar / Companion_OTel_extension）+ 適合仕様軸

## 抽象化レベル（tier1 Library）
- **L3（OSS 中立）**: 業界標準 wire / protocol（OIDC / OTLP / RESP / S3）、OSS 差し替え時業務コード無変更
- **L2\*（同族保証）**: 真に互換性の高い OSS 族、族内差し替えで型置換 + 一部チューニング
- **L1+（単一深耕 + 移行コミットメント）**: 高セマンティクス領域、単一 OSS 全機能を Library API に表現、ライフサイクルイベント時に移行 toolchain で対応

## 抽象化レベル（tier2 業界）
- **業界横断（Cross-industry）**: 通知 / 監査 / 承認 / 帳票 / マスタ管理
- **業界共通（Industry-common）**: 業界 pack 内で複数ドメインに共通（製造業: 設備 / ロット / 品目 / 拠点 / BOM）
- **業界固有（Industry-specific）**: 業界 pack 内ドメイン固有（FA / 調達 / 検査）
- **テナント固有（Tenant-specific）**: テナント差分（マスタ / ルール / 権限）

## 業界 pack
- 1 業界 = 1 pack を原則。1.0.0 では製造業 pack のみ ship
- 業界 pack 並立構造は day-1 から有効（業界中立性の 3 種機械的担保）

## 4 layer client state
- **Server Truth (ST)**: domain_event_subscription / api_response_2xx 由来、in-memory cache
- **Optimistic Local (OL)**: mutation_in_flight 由来、ack で promote / reject で rollback
- **Pending Queue (PQ)**: user_action 由来、IndexedDB encrypted、24h Idempotency-Key TTL
- **Draft (DR)**: form_input 由来、IndexedDB encrypted、confirm 成功で消滅

## BusinessConflict subtype（4 種）
- **stale_write**: base_version < ST.version、field-level disjoint → auto rebase + chained idempotency_key で auto resend
- **lost_update**: base_version < ST.version、field-level intersect → 3-way merge UI
- **supersede**: 同 actor 後続 op 検出 → silent toast
- **concurrent_edit**: presence indicator → user choice

## KEK shamir M-of-N
- N（share total、region 数）= 5、M（share threshold）= 3
- 2 region 完全断まで KEK 物理復元継続可能、3 region 同時断は意図的に「鍵を取れない」状態
- 復元 ceremony は air-gapped host + break-glass two-person rule

## atomic 三表書込（P1〜P4 invariant）
- State change + Outbox + Audit を同一 DB トランザクションに書く
- subscribe 失敗で audit が欠落する事故を構造的に不可能化
- P1: aggregate 状態変更時、必ず Outbox + Audit が同 transaction に書込
- P2: Outbox 投入失敗時、aggregate 状態変更も rollback
- P3: tenant_id mismatch で WITH CHECK reject
- P4: pgaudit が pii_segregated の全 SELECT / INSERT / UPDATE / DELETE を捕捉、欠落ゼロ

## クライアント状態 lineage
- 各 layer の値が「どの source から来たか」「どの version 起点か」を必ず持つ tuple
- ST: `(server_truth, aggregate.version, trace_id, observed_at)`
- OL: `(optimistic_local, base_version, predicted_next_version, idempotency_key, started_at)`
- PQ: `(pending_queue, base_version, idempotency_key, enqueued_at)`
- DR: `(draft, base_version, form_id, edited_at)`

## 5 proof_class（formal 軸）
- **temporal_safety_proof**: TLA+ + Apalache、19 cell（safety property）
- **temporal_liveness_proof**: TLA+ + Apalache、12 cell（liveness property）
- **refinement_proof**: TLA+ / Stainless、35 pair（refinement relation）
- **program_correctness_proof**: Stainless / Dafny / Lean 4、19 cell（program logic）
- **runtime_modelcheck_proof**: Kani / CBMC、14 cell（bounded model checking）

## defense-in-depth 6 層
- **層 A**: compile（codegen / 型 check）
- **層 B**: lint（規約 check）
- **層 C**: integration test / contract test
- **層 D**: runtime（Library / OSS の物理 enforcement）
- **層 E**: 物理（OS / Kubernetes / OSS の物理機能）
- **層 F**: 数学的 enforcement（formal proof）

## 5 verification_class（test 軸）
- **type_invariant**: 型レベル不変条件
- **property_axiom**: property based test
- **contract_pair**: provider / consumer Pact
- **scenario_replay**: scenario corpus 再現
- **fault_chaos**: fault injection chaos drill

## 5 distribution_class（client 軸）
- **v1_full_native_with_companion**: tier1 Library 直接組込（.NET 8 / Java 21 / Node.js 20 / Rust / Go / Python / Ruby）
- **v1_legacy_dotnet_framework**: レガシー .NET Framework 4.6.2+
- **v1_browser_spa_typescript**: Browser SPA（TypeScript）
- **v1_thick_native_via_tauri**: Tauri デスクトップ
- **v1_thin_business_api_only**: polyglot 環境（OpenAPI 仕様のみ）

## L1+ 移行コミットメント 4 primary pair
- **relational_pg_pair**: CloudNativePG → StackGres
- **messaging_kafka_pair**: Apache Kafka → RedPanda
- **workflow_pair**: Temporal → Cadence
- **rule_engine_pair**: ZEN Engine → 自製 DSL backend

## conformance_class（tier1 Bidi）
- **v1_interactive**: 双方向対話、SESSION_ORDERED + SUPPORTED + REQUIRED + 200ms
- **v1_alert**: server-driven 警報、SESSION_ORDERED + UNUSED + REQUIRED + 200ms
- **v1_event_feed**: server-driven Domain Event、SESSION_ORDERED + UNUSED + REQUIRED + 5000ms
- **v1_live_snapshot**: server-driven 最新値、UNORDERED + UNUSED + NONE + 500ms
- **v1_bulk_upload**: client-driven 大量データ、UNORDERED + SUPPORTED + NONE + 0ms

## auth_class（tier1 認証）
- **v1_human_session**: dpop_bound_jwt + medium lifetime + rotating + on_high_risk step_up
- **v1_workload_jwt**: jwt_short + short lifetime + not_applicable refresh
- **v1_device_attest**: jwt_attested + long lifetime + one_shot refresh
- **v1_federated_exchange**: exchange_jwt + short lifetime（RFC 8693）
- **v1_emergency_step_up**: dpop_bound_jwt + very_short lifetime + always step_up

## table_class（tier2 テナント分離）
- **tenant_scoped**: tenant_id 必須 / RLS / audit_local / outbox 必須
- **tenant_master**: tenant_id 必須 / RLS / 変更 audit / outbox 任意
- **platform_global**: tenant_id 不在 / RLS 無効 / 変更 audit / platform-wide outbox
- **pii_segregated**: tenant_id + 専用 role check / pgaudit + audit_local / redact 後 outbox

## ua_subclass（client transport）
- **chrome_edge_direct**: Chromium 105+ + 直結環境、bidi primary
- **chrome_edge_via_corp_proxy**: 企業 FW 越え、paired_post_sse 降格
- **firefox_safari**: full-duplex 不可、paired_post_sse
- **dotnet_companion**: .NET Framework Companion、sse_paired
- **tauri_native**: Tauri sidecar、connect_bidi

## 関連参照
- [背景と目的](../01_背景と目的/README.md)
- [提供する価値や体験](../02_提供する価値や体験/README.md)
- [03_概要設計 アーキテクチャ概観](../../03_概要設計/01_アーキテクチャ概観/README.md)
- [04_詳細設計 README](../../04_詳細設計/README.md)

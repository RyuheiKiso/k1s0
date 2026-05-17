---
id: arch.client.observability_auto_instrumentation_policy
axis: client
phase: architecture
kind: policy
status: draft
depends_on:
  - arch.client.client_index
  - arch.client.sdk_distribution_policy
covered_by:
  defense_in_depth_layers: [A, C, D, E]
  proof_classes:
    - v1_program_correctness_proof
---

# client 観測 Auto-Instrumentation 方針

## 一文方針
- SDK の観測可能性は OpenTelemetry Auto-Instrumentation の L1+ 単一深耕で実現する。distribution_class 別に attach 方式が固定され、SDK 公開 API として disable 経路を持たない。SemConv は tier1 09_観測適合仕様 と双方向 lock。

## Auto-Instrumentation 方式（distribution_class 別）

| class | mode | 実体 |
|---|---|---|
| `v1_full_native_with_companion` | `agent_attach`（.NET / Java）/ `library_auto`（Node / Python / Ruby / Go / Rust） | OTel .NET Auto-Instrumentation / OTel Java Agent / `@opentelemetry/auto-instrumentations-*` / `opentelemetry-instrumentation` / `opentelemetry-ruby auto-load` / `opentelemetry-go-instrumentation` |
| `v1_legacy_dotnet_framework` | `clr_profiler_attach` | `opentelemetry-dotnet-instrumentation`（CLR Profiler、コード変更不要） |
| `v1_browser_spa_typescript` | `library_auto` | `@opentelemetry/sdk-trace-web` + `@opentelemetry/auto-instrumentations-web` |
| `v1_thick_native_via_tauri` | `dual` | Rust 側: `opentelemetry-rust` / WebView 側: `@opentelemetry/auto-instrumentations-web` |
| `v1_thin_business_api_only` | `side_companion_proxy` | OTel Collector sidecar が独立観測 |

mode の選択は固定。SDK 公開 API として override する経路は持たない。

## 注入されるシグナル
- traces: 全 RPC / DB call / external HTTP / IndexedDB I/O / WebCrypto operation の span を自動生成
- metrics:
  - `rpc.client.duration` / `rpc.client.attempt`
  - `http.client.duration`
  - `retry.attempt_count` / `retry.budget_remaining`
  - `circuit_breaker.state` / `circuit_breaker.transition`
  - `sdk.queue.depth` / `sdk.queue.attempts`
  - `sdk.idempotency_key.ttl_remaining`
  - `sdk.token.refresh.count` / `sdk.token.refresh.failure`
  - `sdk.layer.transition.count`（39 conflict tree event 別）
- logs: WARN / ERROR レベルのみ自動 emit、INFO 以下はアプリ側設定
- audit events: `client.*` event 群

## SemConv 整合
- tier1 09_観測適合仕様 の semconv_version との整合:
  - SDK が起動時に server capability descriptor から `observed_semconv_version` を取得
  - 自分の SDK 内蔵 `semconv_version` との互換性 check
  - 不一致は WARN、major 不一致は `OnSemConvIncompatible` event 発火
- SemConv attribute 命名は OTel 公式 + k1s0 prefix（`k1s0.*`）の二系統。命名 drift は CI で検出（codegen 時に固定）

## サンプリング
- default sampling: `parentbased_traceidratio`（rate 1.0 for error trace, 0.05 for success trace）— 9 軸 sampling.yaml と同期
- sampling rate の override は SDK 公開 API として namespace 単位（`rpc.service` prefix 単位）でのみ提供。method 単位 / 個別 RPC 単位 override は禁止
- sampling rate を 0 にする経路は禁止。最小 sampling rate は 0.001（千件に 1）

## exporter
OTLP exporter wire は distribution_class 別に固定:
- `v1_full_native_with_companion`: OTLP gRPC → OTel Collector sidecar（DaemonSet on infra）
- `v1_legacy_dotnet_framework`: OTLP HTTP/protobuf over HTTP/1.1（Companion NuGet 起動時に `OTEL_EXPORTER_OTLP_PROTOCOL=http/protobuf` を強制 set、HTTP/2 stack 不在の WinHTTP / HttpWebRequest 経路と整合）
- `v1_browser_spa_typescript`: OTLP HTTP/JSON → browser fetch、CORS-allowed Collector edge endpoint
- `v1_thick_native_via_tauri`: Rust 側 OTLP gRPC、WebView 側 OTLP HTTP/JSON、Tauri sidecar で merge 後送出
- `v1_thin_business_api_only`: 独立 sidecar OTel Collector が観測

## Browser SPA / Tauri 特殊事情
- Browser SPA:
  - Resource Timing / User Timing API / Long Tasks API / Navigation Timing API を auto-instr
  - Service Worker fetch event を span 化
  - WebSocket / WebTransport stream event を auto-instr
  - First Input Delay / Largest Contentful Paint / Cumulative Layout Shift を metric として export
- Tauri:
  - Rust 側 IPC call と WebView 側 IPC handler の trace を context propagation で stitch
  - `tauri::api::process` / `fs` / `dialog` 等の OS API call を auto-instr
  - OS keychain access も span 化

## レガシー .NET Framework attach 詳細
- `opentelemetry-dotnet-instrumentation` の CLR Profiler を msi インストール → 環境変数 `CORECLR_ENABLE_PROFILING=1` + `COR_ENABLE_PROFILING=1` で attach
- 対象アセンブリの IL を runtime に rewrite、コード変更不要
- Companion NuGet（`k1s0.Companion.NetFx`）が初期化時に OTel exporter / sampler を default 設定で wire up
- Windows Service / IIS hosted ASP.NET 双方サポート
- Companion NuGet は IL rewrite ではなく通常の NuGet 参照。CLR Profiler 経路（観測専用、役割 A）と独立し、Transport Negotiation 役割（05_tier1/04_Library Companion 役割 B）は本 NuGet が明示参照で提供
- 詳細は [Companion OTel 拡張](../../04_詳細設計/03_クロスカッティング適合仕様/11_companion_otel_extension.md)

## exporter / collector の物理冗長
SDK 出口の OTel Collector は 2 段構成:
- 段 1: per-node DaemonSet が SDK OTLP を受け、`loadbalancingexporter`（routing_key: traceId）で段 2 へ転送（consistent hashing 必須）
- 段 2: gateway-aggregator Deployment（trace 単位に集約）が `tail_sampling` processor を実行し、`clickhouseexporter` で ClickHouse traces table へ送出（`tail_sampling` は単一 Collector プロセス内でしか trace grouping できないため、段 1 のみで完結させる構成は不可）
- logs / metrics / profiles 経路は段 1 から直接 SoR（ClickHouse / Prometheus / Parca）へ送出

SDK 内 buffer: 500 span / 1000 metric で `BatchSpanProcessor` の queue を持ち、Collector 失敗時は SDK 内 retry。buffer overflow は `dropped_span_count` metric として記録。

## PII / sensitive data の redaction
- tier1 09_観測適合仕様 の `redaction.yaml` と同期
- HTTP body は capture しない（`uri` / `method` / `status` のみ）
- SQL は normalize（リテラルを `?` 置換）
- HTTP header は allowlist（`authorization` / `cookie` / `set-cookie` は redact）
- 自由テキスト attr は禁止（SDK 内 lint で deny）

## Continuous Profiling
- Parca による pprof continuous profiling は v1 では server side のみ、SDK 側は v2 候補

## trace context propagation
- W3C Trace Context（`traceparent` / `tracestate`）+ Baggage を全 RPC / queue / IPC で propagate
- Browser → Tauri sidecar / Browser → Service Worker / Tauri Rust → WebView の境界も Baggage 経由で propagate
- SDK が自動 attach、アプリ作者が手動 inject する経路は持たない

## exception / panic の記録
- 全 SDK で uncaught exception / panic を OTel exception span event として記録（stack trace 含む）
- PII を含み得る exception message に対し redaction を適用

## 採用しない選択肢
- 言語別 APM agent（NewRelic / DataDog / AppInsights）
- SDK 公開 API での auto-instr 完全 disable
- 自製 trace context 形式
- Browser SPA で console.log を OTel に乗せる経路

## 至高路線における立ち位置
- 「観測機構が大きいから外す」を理由に Auto-Instrumentation を切らない
- 言語横断で SemConv が完全同一であることを CI 物理検証

## 関連参照
- [client 設計方針 index](README.md)
- [Companion OTel 拡張](../../04_詳細設計/03_クロスカッティング適合仕様/11_companion_otel_extension.md)

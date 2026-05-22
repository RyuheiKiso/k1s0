---
id: detail.cross_edge.companion_otel_extension
axis: cross_edge
phase: cross_cutting
kind: cross_cut_spec
status: published
version: 1.0.0
depends_on:
  - arch.client.observability_auto_instrumentation_policy
  - detail.client.sdk_distribution_conformance
  - detail.infra.clock_integrity_conformance
  - detail.meta.axis_registry_conformance
covered_by:
  defense_in_depth_layers: [B, C, D, E]
  proof_classes:
    - v1_program_correctness_proof
related_axes:
  - client
  - infra
trace:
  fr_ids:
  - FR-cross_edge-002

---

# .NET Framework Companion OTel 拡張（v1）

## 位置づけ
- [client 観測 Auto-Instrumentation 方針](../../03_概要設計/09_client設計方針/07_観測Auto-Instrumentation方針.md) / [クライアント SDK 配布適合仕様](../01_適合仕様/18_クライアントSDK配布適合仕様.md) と双方向 lock
- .NET Framework 4.6.2+ の Companion 役割 A（観測可能性）が HttpClient 経路に閉じず、WCF / HttpWebRequest / WebClient / HttpClient の 4 stack 全てで JWT claim → OTel attribute 注入を達成するための拡張パッケージ `k1s0.Companion.NetFx.OTelExt` の仕様

## 不可避性
- DelegatingHandler チェーンに hook できるのは HttpClient 経由の outbound のみ。.NET Framework のレガシー資産は WCF / ASMX / WebClient / HttpWebRequest / WinHTTP 直叩きが多く、HttpClient pipeline を通らない経路では Companion NuGet startup hook が機能しない
- `opentelemetry-dotnet-instrumentation` の CLR Profiler 経路は span 生成までは行うが、Authorization header decode → claim → Activity attribute 注入は標準計装範囲外
- 結果、JWT claim 注入経路が片肺になり、OTel Collector attributes processor の二重持ちでも server 側 decode に限定される（client 側 attribute 不在）

## 採用方針（自製 CLR Profiler は採用しない）
- CLR Profiler 自製（`ICorProfilerCallback` 実装）は工数極大 + .NET 更新追従コスト極大のため不採用
- 代替として、`signalfx/splunk-otel-dotnet`（Apache 2.0、active maintained、4 stack 対応の bytecode instrumentation を OSS で提供）を fork し、k1s0 用 processor を追加した `k1s0.Companion.NetFx.OTelExt` を配布する

## パッケージ構成
- `k1s0.Companion.NetFx.OTelExt`（NuGet package）
  - 内訳:
    - `signalfx/splunk-otel-dotnet` fork bytecode instrumentation
    - k1s0 processor: Authorization header → JWT claim decode → `Activity.Current.SetTag(tenant_id, subject, actor_id, purpose, delegation_chain)`
    - tier1 OTel Collector OTLP exporter 設定
  - 配布形態: NuGet（k1s0 Harbor + nuget.org mirror）
  - 適用方法: 既定経路は CLR Profiler attach（環境変数 `COR_ENABLE_PROFILING=1` + `COR_PROFILER` に splunk-otel-dotnet fork の CLSID を設定 + `COR_PROFILER_PATH` で profiler DLL 配置、`ICorProfilerCallback::ModuleLoadFinished` で IL rewrite を行い HttpClient / HttpWebRequest / WebClient / WCF の各 stack を計装）。k1s0 processor の登録は profiler 初期化時に自動

> 用語注: .NET 用語の "startup hook"（`DOTNET_STARTUP_HOOKS` 環境変数）は .NET Core 5+ の機構であり .NET Framework には存在しない。本企画では当該語を使用しない（CLR Profiler attach 一本に統一）。CLR Profiler attach 不可な環境（4.6.1 以前）では `Application_Start` (ASP.NET) / `Main` 起動直後 (WinForms / WPF) から手動で `OTelExt.Configure()` を呼出して NuGet 参照型 Companion を初期化する degrade 経路を持つ。

## 4 stack の hook 点
- `HttpClient`: DelegatingHandler 自動 inject + bytecode instrumentation で `SocketsHttpHandler` / `HttpClientHandler` の `SendAsync` を計装
- `HttpWebRequest`: `HttpWebRequest.GetResponse` / `BeginGetResponse` を bytecode 計装、request header 中の Authorization を pickup
- `WebClient`: `WebClient.DownloadString` 系の内部 `HttpWebRequest` 経路を経由するため、上記 `HttpWebRequest` の計装で透過対応
- `WCF`: `ChannelFactory` / `ClientBase` の `Invoke` 経路を bytecode 計装、`OperationContext` から SOAP header の Authorization を pickup（HTTP binding の場合）または Message header から custom binding の token を pickup

## Defense-in-depth 二重持ち
- 役割 A の attribute 注入は Companion OTel 拡張（client 側）と OTel Collector attributes processor（server 側）の二経路で並立。片肺障害でも全 span に attribute が必ず乗る
- server 側 Collector は Authorization header を decode して補完（client 側で注入されていない span でも server で attribute 付与）

## Auto-Instrumentation 非対応環境（4.6.1 以前）
- bytecode instrumentation は CLR 4.6.2+ の Auto-Instrumentation（CLR Profiler attach 経路）に依存する。4.6.1 以前は NuGet 参照型 Companion（Application_Start / Main 起動直後から手動で `OTelExt.Configure()` を呼出して計装を初期化する経路、Auto-Instrumentation の自動 attach は不可）に degrade
- 4.6.1 以前環境では `HttpClient` のみ対応、`WCF` / `HttpWebRequest` / `WebClient` は server 側 Collector 補完に依存

## 整合（cross-axis バインディング）

本拡張仕様は以下の cross-axis バインディングを前提として定義される。

- 整合 1: [client 観測 Auto-Instrumentation 方針](../../03_概要設計/09_client設計方針/07_観測Auto-Instrumentation方針.md) において、Companion の JWT claim pickup の主経路は本拡張の bytecode instrumentation（CLR Profiler IL rewrite 経由 k1s0 processor、4 stack 対応）として位置づけられ、DelegatingHandler は HttpClient stack の補助経路に位置づけられる
- 整合 2: [tier1 Library](../../03_概要設計/02_tier1設計方針/02_Library.md) における Companion 役割 A は「4 stack（WCF / HttpWebRequest / HttpClient / WebClient）全てで claim 注入、4.6.1 以前環境では HttpClient のみ Companion 注入 + 他 stack は Collector 補完で degrade」として規定される
- 整合 3: [tier1 観測適合仕様](../01_適合仕様/03_観測適合仕様.md) の `trace_id` 一致要件は、本拡張により Activity が 4 stack 全てに必ず乗ることで物理的に充足する
- 整合 4: [採用 OSS 一覧](../../02_要件定義/04_技術選定/01_OSS採用一覧.md) の client 軸 Companion セクションには `signalfx/splunk-otel-dotnet` fork（Apache 2.0、`dynamic` linkage、`k1s0.Companion.NetFx.OTelExt` として配布）が登録されている
- 整合 5: [検証規律適合仕様](../01_適合仕様/19_検証規律適合仕様.md) の stress test pack `companion_dotnetfx_observability_e2e`（pack #17）として「.NET Framework 4.6.2+ × WCF / HttpWebRequest / HttpClient / WebClient の各 stack で JWT claim が OTel span attribute に乗ること、4.6.1 以前環境は HttpClient のみ Companion + Collector 補完で同等の attribute 到達を満たすこと」を E2E test として必須化し、いずれの stack で attribute 欠落が観測されたら service deploy block する
- 整合 6: [時刻整合適合仕様](../01_適合仕様/13_時刻整合適合仕様.md)（cross-axis double-bound）。Companion 役割 A の OTel attribute 注入は HLC tuple `(tier1.clock.hlc_physical_ms / tier1.clock.hlc_logical_counter / tier1.clock.hlc_node_id)` の 3 attribute を必須包含する。CLR Profiler 内 k1s0 processor が span 生成時に HLC tuple を attribute として attach し、OTel Collector の attributes processor で defense-in-depth の二重持ちが成立する。Companion は `v1_browser_client_skew_tolerant`（4.6.1 未満）/ `v1_application_hlc_only`（4.6.2 以上、`k1s0_hlc_lib` .NET 実装を NuGet 参照）の 2 clock_integrity_class へ明示的に写像し、wall-clock を TTL に使う経路は SDK 表面から完全削除する。Companion 役割 B（Transport Negotiation、`k1s0.Companion.NetFx`）の deadline propagation も sender-issued HLC + receiver-side HLC compare に固定し、wall-clock subtraction は禁止する

## 関連参照
- [client 観測 Auto-Instrumentation 方針](../../03_概要設計/09_client設計方針/07_観測Auto-Instrumentation方針.md)
- [クライアント SDK 配布適合仕様](../01_適合仕様/18_クライアントSDK配布適合仕様.md)
- [時刻整合適合仕様](../01_適合仕様/13_時刻整合適合仕様.md)
- [Connect-RPC UA-aware adapter](12_UA_aware_adapter.md)

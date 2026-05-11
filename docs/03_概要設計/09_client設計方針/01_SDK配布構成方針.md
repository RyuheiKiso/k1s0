---
id: arch.client.sdk_distribution_policy
axis: client
phase: architecture
kind: policy
status: draft
depends_on:
  - arch.client.client_index
  - req.overview.oss_catalog
  - detail.client.sdk_distribution_conformance
covered_by:
  defense_in_depth_layers: [A, B, E]
  proof_classes:
    - v1_program_correctness_proof
---

# client SDK 配布構成方針

## 一文方針
- SDK は sdk_distribution_class（[クライアント SDK 配布適合仕様](../../04_詳細設計/01_適合仕様/18_クライアントSDK配布適合仕様.md)）の宣言で形が決まり、Buf で codegen され、Harbor mirror から各言語 registry 経由で配信される。手動でパッケージを差し込む経路は持たない。

## パッケージ三層（言語横断同型）

### Library パッケージ
- tier1 04_Library で定義された Library 公開 API を直接組み込めるネイティブ言語向け
- 命名:
  - .NET 8 LTS: `k1s0.Library.Net8`
  - Java 21 LTS: `io.k1s0:library-java`
  - Node.js 20: `@k1s0/library-node`
  - Rust: `k1s0-library`
  - Go: `github.com/k1s0/library-go`
  - Python: `k1s0-library`
  - Ruby: `k1s0-library`
- Browser TS は本来「Library を直接組み込めない」側だが、SPA 用に Browser SDK パッケージとして同型 API を提供: `@k1s0/library-web`
- Tauri は `@k1s0/library-web` + tauri sidecar の組合せ

### Companion パッケージ
- tier1 Library を直接組み込めない言語、レガシー資産向け薄 proxy
- 命名:
  - .NET FW 4.6.2+: `k1s0.Companion.NetFx`
  - Browser TS: `@k1s0/companion-web`
  - Tauri sidecar: `k1s0-companion-tauri`（Rust）
  - Python (薄): `k1s0-companion-python`
  - Ruby (薄): `k1s0-companion-ruby`
- ビジネス API は Envoy Gateway 経由で tier1 Server に到達（unary は gRPC-JSON Transcoder、server-streaming は tier1 Rust ハンドラ直接生成 SSE）

### Auto-Instrumentation パッケージ
- 全 distribution_class で同型に observability を注入
- 命名:
  - .NET FW + .NET 8: `opentelemetry-dotnet-instrumentation`
  - Java 21: `opentelemetry-java-instrumentation`
  - Node.js 20: `@opentelemetry/auto-instrumentations-node`
  - Browser TS: `@opentelemetry/auto-instrumentations-web`
  - Python: `opentelemetry-instrumentation`
  - Ruby: `opentelemetry-ruby auto-load`
  - Go: `opentelemetry-go-instrumentation`（eBPF、v2 候補）
- 1.0.0 では default on、disable 経路を SDK 公開 API として持たない（最小 sampling rate 0.001 のみ許容）

## 5 distribution_class（v1、bundle）

| class | 主用途 |
|---|---|
| `v1_full_native_with_companion` | tier1 Library を直接組み込めるサーバ側 / 太い client（.NET 8 / Java 21 / Node 20 / Rust / Go / Python / Ruby） |
| `v1_legacy_dotnet_framework` | レガシー .NET Framework 4.6.2+ exe / Service |
| `v1_browser_spa_typescript` | Browser SPA（TypeScript、UI framework 非依存） |
| `v1_thick_native_via_tauri` | Tauri デスクトップアプリ（Rust + WebView） |
| `v1_thin_business_api_only` | Library / Companion を組み込めない polyglot 環境（OpenAPI 仕様 + curl / Postman / generator） |

各 class の dimension（language_runtime / distribution_channel / transport_capability_class / auto_instrumentation_mode / companion_provision_mode / device_at_rest_encryption_mode / refresh_token_storage_mode）は [クライアント SDK 配布適合仕様](../../04_詳細設計/01_適合仕様/18_クライアントSDK配布適合仕様.md) の `classes.yaml` が単一所有。

## パッケージ命名規約
- .NET: `k1s0.*`（PascalCase）
- Java: `io.k1s0.*`（reverse DNS）
- Node.js / Browser: `@k1s0/*`（npm scope）
- Python: `k1s0_*`（snake_case）
- Ruby: `k1s0/*`（namespace）+ `k1s0-*`（gem name）
- Rust: `k1s0_*`（crate name は kebab → underscore）
- Go: `github.com/k1s0/*`（module path）
- 全言語で「namespace の root が k1s0」を物理 enforce（CI で公開 API snapshot 検査）

## パッケージライフサイクル
- 構築: proto / Apicurio schema を input として Buf workflow で codegen → 各言語 build → Cosign signed → Harbor push
- 更新: [配布 SemVer 方針](05_配布SemVer方針.md) の lockstep release
- 廃棄: major version の sunset は 14 OSS lifecycle 適合仕様の deprecation_window（180 日 + 1 LTS overlap）を踏襲
- 再現性: SDK のバージョン / SHA / SBOM 場所は build artifact として記録、git に commit
- 旧 SDK は Harbor 上に永久保管（pull 可能、push のみ block）

## Companion sidecar の局所性
- .NET FW Companion: 同 process 内 NuGet
- Browser Companion: Service Worker + main thread
- Tauri Companion: Tauri sidecar（別 OS process、IPC 経由）
- thin API: 独立 sidecar pod（infra 側に置く）
- Companion 自体の wire 規約は SDK と同型（gRPC / gRPC-Web / Connect / WebSocket / SSE）。言語別 transport の差異は Companion 内 adapter で吸収

## legacy 資産配慮
- .NET Framework 4.6.2+: opentelemetry-dotnet-instrumentation の CLR Profiler attach + Companion NuGet 二点セットで `v1_legacy_dotnet_framework` class
- 古い Java 8 などは v1 unsupport（v2 候補）。Java 21 LTS への移行コミットメントを request
- Internet Explorer / Edge Legacy: v1 unsupport。Browser SPA は Chromium 系 + Firefox + Safari の latest LTS のみ

## 至高路線における立ち位置
- 「言語数を絞れば運用が楽」を理由に SDK class を削減しない。1.0.0 で 5 class を全 release
- 「Companion を提供すれば Library を諦める」型の分岐を持たない。distribution_class が一意に決める
- パッケージは Cosign signed のみ install 可能。unsigned パッケージは infra 側 Kyverno admission policy で reject

## 関連参照
- [client 設計方針 index](README.md)
- [Transport 適応方針](02_Transport適応方針.md)
- [配布 SemVer 方針](05_配布SemVer方針.md)
- [クライアント SDK 配布適合仕様](../../04_詳細設計/01_適合仕様/18_クライアントSDK配布適合仕様.md)

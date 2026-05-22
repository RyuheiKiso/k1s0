---
id: detail.cross_edge.dotnet8_connect_inhouse
axis: cross_edge
phase: cross_cutting
kind: cross_cut_spec
status: published
version: 1.0.0
depends_on:
  - arch.client.transport_adaptation_policy
  - detail.client.sdk_distribution_conformance
  - detail.client.client_enforcement
  - detail.meta.axis_registry_conformance
covered_by:
  defense_in_depth_layers: [A, B, C, D, E]
  proof_classes:
    - v1_program_correctness_proof
related_axes:
  - client
  - tier1
trace:
  fr_ids:
  - FR-cross_edge-004

---

# .NET 8 LTS 向け Connect-RPC 自製実装 適合仕様（v1）

## 位置づけ
- 02_採用 OSS 一覧 / [client 設計方針 04_採用 OSS] で `connect_bidi` adapter の .NET 8 LTS 実装として宣言された **`k1s0.Connect.NetCore`**（Apache-2.0、in-house authoritative、NuGet 配布）の不可避性 / wire 同等性 / 配布規律 / lifecycle 規律 / ship blocker 解除条件を一箇所に固定する仕様書
- 14_OSS ライフサイクル適合仕様 の lifecycle_class `v1_inhouse_authoritative` の 1 entry。`oss_inventory.lock.yaml` の `k1s0_connect_netcore` entry と双方向 lock
- 05_tier1/07_Bidi 適合仕様 の `capabilities.lock.yaml` の adapter `connect_bidi` の `implementation_per_runtime.dotnet8` と双方向 lock
- [client Transport 適応方針](../../03_概要設計/09_client設計方針/02_Transport適応方針.md) の adapter `connect_bidi` の .NET 8 実装行と双方向 lock
- [client 強制機構](../02_強制機構/08_client強制機構.md) の Connect Conformance Suite CI gate と双方向 lock

## 不可避性（なぜ自製か）
- Connect 公式（`connectrpc.com`）が公開する実装は `connect-go` / `connect-web` (TS) / `connect-node` (TS) / `connect-kotlin` / `connect-swift` / `connect-query` のみ。.NET 用 Connect-RPC の公式 / L1+ active maintained 実装は v1 着手時点で存在しない
- GitHub に community fork が散発的に存在するが、いずれも maintainer 1〜2 名規模で commit cadence が不安定。L1+ 単一深耕の `health_check_cadence`（quarterly）を community fork に握らせると `maintainer_turnover` signal が外部の人間関係に依存する構造的ギャンブル
- .NET Framework 4.6.2+ Companion は WinHttpHandler の HTTP/2 制約（client-streaming / bidi 構造的不可）により Connect bidi 不採用、`sse_paired` 既定で確定済。したがって .NET 8 LTS のみが `capability_class=full` の Connect bidi primary 候補となり、実装が無いと SDK 表面の同型性が崩壊
- 至高路線（CLAUDE.md）の「機能削減で逃げない」原則により、.NET 8 から Connect 経路を撤回することはできない
- 上記 4 点の AND により、自製禁止規律の例外として `v1_inhouse_authoritative` 区画で自製を選択

## wire 同等性（自製 frame protocol ではない）
- `k1s0.Connect.NetCore` は Connect protocol 公開 spec（`https://connectrpc.com/docs/protocol`）の byte-equal 実装であり、独自 wire を発明しない
- 全 RPC form（unary / server-streaming / client-streaming / bidi-streaming）について Connect Conformance Suite（`github.com/connectrpc/conformance`、Apache-2.0）を CI で実行し、全 case を green にすることを ship blocker
- Conformance Suite の release watch を `spec_drift` signal source として 14_OSS ライフサイクル適合仕様 に登録。spec 改訂時は inhouse track PR を発火し、Conformance Suite green=false が連続 2 週で release を block
- Connect protocol の HTTP/2 framing / Content-Type negotiation / error encoding / streaming envelope flag bit / Trailers の各挙動を `property_axiom`（05_tier1/07_Bidi 適合仕様 と同型）で Testcontainers 上の tier1 Rust Connect server と cross-runtime に検証

## 実装方針（薄く、spec 追従）
- 依存:
  - `System.Net.Http`（HTTP/2 stack、`SocketsHttpHandler` の H2C / TLS h2 / TLS h2 / HTTP/3 オプション）
  - `System.Threading.Channels`（bidi back-pressure）
  - `Google.Protobuf`（Apache-2.0、Microsoft 非公式 protobuf-net ではなく公式 `Google.Protobuf`）
- codegen: Buf（remote plugin / vendored buf binary）で `.proto` から `Connect.NetCore.Generated` namespace に生成。tier1 Rust Connect server と同 `.proto` を SoT として並走 codegen。proto drift は 12_スキーマ進化適合仕様 の 5 層 defense-in-depth に乗る
- bidi back-pressure: `System.Threading.Channels` の Bounded / Unbounded を adapter 内部で隠蔽し、Connect protocol の streaming envelope flag bit による End-Stream 通知と Channel completion を 1:1 mapping
- HttpClient lifecycle: Microsoft 推奨の `IHttpClientFactory` 経由 typed client として配布、connection pooling は `SocketsHttpHandler` の既定値 + Connect-specific keep-alive 設定
- error model: Connect Error spec（HTTP status + JSON body `code/message/details`）を例外 `ConnectException` に mapping。gRPC status code から Connect code への変換表は spec 準拠

## 配布規律
- artifact: NuGet `k1s0.Connect.NetCore.dll`（target framework `net8.0`）、ソースは `github.com/k1s0/k1s0.Connect.NetCore`（public OSS、Apache-2.0）
- 配布経路: NuGet.org（公式 registry）+ Harbor mirror（OCI artifact として `.nupkg` を 2nd ref で publish、Cosign 署名対象は Harbor mirror 側のみ。NuGet.org は NuGet repository signing のみ。client 強制機構 の二重署名規律と整合）
- SBOM: Syft で `.nupkg` content を SPDX として artifact 添付。Trivy / Grype で双方向 scan、critical 不一致は max severity 採用 + ticket（10_security/14 整合 12 と整合、admission block しない）
- SemVer: Connect protocol spec の major up を本 NuGet の major up と 1:1 結合（`spec_drift` trigger に紐付け）。public API の C# レベル breaking change も major up を要する
- LTS: .NET 8 LTS の Microsoft サポート期限（2026-11）を `deprecation_clock` に登録、.NET 10 LTS（2026-11 GA 予定）への移行は `major_up` trigger ではなく runtime upgrade として扱う（対象 framework 追加は minor up）

## lifecycle integration
- 14_OSS ライフサイクル適合仕様 `oss_inventory.lock.yaml` の `k1s0_connect_netcore` entry を本仕様が単一所有
- 5 dimension:
  - `adoption_status`: `inhouse_authoritative`
  - `health_check_cadence`: `quarterly`（class bundle 既定）
  - `migration_trigger_set`: `{ spec_drift, cve_backlog_threshold, conformance_drift, major_up }`
  - `response_action.primary`: `inhouse_spec_track_or_runtime_upgrade`
  - `requires_inhouse_spec_memo`: true（本ファイルが該当）
- signal source 宣言:
  - `spec_drift`: `github.com/connectrpc/connectrpc.com` release watch + Conformance Suite CI green 監視
  - `conformance_drift`: Conformance Suite の test case 増設に対する CI green 監視
  - `cve_backlog_threshold`: 内部 grype scan（`.nupkg` SBOM 対象）
  - `major_up`: 自製 NuGet の semver major up（spec 改訂または public API 破壊）

## 5 層 defense-in-depth
- 層 A: compile 時。`oss_inventory.lock.yaml` に登録、build 時に `inhouse_spec_memo: 12_client/18_dotnet8_connect_inhouse.txt` の存在を CI で確認（dead spec 殺し）
- 層 B: lint。本ファイルが他軸（02 / 04 / 06 / 07 / 14）からの参照経路を持つことを grep で確認。参照切れは CI fail
- 層 C: 08 dry_run scenarios と接続。本 entry には `pair_target` が存在しないため dry_run は対象外。代わりに `spec_drift_detected` scenario（14 scenarios.yaml）を quarterly CI で実行
- 層 D: runtime monitoring。Conformance Suite の green 状態を 13_SLO 適合仕様 の SLI として emit、green=false 連続 2 週で alertmanager → page
- 層 E: admission。NuGet は Harbor mirror 経由のみ deploy 許容、非署名 image は Kyverno で block（10_security/14 と整合）

## ship blocker 解除条件（1.0.0）
- SB-1: Connect Conformance Suite 全 case green（unary / server-streaming / client-streaming / bidi-streaming、HTTP/2 + HTTP/3、protobuf binary + JSON）
- SB-2: tier1 Rust Connect server との Testcontainers cross-runtime test green（bidi back-pressure、End-Stream 順序、Trailers 整合）
- SB-3: NuGet 配布 + Cosign 署名（Harbor mirror 経由）+ SBOM attestation + Trivy/Grype 双方向 scan clean
- SB-4: 14_OSS ライフサイクル適合仕様 `oss_inventory.lock.yaml` の `k1s0_connect_netcore` entry が build artifact 生成 dry-run で生成されること（手書き drift 禁止）
- SB-5: 本仕様が 14 / 02 / 04 / 06 / 07 から参照されること（層 B grep）

## 残リスク
- Connect spec 改訂（特に streaming envelope flag bit / Trailers encoding の変更）の追従義務: `spec_drift` signal で 2 週以内に inhouse track PR を発火する規律で吸収、release block を許容することで仕様 drift をゼロ化
- .NET ランタイムの HTTP/2 stack バグ（過去事例: HTTP/2 RST_STREAM handling 等）: Microsoft 公式の patch リリースを runtime upgrade として `major_up` ではなく minor 修正として扱う。`System.Net.Http` の挙動変更が spec 違反を起こした場合は Conformance Suite で検出、回避策実装まで release block
- codegen tool（Buf）の依存: Buf 自体は `v1_l1plus_primary`。Buf の lifecycle event は本 entry の `spec_drift` とは独立に処理される

## 関連参照
- [client Transport 適応方針](../../03_概要設計/09_client設計方針/02_Transport適応方針.md)
- [クライアント SDK 配布適合仕様](../01_適合仕様/18_クライアントSDK配布適合仕様.md)
- [client 強制機構](../02_強制機構/08_client強制機構.md)
- [Connect-RPC UA-aware adapter](12_UA_aware_adapter.md)

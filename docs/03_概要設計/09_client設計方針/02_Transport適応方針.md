---
id: arch.client.transport_adaptation_policy
axis: client
phase: architecture
kind: policy
status: draft
depends_on:
  - arch.client.client_index
  - arch.client.sdk_distribution_policy
  - detail.client.sdk_distribution_conformance
covered_by:
  defense_in_depth_layers: [A, C, D]
  proof_classes:
    - v1_program_correctness_proof
    - v1_temporal_safety_proof
---

# client Transport 適応方針

## 一文方針
- SDK Transport は tier1 07_Bidi 適合仕様 の conformance_class に従い、capability matrix を SDK 側 transport_capability_class として投影する。adapter 切替を SDK 公開 API として露出せず、内部で自動 negotiation する。

## Transport の三層
- 上位（公開 API 層）: SDK が公開する Method / Stream / Event Subscription の API。アプリ作者はこの層しか触らない
- 中位（Adapter 層）: gRPC / gRPC-Web / Connect-RPC / HTTP/3 + WebTransport / WebSocket / SSE / business-API-only HTTP-JSON の各 adapter。distribution_class と endpoint capability から自動選択
- 下位（wire 層）: 実際の TCP / QUIC / WebSocket frame / HTTP/1.1 chunk

三層分離は tier1 03_Server 系 Transport Adapter Layer と対をなし、双方向 lock される。

## 4 transport_capability_class（v1、bundle）

| capability_class | サポート RPC 形式 |
|---|---|
| `full` | unary / server-streaming / client-streaming / bidi-streaming（gRPC bidi 完全） |
| `web` | unary / server-streaming（gRPC-Web 標準）+ bidi-streaming（Connect-RPC over HTTP/2 / HTTP/3、Browser fetch full-duplex streams 必須）+ WebTransport optional |
| `limited` | unary / server-streaming（HTTP/1.1 + JSON unary は Envoy gRPC-JSON Transcoder 派生、HTTP/1.1 chunked + SSE server-streaming は tier1 Rust ハンドラ直接生成。bidi / client-streaming は対象外）。.NET Framework Companion 経路 |
| `business_api_only` | unary（HTTP/1.1 + JSON、Envoy Gateway gRPC-JSON Transcoder 経由）+ HTTP/1.1 + SSE server-streaming optional |

distribution_class → capability_class bundle:
- `v1_full_native_with_companion` → `full`
- `v1_legacy_dotnet_framework` → `limited`
- `v1_browser_spa_typescript` → `web`
- `v1_thick_native_via_tauri` → `full`（Tauri sidecar 経由）
- `v1_thin_business_api_only` → `business_api_only`

dimension override 禁止。

## 6 adapter（client 側、tier1 03 と双方向 lock）
1. `grpc_native`: gRPC Core / grpc-dotnet / grpc-java / tonic / @grpc/grpc-js（HTTP/2）
2. `grpc_web_grpc_only`: gRPC-Web binary（HTTP/1.1 over Envoy gRPC-Web filter）。unary / server-streaming のみ
3. `connect_bidi`: Connect-RPC over HTTP/2 / HTTP/3。bidi / client-streaming / server-streaming / unary すべて成立。implementation_per_runtime:
   - Browser SPA: `@connectrpc/connect-web`
   - Node: `@connectrpc/connect-node`
   - Go: `connectrpc.com/connect`
   - Kotlin: `connect-kotlin`
   - Swift: `connect-swift`
   - .NET 8+: `k1s0.Connect.NetCore`（v1_inhouse_authoritative、Connect 公式 .NET 実装が存在しないため自製。詳細は [13_dotnet8_connect_inhouse](../../04_詳細設計/03_クロスカッティング適合仕様/13_dotnet8_connect_inhouse.md)）
4. `webtransport_quic`: HTTP/3 + WebTransport（QUIC stream）
5. `sse_streaming`: HTTP/1.1 chunked + SSE（text/event-stream、event ごとに protobuf-json payload を data: に載せる）。tier1 03_Server 系 sse_paired adapter と双方向 lock。resume_token は SSE id: と Last-Event-ID リクエストヘッダで運ぶ
6. `http_json_unary`: HTTP/1.1 + JSON unary（Envoy Gateway gRPC-JSON Transcoder unary）

採用しない adapter:
- `websocket_bidi`: 自製 protobuf binary frame の WebSocket 上 bidi（gRPC 公式仕様外、自製 frame protocol 禁止と整合。Connect-RPC を Browser SPA bidi primary に格上げ済）
- `raw_field_protocol`: OPC UA Binary / Modbus / EtherNet/IP 等のフィールドプロトコル（v1 SDK 範囲外、v2 候補）

## capability_class → adapter 集合
- `full` → {`grpc_native`, `connect_bidi`[fallback], `webtransport_quic`[opt]}
- `web` → {`grpc_web_grpc_only`[unary/server-streaming], `connect_bidi`[bidi 用], `webtransport_quic`[opt]}
- `limited` → {`sse_streaming`[server-streaming], `http_json_unary`[unary]}
- `business_api_only` → {`sse_streaming`[server-streaming], `http_json_unary`}

## adapter 選択の決定論的順序
1. RPC method の form（unary / server-streaming / client-streaming / bidi-streaming）を proto から取得
2. capability_class が許容する adapter 集合に form が入るか check
3. 入らなければ compile error（codegen 段階で型 reject）
4. 入るなら preferred_adapter table に従い 1 つを選ぶ
5. preferred_adapter が endpoint capability negotiation で利用不可なら fallback_adapter

「実装者の判断」の余地を持たない。adapter 選択は SDK 内部の純粋関数。

## endpoint capability negotiation
- SDK 起動時 / 周期的 health probe で `GET /capability/probe` から JSON / proto descriptor を取得
- 内容: `supports_bidi` / `supports_connect_rpc` / `supports_fetch_full_duplex_streams` / `supports_webtransport` / `max_message_bytes` / `supports_compression` / `observed_semconv_version` 等
- Browser SPA の bidi RPC は `supports_fetch_full_duplex_streams` が true な environment でのみ成立。未対応 UA では bidi RPC を ConfigurationError として SDK が reject（fail-fast）
- probe の cache TTL は 5 分（短期）。endpoint failover で capability が変わる場合は再 probe + adapter 再選択
- capability 不一致は SDK 内部で BusinessConflict ではなく ConfigurationError として throw
- SDK / 端末側のクライアント環境エラー（IndexedDB 容量不足 / Browser fetch full-duplex 非対応）は ClientCapabilityError として throw（business 層に届かない）

## negotiation の build artifact 化
- capability matrix は build artifact として `capability_matrix.lock.yaml` に固定（[クライアント SDK 配布適合仕様](../../04_詳細設計/01_適合仕様/18_クライアントSDK配布適合仕様.md)）
- server 側 03_Server 系 が公開する capability descriptor との双方向整合は CI で検査

## UA aware 経路選択
- Connect-RPC over fetch full-duplex は中間 proxy / TLS インスペクション / 企業 FW 配下で成立しない
- `chrome_edge_direct` / `chrome_edge_via_corp_proxy` / `firefox_safari` / `dotnet_companion` / `tauri_native` の 5 ua_subclass で adapter を分割し、後者は `paired_post_sse`（POST /open + SSE + POST /send + POST /close）に降格
- 詳細は [Connect-RPC UA-aware adapter](../../04_詳細設計/03_クロスカッティング適合仕様/12_UA_aware_adapter.md)

## deadline propagation
- 全 RPC に `grpc-timeout` / `x-k1s0-deadline-unix-ms` ヘッダで deadline を必ず attach
- SDK ユーザが明示しない場合、SDK が default deadline（method annotation の slo_class から導出）を attach
- deadline 残量を retry budget の判定に組込む（[リトライバックオフ方針](06_リトライバックオフ方針.md)）
- deadline は `monotonic_now() + timeout_ms` で算出、wire 上は HLC tuple として propagate（08_infra/16 時刻整合適合仕様 と整合、wall-clock subtraction 禁止）

## 至高路線における立ち位置
- adapter 選択を実装者裁量に委ねず純粋関数化、capability matrix を build artifact 化することで「環境依存で動く / 動かない」をゼロにする
- WebSocket bidi など自製 frame protocol は v1 で削除し、Connect-RPC L1+ 単一深耕に統一

## 関連参照
- [client 設計方針 index](README.md)
- [SDK 配布構成方針](01_SDK配布構成方針.md)
- [Connect-RPC UA-aware adapter](../../04_詳細設計/03_クロスカッティング適合仕様/12_UA_aware_adapter.md)
- [.NET 8 LTS Connect-RPC 自製実装](../../04_詳細設計/03_クロスカッティング適合仕様/13_dotnet8_connect_inhouse.md)

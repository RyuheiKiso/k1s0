---
id: arch.cross_cutting.bidi_adaptation_path
axis: overview
phase: architecture
kind: policy
status: draft
depends_on:
  - arch.cross_cutting.cross_cutting_index
  - detail.tier1.bidi_conformance
covered_by:
  defense_in_depth_layers: [A, B, C, D, E]
  proof_classes:
    - v1_temporal_safety_proof
---

# Bidi 適応経路

## 一文方針
- bidi semantics（ordering / half_close / resumable / max_msg_lag_ms）を proto で形式化し、Internal の bidi state machine を一つだけ持ち、複数の transport adapter（grpc_native / connect_bidi / grpc_web / web_transport / sse_paired / paired_post_sse / long_poll / webhook / messaging_bridge）が同 conformance scenario corpus を等価実装することを CI で物理証明する。

## 5 conformance_class（[Bidi 適合仕様](../../04_詳細設計/01_適合仕様/01_Bidi適合仕様.md)）
- `v1_interactive`: SESSION_ORDERED + SUPPORTED + REQUIRED + 200ms / bidirectional
- `v1_alert`: SESSION_ORDERED + UNUSED + REQUIRED + 200ms / server→client
- `v1_event_feed`: SESSION_ORDERED + UNUSED + REQUIRED + 5000ms / server→client
- `v1_live_snapshot`: UNORDERED + UNUSED + NONE + 500ms / server→client
- `v1_bulk_upload`: UNORDERED + SUPPORTED + NONE + 0ms / client→server

## 8 adapter
- `grpc_native`（HTTP/2 + gRPC）
- `connect_bidi`（HTTP/2 + Connect-RPC / HTTP/3 + Connect-RPC、要 fetch full-duplex streams）
- `grpc_web`（HTTP/1.1 + gRPC-Web filter）
- `web_transport`（HTTP/3 + QUIC + WebTransport、v1 opt-in）
- `sse_paired`（HTTP/1.1 chunked + SSE、.NET Framework Companion）
- `paired_post_sse`（POST /open + SSE + POST /send + POST /close、半二重 emulation）
- `long_poll`（HTTP/1.1 cursor poll）
- `webhook`（inbound POST、IIS / WCF 同居型）
- `messaging_bridge`（Kafka / AMQP）

## UA-aware 経路選択（[Connect-RPC UA-aware adapter](../../04_詳細設計/03_クロスカッティング適合仕様/12_UA_aware_adapter.md)）
5 ua_subclass:
- `chrome_edge_direct`: Chromium 105+ + 直結環境、connect_bidi
- `chrome_edge_via_corp_proxy`: 企業 FW 越え、paired_post_sse 降格
- `firefox_safari`: full-duplex 不可、paired_post_sse
- `dotnet_companion`: .NET Framework Companion、sse_paired
- `tauri_native`: Tauri sidecar、connect_bidi

## Capability Negotiation
- Open RPC で `client_capabilities` を必須宣言
- Gateway が `chosen_transport` を決定論的に選択
- direction × adapter の構造的不両立は起動時制約として固定

## Resume / Replay の transport 非依存化
- `resumable: REQUIRED` の bidi RPC は transport を跨いで session を継続
- Internal state machine が server-assigned monotonic seq を全メッセージに付与
- `resume_token = (session_id, last_ack_seq)` を Open RPC の optional 入力
- 各 adapter は transport ごとの再接続プロトコル（SSE Last-Event-ID / WebSocket resumption frame / long_poll cursor / WebTransport session ticket）を同一 proto-level seq に機械的にマップ

## .NET 8 LTS 向け Connect-RPC 自製実装（[dotnet8_connect_inhouse](../../04_詳細設計/03_クロスカッティング適合仕様/13_dotnet8_connect_inhouse.md)）
- `k1s0.Connect.NetCore`（v1_inhouse_authoritative、Apache 2.0）
- Connect protocol 公開 spec の byte-equal 実装
- Connect Conformance Suite 全 case green 維持

## HTTP/2 強制（[HTTP/2 enforcement](../../04_詳細設計/03_クロスカッティング適合仕様/01_HTTP2_enforcement.md)）
- 業務 listener は HTTP/2 + HTTP/3 のみ accept、ALPN h2 必須
- per-tab 16 subscription を multiplex で達成
- `v1_legacy_http11` は別ポート 8443-legacy listener に分離

## Conformance Equivalence の機械的担保
- conformance_class ごとに 9 scenario corpus（happy path / 並列メッセージ / half-close / 中断と再開 / slow consumer / proxy buffering 注入 / TLS 切断 / 大メッセージ / 長時間アイドル）
- 全 adapter で同一コーパスを Testcontainers で実行し green を merge 条件
- 個別 adapter が満たさない conformance_class は当該 class の preferred transport から自動除外

## 採用しない設計
- 自製 WebSocket bidi（gRPC 公式仕様外）
- conformance_class dimension override
- proto / 業務コードに transport 種別を露出
- HTTP/2 prior-knowledge / cleartext h2c
- TLS バージョン依存 scenario assertion

## 関連参照
- [Bidi 適合仕様](../../04_詳細設計/01_適合仕様/01_Bidi適合仕様.md)
- [HTTP/2 enforcement](../../04_詳細設計/03_クロスカッティング適合仕様/01_HTTP2_enforcement.md)
- [Connect-RPC UA-aware adapter](../../04_詳細設計/03_クロスカッティング適合仕様/12_UA_aware_adapter.md)
- [.NET 8 LTS Connect-RPC 自製実装](../../04_詳細設計/03_クロスカッティング適合仕様/13_dotnet8_connect_inhouse.md)
- [tier1 Server 系](../02_tier1設計方針/01_Server系.md)
- [client Transport 適応方針](../09_client設計方針/02_Transport適応方針.md)

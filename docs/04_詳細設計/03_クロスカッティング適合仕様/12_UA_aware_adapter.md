---
id: detail.cross_edge.ua_aware_adapter
axis: cross_edge
phase: cross_cutting
kind: cross_cut_spec
status: published
version: 1.0.0
depends_on:
  - arch.client.transport_adaptation_policy
  - detail.client.sdk_distribution_conformance
  - detail.meta.axis_registry_conformance
covered_by:
  defense_in_depth_layers: [A, C, D]
  proof_classes:
    - v1_program_correctness_proof
related_axes:
  - client
  - tier1
trace:
  fr_ids:
  - FR-cross_edge-003

---

# Connect-RPC UA-aware adapter（v1）

## 位置づけ
- [client Transport 適応方針](../../03_概要設計/09_client設計方針/02_Transport適応方針.md) / [クライアント SDK 配布適合仕様](../01_適合仕様/18_クライアントSDK配布適合仕様.md) / 05_tier1/07_Bidi 適合仕様 と双方向 lock
- Connect-RPC を Browser SPA bidi primary として宣言する際、UA ごとの fetch full-duplex streams 実装差を SDK 内部で自動吸収するための adapter 仕様

## 不可避性
- fetch full-duplex streams（`duplex: 'half'` で request body ReadableStream を half-close 前 send）:
  - Chromium 105+: 安定実装、HTTPS + HTTP/2 以上で利用可
  - Firefox: Bug 1428273 配下で実装中、2026 年現在 Nightly で flag 配下、安定 release で一般有効化されていない
  - Safari: 17.4 時点でも request body streaming 未サポート（WebKit Bugzilla）
- `@connectrpc/connect-web` 公式 doc も「bidi-streaming は Chromium / Edge のみ、Firefox / Safari では server-streaming へ自動 degrade」と明記
- 「`v1_browser_spa_typescript` が `v1_interactive`（bidi 必須 class）を全 UA で supports」を能力宣言に残すと Safari / Firefox fleet で必ず ConfigurationError となるサイレント片肺。本 adapter で UA 別に経路を分割
- さらに **chrome_edge であっても全二重 fetch は HTTP/2 over TLS 直結 + 中間 proxy / TLS インスペクション なし** の環境のみ成立する。よって Chromium であっても **企業 FW / DLP appliance / TLS インスペクション越え環境では `paired_post_sse` を primary に降格する**。`chrome_edge` を `chrome_edge_direct` と `chrome_edge_via_corp_proxy` の二 cell に分割し、後者は `firefox_safari` と同経路にマップする

## ua_subclass の導入（transport 軸）
- 本軸 `ua_subclass` は **transport 経路分類** であり、`ua_offline_subclass`（offline storage scope 分類、`ios_safari` / `android_chrome` / `windows_chromium_edge`）とは独立した別の軸
- 両軸は併存し、capability cell の一貫性検査では `(capability_class, adapter, ua_subclass, ua_offline_subclass)` の四軸で 1 entry 以上を持つことを要求
- `capability_matrix.lock.yaml` に `ua_subclass` 軸を追加、5 値:
  - `chrome_edge_direct`: Chromium 105+、HTTP/2 over TLS 直結 + 中間 proxy なし、fetch full-duplex 可
  - `chrome_edge_via_corp_proxy`: Chromium 105+ だが企業 FW / DLP / TLS インスペクション appliance 越え、fetch full-duplex 不成立、`firefox_safari` と同経路（`paired_post_sse`）に降格
  - `firefox_safari`: full-duplex 不可、paired-POST + SSE で半二重 emulation
  - `dotnet_companion`: .NET Framework Companion 経路
  - `tauri_native`: Tauri sidecar 経路
- `chrome_edge_direct` / `chrome_edge_via_corp_proxy` の判定は SDK 起動時 probe（HEAD / OPTIONS で h2 ALPN + 全二重 request body の到達性をテスト）で確定し、tenant 単位で `capability_matrix.lock.yaml` に persist する（環境固定のため再計算は `probe_ttl_days` 単位）
- 全 capability cell は `(class, adapter, ua_subclass)` の三軸で 1 entry 以上を持つこと（CI で検査）

## adapter の二系統

### connect_bidi_native
- 適用 ua_subclass: `chrome_edge_direct`, `tauri_native`
- wire: Connect-RPC over HTTP/2 over TLS、fetch full-duplex streams 必須前提（直結環境のみ成立）
- bidi semantics: proto bidi-streaming 完全等価（flow control / backpressure を HTTP/2 stream window に直接マップ）

### paired_post_sse
- 適用 ua_subclass: `chrome_edge_via_corp_proxy`, `firefox_safari`, `dotnet_companion`
- wire:
  - `POST /k1s0/v1/connect/<rpc>/open` で server-side stream の SSE を確立
  - 以後 client → server は `POST /k1s0/v1/connect/<rpc>/send`（id を session token として）
  - server → client は確立済 SSE で push
  - close は `POST /k1s0/v1/connect/<rpc>/close`
- bidi semantics: 半二重 emulation。client side backpressure は HTTP POST の同期完了で表現、server side backpressure は SSE の `Last-Event-ID` resume で表現
- 等価性証明: 05_tier1/07_Bidi 適合仕様 の bidi conformance class に `paired_post_sse` を追加し、`property_axiom` で完全等価を CI 検査

## SDK 表面 API の同型保持
- SDK 業務コードは bidi semantics（同 RPC method、同 message 型）で書く。adapter 内部で UA 検出 → 経路選択
- SDK 公開 API は両 adapter で同型（proto FQN 対応関係と整合）

## SLO 別二系統管理
`connect_bidi_native` と `paired_post_sse` は wire 効率が異なる（後者は POST 往復のため latency / throughput で劣る）。SLO は ua_subclass 別に設定:
- `bidi_request_latency_p99_ms{ua_subclass="chrome_edge_direct"}`: 30
- `bidi_request_latency_p99_ms{ua_subclass="tauri_native"}`: 30
- `bidi_request_latency_p99_ms{ua_subclass="chrome_edge_via_corp_proxy"}`: 80
- `bidi_request_latency_p99_ms{ua_subclass="firefox_safari"}`: 80
- `bidi_request_latency_p99_ms{ua_subclass="dotnet_companion"}`: 80

11_ops/15_運用ループ適合仕様 / 13_SLO 適合仕様 に二系統 SLO を追記。

## 副作用
- tier1 ingress に `paired_post_sse` 用の 3 endpoint（`/open`, `/send`, `/close`）を追加
- `paired_post_sse` の session 管理（同時 open 数 / TTL / GC）を tier1 で実装、Valkey に session metadata を保管
- Firefox / Safari 進化（fetch full-duplex 一般 release 化）後は `ua_subclass` の判定基準を変更すれば SDK 業務コードは無変更で benefit を得る（adapter 切替透過）

## 整合
- 整合 1: `[client Transport 適応方針](../../03_概要設計/09_client設計方針/02_Transport適応方針.md)` の `transport_capability_class` に `paired_post_sse` adapter を追加、`ua_subclass` 軸で経路振分け
- 整合 2: `[クライアント SDK 配布適合仕様](../01_適合仕様/18_クライアントSDK配布適合仕様.md)` `capability_matrix.lock.yaml` に **transport 軸の `ua_subclass`** = 5 値（`chrome_edge_direct` / `chrome_edge_via_corp_proxy` / `firefox_safari` / `dotnet_companion` / `tauri_native`）を追加、全 cell で `(class, adapter, ua_subclass, ua_offline_subclass)` の四軸 entry が必要（offline 軸の `ua_offline_subclass` は既存の `ios_safari` / `android_chrome` / `windows_chromium_edge` を継続）
- 整合 3: 05_tier1/07_Bidi 適合仕様 「unreachable class 検出」は `ua_subclass` 別に成立することを CI で確認
- 整合 4: 11_ops/15_運用ループ適合仕様 / 13_SLO 適合仕様 に `ua_subclass` 別 SLO 二系統を追加
- 整合 5: 13_test/15_検証規律適合仕様 の bidi conformance test に `paired_post_sse` equivalence proof を追加（property_axiom + scenario_replay）

## 関連参照
- [client Transport 適応方針](../../03_概要設計/09_client設計方針/02_Transport適応方針.md)
- [クライアント SDK 配布適合仕様](../01_適合仕様/18_クライアントSDK配布適合仕様.md)
- [.NET 8 LTS Connect-RPC 自製実装](13_dotnet8_connect_inhouse.md)

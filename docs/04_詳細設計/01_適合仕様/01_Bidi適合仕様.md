---
id: detail.tier1.bidi_conformance
axis: tier1
phase: detail
kind: conformance_spec
status: draft
depends_on:
  - arch.tier1.tier1_index
  - arch.tier1.server_systems
  - detail.tier1.tier1_enforcement
  - detail.infra.clock_integrity_conformance
  - detail.meta.axis_registry_conformance
covered_by:
  defense_in_depth_layers: [A, B, C, D, E]
  proof_classes:
    - v1_temporal_safety_proof
    - v1_program_correctness_proof
lock_artifacts:
  - capabilities.lock.yaml
---

# tier1 Bidi 適合仕様（v1）

## 一文定義
- Bidi 適合仕様は、5 conformance_class × 5 dimension（ordering / half_close / resumable / max_msg_lag_ms / direction）の cross-product として宣言される bidi RPC catalog の各 instance に対し、9 scenario × N adapter の conformance corpus を CI で全数検査し、bidi semantics を transport から切り離して等価実装することを物理 enforce する meta-axis である。

## 位置づけ
- [Server 系](../../03_概要設計/02_tier1設計方針/01_Server系.md) Transport Adapter Layer の意味論的不変量宣言（`tier1.bidi.*` option）の規約と、conformance scenario corpus と、adapter capability matrix の三者を機械可読な単一の真として一箇所に固定
- 03_Server 系 が「動く層」を定義するのに対し、本仕様は「層が等価に動いていることを CI で機械的に証明する規約」を定義
- Buf custom lint、conformance test runner、Capability Negotiation のいずれも本仕様が宣言する 3 つの YAML を共通入力として参照

## 設計原則
- **conformance_class は bundle**: class 1 値が他 4 option（ordering / half_close / resumable / max_msg_lag_ms）を一意に導出
- **dimension override 禁止**: 「`v1_event_feed` のうち lag だけ縮めたい」要望は新 class を切る
- **dead spec を CI で殺す**: classes / scenarios / adapter のいずれも参照消失で CI fail
- **defense-in-depth は 5 層**: Buf lint / scenario assertion / Testcontainers / runtime / 物理 transport

## v1 conformance_class セット（5 class）

| class | ordering | half_close | resumable | lag(ms) | direction |
|---|---|---|---|---|---|
| `v1_interactive` | SESSION_ORDERED | SUPPORTED | REQUIRED | 200 | bidirectional |
| `v1_alert` | SESSION_ORDERED | UNUSED | REQUIRED | 200 | server→client |
| `v1_event_feed` | SESSION_ORDERED | UNUSED | REQUIRED | 5000 | server→client |
| `v1_live_snapshot` | UNORDERED | UNUSED | NONE | 500 | server→client |
| `v1_bulk_upload` | UNORDERED | SUPPORTED | NONE | 0 | client→server |

### 各 class の不変条件と典型用途
- `v1_interactive`: 双方向対話、resume 跨ぎで at-most-once-effective。設備リモート操作、collaborative editing、IME 補助
- `v1_alert`: server-driven 警報配信、低 lag + replay。設備異常通知、SLA 違反通知、安全系 alarm
- `v1_event_feed`: server-driven Domain Event 配信、順序保証 + replay。Domain Event sub、監査ログ tail、品質検査結果配信、受注 sub
- `v1_live_snapshot`: server-driven 最新値表示、latest-wins。feature flag 配信、ダッシュボード tile、ライブ KPI
- `v1_bulk_upload`: client-driven 大量データ投入、at-least-once + idempotent 受信。metric / log / profile / IoT sensor / SCADA テレメトリ

## dimension の根拠
- `SESSION_ORDERED + NONE` は v1 で採らない（順序保証はあるが落ちたら抜けるは業務的にレアな矛盾形）
- `UNORDERED + REQUIRED` も v1 で採らない（順不同で resume は意味論として薄い）
- `direction = client→server` の class（`v1_bulk_upload`）は server→client 一方向 wire を内包する adapter（`sse_paired` / `webhook`）から原理的に supports され得ない
- `half_close = SUPPORTED` は client から終端を伝える局面が要る class にだけ付く（`v1_interactive` / `v1_bulk_upload`）
- `max_msg_lag_ms` は class 内固定値、0 は best-effort（assertion から lag 検査を除外する宣言）

## delivery_guarantee を独立次元にしない理由
- 配送回数の性質は v1 の 5 class それぞれに自然に決まる:
  - `v1_interactive` / `v1_alert` / `v1_event_feed` → at-most-once-effective
  - `v1_live_snapshot` → latest-wins（抜け許容）
  - `v1_bulk_upload` → at-least-once（idempotent 受信）
- v2 で「class の組合せが指数化する」現実問題が出た時点で `delivery_guarantee` を独立次元に昇格する余地は残す（破壊的変更）

## 単一の真

### `classes.yaml`（軸 conformance_class enum）
- class bundle 定義。class 名 → 4 option + invariants の純粋関数テーブル

### `scenarios.yaml`（軸 catalog）
- 9 scenario × option 値 → assertion id のマトリクス
- 9 scenario: `parallel_send` / `resume_after_disconnect` / `slow_consumer_backpressure` / `half_close_initiator` / `proxy_buffering_injection` / `tls_disconnect` / `large_message` / `long_idle` / 1 予約
- assertion id は各言語の test runner が attribute / decorator で claim する索引

#### scenario assertion の transport metadata 中立性
- scenario assertion は TLS バージョン依存挙動を assert しない。例: `tls_disconnect` scenario は「切断後 resume_token 経由で再接続時に `rd_seq_continuous_across_resume` が green」を assert するが、切断トリガが TLS 1.3 close_notify か TLS 1.2 close_notify かは assertion 対象としない
- TLS バージョン / cipher suite / ALPN は [観測適合仕様](03_観測適合仕様.md) の transport metadata（`tier1.transport.tls_version` 等）として別経路で SoR に記録
- 本制約により .NET Framework Companion（TLS 1.2 legacy 例外、08_infra/06 deprecation 12 ヶ月）も同じ scenarios.yaml の全 assertion を green にできる

### `capabilities.lock.yaml`（build artifact）
- 各 adapter crate の static 自己宣言を tier1 build script が集約して生成。手書き禁止
- 各 adapter の `supports`（class 集合）と `constraints`（physical な substrate 制約）を宣言

#### adapter ↔ class supports 対応
| adapter | supports |
|---|---|
| `grpc_native` | 全 5 class |
| `connect_bidi` | 全 5 class（要 fetch full-duplex streams、ua_subclass 別 map） |
| `web_transport` | 全 5 class（v1 opt-in、`requires_fallback: true`） |
| `sse_paired` | `v1_alert` / `v1_event_feed` / `v1_live_snapshot`（client→server bulk は構造的不可） |
| `paired_post_sse` | `v1_interactive` / `v1_alert` / `v1_event_feed` / `v1_live_snapshot`（半二重 emulation） |
| `long_poll` | `v1_event_feed` |
| `webhook` | `v1_alert` / `v1_event_feed` / `v1_live_snapshot` |
| `messaging_bridge` | `v1_event_feed` / `v1_live_snapshot` / `v1_bulk_upload`（partition_key = session_id 制約） |

`paired_post_sse` 詳細は [Connect-RPC UA-aware adapter](../03_クロスカッティング適合仕様/12_UA_aware_adapter.md)、`k1s0.Connect.NetCore` 詳細は [.NET 8 LTS Connect-RPC 自製実装](../03_クロスカッティング適合仕様/13_dotnet8_connect_inhouse.md) を参照。

## Buf custom lint の責務（縮約）
- proto に書かれた `conformance_class` が `classes.yaml` の keys に存在すること
- proto に他 `tier1.bidi.*` option が併記されている場合、その値が `classes.yaml` の bundle 値と一致すること
- bidi RPC で `conformance_class` option が欠落していたら fail
- 廃止予定 class の使用に warn

「矛盾」の概念は「テーブル不一致」に縮約される。lint の判断主観が消える。

## test runner と assertion id の連結
各言語側 test runner は `scenarios.yaml` の assertion id を attribute / decorator で claim:
- Rust: `#[conformance_assert("pl_seq_monotonic_per_session")]`
- C# (Companion): `[ConformanceAssert("pl_seq_monotonic_per_session")]`
- TypeScript: `conformanceAssert("pl_seq_monotonic_per_session", ...)`

CI は次の関係性を全数検査:
- `scenarios.yaml` の全 assertion id について、対応 adapter × 対応言語で実装が存在し green
- assertion id の typo / 未参照は fail
- adapter が claim する class について、`scenarios.yaml` が要求する全 assertion id を adapter 側 fixture が走らせる

## 既存軸との接続関係（08_infra/16 時刻整合 cross-axis 双方向 lock）
- `resume_token` の TTL に server-anchor HLC tuple を必須包含、client wall-clock 由来の TTL 計算を禁止
- Browser SPA / .NET Framework Companion / Tauri webview client は `v1_browser_client_skew_tolerant` 経路で server-issued HLC token のみを opaque pass-through、native client は `v1_application_hlc_only` 経路で monotonic + HLC で TTL 判定
- `SESSION_ORDERED` / `CAUSAL` conformance class の `property_axiom` に「±60s skew 注入下で HLC happens-before respected」を必須 assertion として追加
- Last-Event-ID resume / SSE `id:` フィールドの retry 時刻判定は server 側 HLC の delta で行い、client wall-clock subtraction を禁止
- 08_infra/16 の `clock_causality_proof.lock.yaml` と本仕様の bidi conformance test fixture が双方向 lock

## Capability Negotiation との関係
03_Server 系 Capability Negotiation の `chosen_transport` 選択は、起動時に `capabilities.lock.yaml` を読み込んで実装:
1. proto の `conformance_class` C を取得
2. lock.yaml で C を supports に含む adapter 集合 A を抽出
3. constraints 節で fleet の物理条件を満たさない adapter を A から除外
4. クライアントが Open RPC で宣言した `client_capabilities` ∩ A
5. サーバ側で定義された優先順位でソートして `chosen_transport` を返却

`capabilities.lock.yaml` は build artifact のため、サーバの起動とともに固定される。deploy 中に動的に書き換わらない。

## 5 層 defense-in-depth
- 層 A: Buf lint（proto → `classes.yaml` 照合）
- 層 B: scenario assertion（`scenarios.yaml` → test runner attribute claim）
- 層 C: Testcontainers での全 adapter × 全 class × 全 scenario fixture 実行
- 層 D: runtime（Capability Negotiation で `chosen_transport` 選択、実 transport で bidi state machine が動作）
- 層 E: 物理 transport（HTTP/2 / HTTP/3 / WebSocket / SSE / TCP の物理機構）

## CI 不変条件（merge 不可）
- 整合 1: `classes.yaml` の全 class が `scenarios.yaml` の matrix 条件で少なくとも 1 scenario によりカバーされる（dead class 検出）
- 整合 2: `scenarios.yaml` の全 assertion id が、それを必要とする全 class × 全 adapter × 全対応言語で実装されている
- 整合 3: `capabilities.lock.yaml` で claim される class が `classes.yaml` に存在（dangling reference 検出）
- 整合 4: `classes.yaml` に登録された class のうち、いずれの adapter からも supports されない class が存在しないこと（unreachable class 検出）
- 整合 5: proto に `conformance_class` フィールドが書かれた全 RPC が、いずれかの adapter から到達可能であること（unreachable RPC 検出）
- 整合 6: `capabilities.lock.yaml` は手書き禁止のため、build script が生成した内容と git 上の内容が完全一致すること（generation drift 検出）

## 1.0.0 ship blocker
- 5 conformance_class × 8 adapter の完全 conformance green
- 製造業 pack 10 RPC stress test 全 green
- 08_infra/16 clock_causality_proof との triple-bound（Bidi + Clock + Client）green

## 製造業 pack stress test（v1 の正当性証跡）

| # | RPC | class |
|---|---|---|
| 1 | 設備リモート操作 | `v1_interactive` |
| 2 | ライン稼働監視 (live tile) | `v1_live_snapshot` |
| 3 | 品質検査結果配信 | `v1_event_feed` |
| 4 | SCADA テレメトリ収集 | `v1_bulk_upload` |
| 5 | 図面 collaborative review | `v1_interactive` |
| 6 | 在庫最新値表示 | `v1_live_snapshot` |
| 7 | 受注 sub (基幹 → 製造管理) | `v1_event_feed` |
| 8 | 警報配信 | `v1_alert` |
| 9 | 計量装置連続データ | `v1_bulk_upload` |
| 10 | 出荷指示双方向確認 | `v1_interactive` |

## v2 拡張ルール
- 新 class の追加は purely additive。既存 class の dimension 値変更は破壊的変更（major version up）
- 命名は `vN_<usage>` 形式に固定
- dimension override 禁止の規律は v1 → v2 跨ぎでも維持
- `delivery_guarantee` 等の独立次元昇格は v2 以降で検討

## 採用しない設計
- 自製 WebSocket bidi（protobuf binary frame）
- conformance_class dimension override
- proto / 業務コードに transport 種別を露出
- `SESSION_ORDERED + NONE` / `UNORDERED + REQUIRED` の組合せ
- TLS バージョン依存 scenario assertion

## 関連参照
- [Server 系](../../03_概要設計/02_tier1設計方針/01_Server系.md)
- [tier1 強制機構](../02_強制機構/01_tier1強制機構.md)
- [HTTP/2 enforcement](../03_クロスカッティング適合仕様/01_HTTP2_enforcement.md)
- [Connect-RPC UA-aware adapter](../03_クロスカッティング適合仕様/12_UA_aware_adapter.md)
- [.NET 8 LTS Connect-RPC 自製実装](../03_クロスカッティング適合仕様/13_dotnet8_connect_inhouse.md)
- [時刻整合適合仕様](13_時刻整合適合仕様.md)

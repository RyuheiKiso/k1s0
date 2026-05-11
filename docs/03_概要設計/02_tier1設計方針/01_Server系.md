---
id: arch.tier1.server_systems
axis: tier1
phase: architecture
kind: policy
status: draft
depends_on:
  - arch.tier1.tier1_index
  - arch.tier1.libraries
  - detail.tier1.bidi_conformance
  - detail.tier1.schema_evolution_conformance
covered_by:
  defense_in_depth_layers: [A, B, C, D, E]
  proof_classes:
    - v1_program_correctness_proof
    - v1_property_axiom_proof
---

# tier1 Server 系（ランタイム成果物）

## 一文方針
- Server 系は tier1 が「動かす」ランタイム成果物の総称。tier2 / tier3 に「配る」コード成果物である Library と対をなし、`.proto` を単一の真として External / Internal 二層 + Transport Adapter Layer で全 transport / 全言語に同型投影する。

## 5 分類

### Gateway
- 役割: Library を直接組み込めない言語（.NET Framework など）からの呼び口を提供。観測可能性 / 認証コンテキストの基盤レイヤを担う
- 方式:
  - ビジネス API: `.proto` を単一の真とし、HTTP/1.1 + JSON（unary）は Envoy Gateway の gRPC-JSON Transcoder で派生、HTTP/1.1 + SSE（text/event-stream、server-streaming）は tier1 Rust ハンドラ内で SSE フレーミングを直接生成
  - 観測可能性: W3C Trace Context（traceparent / tracestate / baggage）を未着信なら生成、着信していれば継承。リクエストメトリクス（status / latency / size）を Prometheus に送出
  - 認証コンテキスト: `envoy.filters.http.jwt_authn` で OIDC トークンを検証し、クレームを内部ヘッダに展開。tier1 Backend-for-Library 側で span attribute / 構造化ログに注入
- 実装: Envoy Gateway + tier1 製ハンドラ（Rust）

### Sidecar / Agent
- 役割: Library から切り離して常駐させたい処理を担う
- 例: Outbox relay、シークレット同期、トレース送出のバッファ
- 実装: Rust 製常駐プロセス

### Backend-for-Library
- 役割: 業務側プロセスに置けない権限 / 鍵で動く処理を集中化
- 例: トークン検証の集中点、認可ポリシー一括評価、Schema Registry の互換性強制プロキシ
- 実装: Rust gRPC サーバ

### Control Plane
- 役割: Library / Gateway / Sidecar への動的設定配信を担う
- 例: OpenFeature / flagd の前段、テナント設定配布
- 実装: Rust + flagd 連携

### Operator / Controller
- 役割: Kubernetes リソースの宣言的管理を担う
- 例: CloudNativePG、Strimzi、Keycloak 等のテナント別リソース自動化
- 実装: Go（controller-runtime / kubebuilder）

## 配布形態
- コンテナイメージ + Helm / Operator マニフェストとして配布
- Library がパッケージレジストリ配布であるのに対し、Server 系はプラットフォーム側で「動かす」成果物として扱う
- tier2 / tier3 から見た契約面（API / プロトコル）は Library と同様、`.proto` を単一の真とする

## Proto 二層化（External / Internal）

### 背景
- gRPC-JSON Transcoder は streaming RPC、`google.protobuf.Any`、一部 well-known types の JSON 表現に制約がある
- Library 非対応言語の都合で内部 RPC の proto まで妥協する「逆流」を避けるため、`.proto` を External / Internal の 2 層に分離する

### 長期方針（漸進的な単層化）
- External / Internal の二層分離は、現時点での gRPC-JSON Transcoder の制約と Library 非対応言語の receive 能力差を吸収するための過渡期の最適解
- Transport Adapter Layer が整備され、HTTP/3 + WebTransport を含む全 transport で bidi を含む全 RPC 形式の意味論的等価性が conformance proof で保証される段階に達した時点で、External の制約は段階的に解消され、二層は単層 proto + transport ladder へ collapse する

### External Proto
- 用途: Gateway 経由で Library 非対応言語に公開する API 専用
- 制約:
  - Transcoder は unary 専用、streaming 派生は Transcoder に行わせない
  - server-streaming RPC は Envoy Gateway が独立 upstream の tier1 Rust SSE ハンドラ Pod へ HTTP/1.1 chunked で proxy。Pod 間通信は Istio sidecar mTLS で保護。Envoy access log と Rust ハンドラ側 span は trace_id（W3C traceparent）で連結
  - Rust ハンドラ内部で `text/event-stream` の SSE フレーミング（`id:` / `event:` / `data:` / `retry:`）を直接生成。`id:` フィールドは server-assigned monotonic seq に固定（Last-Event-ID として標準化、resume_token 表現に独自拡張不要）
  - client-streaming / bidirectional streaming は External に直接出さず、後述「RPC 形式の射影方針」に従って unary の組み合わせに射影
  - `google.protobuf.Any` 禁止（既知型の oneof に置換、真に動的な場合は `bytes` + content_type）
  - well-known types（Timestamp / Duration / FieldMask 等）の JSON 表現を固定（RFC3339 / `"1.5s"` / カンマ区切り など）
  - oneof は discriminator フィールドを併記
  - 全 RPC に `google.api.http` アノテーションを必須とし、HTTP メソッド / パス / pagination フィールド命名を規約化
- 強制: Buf の lint ルールとして CI で機械的に強制
- 互換性: レガシー側の互換性を最優先、破壊的変更は厳格に管理（buf breaking 相当の差分検査を CI）

### Internal Proto
- 用途: tier1 Server 系内部 / Library を組み込んだ言語（Rust / C# / Go / TypeScript）間の RPC
- 制約: proto のフル機能（streaming、Any、深いネスト等）を許容、External の制約は適用しない

### 変換境界
- External Proto を受けた tier1 ハンドラ（Gateway 配下）で Internal Proto への射影を行う。Backend-for-Library 配下の権限処理（トークン検証 / 認可ポリシー評価等）は Internal Proto で受ける
- 「External 経路の都合で Internal Proto を曲げない」ことを構造的に担保するため、External ↔ Internal の対応関係（フィールド射影、欠落フィールドの既定値）は tier1 リポジトリ内で宣言的に管理し、CI で差分検査

### RPC 形式の射影方針
- unary: External はそのまま JSON 派生
- server-streaming: External は SSE 派生。Envoy が unary を Transcoder で派生、server-streaming は tier1 Rust ハンドラにルーティングして Rust ハンドラ内で SSE フレーミングを直接生成。レガシー側の event 消費 / resume は Companion 役割 B の `sse_paired` adapter に委譲
- client-streaming: External には直接出さず、resumable upload 形式（Init → 分割 PUT → Commit）の unary 3 連 RPC に射影
- bidirectional streaming: External には Internal の bidi RPC をそのまま「bidi の意味論を持つ proto 表面」として公開。射影は wire 形式ではなく Transport Adapter Layer が担う
- 長時間ジョブの進捗: 進捗を streaming で外に出さない。「Start RPC → cursor 付き GetStatus / Poll RPC」のペアで表現（Temporal の Query / Signal と整合）

「.proto を単一の真」とする原則は維持され、真とは External / Internal の両 proto セットの集合。

## Transport Adapter Layer

### 位置づけ
- bidi を含む RPC 形式を、transport（wire 上の搬送方式）から切り離して提供する Gateway 内部のレイヤ
- tier2 / tier3 から見える `.proto` を「bidi semantics の単一の真」とし、それを実現する transport を複数並立させる
- 本節は方針定義に留め、conformance_class の bundle 定義 / scenario × class assertion id マトリクス / adapter capability matrix の機械可読仕様は [Bidi 適合仕様](../../04_詳細設計/01_適合仕様/01_Bidi適合仕様.md) に分離

### 設計の核
- bidi semantics を proto で形式化
- Internal の bidi state machine を一つだけ持ち、複数の transport adapter がその front-end として dispatch
- 各 adapter は同一の conformance scenario corpus に対して green を merge 条件とし、意味論的等価性を CI で証明
- クライアントが capability を宣言し、Gateway が「conformance を満たす中で最高優先度」の transport を選択

### bidi semantics の proto 形式化
bidi RPC には次の option を必須とする:
- `tier1.bidi.ordering`: `SESSION_ORDERED` | `CAUSAL` | `UNORDERED`
- `tier1.bidi.half_close`: `SUPPORTED` | `UNUSED`
- `tier1.bidi.resumable`: `REQUIRED` | `NONE`
- `tier1.bidi.max_msg_lag_ms`: realtime 上限（数値、0 = 無指定）
- `tier1.bidi.conformance_class`: テストケース集合の識別子（例 `"tier1.bidi.v1"`）

`conformance_class` は bundle として他 4 option の値を一意に導出。RPC 作者が dimension を個別に override する経路は持たない。class カタログ（v1 では 5 class）の確定形と bundle 値は [Bidi 適合仕様](../../04_詳細設計/01_適合仕様/01_Bidi適合仕様.md) `classes.yaml` 節を参照。

### Adapter 一覧（初期セット）

| adapter | substrate | 主な対象 |
|---|---|---|
| `grpc_native` | HTTP/2 + gRPC | Library 対応言語（直結） |
| `connect_bidi` | HTTP/2 + Connect-RPC / HTTP/3 + Connect-RPC | Browser SPA bidi primary（Browser fetch full-duplex streams 必須） |
| `grpc_web` | HTTP/1.1 + gRPC-Web filter | Browser SPA unary / server-streaming |
| `web_transport` | HTTP/3 + QUIC + WebTransport | 将来のモダンクライアント（v1 opt-in） |
| `sse_paired` | HTTP/1.1 chunked + SSE（text/event-stream） | .NET Framework 4.6.2+ outbound |
| `long_poll` | HTTP/1.1 cursor poll | 長時間接続を切る proxy 越え |
| `webhook` | inbound POST | IIS / WCF 同居型レガシー |
| `messaging_bridge` | Kafka / AMQP | ネットワーク完全分離環境 |

採用しない adapter:
- 自製 WebSocket bidi（protobuf binary frame）。gRPC-over-WebSocket は gRPC 公式仕様外、Connect-RPC を Browser SPA bidi primary に格上げ済のため不要

### Capability Negotiation
- Open RPC で `client_capabilities`（サポート adapter 一覧 / TLS バージョン / inbound 可否 / max message size 等）を必須宣言
- Gateway は declared capability ∩ proto の `conformance_class` を満たす adapter set の中から、サーバ側で定義した優先順位に従って `chosen_transport` を返す
- direction × adapter の構造的不両立を Negotiation の起動時制約として固定。`direction = client→server` の class（`v1_bulk_upload`）は、server→client 単方向 wire を内包する adapter（`sse_paired` / `webhook`）の supports から除外され、bulk 経路は `grpc_native` / `connect_bidi` / `web_transport` / `messaging_bridge` のみが選択肢
- 同一 fleet 内で transport の混在を許容。古いクライアントは next-best へ自動的に落ち、新しいクライアントは最良 transport を享受

### Conformance Equivalence の機械的担保
- `conformance_class`（`tier1.bidi.v1` など）ごとに固定の scenario corpus を tier1 リポジトリで保持
  - happy path / 並列メッセージ / half-close / 中断と再開 / slow consumer / proxy buffering 注入 / TLS 切断 / 大メッセージ / 長時間アイドル
- 全 adapter で同一コーパスを Testcontainers で実行し green を merge 条件
- 個別 adapter が満たさない `conformance_class` が出た場合、その adapter は当該 class の preferred transport から自動除外。proto 側 / 業務コード側の修正は不要

### Resume / Replay の transport 非依存化
- `resumable: REQUIRED` の bidi RPC は、transport を跨いで session を継続できることを保証
- Internal state machine が server-assigned monotonic seq を全メッセージに付与し、`resume_token = (session_id, last_ack_seq)` を Open RPC の optional 入力として受ける
- 各 adapter は transport ごとの再接続プロトコル（SSE の Last-Event-ID / WebSocket の resumption frame / long_poll の cursor / WebTransport の session ticket 等）を、同一の proto-level seq に機械的にマップ
- ネットワーク瞬断時に adapter を跨いだ降格（例: `connect_bidi` → `sse_paired`）でも session を継続

### 長期収斂
- Adapter Layer の整備が進み、HTTP/3 + WebTransport が標準化と runtime 普及を達成した段階で、External Proto の制約および tier1 Rust ハンドラ内 SSE フレーミング生成は不要になる
- この時点で Proto 二層化は単層へ collapse し、`sse_paired` adapter および Rust ハンドラ内 SSE フレーミング生成経路は同期して退役
- Transcoder upstream への SSE 派生 option 寄稿は本企画のスコープ外であり、Transcoder 由来制約の解消は WebTransport 普及による `sse_paired` 退役そのもので達成
- 古い adapter（`webhook` / `long_poll` / `sse_paired`）は capability 宣言から段階的に退役。proto と業務コードは無変更で遷移
- 本レイヤの哲学は「単一 OSS 深耕 + 移行コミットメント」を transport 軸に転写したもの

## 関連参照
- [tier1 設計方針 index](README.md)
- [Library](02_Library.md)
- [Bidi 適合仕様](../../04_詳細設計/01_適合仕様/01_Bidi適合仕様.md)
- [スキーマ進化適合仕様](../../04_詳細設計/01_適合仕様/06_スキーマ進化適合仕様.md)
- [HTTP/2 enforcement](../../04_詳細設計/03_クロスカッティング適合仕様/01_HTTP2_enforcement.md)

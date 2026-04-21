# 03. Go / Rust 間言語境界方式

本ファイルは IPA 共通フレーム 2013 の **ソフトウェア方式設計プロセス 7.1.2.2** のうち、tier1 内部の Go（Dapr ファサード層）と Rust（自作領域）の言語境界における通信・型マッピング・エラー統一・OTel span 伝搬・ライフサイクル制御を方式として固定化する。共有メモリ / FFI / UNIX socket は禁止し、Protobuf gRPC のみを唯一の境界プロトコルとする。

## 本ファイルの位置付け

構想設計 ADR-TIER1-001（Dapr Go SDK + 自作 Rust のハイブリッド）が言語分担を確定し、ADR-TIER1-002（Protobuf gRPC 必須）が境界プロトコルを確定している。本ファイルはこれらの制約の下で、実装レベルでの境界挙動を仕様化する。Go と Rust という 2 言語を混在させる以上、言語境界での型表現・エラー伝搬・ライフサイクル制御のズレが runtime 失敗の主要原因となるため、境界を厳密に設計することが必要である。

tier1 の 6 Pod のうち facade 3 Pod（Go）と自作 3 Pod（Rust）の間に実際に発生する呼び出しは、STATE → DECISION、STATE → PII、STATE → AUDIT、WORKFLOW → DECISION、WORKFLOW → AUDIT、SECRET → AUDIT の 6 主要パスである（AUDIT は Kafka 経由の非同期も併用、[../01_コンポーネント方式設計/05_モジュール依存関係.md](../01_コンポーネント方式設計/05_モジュール依存関係.md) 参照）。これらのパスで言語境界を跨ぐ全てのデータと呼び出しが、本ファイルの規約に従う。

## 設計 ID の採番

本ファイルで採番する設計 ID は `DS-SW-IIF-050` 〜 `DS-SW-IIF-069` の 20 件である。通番は [02_イベントスキーマ方式.md](02_イベントスキーマ方式.md) の DS-SW-IIF-049 から連続し、本ファイルで `DS-SW-IIF-*` カテゴリを完了する。

## 境界プロトコルの選定

### DS-SW-IIF-050 Protobuf gRPC のみ許可

Go ⇔ Rust の通信は Protobuf gRPC のみを許可する。以下は禁止: (a) CGo / FFI（Go から Rust を direct link で呼ぶ）、(b) 共有メモリ（Pod 間 / プロセス間）、(c) UNIX domain socket（Pod 内 multi-container でも禁止）、(d) HTTP/JSON、(e) 独自バイナリプロトコル。根拠は [../01_コンポーネント方式設計/05_モジュール依存関係.md](../01_コンポーネント方式設計/05_モジュール依存関係.md) DS-SW-COMP-100/101 と重複するが、言語境界特有の理由として (i) CGo は Go の goroutine スケジューラを阻害する、(ii) 共有メモリは Kubernetes Pod 跨ぎでは不可、(iii) UNIX socket は Pod 跨ぎで使えず Pod 設計を崩す、(iv) Protobuf は両言語で stable な SDK があり型整合を保証しやすい、という 4 点がある。

**確定フェーズ**: Phase 0。**対応要件**: ADR-TIER1-001、ADR-TIER1-002、NFR-E-NW-001〜004。

### DS-SW-IIF-051 gRPC-Web / gRPC-Gateway は未使用

tier1 内部通信で gRPC-Web（ブラウザ用）/ gRPC-Gateway（REST 変換）は使用しない。ブラウザ向けはそもそも tier1 内部通信の対象外で、REST 変換は tier2/3 への公開 API 側（[../02_外部インタフェース方式設計/](../02_外部インタフェース方式設計/)）で必要に応じて実装する。内部では HTTP/2 ネイティブの gRPC のみを使い、変換層を挟まないことでレイテンシと観測性を最適化する。

**確定フェーズ**: Phase 0。**対応要件**: NFR-B-PERF-\*、NFR-D-TRACE-\*。

## 型マッピング

### DS-SW-IIF-052 基本型の言語マッピング

Protobuf スカラ型は Go / Rust で以下に対応する。

| Proto 型 | Go 型 | Rust 型 |
|---|---|---|
| `int32` | `int32` | `i32` |
| `int64` | `int64` | `i64` |
| `uint32` | `uint32` | `u32` |
| `uint64` | `uint64` | `u64` |
| `bool` | `bool` | `bool` |
| `float` | `float32` | `f32` |
| `double` | `float64` | `f64` |
| `string` | `string` | `String` |
| `bytes` | `[]byte` | `Vec<u8>` |
| `google.protobuf.Timestamp` | `*timestamppb.Timestamp` | `prost_types::Timestamp` |
| `google.protobuf.Duration` | `*durationpb.Duration` | `prost_types::Duration` |

このマッピングは `protoc-gen-go` と `prost` の標準マッピングに一致する。例外として時刻型は wrapper 経由で言語標準型（Go `time.Time`、Rust `chrono::DateTime<Utc>`）に変換してアプリケーションコードで扱う。

**確定フェーズ**: Phase 1a。**対応要件**: ADR-TIER1-002。

### DS-SW-IIF-053 時刻型の言語間変換

`google.protobuf.Timestamp` を Go / Rust のネイティブ時刻型に変換する規約は以下。Go 側は `timestamppb.Timestamp.AsTime() time.Time` で UTC `time.Time` に変換。Rust 側は `chrono::DateTime<Utc>::from_timestamp(seconds, nanos)` で UTC に変換。両言語ともタイムゾーンは UTC 固定で、表示時のみアプリケーションが i18n 変換する（JST は `time.FixedZone("JST", 9*3600)` / `chrono_tz::Asia::Tokyo`）。ナノ秒精度まで保持するが、DB 永続化時は Phase 1b では マイクロ秒精度に切り詰める（PostgreSQL `timestamp(6)`）。

**確定フェーズ**: Phase 1a。**対応要件**: NFR-H-INT-\*、NFR-D-TRACE-\*。

### DS-SW-IIF-054 Option / nil の言語間変換

Go の `nil`（ポインタ / slice / map / interface）と Rust の `Option::None` は言語境界で以下のように対応する。Protobuf の optional フィールドは Go では `*T`（ポインタ）、Rust では `Option<T>` に生成される。Protobuf 3 の proto3 optional は両言語で同じ意味を持つ。`nil pointer` は gRPC wire 上は「フィールド省略」として送信され、受信側でも「フィールド省略」として認識される。空 string / 空 slice と未設定の区別が必要な場合は `google.protobuf.StringValue` / `google.protobuf.BytesValue` を使う。

**確定フェーズ**: Phase 1a。**対応要件**: ADR-TIER1-002、NFR-E-ERR-\*。

### DS-SW-IIF-055 列挙型の言語間対応

Protobuf `enum` は Go では `int32` alias 型、Rust では `#[repr(i32)] enum` に生成される。両言語で数値比較は同じ結果を返す。`*_UNSPECIFIED = 0` は明示的 default で、Go の `enum == 0` と Rust の `matches!(e, Enum::Unspecified)` は等価。新 enum 値の追加は後方互換だが、古い consumer は unknown enum 値を `UNSPECIFIED` として扱うため、producer 側で意味を上書きしないよう注意する。

**確定フェーズ**: Phase 1a。**対応要件**: ADR-TIER1-002。

### DS-SW-IIF-056 map / repeated の順序保証

Protobuf の `repeated` は Go の `[]T` / Rust の `Vec<T>` に対応し、順序を保証する。`map<K, V>` は Go の `map[K]V` / Rust の `HashMap<K, V>` に対応するが、map は wire 順序を保証しない（Protobuf 仕様上の制約）。ハッシュチェーン計算や署名検証など順序依存の処理で map を使う場合は、処理前に key ソートしてから `repeated Entry { key; value }` に変換する。この変換規約は `k1s0-common` の共通関数で提供する（[../01_コンポーネント方式設計/05_モジュール依存関係.md](../01_コンポーネント方式設計/05_モジュール依存関係.md) 参照）。

**確定フェーズ**: Phase 1a。**対応要件**: NFR-H-INT-001、ADR-TIER1-002。

## エラー統一

### DS-SW-IIF-057 エラーコードの Protobuf enum 統一

Go / Rust の両側で統一的にエラーコードを扱うため、`k1s0.internal.errors.v1.ErrorCode` enum を定義する。gRPC 標準の `google.rpc.Code` より粒度が細かい tier1 固有のエラー分類（例: `AUDIT_DUPLICATE_EVENT`、`DECISION_RULE_NOT_FOUND`、`PII_RULE_TIMEOUT`）を持つ。Go 側は `*status.Status.Details()` から、Rust 側は `tonic::Status::details()` から `ErrorCode` を抽出する。application 層ではこの enum で分岐処理する。

**確定フェーズ**: Phase 1a。**対応要件**: NFR-E-ERR-\*、DX-RB-\*。

### DS-SW-IIF-058 エラー変換の wrap 規約

Go 側は `fmt.Errorf("context: %w", err)` で wrap し、Rust 側は `thiserror` の `#[from]` または `anyhow::Context::with_context` で wrap する。gRPC レスポンスに変換する際は、Go は `status.Convert(err)` + custom error type の `GRPCStatus() *status.Status` で、Rust は `impl From<MyError> for tonic::Status` で実装する。wrap されたエラーの root cause は両言語で `errors.Unwrap` / `source()` で辿れる。

**確定フェーズ**: Phase 1a。**対応要件**: NFR-E-ERR-\*、DX-GP-\*。

### DS-SW-IIF-059 panic / unwrap の禁止範囲

Rust の `panic!()` / `unwrap()` / `expect()` は prod コードでは禁止する（test コードは除く）。代わりに `Result<T, E>` で明示的にエラー返却する。Go の同等は `panic()` の禁止で、ただし Interceptor の Recovery で補足して INTERNAL に変換する。言語境界をまたぐエラーは必ず `ErrorCode` に分類され、unknown 例外（INTERNAL）が外に出ないようにする。

**確定フェーズ**: Phase 1a。**対応要件**: NFR-E-ERR-\*、NFR-A-FT-001。

## OTel span 伝搬

### DS-SW-IIF-060 W3C Trace Context の propagation

OTel span は W3C Trace Context（`traceparent` / `tracestate` ヘッダ）で両言語間を伝搬する。Go 側は `otelgrpc.UnaryClientInterceptor` + `otel.GetTextMapPropagator().Inject`、Rust 側は `tonic` + `tracing-opentelemetry` + `opentelemetry::global::get_text_map_propagator().extract` で実装する。caller 側の現在 span が callee 側で parent span として認識され、Jaeger / Tempo 上で Go → Rust の trace が連結表示される。

**確定フェーズ**: Phase 1a。**対応要件**: NFR-D-TRACE-\*、FR-T1-TELEMETRY-\*。

### DS-SW-IIF-061 baggage の伝搬

OTel baggage（key-value の context 情報）は `baggage` ヘッダで伝搬する。tier1 では baggage に `tenant_id` / `user_id` / `session_id` を載せ、downstream の Rust Pod で log / metric の label に自動付与する（ただし label 爆発を避けるため high-cardinality の user_id は metric には付けず log のみ）。baggage のサイズ上限は 8 KB で、超過すると HTTP/2 header 上限に抵触するため注意する。

**確定フェーズ**: Phase 1b。**対応要件**: NFR-D-TRACE-\*、NFR-D-MON-\*。

### DS-SW-IIF-062 span の命名規約

Go / Rust 両言語で span 名を `<service>/<method>` 形式で統一する。例: `k1s0.internal.decision.v1.DecisionService/EvaluateDecision`。gRPC の Interceptor / middleware は標準でこの命名を生成するため、アプリ側で override しない。内部関数の span は `<crate>::<function>` 形式で、tracing library のデフォルト命名に従う（Go の `otel.Tracer` / Rust の `tracing::instrument`）。

**確定フェーズ**: Phase 1a。**対応要件**: NFR-D-TRACE-\*、DX-MET-\*。

## ライフサイクル制御

### DS-SW-IIF-063 Rust 側サーバの graceful shutdown

Rust 側 gRPC サーバは tonic の `serve_with_shutdown` を使い、SIGTERM / SIGINT を契機に以下の順で shutdown する。(1) 新規 connection の accept 停止、(2) inflight request の完了待機（最大 30 秒）、(3) backend 接続（PG / Kafka / Valkey）を順次 close、(4) tokio runtime の graceful terminate。30 秒超過時は残リクエストを中断して強制 shutdown する。Kubernetes の `terminationGracePeriodSeconds` は 35 秒で 5 秒バッファを確保する（AUDIT は StatefulSet のため 60 秒に延長、[../01_コンポーネント方式設計/03_自作Rust領域コンポーネント.md](../01_コンポーネント方式設計/03_自作Rust領域コンポーネント.md) DS-SW-COMP-079 参照）。

**確定フェーズ**: Phase 1b。**対応要件**: NFR-A-FT-001、NFR-A-REC-001。

### DS-SW-IIF-064 Go 側クライアントの再接続戦略

Go 側クライアントは Rust 側 Pod の ready / not-ready を意識せずに呼ぶ（Istio Ambient の waypoint が transparent に振り分けるため）。ただし Pod 死亡時の接続切断（HTTP/2 GOAWAY frame）は grpc-go が自動検出し、新しい Pod への再接続を 100ms 以内に完了する。再接続中のインフライトリクエストは `UNAVAILABLE` で失敗するため、caller 側でリトライ（指数バックオフ 3 回）を実装する（[01_内部gRPC契約方式.md](01_内部gRPC契約方式.md) DS-SW-IIF-028 参照）。

**確定フェーズ**: Phase 1b。**対応要件**: NFR-A-FT-001、NFR-A-CONT-\*。

### DS-SW-IIF-065 Rust 側の startup probe

Rust Pod は `/readyz` endpoint で startup 完了を返す。tonic server が listen 開始 + backend 接続（該当する場合）+ cache warm-up（Decision のルール prefetch、[../01_コンポーネント方式設計/03_自作Rust領域コンポーネント.md](../01_コンポーネント方式設計/03_自作Rust領域コンポーネント.md) DS-SW-COMP-065 参照）が完了したとき 200 を返す。Kubernetes の startupProbe は initialDelaySeconds 5s / periodSeconds 2s / failureThreshold 30（最大 60 秒起動を許容）で設定する。起動に失敗し続ける Pod は再作成される。

**確定フェーズ**: Phase 1a。**対応要件**: NFR-A-FT-001、NFR-A-REC-001。

### DS-SW-IIF-066 Go 側 daprd との連携ライフサイクル

Go Pod は daprd サイドカーと起動順序を調整する。Pod 内で daprd が先に起動し、アプリコンテナが daprd の `/v1.0/healthz/outbound` を polling して ready 確認後に tonic server を起動する。これは Dapr 公式パターンで、`dapr.io/sidecar-listen-port` と `dapr.io/app-port` の annotation で制御する。daprd 死亡時はアプリコンテナも `liveness failed` で再起動する（Pod 単位で同期再起動）。

**確定フェーズ**: Phase 1a。**対応要件**: NFR-A-FT-001、ADR-TIER1-001。

## Istio Ambient との連携

### DS-SW-IIF-067 mTLS の Istio 側任せ

Go ⇔ Rust 間の gRPC 通信は Istio Ambient の ztunnel が mTLS を透過的に付与する。アプリ側（Go / Rust）は TLS 設定を持たず plain HTTP/2 で listen / dial する。mTLS 証明書のローテーション（SPIFFE 30 日）は Istio が自動実施する。Dapr の内蔵 mTLS 機能は有効化しない（Istio と二重 mTLS になるため）。

**確定フェーズ**: Phase 1b。**対応要件**: NFR-E-AC-001〜005、NFR-E-ENC-001、ADR-TIER1-001。**参照**: 構想設計 ADR-0001（Istio Ambient Mesh）。

### DS-SW-IIF-068 L7 authz は waypoint に委譲

gRPC メソッドレベルの認可（例: "STATE から DECISION の EvaluateDecision のみ許可"）は Istio Ambient の waypoint で `AuthorizationPolicy` によって実施する。アプリ側（Go / Rust）は waypoint を通過してきたリクエストの caller を信頼し、JWT 検証（Policy Enforcer）のみを行う。waypoint の policy は tier1 namespace で ConfigMap として GitOps 管理する（[../01_コンポーネント方式設計/02_Daprファサード層コンポーネント.md](../01_コンポーネント方式設計/02_Daprファサード層コンポーネント.md) 参照）。

**確定フェーズ**: Phase 1c。**対応要件**: NFR-E-AC-001〜005。

### DS-SW-IIF-069 Circuit Breaker と Retry の Istio 側任せ

gRPC レベルの Circuit Breaker / Retry は Istio Ambient の waypoint で `DestinationRule` + `VirtualService` によって実施する。アプリ側の grpc-go / tonic にも retry 機能はあるが、Istio と二重になると過剰 retry（実質 9 回）でバックエンドを圧迫するため、アプリ側 retry は無効化する。Istio policy は「連続 10 エラーで 30 秒 open、半開で 1 req 成功で close、最大 3 retry」を既定とする（詳細は [../../40_制御方式設計/](../../40_制御方式設計/) で規定）。

**確定フェーズ**: Phase 1c。**対応要件**: NFR-A-CONT-003、NFR-A-FT-001、ADR-0001。

## 章末サマリ

### 設計 ID 一覧

| 設計 ID | 内容 | 確定フェーズ |
|---|---|---|
| DS-SW-IIF-050 | Protobuf gRPC のみ許可（FFI/共有メモリ禁止） | Phase 0 |
| DS-SW-IIF-051 | gRPC-Web / gRPC-Gateway は未使用 | Phase 0 |
| DS-SW-IIF-052 | 基本型の言語マッピング | Phase 1a |
| DS-SW-IIF-053 | 時刻型の言語間変換 | Phase 1a |
| DS-SW-IIF-054 | Option / nil の言語間変換 | Phase 1a |
| DS-SW-IIF-055 | 列挙型の言語間対応 | Phase 1a |
| DS-SW-IIF-056 | map / repeated の順序保証 | Phase 1a |
| DS-SW-IIF-057 | エラーコードの Protobuf enum 統一 | Phase 1a |
| DS-SW-IIF-058 | エラー変換の wrap 規約 | Phase 1a |
| DS-SW-IIF-059 | panic / unwrap の禁止範囲 | Phase 1a |
| DS-SW-IIF-060 | W3C Trace Context propagation | Phase 1a |
| DS-SW-IIF-061 | baggage の伝搬 | Phase 1b |
| DS-SW-IIF-062 | span の命名規約 | Phase 1a |
| DS-SW-IIF-063 | Rust 側サーバ graceful shutdown | Phase 1b |
| DS-SW-IIF-064 | Go 側クライアント再接続戦略 | Phase 1b |
| DS-SW-IIF-065 | Rust 側 startup probe | Phase 1a |
| DS-SW-IIF-066 | Go 側 daprd 連携ライフサイクル | Phase 1a |
| DS-SW-IIF-067 | mTLS は Istio 側任せ | Phase 1b |
| DS-SW-IIF-068 | L7 authz は waypoint 委譲 | Phase 1c |
| DS-SW-IIF-069 | Circuit Breaker / Retry は Istio 側 | Phase 1c |

## 対応要件一覧

- NFR-A-CONT-003（障害影響限定）、NFR-A-FT-001（Pod 復旧）、NFR-A-REC-001（再開）
- NFR-B-PERF-001 / NFR-B-PERF-004 / NFR-B-PERF-006（性能）
- NFR-D-TRACE-\* / NFR-D-MON-\*（観測）
- NFR-E-AC-001〜005 / NFR-E-ENC-001 / NFR-E-NW-001〜004（セキュリティ・通信）
- NFR-H-INT-001（完整性）
- FR-T1-TELEMETRY-\*（OTel 伝搬）
- DX-GP-\* / DX-RB-\* / DX-MET-\*（開発者体験）

構想設計 ADR-TIER1-001（Go+Rust ハイブリッド）、ADR-TIER1-002（Protobuf gRPC）、ADR-0001（Istio Ambient Mesh）と双方向トレースする。

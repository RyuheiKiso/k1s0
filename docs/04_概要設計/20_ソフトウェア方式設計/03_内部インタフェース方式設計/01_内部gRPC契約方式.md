# 01. 内部 gRPC 契約方式

本ファイルは IPA 共通フレーム 2013 の **ソフトウェア方式設計プロセス 7.1.2.2（外部及びコンポーネント間のインタフェース方式設計）** のうち、tier1 内部コンポーネント間の gRPC 契約を方式として固定化する。Protobuf 必須（HTTP/JSON 禁止）、`.proto` 一元管理、buf CLI による breaking change 検出、サービス定義命名規約、メソッド命名、エラー返却、deadline 伝搬、メタデータ伝搬、バージョニングを対象とする。

## 本ファイルの位置付け

構想設計 ADR-TIER1-002（Protobuf gRPC 必須）で確定した方針を、概要設計の方式設計書本体として具体化する。tier1 内部通信が Protobuf gRPC のみであることは設計原則 4（[../../00_設計方針/02_設計原則と制約.md](../../00_設計方針/02_設計原則と制約.md) 原則 4）でも強調されており、本ファイルはこの前提の下で「どう書くか」を確定する。

内部 gRPC 契約の設計が緩いと、(a) Go ⇔ Rust の型不整合で runtime 失敗、(b) 破壊的変更が検知されずに deployment が壊れる、(c) deadline / trace が言語境界で欠落して SLO 違反の原因追跡が破綻、といった事象が起きる。これらは Phase 1b 以降の運用を崩壊させるため、Phase 0 段階で契約方式を厳密に固定化することが必要である。

## 設計 ID の採番

本ファイルで採番する設計 ID は `DS-SW-IIF-001` 〜 `DS-SW-IIF-029` の 29 件である。通番は本カテゴリ（SW-IIF）の先頭から開始し、[02_イベントスキーマ方式.md](02_イベントスキーマ方式.md) で DS-SW-IIF-030、[03_Go_Rust間言語境界方式.md](03_Go_Rust間言語境界方式.md) で DS-SW-IIF-050 に継続する。

## Protobuf 必須とその根拠

### DS-SW-IIF-001 HTTP/JSON 禁止・Protobuf gRPC 必須

tier1 内部のコンポーネント間通信は Protobuf gRPC（HTTP/2）のみを許可する。HTTP/JSON、MessagePack、独自バイナリプロトコルは禁止する。この制約は構想設計 ADR-TIER1-002 で確定しており、概要設計で緩めることはできない。根拠は以下 4 点である。第一に、`.proto` を単一の真実として Go / Rust の両言語コードを自動生成することで型不整合が構造的に発生しない。第二に、gRPC の HTTP/2 multiplexing と keep-alive でレイテンシが HTTP/1.1 + JSON より数倍高速。第三に、`buf breaking` で破壊的変更を CI 検出できる。第四に、deadline / metadata 伝搬が仕様で規定されているため言語横断で一貫する。

**確定フェーズ**: Phase 0。**対応要件**: ADR-TIER1-002、NFR-B-PERF-004、NFR-B-PERF-006。

### DS-SW-IIF-002 .proto ファイルの一元管理

`.proto` ファイルは `src/tier1/contracts/v1/` に一元配置し、Go / Rust コード生成の source of truth とする（[../01_コンポーネント方式設計/06_パッケージ構成_Rust_Go.md](../01_コンポーネント方式設計/06_パッケージ構成_Rust_Go.md) DS-SW-COMP-121 参照）。tier1 内部 gRPC は tier2 / tier3 への公開 API（`src/tier1/contracts/public/v1/`）と別空間に分離し、内部契約の変更が外部契約に伝播しない構造を維持する。内部契約の `.proto` は PR レビューで tier1 アーキテクトの承認必須とし、外部契約より厳密に管理する（内部の型は公開 API 経由で外部に露出し得るため）。

**確定フェーズ**: Phase 1a。**対応要件**: ADR-TIER1-002、NFR-C-NOP-002。

### DS-SW-IIF-003 buf CLI による破壊的変更検出

`buf breaking` を CI で必須実行し、既存フィールド削除・フィールド番号変更・enum 値削除・required → optional 変更などの破壊的変更を自動検出する。検出時は CI が fail し、PR マージ不可となる。`buf.yaml` では `breaking.use: [FILE]`（FILE レベル）で設定し、WIRE_JSON レベルでは追加検出（例: JSON tag 不整合）は Phase 2 で拡張する。`buf format` も CI で必須化し、フォーマット統一を強制する。

**確定フェーズ**: Phase 1a。**対応要件**: ADR-TIER1-002、DX-CICD-\*。

## サービス・メソッド命名規約

### DS-SW-IIF-004 サービス定義命名規約

gRPC サービス定義は `k1s0.internal.<comp>.v<version>.<ServiceName>` の形式で統一する。例として COMP-T1-AUDIT の監査イベント受付サービスは `k1s0.internal.audit.v1.AuditService`、COMP-T1-DECISION の評価サービスは `k1s0.internal.decision.v1.DecisionService` となる。`internal` 名前空間は公開 API の `k1s0.v1.*` と明確に分離することで、誤って内部サービスを tier2 が import することを防ぐ。`<comp>` は 6 コンポーネントの short id（`state` / `secret` / `workflow` / `audit` / `decision` / `pii`）を使い、Pod 固有の識別子に揃える。

**確定フェーズ**: Phase 1a。**対応要件**: ADR-TIER1-002、NFR-C-NOP-002。

### DS-SW-IIF-005 メソッド命名規約

gRPC メソッドは動詞形で命名する。CRUD 系は `Get*` / `List*` / `Create*` / `Update*` / `Delete*`、業務動作は動詞で始める（`Evaluate*` / `Mask*` / `Verify*` / `Issue*` / `Renew*`）。メソッド名に `Internal` のような冗長プレフィクスは付けない（service 名が既に `internal` を含むため）。Request / Response メッセージ名は `<Method>Request` / `<Method>Response` で統一する（例: `EvaluateDecisionRequest` / `EvaluateDecisionResponse`）。

**確定フェーズ**: Phase 1a。**対応要件**: ADR-TIER1-002、DX-GP-\*。

### DS-SW-IIF-006 Unary / Streaming の使い分け

gRPC の 4 通信モードは以下の基準で使い分ける。Unary（単一 request / 単一 response）は 95% 以上の標準ケースで使用。Server Streaming（単一 request / 複数 response）は AUDIT の `VerifyChain`（長期間の検証 progress 返却）のように、結果が段階的に返る場合に使う。Client Streaming（複数 request / 単一 response）は AUDIT の監査イベント batch 受付のような高スループット書込で Phase 2 以降に検討。Bidirectional（複数 request / 複数 response）は現在のユースケースでは使わず、将来的な Workflow のイベントプッシュで検討する。

**確定フェーズ**: Phase 1a（Unary）、Phase 1c（Server Streaming）、Phase 2（他）。**対応要件**: ADR-TIER1-002、NFR-B-PERF-\*。

## Protobuf 型規約

### DS-SW-IIF-007 基本型の利用規約

Protobuf のスカラ型は以下で使用する。`int64` / `int32` は整数、`string` は UTF-8 文字列、`bytes` は任意バイト列、`bool` は真偽値、`double` は倍精度浮動小数点（金額など精度重視用途では `string` + 文字列表現を推奨）。`google.protobuf.Timestamp` は時刻、`google.protobuf.Duration` は期間、`google.protobuf.Struct` は任意 JSON 互換構造（ZEN 評価入出力など）、`google.protobuf.Empty` は空 request / response に使う。`fixed32` / `sfixed32` は使用しない（WIRE 効率が low cardinality では悪い）。

**確定フェーズ**: Phase 1a。**対応要件**: ADR-TIER1-002。

### DS-SW-IIF-008 フィールド番号の予約

フィールド番号 1〜15 は 1 byte で encode されるため、高頻度フィールドに優先割当する（例: `tenant_id` / `request_id` / `trace_id` は必ず 1-3 番に割当）。16 以降は補助フィールドに使う。削除済みフィールド番号は `reserved` で永久保留する（例: `reserved 5;`）。フィールド番号の重複は buf lint で検出されるが、レビューでも確認する。proto 2 の required フィールドは proto 3 で使えないため、必須制約は application-level で担保する。

**確定フェーズ**: Phase 1a。**対応要件**: ADR-TIER1-002、NFR-B-PERF-\*。

### DS-SW-IIF-009 Enum 命名規約

Enum 名は UpperCamelCase、値は UPPER_SNAKE_CASE で enum 名を prefix に含める（例: `enum Severity { SEVERITY_UNSPECIFIED = 0; SEVERITY_INFO = 1; SEVERITY_WARN = 2; ... }`）。`*_UNSPECIFIED = 0` を必ず先頭に置き、明示的な default として扱う。enum 値の削除は breaking change なので、削除時は `reserved` で番号を永久保留する。

**確定フェーズ**: Phase 1a。**対応要件**: ADR-TIER1-002。

### DS-SW-IIF-010 message のネスト深さ制限

message のネスト深さは 3 階層以内を推奨し、4 階層以上は型名を切り出して lib 側で定義する（深いネストは生成コードの可読性を下げ、buf lint で警告される）。Repeated フィールドは複数形で命名（`audit_events`、`tenants`）、map は `map<string, bytes> attributes` の形式で使用する（key は string / int 型のみ許容）。

**確定フェーズ**: Phase 1a。**対応要件**: ADR-TIER1-002、DX-GP-\*。

## エラー返却規約

### DS-SW-IIF-011 google.rpc.Status によるエラー返却

エラーは gRPC 標準の `status.Status`（内部的に `google.rpc.Status`）で返却する。`code` は `google.rpc.Code` enum（`OK` / `CANCELLED` / `INVALID_ARGUMENT` / `DEADLINE_EXCEEDED` / `NOT_FOUND` / `FAILED_PRECONDITION` / `ABORTED` / `UNAVAILABLE` / `INTERNAL` / `UNAUTHENTICATED` / `PERMISSION_DENIED` / `RESOURCE_EXHAUSTED` など）を使う。HTTP status への写像は gRPC-HTTP gateway が自動実施（詳細は [../02_外部インタフェース方式設計/](../02_外部インタフェース方式設計/) 参照）。

**確定フェーズ**: Phase 1a。**対応要件**: NFR-E-ERR-\*、NFR-D-TRACE-\*。

### DS-SW-IIF-012 エラー詳細の構造化

`status.Status.details` フィールドには `k1s0.internal.errors.v1.ErrorDetail` 型を含める。`ErrorDetail` は `error_code` (string、ドキュメント参照用、例: `T1.AUDIT.DUP_EVENT`)、`trace_id` (string、分散トレース ID)、`retryable` (bool、再試行可能か)、`retry_after_seconds` (int32、待機秒)、`docs_url` (string、エラー詳細ドキュメント URL)、`context` (map<string, string>、追加情報) を持つ。error_code は 4 階層 `T1.<comp>.<category>` 形式で命名し、tier1 全体で uniqueness を維持する。

**確定フェーズ**: Phase 1a。**対応要件**: NFR-E-ERR-\*、DX-RB-\*。

### DS-SW-IIF-013 エラーコードタクソノミ

tier1 内部エラーは以下 6 カテゴリに分類する。`VALIDATION`（入力不正）、`AUTH`（認証・認可失敗）、`BACKEND`（バックエンド OSS 障害）、`TIMEOUT`（deadline 超過）、`CONFLICT`（etag 衝突・リース衝突）、`INTERNAL`（予期しないエラー）。各カテゴリは gRPC code への写像が決まっており、例えば VALIDATION → `INVALID_ARGUMENT`、AUTH → `UNAUTHENTICATED` / `PERMISSION_DENIED`、BACKEND → `UNAVAILABLE`、TIMEOUT → `DEADLINE_EXCEEDED`。詳細は [../../30_共通機能方式設計/](../../30_共通機能方式設計/) のエラーハンドリング方式で規定する。

**確定フェーズ**: Phase 1a。**対応要件**: NFR-E-ERR-\*、DX-RB-\*。

## Deadline 伝搬

### DS-SW-IIF-014 gRPC deadline の必須設定

全内部 gRPC 呼び出しは必ず deadline を設定する。deadline なしの呼び出しは CI で lint 検出する（Go の `linters-settings.ctxcheck`、Rust の自作 clippy lint）。deadline は caller 側で `context.WithDeadline` / `tonic::Request::set_timeout` で設定し、値は呼び出し元のタイムアウト階層（[../01_コンポーネント方式設計/02_Daprファサード層コンポーネント.md](../01_コンポーネント方式設計/02_Daprファサード層コンポーネント.md) DS-SW-COMP-048 参照）から導出する。

**確定フェーズ**: Phase 1a。**対応要件**: NFR-B-PERF-\*、NFR-A-CONT-\*。

### DS-SW-IIF-015 deadline の伝搬方式

gRPC は `grpc-timeout` メタデータヘッダで deadline を自動伝搬する。caller が 200ms の deadline を設定すると、HTTP/2 ヘッダに `grpc-timeout: 200m` が乗り、callee 側の gRPC library が自動的に context / tonic::Request に deadline を設定する。multi-hop 呼び出し（facade → rust → 別 rust）では、各 hop で残時間が伝搬され、深い階層でも適切に short-circuit する。caller 側で deadline が既に expire していた場合、callee は `DEADLINE_EXCEEDED` を即時返却する。

**確定フェーズ**: Phase 1a。**対応要件**: NFR-B-PERF-001、NFR-A-CONT-\*。

### DS-SW-IIF-016 deadline 残時間のバッファ戦略

callee が downstream を呼ぶ際は、残時間から安全バッファ（30ms）を差し引いた deadline で downstream を呼ぶ。これにより downstream が timeout する前に callee 側でエラー処理を行う時間を確保する。バッファは通信遅延・レスポンス生成・エラーラッピングに使われる。バッファ値は SLO（p99 500ms）とネットワーク特性（Istio Ambient の遅延 5-10ms）から 30ms を既定とする。

**確定フェーズ**: Phase 1b。**対応要件**: NFR-B-PERF-001、NFR-A-CONT-\*。

## メタデータ伝搬

### DS-SW-IIF-017 必須メタデータ項目

gRPC メタデータ（HTTP/2 header）で以下を必須伝搬する。`traceparent`（W3C Trace Context、OTel 標準）、`tracestate`（W3C Trace Context、vendor specific）、`k1s0-tenant-id`（テナント識別子）、`k1s0-request-id`（リクエスト追跡 ID、UUID v4）、`k1s0-user-id`（ユーザー ID、tenant 内で unique）、`k1s0-session-id`（セッション ID、optional）、`k1s0-feature-flags`（feature flag の evaluation hint、optional）。metadata 名は `k1s0-` prefix で衝突を避ける。

**確定フェーズ**: Phase 1a。**対応要件**: NFR-D-TRACE-\*、NFR-E-AC-001〜005、FR-T1-LOG-\*。

### DS-SW-IIF-018 メタデータの言語境界伝搬

Go 側の metadata は `google.golang.org/grpc/metadata.NewOutgoingContext(ctx, md)` で設定し、Rust 側は `tonic::Request::metadata_mut()` で読み書きする。両言語で metadata key は lower-case 正規化され、値は UTF-8 bytes として伝搬される。bynary metadata（`*-bin` suffix）は使用しない（可読性とデバッグ性のため）。metadata のサイズ上限は 8 KiB（HTTP/2 HPACK テーブル考慮）で、これを超える情報は message body に入れる。

**確定フェーズ**: Phase 1a。**対応要件**: NFR-D-TRACE-\*、NFR-B-PERF-\*。

### DS-SW-IIF-019 metadata 未設定時の対応

必須 metadata が未設定の場合、callee は `INVALID_ARGUMENT` で reject する。例外として `k1s0-session-id` と `k1s0-feature-flags` は optional なので未設定でも受け付ける。必須 metadata が偽造されている疑い（例: tenant_id が JWT の tenant と不一致）は `UNAUTHENTICATED` / `PERMISSION_DENIED` で reject する。この検査は Policy Enforcer の第一層で実施する（[../01_コンポーネント方式設計/02_Daprファサード層コンポーネント.md](../01_コンポーネント方式設計/02_Daprファサード層コンポーネント.md) DS-SW-COMP-036 参照）。

**確定フェーズ**: Phase 1a。**対応要件**: NFR-E-AC-001〜005。

## バージョニング

### DS-SW-IIF-020 サービスバージョニング戦略

内部 gRPC サービスは `v1` / `v2` のメジャーバージョンを package 名に含めて並行公開する。破壊的変更が必要な場合、新 package（`k1s0.internal.<comp>.v2.<ServiceName>`）を追加して旧版と並行 serving し、6 か月の移行期間を経て旧版を削除する。移行期間中は caller 側を Canary で新版に切替、問題なければ全面移行する。削除時は ADR 起票で全利用者への通知と移行完了を確認する。

**確定フェーズ**: Phase 1a（ルール）、Phase 2 以降（v2 発生時）。**対応要件**: ADR-TIER1-002、NFR-C-NOP-002。

### DS-SW-IIF-021 後方互換性の維持基準

マイナーバージョンアップ（v1 内の追加）では後方互換性を厳守する。以下は後方互換 OK: 新フィールド追加（新番号）、新 enum 値追加、新メソッド追加、新サービス追加。以下は後方互換 NG: フィールド削除、フィールド型変更、フィールド番号変更、enum 値削除、メソッド削除、サービス削除、required ↔ optional 変更。`buf breaking` で自動検出される範囲は OK / NG を機械判定する。

**確定フェーズ**: Phase 1a。**対応要件**: ADR-TIER1-002、DX-CICD-\*。

### DS-SW-IIF-022 非推奨化の手順

サービス・メソッド・フィールドの非推奨化は以下 4 段階で実施する。(1) `deprecated = true` オプション付与 + ドキュメント更新、(2) caller 側に Prometheus counter `k1s0_deprecated_grpc_call_total{service, method, version}` で使用状況を追跡、(3) 並行期間 6 か月（または 2 メジャーバージョン）で全 caller の移行を確認、(4) 削除時は buf breaking で明示検出して CI fail を経由し ADR 承認で release。

**確定フェーズ**: Phase 1c。**対応要件**: NFR-C-NOP-002、DX-CICD-\*。

## 内部 gRPC サーバ実装規約

### DS-SW-IIF-023 Interceptor の共通化

gRPC サーバの Interceptor は全 Pod で以下 5 段を必ず登録する。(1) Recovery（panic → gRPC INTERNAL 変換）、(2) Tracing（OTel span 生成と metadata propagation）、(3) Metrics（RED メトリック記録）、(4) Auth（JWT 検証と tenant 境界確認）、(5) Logging（request/response の構造化ログ）。登録順序は Recovery が最外で、Logging が最内とする。Go は `grpc-go` の `UnaryServerInterceptor` / `StreamServerInterceptor`、Rust は `tonic::service::interceptor_fn` を使う。

**確定フェーズ**: Phase 1a。**対応要件**: NFR-D-MON-\*、NFR-D-TRACE-\*、NFR-E-AC-001〜005。

### DS-SW-IIF-024 Keepalive 設定

gRPC サーバは以下 keepalive を設定する。`time` 30s（ping 送信間隔）、`timeout` 10s（ping 応答待機）、`min_time` 10s（client 側の最小 ping 間隔許容）、`permit_without_stream` true（stream なしでも ping 許容）。これにより Istio Ambient の透過プロキシや NAT で接続が切断される前に検知できる。client 側も同じ値を設定する。

**確定フェーズ**: Phase 1a。**対応要件**: NFR-A-FT-001、NFR-E-NW-001〜004。

### DS-SW-IIF-025 Message size 制限

gRPC message の上限は request 4 MiB、response 4 MiB とする。これを超える場合は Server Streaming で分割送信する。上限値は `grpc.MaxRecvMsgSize` / `grpc.MaxSendMsgSize` で設定する。大量データ（例: AUDIT の月次エクスポート）は内部 gRPC で扱わず、MinIO への直接書出で対処する。

**確定フェーズ**: Phase 1a。**対応要件**: NFR-B-CAP-\*、NFR-B-PERF-\*。

### DS-SW-IIF-026 Concurrent stream 制限

gRPC サーバは HTTP/2 の concurrent stream 上限を 100 に設定する。これを超えるリクエストはクライアント側で queue される。Istio Ambient の waypoint では `max_concurrent_streams=500` を設定し、Pod 側（100）のほうが厳しく制限して Pod 保護を優先する。上限到達は Prometheus metric `k1s0_grpc_concurrent_streams` で監視し、HPA スケール trigger に使える（Phase 2 以降）。

**確定フェーズ**: Phase 1b。**対応要件**: NFR-B-WL-\*、NFR-B-CAP-\*。

## 内部 gRPC クライアント実装規約

### DS-SW-IIF-027 Connection Pool とロードバランシング

クライアントは pod 単位で 1 ChannelPool を持ち、Pool 内に複数 connection を張る。Go は `grpc-go` の `grpc.WithDefaultServiceConfig`（`loadBalancingPolicy: round_robin`）、Rust は `tonic` + `tower::balance::p2c::Balance`（Power of Two Choices）を使う。Istio Ambient の waypoint が実際のロードバランシングを担うため、client 側は同じ waypoint 宛に接続を集約する（DNS resolver: `<service>.<ns>.svc.cluster.local` を waypoint が transparent hijack）。

**確定フェーズ**: Phase 1b。**対応要件**: NFR-A-CONT-003、NFR-B-PERF-\*。

### DS-SW-IIF-028 リトライ戦略

gRPC レスポンスが `UNAVAILABLE` / `DEADLINE_EXCEEDED` / `ABORTED` の場合、クライアントは最大 3 回まで指数バックオフ（100ms / 200ms / 400ms）でリトライする。`INVALID_ARGUMENT` / `FAILED_PRECONDITION` / `PERMISSION_DENIED` はリトライしない。リトライは `retryable` フラグを response の `ErrorDetail` で確認して決定する。リトライ設定は Go では `grpc.ServiceConfig` で JSON 定義、Rust では `tonic::service::interceptor` で実装する。詳細は [../../40_制御方式設計/](../../40_制御方式設計/) の リトライ方式で規定する。

**確定フェーズ**: Phase 1b。**対応要件**: NFR-A-FT-001、NFR-A-CONT-\*。

### DS-SW-IIF-029 Circuit Breaker との連携

連続エラー（5 秒間に 10 エラー以上）を検出したら Circuit Breaker が open となり、以降 30 秒間はリクエストを即時 fail させる（fast fail）。half-open 状態で 1 リクエストを試し、成功すれば close に戻る。Circuit Breaker は Istio Ambient の waypoint で実装するため、アプリ側の実装は不要。ただしアプリ側のエラー計数メトリクス（`k1s0_grpc_client_errors_total`）を Istio メトリクスと揃えて監視する。詳細は [../../40_制御方式設計/](../../40_制御方式設計/) で規定する。

**確定フェーズ**: Phase 1b/1c。**対応要件**: NFR-A-CONT-003、NFR-A-FT-001。

## 章末サマリ

### 設計 ID 一覧

| 設計 ID | 内容 | 確定フェーズ |
|---|---|---|
| DS-SW-IIF-001 | HTTP/JSON 禁止・Protobuf gRPC 必須 | Phase 0 |
| DS-SW-IIF-002 | .proto ファイル一元管理 | Phase 1a |
| DS-SW-IIF-003 | buf CLI 破壊的変更検出 | Phase 1a |
| DS-SW-IIF-004 | サービス定義命名規約 | Phase 1a |
| DS-SW-IIF-005 | メソッド命名規約 | Phase 1a |
| DS-SW-IIF-006 | Unary / Streaming の使い分け | Phase 1a/1c/2 |
| DS-SW-IIF-007 | 基本型の利用規約 | Phase 1a |
| DS-SW-IIF-008 | フィールド番号の予約 | Phase 1a |
| DS-SW-IIF-009 | Enum 命名規約 | Phase 1a |
| DS-SW-IIF-010 | message のネスト深さ制限 | Phase 1a |
| DS-SW-IIF-011 | google.rpc.Status によるエラー返却 | Phase 1a |
| DS-SW-IIF-012 | エラー詳細の構造化（ErrorDetail） | Phase 1a |
| DS-SW-IIF-013 | エラーコードタクソノミ | Phase 1a |
| DS-SW-IIF-014 | gRPC deadline 必須設定 | Phase 1a |
| DS-SW-IIF-015 | deadline 伝搬方式 | Phase 1a |
| DS-SW-IIF-016 | deadline 残時間バッファ戦略 | Phase 1b |
| DS-SW-IIF-017 | 必須メタデータ項目 | Phase 1a |
| DS-SW-IIF-018 | metadata 言語境界伝搬 | Phase 1a |
| DS-SW-IIF-019 | metadata 未設定時の対応 | Phase 1a |
| DS-SW-IIF-020 | サービスバージョニング戦略 | Phase 1a/2 |
| DS-SW-IIF-021 | 後方互換性維持基準 | Phase 1a |
| DS-SW-IIF-022 | 非推奨化の手順 | Phase 1c |
| DS-SW-IIF-023 | Interceptor の共通化 | Phase 1a |
| DS-SW-IIF-024 | Keepalive 設定 | Phase 1a |
| DS-SW-IIF-025 | Message size 制限 | Phase 1a |
| DS-SW-IIF-026 | Concurrent stream 制限 | Phase 1b |
| DS-SW-IIF-027 | Connection Pool とロードバランシング | Phase 1b |
| DS-SW-IIF-028 | リトライ戦略 | Phase 1b |
| DS-SW-IIF-029 | Circuit Breaker との連携 | Phase 1b/1c |

## 対応要件一覧

- FR-T1-LOG-\*（trace_id / tenant_id 伝搬）、FR-T1-TELEMETRY-\*（span 伝搬）
- NFR-A-CONT-001 / NFR-A-CONT-003 / NFR-A-FT-001（耐障害性）
- NFR-B-PERF-001 / NFR-B-PERF-004 / NFR-B-PERF-006 / NFR-B-WL-\* / NFR-B-CAP-\*（性能）
- NFR-C-NOP-002（可視性）、NFR-D-MON-\* / NFR-D-TRACE-\*（観測）
- NFR-E-AC-001〜005 / NFR-E-NW-001〜004（セキュリティ・通信）
- DX-CICD-\* / DX-GP-\* / DX-RB-\*（開発者体験）

構想設計 ADR-TIER1-002（Protobuf gRPC 必須）と双方向トレースする。

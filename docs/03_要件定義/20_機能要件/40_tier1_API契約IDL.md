# 40. tier1 API 契約 IDL

本書は tier1 が公開する 11 API のインタフェース契約（Protobuf IDL）をスケルトン形式で定義する。各 API の要件詳細は [10_tier1_API要件/](10_tier1_API要件/) に記述されており、本書はそれに対応する機械可読な契約骨格を提供する。tier2/tier3 はこの IDL から生成されるクライアントライブラリ経由でのみ tier1 を利用し、内部実装言語（Go/Rust）には依存しない。

## 本書の位置付け

要件記述だけではインタフェース契約は一意に定まらない。例えば「State API は取得・更新・削除を提供する」と書かれていても、エラーコードの体系、冪等性キー、ETag、トランザクション境界、バルク操作可否といった具体は IDL を見ないと確定しない。tier2/tier3 開発者は IDL からクライアントを生成してから開発に入るため、IDL が無いと並行開発が始められない。

本書の IDL は「要件定義段階で確定すべき契約の最小限」であり、詳細設計で message フィールドの追加・RPC の分割統合が行われる。ただし以下は要件定義の合意事項として本書で固定する。

- 全 RPC は gRPC over HTTP/2、mTLS 必須、ヘッダで `x-tenant-id` `x-correlation-id` を伝搬
- 全エラーは `google.rpc.Status` を使い、`details` に `ErrorDetail`（共通エラー体系）を載せる
- メッセージのフィールドタグは 1〜15 の空きを予約領域として将来拡張用に空ける

## 共通型定義

全 API で参照する共通メッセージを先に定義する。

```protobuf
// 共通型定義: 全 tier1 API が import して利用する基盤型
syntax = "proto3";
// パッケージ命名規則: k1s0.tier1.<api>.<version>
package k1s0.tier1.common.v1;

// 呼出元テナントを特定する識別子 (ヘッダとの整合性は interceptor で検証)
message TenantContext {
  // テナント ID (UUID v4 文字列)
  string tenant_id = 1;
  // 呼出元の主体 (workload_id / user_id のいずれか)
  string subject = 2;
  // 相関 ID (トレース連携のため OTel traceparent と紐付ける)
  string correlation_id = 3;
}

// エラー詳細: google.rpc.Status.details に埋め込む
message ErrorDetail {
  // エラーコード (E-<CATEGORY>-<MODULE>-<NUMBER> 形式)
  string code = 1;
  // 人間可読なメッセージ (テナント表示可)
  string message = 2;
  // 再試行可否 (true の場合クライアントは指数バックオフで再試行)
  bool retryable = 3;
  // 再試行までの推奨待機時間 (ミリ秒)
  int32 retry_after_ms = 4;
}
```

## 01. Service Invoke API

サービス間の RPC を tier1 ファサード経由で仲介する。サービス名解決、ロードバランシング、mTLS、リトライ、ヘッダ伝搬を隠蔽する。

```protobuf
// サービス間呼出を仲介する API (FR-T1-INVOKE-001〜005)
syntax = "proto3";
package k1s0.tier1.invoke.v1;
import "k1s0/tier1/common/v1/common.proto";

service InvokeService {
  // 任意サービスの任意メソッドを呼び出す (app_id は Dapr の app_id 概念と互換)
  rpc Invoke(InvokeRequest) returns (InvokeResponse);
  // ストリーミング呼出 (大容量応答や段階出力)
  rpc InvokeStream(InvokeRequest) returns (stream InvokeChunk);
}

// Invoke リクエスト
message InvokeRequest {
  // 呼出先のアプリ識別子
  string app_id = 1;
  // 呼出先のメソッド名 (HTTP の場合 path に相当)
  string method = 2;
  // 呼出データ (bytes で透過伝搬、encoding は content_type で示す)
  bytes data = 3;
  // Content-Type (application/json, application/grpc, application/protobuf 等)
  string content_type = 4;
  // 呼出元コンテキスト
  k1s0.tier1.common.v1.TenantContext context = 5;
  // タイムアウト (ミリ秒、省略時は 5000ms)
  int32 timeout_ms = 6;
}

// Invoke 応答
message InvokeResponse {
  // 応答データ
  bytes data = 1;
  // Content-Type
  string content_type = 2;
  // HTTP ステータス相当 (成功 200、失敗時は詳細を Status に載せる)
  int32 status = 3;
}

// ストリーム応答のチャンク
message InvokeChunk {
  bytes data = 1;
  // ストリーム終端フラグ
  bool eof = 2;
}
```

## 02. State API

KV / Relational / Document の状態管理を抽象化する。ETag による楽観的排他、TTL、バルク操作、トランザクションを提供する。

```protobuf
// 状態管理 API (FR-T1-STATE-001〜005)
syntax = "proto3";
package k1s0.tier1.state.v1;
import "k1s0/tier1/common/v1/common.proto";

service StateService {
  // キー単位の取得
  rpc Get(GetRequest) returns (GetResponse);
  // キー単位の保存 (ETag 不一致時は FAILED_PRECONDITION)
  rpc Set(SetRequest) returns (SetResponse);
  // キー単位の削除
  rpc Delete(DeleteRequest) returns (DeleteResponse);
  // 複数キーの一括取得
  rpc BulkGet(BulkGetRequest) returns (BulkGetResponse);
  // トランザクション境界付きの複数操作 (全 Store で対応するわけではない)
  rpc Transact(TransactRequest) returns (TransactResponse);
}

// Get リクエスト
message GetRequest {
  // Store 名 (valkey-default / postgres-tenant 等、運用側で設定)
  string store = 1;
  // キー (テナント境界は tier1 が自動付与、クライアントはテナント内キーのみ指定)
  string key = 2;
  k1s0.tier1.common.v1.TenantContext context = 3;
}

message GetResponse {
  bytes data = 1;
  // 楽観的排他のための ETag
  string etag = 2;
  // キー未存在時は true
  bool not_found = 3;
}

message SetRequest {
  string store = 1;
  string key = 2;
  bytes data = 3;
  // 期待 ETag (空は未存在前提)
  string expected_etag = 4;
  // TTL (秒、0 は永続)
  int32 ttl_sec = 5;
  k1s0.tier1.common.v1.TenantContext context = 6;
}

message SetResponse {
  string new_etag = 1;
}

message DeleteRequest {
  string store = 1;
  string key = 2;
  string expected_etag = 3;
  k1s0.tier1.common.v1.TenantContext context = 4;
}

message DeleteResponse {
  bool deleted = 1;
}

message BulkGetRequest {
  string store = 1;
  repeated string keys = 2;
  k1s0.tier1.common.v1.TenantContext context = 3;
}

message BulkGetResponse {
  map<string, GetResponse> results = 1;
}

// トランザクション内の 1 操作
message TransactOp {
  oneof op {
    SetRequest set = 1;
    DeleteRequest delete = 2;
  }
}

message TransactRequest {
  string store = 1;
  repeated TransactOp operations = 2;
  k1s0.tier1.common.v1.TenantContext context = 3;
}

message TransactResponse {
  bool committed = 1;
}
```

## 03. PubSub API

Kafka バックエンドを抽象化する Publish / Subscribe API。At-least-once 配信、冪等性キー、Dead Letter Queue を提供する。

```protobuf
// PubSub API (FR-T1-PUBSUB-001〜005)
syntax = "proto3";
package k1s0.tier1.pubsub.v1;
import "k1s0/tier1/common/v1/common.proto";

service PubSubService {
  // 単発 Publish
  rpc Publish(PublishRequest) returns (PublishResponse);
  // バッチ Publish (冪等性のため idempotency_key 必須)
  rpc BulkPublish(BulkPublishRequest) returns (BulkPublishResponse);
  // サブスクリプション (tier2/tier3 側は HTTP コールバック登録 / gRPC ストリームのいずれか)
  rpc Subscribe(SubscribeRequest) returns (stream Event);
}

message PublishRequest {
  // トピック名 (テナント接頭辞は自動付与)
  string topic = 1;
  // イベント本文
  bytes data = 2;
  string content_type = 3;
  // 冪等性キー (重複 Publish を抑止、TTL 24h)
  string idempotency_key = 4;
  // メタデータ (partition_key, trace_id 等)
  map<string, string> metadata = 5;
  k1s0.tier1.common.v1.TenantContext context = 6;
}

message PublishResponse {
  // Kafka 側のオフセット
  int64 offset = 1;
}

message BulkPublishRequest {
  string topic = 1;
  repeated PublishRequest entries = 2;
}

message BulkPublishResponse {
  // 各エントリの結果 (失敗時はエラー詳細)
  repeated BulkPublishEntry results = 1;
}

message BulkPublishEntry {
  int32 entry_index = 1;
  int64 offset = 2;
  string error_code = 3;
}

message SubscribeRequest {
  string topic = 1;
  // コンシューマグループ (テナント単位で分離)
  string consumer_group = 2;
  k1s0.tier1.common.v1.TenantContext context = 3;
}

message Event {
  string topic = 1;
  bytes data = 2;
  string content_type = 3;
  int64 offset = 4;
  map<string, string> metadata = 5;
}
```

## 04. Secrets API

OpenBao をバックエンドとする秘密情報取得 API。Read-only、テナント境界で分離、監査ログ必須。

```protobuf
// Secrets API (FR-T1-SECRETS-001〜004)
syntax = "proto3";
package k1s0.tier1.secrets.v1;
import "k1s0/tier1/common/v1/common.proto";

service SecretsService {
  // 単一シークレット取得
  rpc Get(GetSecretRequest) returns (GetSecretResponse);
  // 一括取得 (テナントに割当された全シークレット)
  rpc BulkGet(BulkGetSecretRequest) returns (BulkGetSecretResponse);
}

message GetSecretRequest {
  // シークレット名 (テナント境界を超えた参照は即 PermissionDenied)
  string name = 1;
  k1s0.tier1.common.v1.TenantContext context = 2;
}

message GetSecretResponse {
  // 値 (Base64 エンコード必要時はクライアント側で判断)
  map<string, string> values = 1;
  // バージョン (ローテーション追跡用)
  int32 version = 2;
}

message BulkGetSecretRequest {
  k1s0.tier1.common.v1.TenantContext context = 1;
}

message BulkGetSecretResponse {
  map<string, GetSecretResponse> results = 1;
}
```

## 05. Binding API

外部 HTTP/SMTP/S3 との入出力連携を抽象化する。入力バインディング（外部 → tier1）と出力バインディング（tier1 → 外部）の両方を提供。

```protobuf
// Binding API (FR-T1-BINDING-001〜004)
syntax = "proto3";
package k1s0.tier1.binding.v1;
import "k1s0/tier1/common/v1/common.proto";

service BindingService {
  // 出力バインディング呼出 (tier1 → 外部システムへ送信)
  rpc Invoke(InvokeBindingRequest) returns (InvokeBindingResponse);
}

message InvokeBindingRequest {
  // バインディング名 (運用側で事前設定、例: s3-archive / smtp-notify)
  string name = 1;
  // 操作種別 (create / get / list / delete / send 等、バインディング型依存)
  string operation = 2;
  bytes data = 3;
  map<string, string> metadata = 4;
  k1s0.tier1.common.v1.TenantContext context = 5;
}

message InvokeBindingResponse {
  bytes data = 1;
  map<string, string> metadata = 2;
}
```

## 06. Workflow API

Temporal バックエンドによる長時間ワークフロー API。Start / Signal / Query / Cancel / Terminate を提供する。

```protobuf
// Workflow API (FR-T1-WORKFLOW-001〜005)
syntax = "proto3";
package k1s0.tier1.workflow.v1;
import "k1s0/tier1/common/v1/common.proto";

service WorkflowService {
  // ワークフロー開始
  rpc Start(StartRequest) returns (StartResponse);
  // シグナル送信 (ワークフローへの入力イベント)
  rpc Signal(SignalRequest) returns (SignalResponse);
  // クエリ (ワークフロー状態の読取り、副作用なし)
  rpc Query(QueryRequest) returns (QueryResponse);
  // 正常終了の依頼 (キャンセル)
  rpc Cancel(CancelRequest) returns (CancelResponse);
  // 強制終了
  rpc Terminate(TerminateRequest) returns (TerminateResponse);
  // 状態取得
  rpc GetStatus(GetStatusRequest) returns (GetStatusResponse);
}

message StartRequest {
  // ワークフロー種別 (tier2 で登録されたコード名)
  string workflow_type = 1;
  // 実行 ID (指定なければ tier1 が UUID を生成)
  string workflow_id = 2;
  // 初期入力
  bytes input = 3;
  // 冪等性 (同一 workflow_id の重複開始は既存実行を返す)
  bool idempotent = 4;
  k1s0.tier1.common.v1.TenantContext context = 5;
}

message StartResponse {
  string workflow_id = 1;
  string run_id = 2;
}

message SignalRequest {
  string workflow_id = 1;
  string signal_name = 2;
  bytes payload = 3;
  k1s0.tier1.common.v1.TenantContext context = 4;
}

message SignalResponse {}

message QueryRequest {
  string workflow_id = 1;
  string query_name = 2;
  bytes payload = 3;
  k1s0.tier1.common.v1.TenantContext context = 4;
}

message QueryResponse {
  bytes result = 1;
}

message CancelRequest {
  string workflow_id = 1;
  string reason = 2;
  k1s0.tier1.common.v1.TenantContext context = 3;
}

message CancelResponse {}

message TerminateRequest {
  string workflow_id = 1;
  string reason = 2;
  k1s0.tier1.common.v1.TenantContext context = 3;
}

message TerminateResponse {}

message GetStatusRequest {
  string workflow_id = 1;
  k1s0.tier1.common.v1.TenantContext context = 2;
}

// 実行状態の列挙
enum WorkflowStatus {
  RUNNING = 0;
  COMPLETED = 1;
  FAILED = 2;
  CANCELED = 3;
  TERMINATED = 4;
  CONTINUED_AS_NEW = 5;
}

message GetStatusResponse {
  WorkflowStatus status = 1;
  string run_id = 2;
  // 完了時の出力 (status = COMPLETED の時のみ)
  bytes output = 3;
  // 失敗時のエラー詳細
  k1s0.tier1.common.v1.ErrorDetail error = 4;
}
```

## 07. Log API

構造化ログ送信 API。OpenTelemetry Logs に準拠し、Grafana Loki へ集約する。

```protobuf
// Log API (FR-T1-LOG-001〜004)
syntax = "proto3";
package k1s0.tier1.log.v1;
import "k1s0/tier1/common/v1/common.proto";
import "google/protobuf/timestamp.proto";

service LogService {
  rpc Send(SendLogRequest) returns (SendLogResponse);
  rpc BulkSend(BulkSendLogRequest) returns (BulkSendLogResponse);
}

// 重大度 (OpenTelemetry Log Severity と整合)
enum Severity {
  TRACE = 0;
  DEBUG = 5;
  INFO = 9;
  WARN = 13;
  ERROR = 17;
  FATAL = 21;
}

message LogEntry {
  google.protobuf.Timestamp timestamp = 1;
  Severity severity = 2;
  // メッセージ本文 (PII 自動検出対象)
  string body = 3;
  // 属性 (service.name / env / trace_id / span_id を含む)
  map<string, string> attributes = 4;
  // 関連する例外スタック (オプション)
  string stack_trace = 5;
}

message SendLogRequest {
  LogEntry entry = 1;
  k1s0.tier1.common.v1.TenantContext context = 2;
}

message SendLogResponse {}

message BulkSendLogRequest {
  repeated LogEntry entries = 1;
  k1s0.tier1.common.v1.TenantContext context = 2;
}

message BulkSendLogResponse {
  int32 accepted = 1;
  int32 rejected = 2;
}
```

## 08. Telemetry API

メトリクス・トレース送信 API。OpenTelemetry に準拠し、Grafana Mimir / Tempo へ集約する。

```protobuf
// Telemetry API (FR-T1-TELEMETRY-001〜004)
syntax = "proto3";
package k1s0.tier1.telemetry.v1;
import "k1s0/tier1/common/v1/common.proto";
import "google/protobuf/timestamp.proto";

service TelemetryService {
  rpc EmitMetric(EmitMetricRequest) returns (EmitMetricResponse);
  rpc EmitSpan(EmitSpanRequest) returns (EmitSpanResponse);
}

// メトリクス種別
enum MetricKind {
  COUNTER = 0;
  GAUGE = 1;
  HISTOGRAM = 2;
}

message Metric {
  string name = 1;
  MetricKind kind = 2;
  double value = 3;
  map<string, string> labels = 4;
  google.protobuf.Timestamp timestamp = 5;
}

message EmitMetricRequest {
  repeated Metric metrics = 1;
  k1s0.tier1.common.v1.TenantContext context = 2;
}

message EmitMetricResponse {}

message Span {
  string trace_id = 1;
  string span_id = 2;
  string parent_span_id = 3;
  string name = 4;
  google.protobuf.Timestamp start_time = 5;
  google.protobuf.Timestamp end_time = 6;
  map<string, string> attributes = 7;
}

message EmitSpanRequest {
  repeated Span spans = 1;
  k1s0.tier1.common.v1.TenantContext context = 2;
}

message EmitSpanResponse {}
```

## 09. Decision API

ZEN Engine による JDM (JSON Decision Model) 評価 API。ルール評価、結果の根拠（trace）を返す。

```protobuf
// Decision API (FR-T1-DECISION-001〜004)
syntax = "proto3";
package k1s0.tier1.decision.v1;
import "k1s0/tier1/common/v1/common.proto";

service DecisionService {
  // ルール評価 (同期)
  rpc Evaluate(EvaluateRequest) returns (EvaluateResponse);
  // バッチ評価 (複数入力を一括評価)
  rpc BatchEvaluate(BatchEvaluateRequest) returns (BatchEvaluateResponse);
}

message EvaluateRequest {
  // ルール ID (tier2 で登録した JDM 文書の識別子)
  string rule_id = 1;
  // ルールバージョン (省略時は最新有効)
  string rule_version = 2;
  // 入力 (JDM の context に相当、任意 JSON)
  bytes input_json = 3;
  // trace 情報を返すか (デバッグ用、PII を含む可能性あり)
  bool include_trace = 4;
  k1s0.tier1.common.v1.TenantContext context = 5;
}

message EvaluateResponse {
  // 出力 (JDM 評価結果、任意 JSON)
  bytes output_json = 1;
  // 評価されたノードのトレース (include_trace=true の時のみ)
  bytes trace_json = 2;
  // 評価にかかった時間 (マイクロ秒)
  int64 elapsed_us = 3;
}

message BatchEvaluateRequest {
  string rule_id = 1;
  string rule_version = 2;
  repeated bytes inputs_json = 3;
  k1s0.tier1.common.v1.TenantContext context = 4;
}

message BatchEvaluateResponse {
  repeated bytes outputs_json = 1;
}
```

## 10. Audit / Pii API

監査イベント記録と PII 自動判定 API。監査は WORM 保管、PII 判定は Decision API を内部的に利用する。

```protobuf
// Audit / Pii API (FR-T1-AUDIT-001〜003 / FR-T1-PII-001〜002)
syntax = "proto3";
package k1s0.tier1.audit.v1;
import "k1s0/tier1/common/v1/common.proto";
import "google/protobuf/timestamp.proto";

service AuditService {
  rpc Record(RecordAuditRequest) returns (RecordAuditResponse);
  rpc Query(QueryAuditRequest) returns (QueryAuditResponse);
}

service PiiService {
  rpc Classify(ClassifyRequest) returns (ClassifyResponse);
  rpc Mask(MaskRequest) returns (MaskResponse);
}

message AuditEvent {
  google.protobuf.Timestamp timestamp = 1;
  // 操作主体 (user_id / workload_id)
  string actor = 2;
  // 操作種別 (CREATE / READ / UPDATE / DELETE / LOGIN / EXPORT)
  string action = 3;
  // 対象リソース (URN 形式: k1s0:tenant:<tid>:resource:<type>/<id>)
  string resource = 4;
  // 操作結果 (SUCCESS / DENIED / ERROR)
  string outcome = 5;
  // 追加コンテキスト
  map<string, string> attributes = 6;
}

message RecordAuditRequest {
  AuditEvent event = 1;
  k1s0.tier1.common.v1.TenantContext context = 2;
}

message RecordAuditResponse {
  // WORM ストアでの固有 ID
  string audit_id = 1;
}

message QueryAuditRequest {
  // 範囲指定
  google.protobuf.Timestamp from = 1;
  google.protobuf.Timestamp to = 2;
  // フィルタ (任意の attributes 等価一致)
  map<string, string> filters = 3;
  int32 limit = 4;
  k1s0.tier1.common.v1.TenantContext context = 5;
}

message QueryAuditResponse {
  repeated AuditEvent events = 1;
}

message ClassifyRequest {
  // 判定対象テキスト
  string text = 1;
  k1s0.tier1.common.v1.TenantContext context = 2;
}

message PiiFinding {
  // 検出された PII 種別 (NAME / EMAIL / PHONE / MYNUMBER / CREDITCARD 等)
  string type = 1;
  // 文字列内の位置 (start, end)
  int32 start = 2;
  int32 end = 3;
  // 信頼度 (0.0〜1.0)
  double confidence = 4;
}

message ClassifyResponse {
  repeated PiiFinding findings = 1;
  // PII を含むか (findings が空でなければ true)
  bool contains_pii = 2;
}

message MaskRequest {
  string text = 1;
  k1s0.tier1.common.v1.TenantContext context = 2;
}

message MaskResponse {
  // マスク後のテキスト (氏名 → [NAME]、メール → [EMAIL])
  string masked_text = 1;
  repeated PiiFinding findings = 2;
}
```

## 11. Feature API

Feature Flag 評価 API。flagd / OpenFeature 準拠、Release/Experiment/Ops/Permission の 4 種別を区別する。

```protobuf
// Feature API (FR-T1-FEATURE-001〜004)
syntax = "proto3";
package k1s0.tier1.feature.v1;
import "k1s0/tier1/common/v1/common.proto";

service FeatureService {
  // Boolean Flag 評価
  rpc EvaluateBoolean(EvaluateRequest) returns (BooleanResponse);
  // String Flag 評価 (Variant)
  rpc EvaluateString(EvaluateRequest) returns (StringResponse);
  // 数値 Flag 評価
  rpc EvaluateNumber(EvaluateRequest) returns (NumberResponse);
  // JSON オブジェクト Flag 評価
  rpc EvaluateObject(EvaluateRequest) returns (ObjectResponse);
}

// Flag 評価の共通入力
message EvaluateRequest {
  // Flag キー (命名規則: <tenant>.<component>.<feature>)
  string flag_key = 1;
  // 評価コンテキスト (targetingKey は subject と同一)
  map<string, string> evaluation_context = 2;
  k1s0.tier1.common.v1.TenantContext context = 3;
}

// Flag の種別 (OpenFeature / k1s0 固有)
enum FlagKind {
  RELEASE = 0;
  EXPERIMENT = 1;
  OPS = 2;
  PERMISSION = 3;
}

message FlagMetadata {
  FlagKind kind = 1;
  // バリアント名 (有効化理由の参考)
  string variant = 2;
  // 評価の理由 (DEFAULT / TARGETING_MATCH / SPLIT / ERROR)
  string reason = 3;
}

message BooleanResponse {
  bool value = 1;
  FlagMetadata metadata = 2;
}

message StringResponse {
  string value = 1;
  FlagMetadata metadata = 2;
}

message NumberResponse {
  double value = 1;
  FlagMetadata metadata = 2;
}

message ObjectResponse {
  bytes value_json = 1;
  FlagMetadata metadata = 2;
}
```

## IDL バージョニングと配布

tier1 API の IDL は SemVer で管理する。MAJOR は破壊的変更（メッセージ削除、RPC 削除）、MINOR は追加（新 RPC、新フィールド tag）、PATCH はドキュメント修正のみ。破壊的変更は OR-EOL-001 の非推奨ライフサイクルに従い 12 か月前告知。

IDL ファイルは Git モノレポ内の `proto/k1s0/tier1/` 配下で管理し、Buf（buf.build）で lint/breaking check を CI で強制する。tier2/tier3 クライアントライブラリは buf generate で Rust/Go/C# から生成、Nexus/Artifactory に公開する。

## メンテナンス

IDL の変更は ADR-TIER1-002（内部通信 Protobuf gRPC）と連動して行う。要件変更時に本書の IDL スケルトンが整合しない場合、PR で同時更新必須。四半期ごとに Product Council で IDL の網羅性と SemVer 適合をレビュー。

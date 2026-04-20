# 40. tier1 API 契約 IDL

本書は tier1 が公開する 11 API のインタフェース契約（Protobuf IDL）をスケルトン形式で定義する。各 API の要件詳細は [10_tier1_API要件/](10_tier1_API要件/) に記述されており、本書はそれに対応する機械可読な契約骨格を提供する。tier2/tier3 はこの IDL から生成されるクライアントライブラリ経由でのみ tier1 を利用し、内部実装言語（Go/Rust）には依存しない。

## 本書の位置付け

要件記述だけではインタフェース契約は一意に定まらない。例えば「State API は取得・更新・削除を提供する」と書かれていても、エラーコードの体系、冪等性キー、ETag、トランザクション境界、バルク操作可否といった具体は IDL を見ないと確定しない。tier2/tier3 開発者は IDL からクライアントを生成してから開発に入るため、IDL が無いと並行開発が始められない。

本書の IDL は「要件定義段階で確定すべき契約の最小限」であり、詳細設計で message フィールドの追加・RPC の分割統合が行われる。ただし以下は要件定義の合意事項として本書で固定する。

- 全 RPC は gRPC over HTTP/2、mTLS 必須、ヘッダで `x-tenant-id` `x-correlation-id` を伝搬
- 全エラーは `google.rpc.Status` を使い、`details` に `ErrorDetail`（共通エラー体系）を載せる
- メッセージのフィールドタグは 1〜15 の空きを予約領域として将来拡張用に空ける

## 責任分界表: 要件定義 / 基本設計の IDL 変更ルール

本書の IDL は骨格であり、詳細設計で細案化される。tier2/tier3 開発者は IDL を「契約」として生成コードに取り込むため、どの要素が誰の責任で、どの段階で固定され、どう変わりうるかを明示しない限り、並行開発で破壊的変更による手戻りが発生する。本節は IDL 各要素の所有権とライフサイクルを定義する。

| IDL 要素 | 所有者 | 要件定義での扱い | 基本設計での変更ルール | 後方互換性 | 変更手続き |
|---|---|---|---|---|---|
| `service` / RPC メソッド名・シグネチャ | tier1 テックリード | **固定**（本書で確定） | 破壊的変更禁止（semver major 相当） | クライアント再生成で継続動作 | ADR 記録必須、Product Council 承認 |
| `rpc` の引数型 / 戻り型 | tier1 テックリード | **固定** | 破壊的変更禁止（型置換は別 RPC を新設） | 同上 | 同上 |
| `message` のフィールド追加 | tier1 API 担当 | スケルトン、詳細設計で追加 | 後方互換を保ち追加のみ可、`reserved` で削除範囲宣言 | proto3 のデフォルト値で後方互換 | PR レビューのみ |
| `message` のフィールド削除 | tier1 API 担当 | 非推奨化は IDL コメントで宣言 | 1 年間の非推奨期間を経て削除、`reserved` 化 | 1 年猶予で互換担保 | ADR 記録、OR-EOL-* 連動 |
| フィールドのタグ番号（1〜15） | tier1 テックリード | **予約領域**として確保 | 後方互換リリースまで未使用フィールドを占有 | タグ番号変更は破壊的 | ADR 記録必須 |
| `ErrorDetail.code` の値体系 | tier1 テックリード + セキュリティ | **固定**（`E-<CATEGORY>-<MODULE>-<NUMBER>`） | 新コード追加のみ、既存コード意味変更禁止 | 未知コードは `UNKNOWN` 扱い | PR レビューのみ |
| `TenantContext` スキーマ | tier1 横断 | **固定**（全 API 共通） | 追加のみ、既存フィールド削除禁止 | 同上 | ADR 記録必須 |
| gRPC ステータスコードマッピング | tier1 テックリード | 本書で確定（UNAVAILABLE → 503 等） | 新規マッピング追加のみ | 既存マッピング変更は破壊的 | ADR 記録必須 |
| ストリーミング RPC の追加 | tier1 API 担当 | スケルトン（InvokeStream 等） | 追加のみ、既存 unary を stream に置換禁止 | unary → stream は破壊的 | ADR 記録必須 |
| `option` / `extension` | tier1 テックリード | 未使用 | 詳細設計で Dapr 互換 option を追加可 | gRPC の拡張性で互換担保 | PR レビューのみ |

### TenantContext の伝搬方式

`TenantContext` は全 API の `message` に埋め込む形式（各 Request の `context` フィールド）と、gRPC メタデータヘッダ（`x-tenant-id` / `x-correlation-id` / `x-subject`）の二重伝搬とする。メタデータヘッダは Istio Ambient L7 ポリシーと OpenTelemetry トレース伝播が参照できる形式として必須、message 埋め込みは RPC 内での参照容易性とテストでの明示性を担保する。tier1 ファサードの interceptor は両者の整合性を検証し、不一致時は `E-AUTH-CTX-MISMATCH` を返す。詳細設計で interceptor 実装方針を ADR に記録する。

### API 要件ファイルとの対応

本 IDL の各 API ブロックは、`10_tier1_API要件/` 配下の要件ファイルと 1 対 1 で対応する。要件の散文記述（現状→達成後→崩れた時）と IDL（機械可読な契約骨格）は相補的であり、要件ファイルの「入出力仕様」セクションは本書の該当ブロックへのアンカーリンクで参照する運用とする。

| API 番号 | 要件ファイル | IDL セクション |
|---|---|---|
| 01 | [01_Service_Invoke_API.md](10_tier1_API要件/01_Service_Invoke_API.md) | [01. Service Invoke API](#01-service-invoke-api) |
| 02 | [02_State_API.md](10_tier1_API要件/02_State_API.md) | [02. State API](#02-state-api) |
| 03 | [03_PubSub_API.md](10_tier1_API要件/03_PubSub_API.md) | [03. PubSub API](#03-pubsub-api) |
| 04 | [04_Secrets_API.md](10_tier1_API要件/04_Secrets_API.md) | [04. Secrets API](#04-secrets-api) |
| 05 | [05_Binding_API.md](10_tier1_API要件/05_Binding_API.md) | [05. Binding API](#05-binding-api) |
| 06 | [06_Workflow_API.md](10_tier1_API要件/06_Workflow_API.md) | [06. Workflow API](#06-workflow-api) |
| 07 | [07_Log_API.md](10_tier1_API要件/07_Log_API.md) | [07. Log API](#07-log-api) |
| 08 | [08_Telemetry_API.md](10_tier1_API要件/08_Telemetry_API.md) | [08. Telemetry API](#08-telemetry-api) |
| 09 | [09_Decision_API.md](10_tier1_API要件/09_Decision_API.md) | [09. Decision API](#09-decision-api) |
| 10 | [10_Audit_Pii_API.md](10_tier1_API要件/10_Audit_Pii_API.md) | [10. Audit / Pii API](#10-audit--pii-api) |
| 11 | [11_Feature_API.md](10_tier1_API要件/11_Feature_API.md) | [11. Feature API](#11-feature-api) |

要件ファイル側の「入出力仕様」セクションに疑似インタフェースを残す場合、本書 IDL との対応（例: 疑似 `options.timeout_seconds` は本書 `InvokeRequest.timeout_ms`）を明記する。対応記述のない疑似インタフェースは、要件ファイル更新時に IDL 側を破壊的に変更してしまうリスクがあるため、レビューで指摘される。

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
  // エラーコード (E-<CATEGORY>-<MODULE>-<NUMBER> 形式、詳細検索用)
  string code = 1;
  // 機械可読カテゴリ (switch 分岐用、enum 追加時は後方互換を維持)
  K1s0ErrorCategory category = 5;
  // 人間可読なメッセージ (テナント表示可)
  string message = 2;
  // 再試行可否 (true の場合クライアントは指数バックオフで再試行)
  bool retryable = 3;
  // 再試行までの推奨待機時間 (ミリ秒)
  int32 retry_after_ms = 4;
}

// 機械可読なエラーカテゴリ。00_tier1_API共通規約.md の 8 カテゴリに対応。
// 新規追加は MINOR バージョンで許可、既存値の削除・意味変更は禁止。
// 未知カテゴリを受け取った tier2/tier3 SDK は UNSPECIFIED として扱う。
enum K1s0ErrorCategory {
  // 未指定 (既定値、クライアントは UNKNOWN として扱う)
  K1S0_ERROR_UNSPECIFIED = 0;
  // 呼出側入力誤り (リトライ不可)
  K1S0_ERROR_INVALID_ARGUMENT = 1;
  // JWT 不在・署名不正・期限切れ
  K1S0_ERROR_UNAUTHENTICATED = 2;
  // RBAC 拒否・テナント越境・allowlist 外
  K1S0_ERROR_PERMISSION_DENIED = 3;
  // キー / バージョン / リソース未存在
  K1S0_ERROR_NOT_FOUND = 4;
  // ETag 不一致・冪等性キー衝突
  K1S0_ERROR_CONFLICT = 5;
  // レート制限・クォータ超過 (retry_after_ms 必須)
  K1S0_ERROR_RESOURCE_EXHAUSTED = 6;
  // 一時的バックエンド不能 (retry_after_ms 必須、指数バックオフ)
  K1S0_ERROR_UNAVAILABLE = 7;
  // tier1 バグ・未分類 (Audit に Severity 2 で記録)
  K1S0_ERROR_INTERNAL = 8;
  // gRPC Deadline 超過 (副作用未発生扱い、再試行可)
  K1S0_ERROR_DEADLINE_EXCEEDED = 9;
}

// 具体コードの命名規約 (string code の値):
//   E-<CATEGORY>-<MODULE>-<NUMBER>
// 例:
//   E-INVALID_ARGUMENT-INVOKE-001 (Service Invoke の引数不正)
//   E-UNAUTHENTICATED-AUTH-001    (JWT 署名検証失敗)
//   E-PERMISSION_DENIED-AUTH-002  (tenant_id 越境)
//   E-NOT_FOUND-STATE-001         (Key 未存在)
//   E-CONFLICT-STATE-002          (ETag 不一致)
//   E-RESOURCE_EXHAUSTED-RATELIMIT-001 (Per-tenant RPS 上限超過)
//   E-UNAVAILABLE-KAFKA-001       (Kafka ブローカー接続不可)
//   E-INTERNAL-DECISION-001       (ZEN Engine 内部例外)
// MODULE 値の全列挙:
//   AUTH / INVOKE / STATE / PUBSUB / SECRETS / BINDING / WORKFLOW /
//   LOG / TELEMETRY / DECISION / AUDIT / PII / FEATURE /
//   RATELIMIT / KAFKA / POSTGRES / VALKEY / OPENBAO / CTX
// NUMBER は 001 から連番、欠番許容、001〜099 は tier1 共通予約。
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

OpenBao をバックエンドとする秘密情報取得・ローテーション API。Read / Rotate の両面を持ち、テナント境界で分離、監査ログ必須（Rotate は NFR-E-MON-001 で WORM 記録）。

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
  // ローテーション実行 (FR-T1-SECRETS-004)
  // 成功時は new_version を返し、旧バージョンは grace_period_sec まで Get 可能
  // 失敗時は K1s0Error を返し OpenBao 側は不変 (トランザクショナル)
  rpc Rotate(RotateSecretRequest) returns (RotateSecretResponse);
}

message GetSecretRequest {
  // シークレット名 (テナント境界を超えた参照は即 PermissionDenied)
  string name = 1;
  k1s0.tier1.common.v1.TenantContext context = 2;
  // 省略時は最新、明示で旧バージョン取得可 (grace_period 中のみ)
  optional int32 version = 3;
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

message RotateSecretRequest {
  // ローテーション対象シークレット名
  string name = 1;
  k1s0.tier1.common.v1.TenantContext context = 2;
  // 旧バージョンの猶予時間 (0 は即無効、既定 3600 秒)
  // tier2 側の接続プール drain 時間を想定
  int32 grace_period_sec = 3;
  // 動的シークレット (DB 資格情報等) の場合の発行ポリシー名
  optional string policy = 4;
  // 冪等性キー (同一キーでの再試行は同じ new_version を返す)
  string idempotency_key = 5;
}

message RotateSecretResponse {
  // ローテーション後の新バージョン
  int32 new_version = 1;
  // 旧バージョン (grace_period_sec まで Get 可能)
  int32 previous_version = 2;
  // 新バージョン発効時刻
  int64 rotated_at_ms = 3;
  // 動的シークレット時の TTL (静的シークレットでは 0)
  int32 ttl_sec = 4;
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

### JDM (JSON Decision Model) スキーマ

tier2 が登録する JDM 文書は以下の JSON Schema（抜粋、機械可読な完全版は `proto/k1s0/tier1/decision/v1/jdm_schema.json` で管理）に従う。ZEN Engine v0.36 の JDM v1 仕様に準拠し、k1s0 独自の非決定要素禁止ルールを schema validator で強制する。

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://k1s0.jtc.local/schema/jdm/v1",
  "type": "object",
  "required": ["contentType", "nodes", "edges"],
  "properties": {
    "contentType": { "const": "application/vnd.gorules.decision" },
    "nodes": {
      "type": "array",
      "items": {
        "oneOf": [
          { "$ref": "#/$defs/inputNode" },
          { "$ref": "#/$defs/outputNode" },
          { "$ref": "#/$defs/decisionTableNode" },
          { "$ref": "#/$defs/expressionNode" },
          { "$ref": "#/$defs/functionNode" },
          { "$ref": "#/$defs/switchNode" }
        ]
      }
    },
    "edges": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["sourceId", "targetId"],
        "properties": {
          "sourceId": { "type": "string" },
          "targetId": { "type": "string" },
          "sourceHandle": { "type": "string" }
        }
      }
    }
  },
  "$defs": {
    "decisionTableNode": {
      "type": "object",
      "required": ["id", "type", "content"],
      "properties": {
        "type": { "const": "decisionTableNode" },
        "content": {
          "type": "object",
          "required": ["inputs", "outputs", "rules", "hitPolicy"],
          "properties": {
            "hitPolicy": { "enum": ["first", "collect"] },
            "inputs":  { "type": "array", "items": { "$ref": "#/$defs/columnDef" } },
            "outputs": { "type": "array", "items": { "$ref": "#/$defs/columnDef" } },
            "rules":   { "type": "array", "items": { "type": "object" } }
          }
        }
      }
    },
    "columnDef": {
      "type": "object",
      "required": ["id", "name", "field"],
      "properties": {
        "id":   { "type": "string" },
        "name": { "type": "string" },
        "field":{ "type": "string" }
      }
    }
  }
}
```

### JDM 非決定要素禁止ルール（k1s0 独自）

以下を含む JDM 文書は schema validator で reject する（NFR-I-SLO-009 Correctness 100% 担保のため）。

- `time.Now()` / `now()` / 現在時刻関数呼出し（代わりに評価時に `evaluation_context.now` で注入）
- `random()` / 乱数関数
- 外部 HTTP / DB アクセス（expression 内）
- 再帰深度 10 階層超（Decision graph の depth）
- 1 decision table の rule 数 10,000 超

これらは CI の JDM lint で検出し、違反は PR マージ不可とする。

### Decision API Protobuf

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

// JDM ルール文書の登録・バージョン管理 (Phase 1b で proto 追加予定)
service DecisionAdminService {
  // JDM 文書の登録 (schema validator と非決定要素 linter を通過必須)
  rpc RegisterRule(RegisterRuleRequest) returns (RegisterRuleResponse);
  // バージョン一覧
  rpc ListVersions(ListVersionsRequest) returns (ListVersionsResponse);
  // 特定バージョンの取得 (レビュー用)
  rpc GetRule(GetRuleRequest) returns (GetRuleResponse);
}

message RegisterRuleRequest {
  // ルール ID (tenant 内で一意)
  string rule_id = 1;
  // JDM 文書 (前節 JSON Schema に準拠、UTF-8 JSON)
  bytes jdm_document = 2;
  // Sigstore 署名 (ADR-RULE-001、registry に登録する署名)
  bytes sigstore_signature = 3;
  // コミット ID (Git commit hash、JDM バージョン追跡用)
  string commit_hash = 4;
  k1s0.tier1.common.v1.TenantContext context = 5;
}

message RegisterRuleResponse {
  // 採番されたバージョン (tenant + rule_id 内で一意、単調増加)
  string rule_version = 1;
  // 発効可能となる時刻 (即時なら registered_at と同じ)
  int64 effective_at_ms = 2;
}

message ListVersionsRequest {
  string rule_id = 1;
  k1s0.tier1.common.v1.TenantContext context = 2;
}

message ListVersionsResponse {
  repeated RuleVersionMeta versions = 1;
}

message RuleVersionMeta {
  string rule_version = 1;
  string commit_hash = 2;
  int64 registered_at_ms = 3;
  string registered_by = 4;
  // DEPRECATED 状態 (非推奨のみ true、廃止後は ListVersions から消える)
  bool deprecated = 5;
}

message GetRuleRequest {
  string rule_id = 1;
  string rule_version = 2;
  k1s0.tier1.common.v1.TenantContext context = 3;
}

message GetRuleResponse {
  bytes jdm_document = 1;
  RuleVersionMeta meta = 2;
}
```

### Decision API 実装方式（要件段階の合意事項）

- **IPC 方式**: Go ファサード（Dapr 経由）→ ZEN Engine (Rust) は **gRPC over Unix domain socket**（`unix:///var/run/k1s0/zen.sock`）で呼出し。FFI は Rust side の panic が Go プロセス全体を落とすリスクがあり採用しない
- **JDM バージョニング**: Git commit hash（rule_version として返却）。古いバージョンは 90 日間並走可能、その後は ListVersions から消える
- **決定論的性**: 同一 `(rule_version, evaluation_context)` は 100% 同一出力（NFR-I-SLO-009）。外部参照禁止 linter で担保
- **キャッシュ**: 評価結果は evaluation_context の SHA-256 を key とした in-memory LRU（1,000 エントリ、TTL 1 分）を Go 側で保持。Decision 評価自体は sub-ms だが、tier1 側での network/marshal コストを抑制

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

// Flag 定義の登録・更新 (Phase 1b 提供)
service FeatureAdminService {
  rpc RegisterFlag(RegisterFlagRequest) returns (RegisterFlagResponse);
  rpc GetFlag(GetFlagRequest) returns (GetFlagResponse);
  rpc ListFlags(ListFlagsRequest) returns (ListFlagsResponse);
}

// flagd 互換の Flag 定義 (k1s0 は OpenFeature / flagd 仕様に準拠)
message FlagDefinition {
  // Flag キー (命名規則: <tenant>.<component>.<feature>)
  string flag_key = 1;
  // Flag 種別 (RELEASE / EXPERIMENT / OPS / PERMISSION)
  FlagKind kind = 2;
  // 戻り値型 (boolean / string / number / object)
  FlagValueType value_type = 3;
  // デフォルト variant の名前 (下記 variants にキーが存在すること)
  string default_variant = 4;
  // variants 定義: variant 名 → 値 (value_type に応じた JSON literal)
  map<string, google.protobuf.Value> variants = 5;
  // targeting ルール (先頭から評価、最初に match したもの採用)
  repeated TargetingRule targeting = 6;
  // 状態 (ENABLED / DISABLED / ARCHIVED)
  FlagState state = 7;
  // 説明 (監査・運用者向け)
  string description = 8;
}

enum FlagValueType {
  FLAG_VALUE_UNSPECIFIED = 0;
  FLAG_VALUE_BOOLEAN = 1;
  FLAG_VALUE_STRING = 2;
  FLAG_VALUE_NUMBER = 3;
  FLAG_VALUE_OBJECT = 4;
}

enum FlagState {
  FLAG_STATE_UNSPECIFIED = 0;
  FLAG_STATE_ENABLED = 1;
  FLAG_STATE_DISABLED = 2;
  FLAG_STATE_ARCHIVED = 3;
}

// targeting ルール (JsonLogic 互換、flagd 仕様準拠)
// 例: { "if": [ { "==": [{ "var": "userRole" }, "admin"] }, "blue-variant", "red-variant" ] }
message TargetingRule {
  // ルール ID (監査用、tenant+flag 内で一意)
  string rule_id = 1;
  // JsonLogic 式 (bytes で保持、登録時に schema validator 通過必須)
  bytes json_logic_expr = 2;
  // 評価成立時に返す variant 名
  string variant_if_match = 3;
  // Fractional split (A/B テスト用、weights 合計 100 必須)
  repeated FractionalSplit fractional = 4;
}

// Experiment 種別の Flag で A/B 比率を指定
message FractionalSplit {
  string variant = 1;
  // 重み (0〜100、全エントリ合計 100 必須)
  int32 weight = 2;
}

message RegisterFlagRequest {
  FlagDefinition flag = 1;
  // 変更理由 (permission 種別 Flag の場合 Product Council 承認番号必須)
  string change_reason = 2;
  // permission 種別時の承認番号 (空値は permission 種別で reject)
  string approval_id = 3;
  k1s0.tier1.common.v1.TenantContext context = 4;
}

message RegisterFlagResponse {
  // バージョン (flag_key 内で単調増加)
  int64 version = 1;
}

message GetFlagRequest {
  string flag_key = 1;
  // バージョン (省略時は最新)
  optional int64 version = 2;
  k1s0.tier1.common.v1.TenantContext context = 3;
}

message GetFlagResponse {
  FlagDefinition flag = 1;
  int64 version = 2;
}

message ListFlagsRequest {
  // 種別フィルタ (省略で全種別)
  optional FlagKind kind = 1;
  // 状態フィルタ (省略で ENABLED のみ)
  optional FlagState state = 2;
  k1s0.tier1.common.v1.TenantContext context = 3;
}

message ListFlagsResponse {
  repeated FlagDefinition flags = 1;
}
```

### Feature API 4 種別の運用ルール

- **RELEASE**: 新機能の段階解放。variants は `{on, off}` が基本、targeting で canary → GA の段階拡大。廃止期限（sunset date）を必須項目とし、90 日超の放置は自動 ARCHIVED
- **EXPERIMENT**: A/B テスト。FractionalSplit を使用、最低 2 variants + 1 control。実験終了後は勝ち variant を default に昇格して ARCHIVED
- **OPS**: 運用 kill switch（例: 外部 API 連携を緊急遮断）。variants は `{enabled, disabled}` のみ、targeting は最小限
- **PERMISSION**: 権限変更（feature 可視性）。登録時に Product Council の approval_id 必須、未指定は reject。変更履歴は 7 年保管（NFR-E-MON-001 に準拠）

## IDL バージョニングと配布

tier1 API の IDL は SemVer で管理する。MAJOR は破壊的変更（メッセージ削除、RPC 削除）、MINOR は追加（新 RPC、新フィールド tag）、PATCH はドキュメント修正のみ。破壊的変更は OR-EOL-001 の非推奨ライフサイクルに従い 12 か月前告知。

IDL ファイルは Git モノレポ内の `proto/k1s0/tier1/` 配下で管理し、Buf（buf.build）で lint/breaking check を CI で強制する。tier2/tier3 クライアントライブラリは buf generate で Rust/Go/C# から生成、Nexus/Artifactory に公開する。

## メンテナンス

IDL の変更は ADR-TIER1-002（内部通信 Protobuf gRPC）と連動して行う。要件変更時に本書の IDL スケルトンが整合しない場合、PR で同時更新必須。四半期ごとに Product Council で IDL の網羅性と SemVer 適合をレビュー。

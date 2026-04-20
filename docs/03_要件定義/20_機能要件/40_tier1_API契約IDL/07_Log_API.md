# 07. Log API

構造化ログ送信 API。OpenTelemetry Logs の仕様に準拠し、Grafana Loki へ集約する。tier2/tier3 はアプリケーション固有のログライブラリを使っても、最終的に本 API 経由で tier1 ファサードへ送信することで、`service.name` / `env` / `trace_id` / `span_id` 等の属性が自動付与され、相関分析が可能となる。

## 要件対応

- 要件ファイル: [../10_tier1_API要件/07_Log_API.md](../10_tier1_API要件/07_Log_API.md)
- 要件 ID: FR-T1-LOG-001〜004
- 共通型: [00_共通型定義.md](00_共通型定義.md)

## 設計のポイント

`Severity` 列挙は OpenTelemetry Log Severity の値（TRACE=1, DEBUG=5, INFO=9, WARN=13, ERROR=17, FATAL=21）と整合させ、他 OTel プロダクトとの相互運用性を保つ。`body` は PII 自動検出の対象として tier1 側で判定され、検出時は監査ログへの警告記録と同時にマスク処理が適用される。単発送信 `Send` は同期応答で tier2/tier3 の制御フローに組み込みやすくし、高頻度送信には `BulkSend` でバッチ化することで Loki 側の ingestion コストを抑制する。

## Protobuf 定義

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

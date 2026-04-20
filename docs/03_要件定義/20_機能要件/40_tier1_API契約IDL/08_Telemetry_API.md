# 08. Telemetry API

メトリクス・トレース送信 API。OpenTelemetry に準拠し、メトリクスは Grafana Mimir、トレースは Grafana Tempo へ集約する。tier2/tier3 は OpenTelemetry SDK の計装を直接使うことも、本 API 経由で送信することもでき、後者は SDK を含められない軽量環境（CLI / レガシーランタイム）向けのフォールバック経路として位置付ける。

## 要件対応

- 要件ファイル: [../10_tier1_API要件/08_Telemetry_API.md](../10_tier1_API要件/08_Telemetry_API.md)
- 要件 ID: FR-T1-TELEMETRY-001〜004
- 共通型: [00_共通型定義.md](00_共通型定義.md)

## 設計のポイント

メトリクス種別 `MetricKind` は Counter（単調増加）、Gauge（任意値）、Histogram（分布）の 3 種に限定する。Summary は Mimir の推奨に従い除外し、必要があれば Histogram + recording rule で代替する。`Span` は W3C Trace Context の `trace_id` / `span_id` / `parent_span_id` を素通しで受け取り、tier1 側では属性の補強（`tenant_id` / `k1s0.tier` の自動付与）のみを行う。tier2 側で OTel SDK を使う場合は OTLP エンドポイントを直接利用し、本 API は経由しない（重複送信を避けるため）。

## Protobuf 定義

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

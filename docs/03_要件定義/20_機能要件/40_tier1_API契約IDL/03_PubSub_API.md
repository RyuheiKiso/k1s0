# 03. PubSub API

Kafka をバックエンドとする Publish / Subscribe API。At-least-once 配信、冪等性キーによる重複抑止、Dead Letter Queue への移送を提供し、tier2/tier3 が Kafka プロデューサ / コンシューマの詳細（パーティション戦略、ACK モード、コンシューマグループ管理）を直接扱わずにイベント駆動処理を実装できるようにする。

## 要件対応

- 要件ファイル: [../10_tier1_API要件/03_PubSub_API.md](../10_tier1_API要件/03_PubSub_API.md)
- 要件 ID: FR-T1-PUBSUB-001〜005
- 共通型: [00_共通型定義.md](00_共通型定義.md)

## 設計のポイント

トピック名は tier1 がテナント接頭辞を自動付与し、クライアントから見えるのは「テナント内トピック名」に限る。`idempotency_key` は TTL 24 時間の範囲で重複 Publish を抑止し、tier1 側で Valkey にキーを保持する想定。サブスクリプションは HTTP コールバック登録方式と gRPC ストリーム方式の両方を許容し、運用環境（Istio Ambient の外側からの呼出可否）によって選択する。`BulkPublish` は冪等性のため `idempotency_key` 必須とし、配列内の各エントリに個別の結果を返す（部分成功あり）。

## Protobuf 定義

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

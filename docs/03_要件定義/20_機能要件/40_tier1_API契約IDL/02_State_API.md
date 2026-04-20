# 02. State API

KV / Relational / Document の状態管理を抽象化する API。ETag による楽観的排他、TTL、バルク操作、トランザクションを提供し、tier2/tier3 が Valkey / PostgreSQL / ドキュメントストアなどの具体バックエンドを意識せずに状態を保存・取得できるようにする。

## 要件対応

- 要件ファイル: [../10_tier1_API要件/02_State_API.md](../10_tier1_API要件/02_State_API.md)
- 要件 ID: FR-T1-STATE-001〜005
- 共通型: [00_共通型定義.md](00_共通型定義.md)

## 設計のポイント

`store` フィールドで論理的なストア名（例: `valkey-default` / `postgres-tenant`）を指定し、物理的なバックエンド割当は運用側の設定に委ねる。テナント境界は tier1 がキーに自動付与するため、クライアントは「テナント内キー」のみを渡す。`Transact` はバックエンドごとに対応範囲が異なり、全 Store で使えるわけではない点を IDL コメントに明記しており、詳細設計で Store ごとのケイパビリティ行列を固定する。ETag は `Set` / `Delete` の `expected_etag` で期待値を渡し、不一致時は `CONFLICT` カテゴリ（`E-CONFLICT-STATE-002`）で返す。

## Protobuf 定義

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

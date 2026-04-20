# 01. Service Invoke API

サービス間の RPC を tier1 ファサード経由で仲介する API。サービス名解決、ロードバランシング、mTLS、リトライ、ヘッダ伝搬を隠蔽し、tier2/tier3 のアプリケーションコードから「相手サービスの接続情報」「再試行戦略」「トレース ID の伝搬」といったインフラ関心事を追い出す。

## 要件対応

- 要件ファイル: [../10_tier1_API要件/01_Service_Invoke_API.md](../10_tier1_API要件/01_Service_Invoke_API.md)
- 要件 ID: FR-T1-INVOKE-001〜005
- 共通型: [00_共通型定義.md](00_共通型定義.md)

## 設計のポイント

`Invoke` RPC はサービス間の単発呼出を担い、`data` は `bytes` で透過的に伝搬する。エンコーディングは `content_type` で識別し、JSON / gRPC / Protobuf の混在に耐える。ストリーミング応答（大容量ファイルや段階出力）には `InvokeStream` を用意し、unary → stream の置換は破壊的変更となるため別 RPC として共存させる方針を取る。`timeout_ms` の省略時の既定値 5000ms は要件定義段階の合意値であり、詳細設計で SLO 配分と突き合わせて最終確定する。

## Protobuf 定義

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

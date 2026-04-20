# 05. Binding API

外部 HTTP / SMTP / S3 / ファイルストレージ等との入出力連携を抽象化する API。入力バインディング（外部 → tier1 → tier2/tier3）と出力バインディング（tier2/tier3 → tier1 → 外部）の両方向を提供するが、要件定義段階では tier1 主導の出力バインディング（`Invoke`）のみ IDL 化し、入力バインディングは運用設定に基づく PubSub トピックへの注入として実現する。

## 要件対応

- 要件ファイル: [../10_tier1_API要件/05_Binding_API.md](../10_tier1_API要件/05_Binding_API.md)
- 要件 ID: FR-T1-BINDING-001〜004
- 共通型: [00_共通型定義.md](00_共通型定義.md)

## 設計のポイント

バインディング設定（接続先 URL、認証情報、許可される操作種別）は運用側で事前登録し、クライアントは `name` で論理名を指定する。`operation` はバインディング型ごとに異なる（HTTP: `post`/`get` / SMTP: `send` / S3: `create`/`get`/`list`/`delete`）ため、IDL では `string` のまま保持し、有効値は詳細設計で Binding 型ごとに enum 化する。データは `bytes` で透過的に渡し、`metadata` に宛先の詳細（HTTP ヘッダ、SMTP の To/Cc、S3 のキー名）を載せる。秘密情報は Binding 設定側で保持し、リクエストに含めてはならない（含まれていた場合は `InvalidArgument` で拒否）。

## Protobuf 定義

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

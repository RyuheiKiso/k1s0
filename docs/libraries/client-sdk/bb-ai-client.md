# bb-ai-client 設計

AI Gateway サーバーへの HTTP クライアントライブラリ。4言語対応。

## 概要

| 項目 | 内容 |
|------|------|
| ライブラリ名 | `bb-ai-client` |
| 対応言語 | Go / Rust / TypeScript / Dart |
| 主要利用者 | `ai-agent-server`（Rust）、各クライアントアプリ（TS/Dart） |

---

## 実装パス

| 言語 | パッケージ名 | パス |
|------|------------|------|
| Go | `bbaiclient` | `regions/system/library/go/bb-ai-client/` |
| Rust | `k1s0-bb-ai-client` | `regions/system/library/rust/bb-ai-client/` |
| TypeScript | `@k1s0/bb-ai-client` | `regions/system/library/typescript/bb-ai-client/` |
| Dart | `k1s0_bb_ai_client` | `regions/system/library/dart/bb-ai-client/` |

---

## 設計方針

- `AiClient` インターフェース/トレイト/抽象クラスにより実装を差し替え可能にする
- `HttpAiClient` が AI Gateway の REST エンドポイントを呼び出す
- `InMemoryAiClient` がユニットテスト用のモック実装として機能する
- 型定義は Go の `types.go` を正とし、4言語で整合させる

---

## エンドポイント

| メソッド | エンドポイント | 説明 |
|---------|--------------|------|
| POST | `/v1/complete` | チャット補完 |
| POST | `/v1/embed` | テキスト埋め込み |
| GET | `/v1/models` | モデル一覧取得 |

---

## 型定義

### 共通型（4言語で整合）

| 型 | フィールド | 説明 |
|----|-----------|------|
| `ChatMessage` | role, content | role は "user" / "assistant" / "system" |
| `CompleteRequest` | model, messages, max_tokens?, temperature?, stream? | チャット補完リクエスト |
| `Usage` | input_tokens, output_tokens | トークン使用量 |
| `CompleteResponse` | id, model, content, usage | チャット補完レスポンス |
| `EmbedRequest` | model, texts | 埋め込みリクエスト |
| `EmbedResponse` | model, embeddings | 埋め込みレスポンス |
| `ModelInfo` | id, name, description | モデル基本情報 |

> **注意**: TypeScript/Dart では JSON snake_case（`input_tokens`）と camelCase（`inputTokens`）の変換を内部で行う。

---

## モジュール構成

### Go

```
go/bb-ai-client/
  types.go      — 型定義
  client.go     — AiClient インターフェース
  http_client.go — HttpClient（net/http 実装）
  mock_client.go — MockClient（テスト用）
  error.go      — エラー型
```

### Rust

```
rust/bb-ai-client/src/
  lib.rs        — エントリポイント・再エクスポート
  types.rs      — リクエスト/レスポンス型定義
  traits.rs     — AiClient トレイト（mock feature で mockall 対応）
  client.rs     — HttpAiClient（reqwest 実装）
  memory.rs     — InMemoryAiClient（テスト用）
tests/
  bb_ai_client_test.rs — 統合テスト
```

### TypeScript

```
typescript/bb-ai-client/src/
  index.ts      — 全型・クラスを一括エクスポート
__tests__/
  bb-ai-client.test.ts — vitest テスト
```

### Dart

```
dart/bb-ai-client/lib/
  bb_ai_client.dart    — ライブラリエントリポイント
  src/types.dart       — 型定義
  src/client.dart      — AiClient 抽象クラス
  src/http_client.dart — HttpAiClient（http パッケージ）
  src/memory_client.dart — InMemoryAiClient（テスト用）
test/
  bb_ai_client_test.dart — dart test
```

---

## 使用例

### Go

```go
import bbaiclient "github.com/k1s0/system/library/go/bb-ai-client"

client := bbaiclient.NewHTTPClient(bbaiclient.HTTPClientConfig{
    BaseURL: "http://ai-gateway:8080",
    APIKey:  "sk-xxx",
})

resp, err := client.Complete(ctx, bbaiclient.CompleteRequest{
    Model: "claude-3-5-sonnet",
    Messages: []bbaiclient.ChatMessage{
        {Role: "user", Content: "Goとは？"},
    },
    MaxTokens: 512,
})
fmt.Println(resp.Content)
```

### Rust

```rust
use k1s0_bb_ai_client::{HttpAiClient, AiClient, CompleteRequest, ChatMessage};

let client = HttpAiClient::new(
    "http://ai-gateway:8080".to_string(),
    "sk-xxx".to_string(),
);

let req = CompleteRequest {
    model: "claude-3-5-sonnet".to_string(),
    messages: vec![ChatMessage {
        role: "user".to_string(),
        content: "Rustとは？".to_string(),
    }],
    max_tokens: Some(512),
    temperature: None,
    stream: None,
};
let resp = client.complete(&req).await?;
println!("{}", resp.content);
```

### TypeScript

```typescript
import { HttpAiClient } from '@k1s0/bb-ai-client';

const client = new HttpAiClient({
  baseUrl: 'http://ai-gateway:8080',
  apiKey: 'sk-xxx',
});

const resp = await client.complete({
  model: 'claude-3-5-sonnet',
  messages: [{ role: 'user', content: 'TypeScriptとは？' }],
  maxTokens: 512,
});
console.log(resp.content);
```

### Dart

```dart
import 'package:k1s0_bb_ai_client/bb_ai_client.dart';

final client = HttpAiClient(
  baseUrl: 'http://ai-gateway:8080',
  apiKey: 'sk-xxx',
);

final resp = await client.complete(CompleteRequest(
  model: 'claude-3-5-sonnet',
  messages: [const ChatMessage(role: 'user', content: 'Dartとは？')],
  maxTokens: 512,
));
print(resp.content);
```

---

## テスト用 InMemoryAiClient

### Go

```go
mockClient := bbaiclient.NewMockClient()
// MockClient は gomock ベースの自動生成モック
```

### Rust

```rust
use k1s0_bb_ai_client::{InMemoryAiClient, CompleteResponse, Usage};

// レスポンスキューを事前設定する方式
let client = InMemoryAiClient::new(
    vec![CompleteResponse {
        id: "test-id".to_string(),
        model: "claude-3".to_string(),
        content: "テスト応答".to_string(),
        usage: Usage { input_tokens: 10, output_tokens: 5 },
    }],
    vec![], // embed_responses
);
```

### TypeScript

```typescript
import { InMemoryAiClient } from '@k1s0/bb-ai-client';

// カスタムハンドラを注入する方式
const client = new InMemoryAiClient({
  complete: async (req) => ({
    id: 'test-id',
    model: req.model,
    content: 'テスト応答',
    usage: { inputTokens: 10, outputTokens: 5 },
  }),
});
```

### Dart

```dart
import 'package:k1s0_bb_ai_client/bb_ai_client.dart';

// カスタムハンドラを注入する方式
final client = InMemoryAiClient(
  complete: (req) async => CompleteResponse(
    id: 'test-id',
    model: req.model,
    content: 'テスト応答',
    usage: const Usage(inputTokens: 10, outputTokens: 5),
  ),
);
```

---

## 依存関係

### Rust Cargo.toml

```toml
[dependencies]
k1s0-bb-ai-client = { path = "../../../library/rust/bb-ai-client" }

# テスト用モックを有効化する場合
[dev-dependencies]
k1s0-bb-ai-client = { path = "../../../library/rust/bb-ai-client", features = ["mock"] }
```

### TypeScript package.json

```json
{
  "dependencies": {
    "@k1s0/bb-ai-client": "workspace:*"
  }
}
```

### Dart pubspec.yaml

```yaml
dependencies:
  k1s0_bb_ai_client:
    path: ../../../library/dart/bb-ai-client
```

---

## 関連ドキュメント

- [ai-gateway サーバー設計](../../servers/ai-gateway/server.md)
- [ai-agent サーバー設計](../../servers/ai-agent/server.md)

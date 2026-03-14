# system-library-bb-ai-client 設計

AI Gatewayサーバーへの HTTP クライアントライブラリ。Rust サーバー専用。

## 概要

| 項目 | 内容 |
|------|------|
| パッケージ名 | `k1s0-bb-ai-client` |
| 言語 | Rust（Rust サーバー向け内部ライブラリ） |
| 実装パス | `regions/system/library/rust/bb-ai-client/` |
| 主要利用者 | `ai-agent-server` |

---

## 設計方針

- `AiClient` トレイトにより実装を差し替え可能にする（テスト・本番を分離）
- `HttpAiClient` が ai-gateway の OpenAI互換エンドポイントを呼び出す
- `InMemoryAiClient` がユニットテスト用のモックとして機能する
- `mock` feature フラグで `mockall` による自動モック生成を有効化

---

## モジュール構成

```
src/
  lib.rs        — エントリポイント・再エクスポート
  types.rs      — リクエスト/レスポンス型定義
  traits.rs     — AiClient trait
  client.rs     — HttpAiClient（reqwest実装）
  memory.rs     — InMemoryAiClient（テスト用）
```

---

## AiClient トレイト

```rust
#[async_trait::async_trait]
pub trait AiClient: Send + Sync {
    async fn complete(&self, req: &CompleteRequest) -> Result<CompleteResponse, AiClientError>;
    async fn embed(&self, req: &EmbedRequest) -> Result<EmbedResponse, AiClientError>;
    async fn list_models(&self) -> Result<Vec<AiModel>, AiClientError>;
}
```

---

## 主要型

| 型 | 説明 |
|----|------|
| `ChatMessage` | role（user/assistant/system）+ content |
| `CompleteRequest` | model, messages, max_tokens, temperature, stream |
| `CompleteResponse` | id, model, content, prompt_tokens, completion_tokens |
| `EmbedRequest` | model, inputs |
| `EmbedResponse` | model, embeddings（Vec<Vec<f32>>） |
| `AiModel` | id, name, provider, context_window, enabled |
| `AiClientError` | HttpError / JsonError / Unavailable |

---

## 使用例

```rust
use k1s0_bb_ai_client::{HttpAiClient, AiClient, CompleteRequest, ChatMessage};

// クライアント初期化
let client = HttpAiClient::new(
    "http://ai-gateway:8080".to_string(),
    "Bearer sk-xxx".to_string(),
);

// 補完リクエスト
let req = CompleteRequest {
    model: "gpt-4o-mini".to_string(),
    messages: vec![ChatMessage {
        role: "user".to_string(),
        content: "Rustとは？".to_string(),
    }],
    max_tokens: Some(512),
    temperature: Some(0.7),
    stream: false,
};
let resp = client.complete(&req).await?;
println!("{}", resp.content);
```

---

## テスト用 InMemoryAiClient

```rust
use k1s0_bb_ai_client::{InMemoryAiClient, CompleteResponse};

let mut client = InMemoryAiClient::new();
client.push_complete_response(CompleteResponse {
    id: "test-id".to_string(),
    model: "gpt-4o-mini".to_string(),
    content: "テスト応答".to_string(),
    prompt_tokens: 10,
    completion_tokens: 5,
});

// テストコードで client を AiClient として注入
```

---

## Cargo.toml 依存

```toml
[dependencies]
k1s0-bb-ai-client = { path = "../../../library/rust/bb-ai-client" }

# テスト用モックを有効化する場合
k1s0-bb-ai-client = { path = "../../../library/rust/bb-ai-client", features = ["mock"] }
```

---

## 関連ドキュメント

- [ai-gateway サーバー設計](../../servers/ai-gateway/server.md)
- [ai-agent サーバー設計](../../servers/ai-agent/server.md)

# system-ai-gateway-server 設計

LLMプロバイダーへのリクエストルーティング・ガードレール・使用量トラッキングを提供するAI Gatewayサーバー。

## 概要

### RBAC対応表

| ロール名 | リソース/アクション |
|---------|-----------------|
| sys_auditor 以上 | ai/read（使用量参照） |
| sys_operator 以上 | ai/write（補完・エンベディング実行） |
| sys_admin のみ | ai/admin（モデル設定変更） |

system tier のAI Gatewayサーバーは以下の機能を提供する。

| 機能 | 説明 |
| --- | --- |
| テキスト補完 | OpenAI互換APIへのチャット補完リクエストをルーティング |
| ストリーミング補完 | Server-Sent Events / gRPC ストリーミングによる逐次レスポンス |
| エンベディング | テキストのベクトル変換（RAGパイプライン向け） |
| モデル一覧 | 利用可能なLLMモデルの一覧取得 |
| 使用量トラッキング | テナント別トークン使用量・コスト集計 |
| ガードレール | Prompt Injection検出・有害コンテンツフィルタリング |
| ルーティング戦略 | コスト最適化・レイテンシ最適化の動的モデル選択 |

### 技術スタック

> 共通技術スタックは [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md#共通技術スタック) を参照。

追加依存:
- `redis`: レスポンスキャッシュ（オプション）
- `reqwest`: OpenAI互換HTTP クライアント

### 配置パス

配置: `regions/system/server/rust/ai-gateway/`

---

## 設計方針

[認証認可設計.md](../../architecture/auth/認証認可設計.md) の RBAC モデルに基づき、以下の方針で実装する。

| 項目 | 設計 |
| --- | --- |
| 実装言語 | Rust |
| LLMプロバイダー互換性 | OpenAI API互換（`/v1/chat/completions`, `/v1/embeddings`, `/v1/models`） |
| ルーティング戦略 | `cost`（コスト最小化）/ `latency`（レイテンシ最小化）を設定で切替 |
| ガードレール | 正規表現ベースのPrompt Injection検出（`ignore previous instructions` 等） |
| キャッシュ | Redis による補完レスポンスキャッシュ（TTL設定可能、オプション） |
| 使用量記録 | リクエスト完了後にprompt_tokens / completion_tokens / cost_usdをDBに非同期保存 |
| DB | PostgreSQL の `k1s0_system` DB（ai_models, usage_records, routing_rules テーブル） |
| 認証 | JWTによる認可 |
| ポート | 8120（REST）/ 50061（gRPC） |

---

## API 定義

### REST API エンドポイント

| メソッド | パス | 説明 | 認可 |
|--------|------|------|------|
| POST | /api/v1/complete | テキスト補完 | ai/write |
| POST | /api/v1/complete/stream | ストリーミング補完 | ai/write |
| POST | /api/v1/embed | エンベディング | ai/write |
| GET | /api/v1/models | モデル一覧 | ai/read |
| GET | /api/v1/usage | 使用量取得 | ai/read |
| GET | /healthz | ヘルスチェック | 不要 |
| GET | /readyz | レディネスチェック | 不要 |
| GET | /metrics | Prometheusメトリクス | 不要 |

#### POST /api/v1/complete

```json
// リクエスト
{
  "model": "gpt-4o-mini",
  "messages": [
    { "role": "user", "content": "Rustとは何ですか？" }
  ],
  "max_tokens": 1024,
  "temperature": 0.7,
  "strategy": "cost"
}

// レスポンス
{
  "id": "cmpl-abc123",
  "model": "gpt-4o-mini",
  "content": "Rustはシステムプログラミング言語で...",
  "prompt_tokens": 20,
  "completion_tokens": 150
}
```

#### GET /api/v1/usage

```
GET /api/v1/usage?tenant_id=t1&start_date=2026-03-01&end_date=2026-03-31
```

```json
{
  "tenant_id": "t1",
  "total_prompt_tokens": 50000,
  "total_completion_tokens": 120000,
  "total_cost_usd": 3.45
}
```

### gRPC API

Proto定義: `api/proto/k1s0/system/ai_gateway/v1/ai_gateway.proto`

| RPC | 説明 |
|-----|------|
| Complete | テキスト補完（Unary） |
| CompleteStream | ストリーミング補完（Server Stream） |
| Embed | エンベディング（Unary） |
| ListModels | モデル一覧（Unary） |
| GetUsage | 使用量取得（Unary） |

---

## アーキテクチャ

```
HTTP/gRPC
    ↓
Adapter Layer
  ├── REST Handler (axum)      ← /api/v1/*
  └── gRPC Handler (tonic)     ← AiGatewayService
    ↓
UseCase Layer
  ├── CompleteUseCase          ← ガードレール → ルーティング → LLM → 使用量記録
  ├── EmbedUseCase
  ├── ListModelsUseCase
  └── GetUsageUseCase
    ↓
Domain Layer
  ├── GuardrailService         ← Prompt Injection正規表現チェック
  ├── RoutingService           ← コスト/レイテンシ戦略でモデル選択
  └── Repository traits
    ↓
Infrastructure Layer
  ├── LlmClient               ← reqwest (OpenAI互換HTTP)
  ├── RedisCache              ← オプションキャッシュ
  └── PostgreSQL              ← モデル・使用量・ルーティングルール
```

---

## 関連ドキュメント

- [ai-gateway データベース設計](./database.md)
- [ai-agent サーバー設計](../ai-agent/server.md)
- [bb-ai-client ライブラリ設計](../../libraries/client-sdk/bb-ai-client.md)

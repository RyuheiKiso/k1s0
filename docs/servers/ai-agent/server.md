# system-ai-agent-server 設計

ReActループによるLLMエージェント実行・ツール呼び出し・ステップレビューを提供するAI Agentサーバー。

## 概要

### RBAC対応表

| ロール名 | リソース/アクション |
|---------|-----------------|
| sys_auditor 以上 | agents/read（実行履歴参照） |
| sys_operator 以上 | agents/write（エージェント実行・定義作成） |
| sys_admin のみ | agents/admin（エージェント定義の削除・強制停止） |

system tier のAI Agentサーバーは以下の機能を提供する。

| 機能 | 説明 |
| --- | --- |
| エージェント定義管理 | モデル・システムプロンプト・ツール一覧・最大ステップ数の定義CRUD |
| エージェント実行 | ReActループ（Thought→Action→Observation）によるマルチステップ推論 |
| ストリーミング実行 | gRPC Server Streamによる実行イベントの逐次配信 |
| ステップレビュー | Human-in-the-Loop：承認/拒否による実行制御 |
| 実行履歴管理 | エージェント実行履歴・ステップ詳細の参照 |
| ツールレジストリ | OpenAPI JSONスキーマからLLM向けツール定義への変換 |

### 技術スタック

> 共通技術スタックは [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md#共通技術スタック) を参照。

追加依存:
- `k1s0-bb-ai-client`: AI Gatewayへのリクエストクライアント

### 配置パス

配置: `regions/system/server/rust/ai-agent/`

### CI ステータス

> **実験系クレート**: ai-agent は stable CI ゲート（lint-rust / test-rust / build-rust）から除外されている。
> `check-ai-experimental` ジョブ（`continue-on-error: true`）で可視性を維持しつつ、CI 全体のグリーンには影響しない。
> bb-ai-client との型整合が安定次第、stable ゲートに復帰予定。

---

## 設計方針

| 項目 | 設計 |
| --- | --- |
| 実装言語 | Rust |
| 推論エンジン | ReActパターン（Reasoning + Acting）によるループ制御 |
| LLM連携 | ai-gateway-server（ポート 8080/Docker, 8120/ローカル）経由でアクセス |
| ツール定義 | OpenAPI 3.0 JSONスキーマをLLM Function Calling形式に変換 |
| 最大ステップ数 | エージェント定義の `max_steps` で制限（デフォルト10） |
| Human-in-the-Loop | `requires_review: true` ステップで実行を一時停止し承認待ち |
| DB | PostgreSQL の `k1s0_system` DB（agent_definitions, executions テーブル） |
| 認証 | JWTによる認可 |
| ポート | 8121（REST）/ 50062（gRPC） |

---

## ReAct ループ設計

```
入力: input
  ↓
[Thought] LLMに推論させる
  "ユーザーの質問はXで、Yツールを使うべき"
  ↓
[Action] ツール呼び出しをJSON解析
  { "tool": "search", "args": { "query": "..." } }
  ↓
[Observation] ツール実行結果をコンテキストに追加
  "検索結果: ..."
  ↓
max_stepsに達するか、Finishアクションが出るまで繰り返す
  ↓
出力: output
```

---

## API 定義

### REST API エンドポイント

| メソッド | パス | 説明 | 認可 |
|--------|------|------|------|
| POST | /api/v1/agents | エージェント定義作成 | agents/write |
| GET | /api/v1/agents | エージェント定義一覧 | agents/read |
| POST | /api/v1/agents/{id}/execute | エージェント実行 | agents/write |
| GET | /api/v1/executions | 実行履歴一覧 | agents/read |
| POST | /api/v1/executions/{id}/review | ステップレビュー | agents/write |
| GET | /healthz | ヘルスチェック | 不要 |
| GET | /readyz | レディネスチェック | 不要 |
| GET | /metrics | Prometheusメトリクス | 不要 |

#### POST /api/v1/agents/{id}/execute

```json
// リクエスト
{
  "input": "現在の為替レートを調べてUSD→JPYに換算してください",
  "session_id": "sess-abc",
  "tenant_id": "tenant-1",
  "context": { "user_locale": "ja-JP" }
}

// レスポンス
{
  "execution_id": "exec-xyz",
  "status": "completed",
  "output": "現在のレートは150.5円/USDです。1000USDは150,500円です。",
  "steps": [
    { "index": 0, "step_type": "thought", "output": "為替レート検索ツールを使う" },
    { "index": 1, "step_type": "action", "tool_name": "exchange_rate", "input": "{\"from\":\"USD\",\"to\":\"JPY\"}" },
    { "index": 2, "step_type": "observation", "output": "150.5" }
  ]
}
```

#### POST /api/v1/executions/{id}/review

```json
// リクエスト
{
  "step_index": 2,
  "approved": true,
  "feedback": "続けてください"
}

// レスポンス
{
  "execution_id": "exec-xyz",
  "resumed": true
}
```

### gRPC API

Proto定義: `api/proto/k1s0/system/ai_agent/v1/ai_agent.proto`

| RPC | 説明 |
|-----|------|
| Execute | エージェント実行（Unary） |
| ExecuteStream | ストリーミング実行（Server Stream） |
| CancelExecution | 実行キャンセル（Unary） |
| ReviewStep | ステップレビュー（Unary） |

---

## アーキテクチャ

```
HTTP/gRPC
    ↓
Adapter Layer
  ├── REST Handler (axum)
  └── gRPC Handler (tonic)
    ↓
UseCase Layer
  ├── CreateAgentUseCase
  ├── ExecuteAgentUseCase      ← ReActEngineを呼び出す
  ├── ReviewStepUseCase
  └── ListExecutionsUseCase
    ↓
Domain Layer
  ├── ReActEngine              ← Thought→Action→Observationループ
  ├── ToolRegistry             ← OpenAPI→LLM Function Calling変換
  └── Repository traits
    ↓
Infrastructure Layer
  ├── AiGatewayClient         ← bb-ai-client経由でai-gateway呼び出し
  └── PostgreSQL              ← エージェント定義・実行履歴
```

---

## 関連ドキュメント

- [ai-agent データベース設計](./database.md)
- [ai-gateway サーバー設計](../ai-gateway/server.md)
- [bb-ai-client ライブラリ設計](../../libraries/client-sdk/bb-ai-client.md)

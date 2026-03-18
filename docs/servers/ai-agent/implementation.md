# system-ai-agent-server 実装設計

> **注記**: 本ドキュメントは ai-agent-server の実装仕様を含む。共通パターンは [Rust共通実装.md](../_common/Rust共通実装.md) を参照。

system-ai-agent-server（AI エージェントサーバー）の Rust 実装仕様。概要・API 定義・アーキテクチャは [server.md](server.md) を参照。

---

## アーキテクチャ概要

Clean Architecture に基づく 4 層構成を採用する。

| レイヤー | 責務 | 依存方向 |
|---------|------|---------|
| domain | エンティティ・リポジトリトレイト・ドメインサービス | なし（最内層） |
| usecase | ビジネスロジック（エージェント作成・実行・レビュー） | domain のみ |
| adapter | REST/gRPC ハンドラー・ミドルウェア・リポジトリ実装 | usecase, domain |
| infrastructure | 設定・DB接続・外部クライアント・起動シーケンス | 全レイヤー |

---

## Rust 実装 (regions/system/server/rust/ai-agent/)

### ディレクトリ構成

```
regions/system/server/rust/ai-agent/
├── src/
│   ├── main.rs                              # エントリポイント（startup::run() 委譲）
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── entity/
│   │   │   ├── mod.rs
│   │   │   ├── agent_definition.rs          # AgentDefinition エンティティ（モデル・プロンプト・ツール・max_steps）
│   │   │   ├── execution.rs                 # Execution エンティティ（実行状態・ステップ履歴）
│   │   │   └── tool.rs                      # Tool エンティティ（OpenAPI JSONスキーマ定義）
│   │   ├── repository/
│   │   │   ├── mod.rs
│   │   │   ├── agent_repository.rs          # AgentRepository トレイト
│   │   │   └── execution_repository.rs      # ExecutionRepository トレイト
│   │   └── service/
│   │       ├── mod.rs
│   │       ├── react_engine.rs              # ReActループ制御（Thought→Action→Observation）
│   │       └── tool_registry.rs             # ツールレジストリ（OpenAPI→LLM Function Calling変換）
│   ├── usecase/
│   │   ├── mod.rs
│   │   ├── create_agent.rs                  # エージェント定義作成
│   │   ├── execute_agent.rs                 # エージェント実行（ReActループ）
│   │   ├── list_executions.rs               # 実行履歴一覧取得
│   │   └── review_step.rs                   # ステップレビュー（Human-in-the-Loop）
│   ├── adapter/
│   │   ├── mod.rs
│   │   ├── handler/
│   │   │   ├── mod.rs
│   │   │   ├── agent_handler.rs             # axum REST ハンドラー
│   │   │   └── health.rs                    # ヘルスチェック
│   │   ├── grpc/
│   │   │   ├── mod.rs
│   │   │   ├── agent_grpc.rs                # gRPC サービス実装
│   │   │   └── tonic_service.rs             # tonic サービスラッパー
│   │   ├── middleware/
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs                      # JWT 認証ミドルウェア
│   │   │   ├── grpc_auth.rs                 # gRPC 認証ミドルウェア
│   │   │   └── rbac.rs                      # RBAC ミドルウェア
│   │   └── repository/
│   │       ├── mod.rs
│   │       ├── agent_postgres.rs            # AgentRepository PostgreSQL 実装
│   │       └── execution_postgres.rs        # ExecutionRepository PostgreSQL 実装
│   ├── infrastructure/
│   │   ├── mod.rs
│   │   ├── config.rs                        # 設定構造体・読み込み
│   │   ├── database.rs                      # DB 接続プール
│   │   ├── ai_gateway_client.rs             # AI Gateway への HTTP クライアント
│   │   ├── in_memory.rs                     # InMemory リポジトリ（dev/test 用）
│   │   └── startup.rs                       # 起動シーケンス・DI
│   └── proto/                               # tonic-build 生成コード
├── config/
│   └── config.yaml
├── build.rs
├── Cargo.toml
└── Dockerfile
```

### 主要コンポーネント

#### ドメインサービス

- **ReActEngine**: ReAct パターン（Reasoning + Acting）によるマルチステップ推論ループを制御する。Thought→Action→Observation のサイクルを `max_steps` まで繰り返し、Finish アクションまたは上限到達で終了する
- **ToolRegistry**: OpenAPI 3.0 JSON スキーマを LLM Function Calling 形式に変換し、エージェントが利用可能なツール一覧を管理する

#### ユースケース

| ユースケース | 責務 |
|------------|------|
| `CreateAgentUseCase` | エージェント定義（モデル・プロンプト・ツール・max_steps）の作成 |
| `ExecuteAgentUseCase` | ReAct ループによるエージェント実行。AI Gateway 経由で LLM を呼び出す |
| `ListExecutionsUseCase` | エージェント実行履歴の一覧取得 |
| `ReviewStepUseCase` | Human-in-the-Loop: `requires_review: true` ステップの承認/拒否 |

#### 外部連携

- **AI Gateway Client** (`infrastructure/ai_gateway_client.rs`): ai-gateway-server（ポート 8120）経由で LLM にリクエストを送信する HTTP クライアント

### エラーハンドリング方針

- ユースケース層で `anyhow::Result` を返却し、adapter 層で HTTP/gRPC ステータスコードに変換する
- エラーコードプレフィックス: `SYS_AI_AGENT_`
- AI Gateway 呼び出し失敗時はリトライせず、実行ステップにエラーを記録して中断する

### テスト方針

| テスト種別 | 対象 | 方針 |
|-----------|------|------|
| 単体テスト | ドメインサービス・ユースケース | mockall によるリポジトリモック |
| 統合テスト | REST/gRPC ハンドラー | axum-test / tonic テストクライアント |
| InMemory テスト | リポジトリ | `in_memory.rs` による DB 不要テスト |

> **CI 注記**: ai-agent は実験系クレートとして stable CI ゲートから除外されている。`check-ai-experimental` ジョブで可視性を維持する。

---

## 関連ドキュメント

- [server.md](server.md) -- 概要・API 定義・ReAct ループ設計
- [Rust共通実装.md](../_common/Rust共通実装.md) -- 共通起動シーケンス・Cargo 依存

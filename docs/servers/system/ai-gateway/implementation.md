# system-ai-gateway-server 実装設計

> **注記**: 本ドキュメントは ai-gateway-server の実装仕様を含む。共通パターンは [Rust共通実装.md](../../_common/Rust共通実装.md) を参照。

system-ai-gateway-server（AI ゲートウェイサーバー）の Rust 実装仕様。概要・API 定義・アーキテクチャは [server.md](server.md) を参照。

---

## アーキテクチャ概要

Clean Architecture に基づく 4 層構成を採用する。

| レイヤー | 責務 | 依存方向 |
|---------|------|---------|
| domain | エンティティ・リポジトリトレイト・ドメインサービス | なし（最内層） |
| usecase | ビジネスロジック（補完・エンベディング・使用量管理） | domain のみ |
| adapter | REST/gRPC ハンドラー・ミドルウェア・リポジトリ実装 | usecase, domain |
| infrastructure | 設定・DB接続・LLMクライアント・Redisキャッシュ・起動シーケンス | 全レイヤー |

---

## Rust 実装 (regions/system/server/rust/ai-gateway/)

### ディレクトリ構成

```
regions/system/server/rust/ai-gateway/
├── src/
│   ├── main.rs                              # エントリポイント（startup::run() 委譲）
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── entity/
│   │   │   ├── mod.rs
│   │   │   ├── model.rs                     # Model エンティティ（LLMモデル定義）
│   │   │   ├── provider.rs                  # Provider エンティティ（プロバイダー定義）
│   │   │   ├── routing_rule.rs              # RoutingRule エンティティ（ルーティング戦略）
│   │   │   └── usage_record.rs              # UsageRecord エンティティ（使用量記録）
│   │   ├── repository/
│   │   │   ├── mod.rs
│   │   │   ├── model_repository.rs          # ModelRepository トレイト
│   │   │   ├── routing_rule_repository.rs   # RoutingRuleRepository トレイト
│   │   │   └── usage_repository.rs          # UsageRepository トレイト
│   │   └── service/
│   │       ├── mod.rs
│   │       ├── guardrail_service.rs         # ガードレール（Prompt Injection検出・有害コンテンツフィルタ）
│   │       └── routing_service.rs           # ルーティング戦略（cost/latency最適化）
│   ├── usecase/
│   │   ├── mod.rs
│   │   ├── complete.rs                      # テキスト補完
│   │   ├── embed.rs                         # エンベディング
│   │   ├── get_usage.rs                     # 使用量取得
│   │   └── list_models.rs                   # モデル一覧取得
│   ├── adapter/
│   │   ├── mod.rs
│   │   ├── handler/
│   │   │   ├── mod.rs
│   │   │   ├── ai_handler.rs               # axum REST ハンドラー
│   │   │   └── health.rs                    # ヘルスチェック
│   │   ├── grpc/
│   │   │   ├── mod.rs
│   │   │   ├── ai_grpc.rs                  # gRPC サービス実装
│   │   │   └── tonic_service.rs             # tonic サービスラッパー
│   │   ├── middleware/
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs                      # JWT 認証ミドルウェア
│   │   │   └── grpc_auth.rs                 # gRPC 認証ミドルウェア
│   │   └── repository/
│   │       ├── mod.rs
│   │       ├── model_postgres.rs            # ModelRepository PostgreSQL 実装
│   │       ├── routing_rule_postgres.rs     # RoutingRuleRepository PostgreSQL 実装
│   │       └── usage_postgres.rs            # UsageRepository PostgreSQL 実装
│   ├── infrastructure/
│   │   ├── mod.rs
│   │   ├── config.rs                        # 設定構造体・読み込み
│   │   ├── database.rs                      # DB 接続プール
│   │   ├── llm_client.rs                    # OpenAI 互換 HTTP クライアント
│   │   ├── redis_cache.rs                   # Redis レスポンスキャッシュ
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

- **GuardrailService**: 正規表現ベースの Prompt Injection 検出（`ignore previous instructions` 等）と有害コンテンツフィルタリングを行う
- **RoutingService**: `cost`（コスト最小化）/ `latency`（レイテンシ最小化）の 2 戦略による動的モデル選択を行う

#### ユースケース

| ユースケース | 責務 |
|------------|------|
| `CompleteUseCase` | OpenAI 互換 API へのチャット補完リクエストのルーティング |
| `EmbedUseCase` | テキストのベクトル変換（RAG パイプライン向け） |
| `GetUsageUseCase` | テナント別トークン使用量・コスト集計の取得 |
| `ListModelsUseCase` | 利用可能な LLM モデルの一覧取得 |

#### 外部連携

- **LLM Client** (`infrastructure/llm_client.rs`): OpenAI 互換 HTTP クライアント。reqwest で外部 LLM プロバイダーにリクエストを送信する
- **Redis Cache** (`infrastructure/redis_cache.rs`): 補完レスポンスの TTL 付きキャッシュ（オプション）

### エラーハンドリング方針

- ユースケース層で `anyhow::Result` を返却し、adapter 層で HTTP/gRPC ステータスコードに変換する
- エラーコードプレフィックス: `SYS_AI_`
- LLM プロバイダー障害時はルーティングサービスが代替プロバイダーにフォールバックする

### テスト方針

| テスト種別 | 対象 | 方針 |
|-----------|------|------|
| 単体テスト | ガードレール・ルーティングサービス | mockall によるリポジトリモック |
| 統合テスト | REST/gRPC ハンドラー | axum-test / tonic テストクライアント |
| InMemory テスト | リポジトリ | DB 不要テスト |

> **CI 注記**: ai-gateway は実験系クレートとして stable CI ゲートから除外されている。`check-ai-experimental` ジョブで可視性を維持する。

---

## 関連ドキュメント

- [server.md](server.md) -- 概要・API 定義・ルーティング設計
- [Rust共通実装.md](../../_common/Rust共通実装.md) -- 共通起動シーケンス・Cargo 依存

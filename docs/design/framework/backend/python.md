# Backend Framework（Python）

k1s0 Backend Framework（Python）は、FastAPI ベースのマイクロサービス開発のための共通 Python パッケージ群を提供します。Rust 版・Go 版・C# 版と同等の機能を Python で実装しています。

## パッケージ一覧

```
framework/backend/python/
├── pyproject.toml              # ワークスペース管理（uv）
├── packages/
│   ├── k1s0-error/            # エラー表現の統一（RFC 7807）
│   ├── k1s0-config/           # YAML 設定読み込み
│   ├── k1s0-validation/       # 入力バリデーション（Pydantic v2）
│   ├── k1s0-observability/    # OpenTelemetry 統合
│   ├── k1s0-grpc-server/      # gRPC サーバー共通基盤
│   ├── k1s0-grpc-client/      # gRPC クライアント共通
│   ├── k1s0-health/           # ヘルスチェック（liveness/readiness）
│   ├── k1s0-db/               # DB 接続（SQLAlchemy 2.0 + asyncpg）
│   ├── k1s0-domain-event/     # ドメインイベント・Outbox パターン
│   ├── k1s0-resilience/       # サーキットブレーカー・リトライ・タイムアウト
│   ├── k1s0-rate-limit/       # レート制限（トークンバケット・スライディングウィンドウ）
│   ├── k1s0-cache/            # Redis キャッシュ・キャッシュパターン
│   ├── k1s0-consensus/        # リーダー選出・分散ロック・Saga オーケストレーション
│   └── k1s0-auth/             # JWT/OIDC 認証・ポリシーエンジン
└── tests/
```

## Tier 構成

### Tier 1（依存なし）

| パッケージ | 説明 | 主要依存 |
|-----------|------|---------|
| k1s0-error | 統一エラーハンドリング。K1s0Error 基底クラス、ErrorCode dataclass、RFC 7807 ProblemDetail 生成 | - |
| k1s0-config | YAML 設定管理。`load_config()` で `--env`/`--secrets-dir` 対応 | PyYAML |
| k1s0-validation | 入力検証。Pydantic v2 BaseModel ベース、カスタムバリデータ | pydantic |

### Tier 2（Tier 1 のみ依存可）

| パッケージ | 説明 | 主要依存 |
|-----------|------|---------|
| k1s0-observability | OpenTelemetry 統合。`setup_observability()` で tracing/metrics/logging を一括初期化 | opentelemetry-sdk, opentelemetry-exporter-otlp |
| k1s0-grpc-server | gRPC サーバー共通基盤。インターセプター、リフレクション、ヘルスサービス | grpcio, grpcio-reflection |
| k1s0-grpc-client | gRPC クライアント共通。チャネル管理、リトライ、デッドライン | grpcio |
| k1s0-health | ヘルスチェック。FastAPI ルーター提供、liveness/readiness プローブ | fastapi |
| k1s0-db | DB 接続管理。SQLAlchemy 2.0 async セッション、マイグレーション（Alembic） | sqlalchemy, asyncpg, alembic |
| k1s0-domain-event | ドメインイベント発行/購読。InMemoryEventBus、Outbox パターン（optional: asyncpg, sqlalchemy） | pydantic |
| k1s0-resilience | 耐障害性パターン。CircuitBreaker、RetryExecutor、TimeoutGuard、ConcurrencyLimiter、Bulkhead | - |
| k1s0-rate-limit | レート制限。TokenBucketLimiter、SlidingWindowLimiter | - |
| k1s0-cache | Redis キャッシュ。CacheClient、CacheAside/WriteThrough/WriteBehind パターン | pydantic, redis（optional） |
| k1s0-consensus | リーダー選出、分散ロック、Saga オーケストレーション | k1s0-db, k1s0-domain-event, k1s0-observability |

### Tier 3（Tier 1/2 依存可）

| パッケージ | 説明 | 主要依存 |
|-----------|------|---------|
| k1s0-auth | JWT/OIDC 認証。JwtVerifier（JWKS 自動取得）、PolicyEvaluator、FastAPI/gRPC ミドルウェア | PyJWT, cryptography, httpx |

## 技術選定

| 機能 | 技術 | 理由 |
|------|------|------|
| Web フレームワーク | FastAPI 0.115+ | 型ヒント・Pydantic 統合、非同期対応、高パフォーマンス |
| ASGI サーバー | Uvicorn | FastAPI 推奨、非同期対応 |
| バリデーション | Pydantic v2 | FastAPI ネイティブ統合、高速 |
| ORM | SQLAlchemy 2.0 | 非同期対応、型安全、成熟したエコシステム |
| DB ドライバ | asyncpg | PostgreSQL 非同期ドライバ、高パフォーマンス |
| gRPC | grpcio + grpcio-tools | 公式 Python gRPC 実装 |
| テスト | pytest + pytest-asyncio + httpx | Python 標準テストフレームワーク |
| ログ/トレース | OpenTelemetry Python SDK | 他言語と統一 |
| パッケージ管理 | uv | Rust 製高速パッケージマネージャ |
| リント/フォーマット | Ruff | Rust 製高速リンター（flake8/black/isort 統合） |
| 型チェック | mypy | Python 標準型チェッカー |

## Tier 依存ルール

Rust/Go/C# と同様:
- **Tier 1**: フレームワーク内依存なし
- **Tier 2**: Tier 1 のみ依存可能
- **Tier 3**: Tier 1 / Tier 2 に依存可能

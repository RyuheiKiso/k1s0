# Framework Backend

バックエンド共通部品（crate/ライブラリ）および共通マイクロサービス。

## ディレクトリ構成

```
backend/
├── rust/
│   ├── crates/       # 共通 crate 群
│   └── services/     # 共通マイクロサービス
├── go/
│   └── pkg/          # 共通パッケージ
├── csharp/           # NuGet パッケージ
├── python/           # Python パッケージ（uv）
└── kotlin/           # Kotlin パッケージ（Gradle）
```

## Rust

### 共通 crate（Tier 1-3）

| crate | 説明 | Tier | ステータス |
|-------|------|:----:|:--------:|
| `k1s0-error` | 統一エラーハンドリング | 1 | ✅ |
| `k1s0-config` | 設定ファイル管理 | 1 | ✅ |
| `k1s0-validation` | 入力バリデーション | 1 | ✅ |
| `k1s0-observability` | ロギング/トレーシング/メトリクス | 2 | ✅ |
| `k1s0-grpc-server` | gRPC サーバー基盤 | 2 | ✅ |
| `k1s0-grpc-client` | gRPC クライアント | 2 | ✅ |
| `k1s0-resilience` | リトライ/サーキットブレーカー | 2 | ✅ |
| `k1s0-health` | ヘルスチェック | 2 | ✅ |
| `k1s0-db` | DB 接続/トランザクション | 2 | ✅ |
| `k1s0-cache` | Redis キャッシュ | 2 | ✅ |
| `k1s0-domain-event` | ドメインイベント発行/購読/Outbox | 2 | ✅ |
| `k1s0-auth` | 認証/認可 | 3 | ✅ |

### 共通マイクロサービス

| サービス | 説明 | ステータス |
|----------|------|:--------:|
| `auth-service` | 認証/認可サービス | ✅ |
| `config-service` | 設定管理サービス | ✅ |
| `endpoint-service` | エンドポイント管理サービス | ✅ |

## Go

### 共通パッケージ（Tier 1-3）

| module | 説明 | Tier | ステータス |
|--------|------|:----:|:--------:|
| `k1s0-error` | 統一エラーハンドリング | 1 | ✅ |
| `k1s0-config` | 設定ファイル管理 | 1 | ✅ |
| `k1s0-validation` | 入力バリデーション | 1 | ✅ |
| `k1s0-observability` | ロギング/トレーシング/メトリクス | 2 | ✅ |
| `k1s0-grpc-server` | gRPC サーバー基盤 | 2 | ✅ |
| `k1s0-grpc-client` | gRPC クライアント | 2 | ✅ |
| `k1s0-resilience` | リトライ/サーキットブレーカー | 2 | ✅ |
| `k1s0-health` | ヘルスチェック | 2 | ✅ |
| `k1s0-db` | DB 接続/トランザクション | 2 | ✅ |
| `k1s0-cache` | Redis キャッシュ | 2 | ✅ |
| `k1s0-domain-event` | ドメインイベント発行/購読/Outbox | 2 | ✅ |
| `k1s0-auth` | 認証/認可 | 3 | ✅ |

## C\#

### 共通パッケージ（Tier 1-3）

| package | 説明 | Tier | ステータス |
|---------|------|:----:|:--------:|
| `K1s0.Error` | 統一エラーハンドリング | 1 | ✅ |
| `K1s0.Config` | 設定ファイル管理 | 1 | ✅ |
| `K1s0.Validation` | 入力バリデーション | 1 | ✅ |
| `K1s0.Observability` | ロギング/トレーシング/メトリクス | 2 | ✅ |
| `K1s0.Grpc.Server` | gRPC サーバー基盤 | 2 | ✅ |
| `K1s0.Grpc.Client` | gRPC クライアント | 2 | ✅ |
| `K1s0.Health` | ヘルスチェック | 2 | ✅ |
| `K1s0.Db` | DB 接続/トランザクション（EF Core） | 2 | ✅ |
| `K1s0.DomainEvent` | ドメインイベント発行/購読/Outbox | 2 | ✅ |
| `K1s0.Resilience` | レジリエンスパターン | 2 | ✅ |
| `K1s0.Cache` | Redis キャッシュ（StackExchange.Redis） | 2 | ✅ |
| `K1s0.Auth` | 認証/認可 | 3 | ✅ |

## Python

### 共通パッケージ（Tier 1-3）

| package | 説明 | Tier | ステータス |
|---------|------|:----:|:--------:|
| `k1s0-error` | 統一エラーハンドリング | 1 | ✅ |
| `k1s0-config` | 設定ファイル管理（YAML） | 1 | ✅ |
| `k1s0-validation` | 入力バリデーション（Pydantic） | 1 | ✅ |
| `k1s0-observability` | ロギング/トレーシング/メトリクス（OpenTelemetry） | 2 | ✅ |
| `k1s0-grpc-server` | gRPC サーバー基盤（grpcio） | 2 | ✅ |
| `k1s0-grpc-client` | gRPC クライアント | 2 | ✅ |
| `k1s0-health` | ヘルスチェック（FastAPI） | 2 | ✅ |
| `k1s0-db` | DB 接続/トランザクション（SQLAlchemy + asyncpg） | 2 | ✅ |
| `k1s0-domain-event` | ドメインイベント発行/購読/Outbox | 2 | ✅ |
| `k1s0-resilience` | レジリエンスパターン | 2 | ✅ |
| `k1s0-cache` | Redis キャッシュ | 2 | ✅ |
| `k1s0-auth` | 認証/認可 | 3 | ✅ |

## Kotlin

### 共通パッケージ（Tier 1-3）

| package | 説明 | Tier | ステータス |
|---------|------|:----:|:--------:|
| `k1s0-error` | 統一エラーハンドリング | 1 | ✅ |
| `k1s0-config` | 設定ファイル管理（YAML） | 1 | ✅ |
| `k1s0-validation` | 入力バリデーション | 1 | ✅ |
| `k1s0-observability` | ロギング/トレーシング/メトリクス（OpenTelemetry） | 2 | ✅ |
| `k1s0-grpc-server` | gRPC サーバー基盤（grpc-kotlin） | 2 | ✅ |
| `k1s0-grpc-client` | gRPC クライアント | 2 | ✅ |
| `k1s0-health` | ヘルスチェック（Ktor） | 2 | ✅ |
| `k1s0-db` | DB（Exposed + HikariCP） | 2 | ✅ |
| `k1s0-domain-event` | ドメインイベント発行/購読/Outbox | 2 | ✅ |
| `k1s0-resilience` | レジリエンスパターン | 2 | ✅ |
| `k1s0-cache` | Redis キャッシュ（Lettuce） | 2 | ✅ |
| `k1s0-auth` | 認証/認可（nimbus-jose-jwt） | 3 | ✅ |

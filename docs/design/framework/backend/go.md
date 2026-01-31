# Go Backend Framework

k1s0 Go Backend Framework は、Go によるマイクロサービス開発のための共通パッケージ群を提供する。各パッケージは独立して使用可能で、Clean Architecture の原則に従って設計されている。

## パッケージ一覧

```
framework/backend/go/
├── k1s0-error/         # エラー表現の統一
├── k1s0-config/        # 設定読み込み（YAML）
├── k1s0-validation/    # 入力バリデーション（go-playground/validator）
├── k1s0-observability/ # ログ/トレース/メトリクス（Zap + OpenTelemetry）
├── k1s0-grpc-server/   # gRPC サーバ共通基盤
├── k1s0-grpc-client/   # gRPC クライアントユーティリティ
├── k1s0-resilience/    # レジリエンスパターン（gobreaker）
├── k1s0-rate-limit/    # レート制限（トークンバケット、スライディングウィンドウ）
├── k1s0-health/        # ヘルスチェックプローブ
├── k1s0-db/            # DB 接続・トランザクション（pgx/v5）
├── k1s0-cache/         # Redis キャッシュ（go-redis + MessagePack）
├── k1s0-domain-event/  # ドメインイベント発行/購読/Outbox
├── k1s0-consensus/     # リーダー選出・分散ロック・Saga オーケストレーション
└── k1s0-auth/          # 認証・認可（golang-jwt + go-oidc）
```

## Tier 構成

### Tier 1: 基盤パッケージ（フレームワーク依存なし）

| Package | 説明 | 主要依存 |
|---------|------|---------|
| k1s0-error | エラー種別（domain/application/infrastructure）、HTTP/gRPC 変換、エラーコード・コンテキスト追跡 | - |
| k1s0-config | 環境別 YAML 設定読み込み（default/dev/stg/prod）、シークレットファイル参照 | gopkg.in/yaml.v3 |
| k1s0-validation | 構造体バリデーション、カスタムバリデーション登録、gRPC Problem Details 形式 | go-playground/validator |

### Tier 2: インフラパッケージ（Tier 1 のみ依存可）

| Package | 説明 | 主要依存 |
|---------|------|---------|
| k1s0-observability | Zap ベース構造化ログ、コンテキストフィールド抽出（TraceID, RequestID, SpanID, UserID, TenantID） | go.uber.org/zap, OpenTelemetry |
| k1s0-grpc-server | サーバライフサイクル管理、インターセプター（recovery, tracing, deadline, logging, error）、ヘルスチェック、リフレクション | k1s0-error, k1s0-observability |
| k1s0-grpc-client | クライアントプール（ヘルスチェック付き）、接続再利用、インターセプター | k1s0-observability, k1s0-resilience |
| k1s0-resilience | 指数バックオフ+ジッター、サーキットブレーカー、タイムアウト、バルクヘッド | gobreaker |
| k1s0-rate-limit | トークンバケット、スライディングウィンドウ、統計追跡、ミドルウェア/gRPC インターセプター | k1s0-error |
| k1s0-health | コンポーネントベースヘルスチェック、Kubernetes プローブ対応、並列チェック+タイムアウト | - |
| k1s0-db | pgx/v5 ラッパー、プール設定、トランザクション、マイグレーション、バックプレッシャー | k1s0-error, pgx/v5 |
| k1s0-cache | Redis クライアント抽象、Cache-Aside パターン（GetOrSet, GetOrSetWithLock）、TTL 管理 | go-redis, msgpack |
| k1s0-domain-event | DomainEvent インターフェース、イベントバス、Outbox パターン | google/uuid |

### Tier 3: アプリケーションパッケージ（Tier 1, 2 依存可）

| Package | 説明 | 主要依存 |
|---------|------|---------|
| k1s0-consensus | リーダー選出（リースベース）、分散ロック（DB + Redis）、Saga オーケストレーション/コレオグラフィ、フェンシングトークン | k1s0-db, k1s0-domain-event, k1s0-observability |
| k1s0-auth | JWT 検証（RS256/HS256/ES256）、OIDC プロバイダ統合、トークンキャッシュ、gRPC/HTTP ミドルウェア | golang-jwt/jwt/v5, go-oidc |

## 依存関係

```
k1s0-error          # Tier 1（依存なし）
k1s0-config         # Tier 1（依存なし）
k1s0-validation     # Tier 1（依存なし）

k1s0-observability  # Tier 2（依存なし）
k1s0-resilience     # Tier 2（依存なし）
k1s0-health         # Tier 2（依存なし）
k1s0-domain-event   # Tier 2（依存なし）

k1s0-rate-limit     # Tier 2
  └── k1s0-error

k1s0-db             # Tier 2
  └── k1s0-error

k1s0-cache          # Tier 2（依存なし）

k1s0-grpc-server    # Tier 2
  ├── k1s0-error
  └── k1s0-observability

k1s0-grpc-client    # Tier 2
  ├── k1s0-observability
  └── k1s0-resilience

k1s0-consensus      # Tier 3
  ├── k1s0-db
  ├── k1s0-domain-event
  ├── k1s0-observability
  └── k1s0-config

k1s0-auth           # Tier 3
```

## ビルド・テスト

```bash
cd framework/backend/go

# 全モジュールのビルド
for dir in */; do
  if [ -f "${dir}go.mod" ]; then
    cd "$dir" && go build ./... && cd ..
  fi
done

# 全モジュールのテスト（race detector 付き）
for dir in */; do
  if [ -f "${dir}go.mod" ]; then
    cd "$dir" && go test -v -race ./... && cd ..
  fi
done

# フォーマットチェック
gofmt -l .

# 静的解析
go vet ./...

# Lint（golangci-lint）
golangci-lint run --timeout=5m ./...
```

## 設計方針

- **環境変数禁止**: `os.Getenv` 等は使用せず、`k1s0-config` 経由で YAML から設定を取得
- **Clean Architecture 準拠**: domain → application → infrastructure → presentation の依存方向を厳守
- **エラーハンドリング**: `k1s0-error` の `ErrorKind` で分類し、presentation 層で HTTP/gRPC ステータスに変換
- **Tier 依存ルール**: Tier 1 は依存なし、Tier 2 は Tier 1 のみ、Tier 3 は Tier 1 + 2（k1s0-consensus は例外的に Tier 2 間依存を許可）

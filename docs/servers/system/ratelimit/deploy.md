# system-ratelimit-server デプロイ設計

system-ratelimit-server のデプロイ仕様を定義する。概要・API 定義・アーキテクチャは [system-ratelimit-server.md](server.md) を参照。

---

## デプロイ概要

| 項目 | 値 |
| --- | --- |
| サーバー名 | ratelimit-server |
| Tier | system |
| 実装言語 | Rust |
| パッケージ名 | `k1s0-ratelimit-server` |
| REST ポート | 8080（コンテナ内部） |
| gRPC ポート | 50051 |
| DB スキーマ | `ratelimit`（PostgreSQL） |
| キャッシュ | Redis（トークンバケット状態管理 + Lua スクリプト） |
| メッセージング | なし（Kafka 不使用） |
| 配置パス | `regions/system/server/rust/ratelimit/` |

> **重要**: レートリミットサーバーは Kong API ゲートウェイからの高頻度 gRPC 呼び出しを受けるため、低レイテンシ・高スループット設計が求められる。

---

## コンテナイメージ

### Dockerfile 構成

| 項目 | 詳細 |
| --- | --- |
| ビルドステージ | `rust:1.93-bookworm`（マルチステージビルド + cargo-chef） |
| ランタイムステージ | `gcr.io/distroless/cc-debian12:nonroot`（最小イメージ） |
| 追加パッケージ | `protobuf-compiler`（proto 生成）、`cmake` + `build-essential`（rdkafka ビルド） |
| libz コピー | distroless には zlib が含まれないため、ビルドステージから手動コピー |
| ビルドコマンド | `cargo build --release -p k1s0-ratelimit-server`（ワークスペースから特定パッケージを指定） |
| ビルドコンテキスト | `regions/system`（`COPY . .` でシステム全体のライブラリ依存を含める） |
| 公開ポート | 8080（REST API）、50051（gRPC） |
| 実行ユーザー | `nonroot:nonroot`（セキュリティベストプラクティス） |

### Dockerfile

```dockerfile
# chef ベースステージ: cargo-chef とビルド依存のインストール
# Note: build context must be ./regions/system (to include library dependencies)
FROM rust:1.93-bookworm AS chef

# cargo-chef のインストール（依存キャッシュ用）
RUN cargo install cargo-chef --locked

# protobuf コンパイラ（tonic-build 用）と cmake + build-essential（rdkafka 用）のインストール
RUN apt-get update && apt-get install -y --no-install-recommends \
    protobuf-compiler \
    cmake \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# planner ステージ: 依存情報レシピの生成
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# cook ステージ: 依存のみビルド（キャッシュ層）
FROM chef AS cook
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json -p k1s0-ratelimit-server

# builder ステージ: ソースコードのビルド
FROM cook AS builder
COPY . .
RUN cargo build --release -p k1s0-ratelimit-server

FROM busybox:1.36.1-musl AS busybox

# runtime ステージ: 最小実行環境
FROM gcr.io/distroless/cc-debian12:nonroot

COPY --from=builder /usr/lib/x86_64-linux-gnu/libz.so.1 /usr/lib/x86_64-linux-gnu/libz.so.1
COPY --from=builder /app/target/release/k1s0-ratelimit-server /k1s0-ratelimit-server
COPY --from=busybox /bin/busybox /busybox

USER nonroot:nonroot
EXPOSE 8080 50051

ENTRYPOINT ["/k1s0-ratelimit-server"]
```

---

## 環境変数

config.yaml で設定する項目の一覧。Vault Agent Injector 経由でシークレットを注入する。

| 環境変数 / 設定キー | 説明 | デフォルト値 | 必須 |
| --- | --- | --- | --- |
| `app.name` | アプリケーション名 | `ratelimit` | Yes |
| `app.version` | バージョン | `0.1.0` | Yes |
| `app.environment` | 実行環境（production / staging / dev） | `production` | Yes |
| `server.host` | バインドホスト | `0.0.0.0` | Yes |
| `server.port` | REST ポート | `8080` | Yes |
| `server.grpc_port` | gRPC ポート | `50051` | Yes |
| `database.host` | PostgreSQL ホスト | - | Yes |
| `database.port` | PostgreSQL ポート | `5432` | Yes |
| `database.name` | データベース名 | `k1s0_system` | Yes |
| `database.user` | DB ユーザー | `app` | Yes |
| `database.password` | DB パスワード（Vault 注入） | - | Yes |
| `database.ssl_mode` | SSL モード | `disable` | No |
| `database.max_open_conns` | 最大接続数 | `25` | No |
| `database.max_idle_conns` | アイドル接続数 | `5` | No |
| `database.conn_max_lifetime` | 接続最大寿命 | `5m` | No |
| `redis.url` | Redis 接続 URL | - | Yes |
| `redis.pool_size` | Redis 接続プールサイズ | `20` | No |
| `redis.timeout_ms` | Redis 操作タイムアウト（ms） | `100` | No |
| `ratelimit.fail_open` | Redis 障害時のフェイルオープン | `true` | No |
| `ratelimit.default_limit` | デフォルトリクエスト上限 | `100` | No |
| `ratelimit.default_window_seconds` | デフォルトウィンドウサイズ（秒） | `60` | No |

### Vault シークレットパス

| シークレット | パス |
| --- | --- |
| DB パスワード | `secret/data/k1s0/system/ratelimit/database` |
| Redis パスワード | `secret/data/k1s0/system/ratelimit/redis` |

---

## DB マイグレーション

レートリミットルールは PostgreSQL の `ratelimit` スキーマに格納する。Redis はトークンバケットの状態管理に使用し、永続化対象外である。詳細スキーマは [database.md](database.md) 参照。

### 対象テーブル

| テーブル名 | 説明 |
| --- | --- |
| `ratelimit.rate_limit_rules` | レートリミットルール定義（scope, limit, window, algorithm 等） |

### マイグレーション実行

```bash
# sqlx-cli によるマイグレーション実行
sqlx migrate run --source regions/system/database/ratelimit-db/migrations/
```

---

## ヘルスチェック

| エンドポイント | 用途 | チェック内容 |
| --- | --- | --- |
| `GET /healthz` | Liveness Probe | プロセス生存確認 |
| `GET /readyz` | Readiness Probe | PostgreSQL 接続確認 + Redis 接続確認 |
| `GET /metrics` | Prometheus メトリクス | OpenTelemetry メトリクス公開 |

### Kubernetes Probe 設定

```yaml
livenessProbe:
  httpGet:
    path: /healthz
    port: 8080
  initialDelaySeconds: 10
  periodSeconds: 10

readinessProbe:
  httpGet:
    path: /readyz
    port: 8080
  initialDelaySeconds: 5
  periodSeconds: 5
```

---

## リソース要件

| 項目 | requests | limits |
| --- | --- | --- |
| CPU | 200m | 1000m |
| Memory | 128Mi | 256Mi |

> **補足**: Kong からの高頻度呼び出しに対応するため、CPU requests は他のサーバーより高めに設定している。Redis Lua スクリプトのアトミック操作によるレイテンシは極めて低い（< 1ms）ため、メモリはコンパクトに保つ。

---

## スケーリング設定

| 項目 | 値 |
| --- | --- |
| 最小レプリカ数 | 3 |
| 最大レプリカ数 | 10 |
| ターゲット CPU 使用率 | 60% |
| スケーリング方式 | HPA（Horizontal Pod Autoscaler） |

### HPA 設定

```yaml
autoscaling:
  enabled: true
  minReplicas: 3
  maxReplicas: 10
  targetCPUUtilizationPercentage: 60
```

> **注意**: レートリミットサーバーは Kong からの全リクエストで呼び出されるため、最小レプリカ数を 3 に設定し高可用性を確保する。CPU ターゲットは 60% と低めに設定し、バースト負荷への応答性を高めている。Redis を状態ストアとして使用するためサーバー自体はステートレスであり、水平スケーリングが安全に行える。

---

## テスト方針

### レイヤー別テスト

| レイヤー | テスト種別 | ツール |
| --- | --- | --- |
| domain/entity | 単体テスト | `#[cfg(test)]` + `assert!` |
| domain/service | 単体テスト | `#[cfg(test)]` + `assert!` |
| usecase | 単体テスト（モック） | `mockall` |
| adapter/handler | 統合テスト（HTTP/gRPC） | `axum::test` + `tokio::test` |
| infrastructure/persistence | 統合テスト（DB） | `testcontainers` |
| infrastructure/cache | 統合テスト（Redis + Lua） | `testcontainers`（Redis） |

---

## Helm values

[helm設計.md](../../infrastructure/kubernetes/helm設計.md) のサーバー用 Helm Chart を使用する。レートリミットサーバー固有の values は以下の通り。

```yaml
# values-ratelimit.yaml
app:
  name: ratelimit-server
  tier: system

image:
  repository: harbor.internal.example.com/k1s0/ratelimit-server
  tag: "0.1.0"

service:
  ports:
    - name: http
      port: 80
      targetPort: 8080
    - name: grpc
      port: 50051
      targetPort: 50051

resources:
  requests:
    cpu: 200m
    memory: 128Mi
  limits:
    cpu: "1"
    memory: 256Mi

# Vault Agent Injector
podAnnotations:
  vault.hashicorp.com/agent-inject: "true"
  vault.hashicorp.com/role: "system"
  vault.hashicorp.com/agent-inject-secret-db-password: "secret/data/k1s0/system/ratelimit/database"
  vault.hashicorp.com/agent-inject-secret-redis-password: "secret/data/k1s0/system/ratelimit/redis"

# ヘルスチェック
livenessProbe:
  httpGet:
    path: /healthz
    port: 8080
  initialDelaySeconds: 10
  periodSeconds: 10

readinessProbe:
  httpGet:
    path: /readyz
    port: 8080
  initialDelaySeconds: 5
  periodSeconds: 5

# ConfigMap マウント
configMap:
  name: ratelimit-server-config
  mountPath: /etc/app/config.yaml
```

---

## Kong ルーティング

[認証認可設計.md](../../architecture/auth/認証認可設計.md) の Kong ルーティング設計に従い、レートリミットサーバーを Kong に登録する。Kong カスタムプラグインからは gRPC で直接 `CheckRateLimit` を呼び出す。

```yaml
services:
  - name: ratelimit-v1
    url: http://ratelimit-server.k1s0-system.svc.cluster.local:80
    routes:
      - name: ratelimit-v1-route
        paths:
          - /api/v1/ratelimit
        strip_path: false
    plugins:
      - name: rate-limiting
        config:
          minute: 3000
          policy: redis
```

---

## 関連ドキュメント

- [system-ratelimit-server.md](server.md) -- 概要・API 定義・アーキテクチャ
- [system-ratelimit-database.md](database.md) -- データベーススキーマ
- [Dockerイメージ戦略.md](../../infrastructure/docker/Dockerイメージ戦略.md) -- マルチステージビルド・ベースイメージ
- [helm設計.md](../../infrastructure/kubernetes/helm設計.md) -- Helm Chart・Vault Agent Injector
- [可観測性設計.md](../../architecture/observability/可観測性設計.md) -- OpenTelemetry・Prometheus・構造化ログ
- [認証認可設計.md](../../architecture/auth/認証認可設計.md) -- Kong ルーティング設計

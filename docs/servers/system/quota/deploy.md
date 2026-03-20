# system-quota-server デプロイ設計

system-quota-server のデプロイ仕様を定義する。概要・API 定義・アーキテクチャは [system-quota-server.md](server.md) を参照。

---

## デプロイ概要

| 項目 | 値 |
| --- | --- |
| サーバー名 | quota-server |
| Tier | system |
| 実装言語 | Rust |
| パッケージ名 | `k1s0-quota-server` |
| REST ポート | 8080（コンテナ内部）/ 8097（ホスト側） |
| gRPC ポート | 50051 |
| DB スキーマ | `quota`（PostgreSQL） |
| キャッシュ | Redis（使用量カウンター） |
| メッセージング | Kafka（超過イベント・閾値通知） |
| 配置パス | `regions/system/server/rust/quota/` |

---

## コンテナイメージ

### Dockerfile 構成

| 項目 | 詳細 |
| --- | --- |
| ビルドステージ | `rust:1.93-bookworm`（マルチステージビルド + cargo-chef） |
| ランタイムステージ | `gcr.io/distroless/cc-debian12:nonroot`（最小イメージ） |
| 追加パッケージ | `protobuf-compiler`（proto 生成）、`cmake` + `build-essential`（rdkafka ビルド） |
| libz コピー | distroless には zlib が含まれないため、ビルドステージから手動コピー |
| ビルドコマンド | `cargo build --release -p k1s0-quota-server`（ワークスペースから特定パッケージを指定） |
| ビルドコンテキスト | `regions/system`（`COPY . .` でシステム全体のライブラリ依存を含める） |
| 公開ポート | 8080（REST API、ホスト側 8097）、50051（gRPC） |
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
RUN cargo chef cook --release --recipe-path recipe.json -p k1s0-quota-server

# builder ステージ: ソースコードのビルド
FROM cook AS builder
COPY . .
RUN cargo build --release -p k1s0-quota-server

FROM busybox:1.36.1-musl AS busybox

# runtime ステージ: 最小実行環境
FROM gcr.io/distroless/cc-debian12:nonroot

COPY --from=builder /usr/lib/x86_64-linux-gnu/libz.so.1 /usr/lib/x86_64-linux-gnu/libz.so.1
COPY --from=builder /app/target/release/k1s0-quota-server /k1s0-quota-server
COPY --from=busybox /bin/busybox /busybox

USER nonroot:nonroot
EXPOSE 8080 50051

ENTRYPOINT ["/k1s0-quota-server"]
```

---

## 環境変数

config.yaml で設定する項目の一覧。Vault Agent Injector 経由でシークレットを注入する。

| 環境変数 / 設定キー | 説明 | デフォルト値 | 必須 |
| --- | --- | --- | --- |
| `app.name` | アプリケーション名 | `quota` | Yes |
| `app.version` | バージョン | `0.1.0` | Yes |
| `app.environment` | 実行環境（production / staging / dev） | `production` | Yes |
| `server.host` | バインドホスト | `0.0.0.0` | Yes |
| `server.port` | REST ポート | `8097` | Yes |
| `server.grpc_port` | gRPC ポート | `50051` | Yes |
| `database.url` | PostgreSQL 接続 URL | - | Yes |
| `database.schema` | DB スキーマ名 | `quota` | Yes |
| `database.max_connections` | 最大接続数 | `10` | No |
| `database.min_connections` | 最小接続数 | `2` | No |
| `database.connect_timeout_seconds` | 接続タイムアウト（秒） | `5` | No |
| `redis.url` | Redis 接続 URL | - | Yes |
| `redis.pool_size` | Redis 接続プールサイズ | `10` | No |
| `redis.key_prefix` | Redis キープレフィックス | `quota:` | No |
| `redis.connect_timeout_seconds` | Redis 接続タイムアウト（秒） | `3` | No |
| `kafka.brokers` | Kafka ブローカーリスト | - | Yes |
| `kafka.security_protocol` | Kafka セキュリティプロトコル | `PLAINTEXT` | No |
| `kafka.topic_exceeded` | クォータ超過トピック | `k1s0.system.quota.exceeded.v1` | Yes |
| `kafka.topic_threshold` | 閾値到達トピック | `k1s0.system.quota.threshold.reached.v1` | Yes |
| `auth.jwks_url` | JWKS エンドポイント URL | - | Yes |
| `quota.reset_schedule.daily` | 日次リセット cron 式 | `0 0 * * *` | No |
| `quota.reset_schedule.monthly` | 月次リセット cron 式 | `0 0 1 * *` | No |

### Vault シークレットパス

| シークレット | パス |
| --- | --- |
| DB パスワード | `secret/data/k1s0/system/quota/database` |
| Redis パスワード | `secret/data/k1s0/system/quota/redis` |
| Kafka SASL | `secret/data/k1s0/system/kafka/sasl` |

---

## DB マイグレーション

クォータポリシーと使用量データは PostgreSQL の `quota` スキーマに格納する。Redis はリアルタイムカウンターとして使用し、永続化対象外である。詳細スキーマは [database.md](database.md) 参照。

### 対象テーブル

| テーブル名 | 説明 |
| --- | --- |
| `quota.quota_policies` | クォータポリシー定義（subject_type, period, limit 等） |
| `quota.quota_usage` | クォータ使用量（PostgreSQL 上の永続化記録） |

### マイグレーション実行

```bash
# sqlx-cli によるマイグレーション実行
sqlx migrate run --source regions/system/database/quota-db/migrations/
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
| CPU | 100m | 500m |
| Memory | 128Mi | 256Mi |

> **補足**: Redis アトミックカウンターによる低レイテンシ要件があるため、CPU throttling を避けるためにリソースリミットに余裕を持たせている。

---

## スケーリング設定

| 項目 | 値 |
| --- | --- |
| 最小レプリカ数 | 2 |
| 最大レプリカ数 | 5 |
| ターゲット CPU 使用率 | 70% |
| スケーリング方式 | HPA（Horizontal Pod Autoscaler） |

### HPA 設定

```yaml
autoscaling:
  enabled: true
  minReplicas: 2
  maxReplicas: 5
  targetCPUUtilizationPercentage: 70
```

> **注意**: クォータサーバーは Redis カウンターを使用するためステートレスであり、水平スケーリングが安全に行える。日次・月次リセットはスケジューラーで実行されるため、レプリカ数に依存しない。

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
| infrastructure/cache | 統合テスト（Redis） | `testcontainers`（Redis） |

---

## Helm values

[helm設計.md](../../infrastructure/kubernetes/helm設計.md) のサーバー用 Helm Chart を使用する。クォータサーバー固有の values は以下の通り。

```yaml
# values-quota.yaml
app:
  name: quota-server
  tier: system

image:
  repository: harbor.internal.example.com/k1s0/quota-server
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
    cpu: 100m
    memory: 128Mi
  limits:
    cpu: 500m
    memory: 256Mi

# Vault Agent Injector
podAnnotations:
  vault.hashicorp.com/agent-inject: "true"
  vault.hashicorp.com/role: "system"
  vault.hashicorp.com/agent-inject-secret-db-password: "secret/data/k1s0/system/quota/database"
  vault.hashicorp.com/agent-inject-secret-redis-password: "secret/data/k1s0/system/quota/redis"
  vault.hashicorp.com/agent-inject-secret-kafka-sasl: "secret/data/k1s0/system/kafka/sasl"

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
  name: quota-server-config
  mountPath: /etc/app/config.yaml
```

---

## Kong ルーティング

[認証認可設計.md](../../architecture/auth/認証認可設計.md) の Kong ルーティング設計に従い、クォータサーバーを Kong に登録する。

```yaml
services:
  - name: quota-v1
    url: http://quota-server.k1s0-system.svc.cluster.local:80
    routes:
      - name: quota-v1-route
        paths:
          - /api/v1/quotas
        strip_path: false
    plugins:
      - name: rate-limiting
        config:
          minute: 3000
          policy: redis
```

---

## 関連ドキュメント

- [system-quota-server.md](server.md) -- 概要・API 定義・アーキテクチャ
- [system-quota-database.md](database.md) -- データベーススキーマ
- [Dockerイメージ戦略.md](../../infrastructure/docker/Dockerイメージ戦略.md) -- マルチステージビルド・ベースイメージ
- [helm設計.md](../../infrastructure/kubernetes/helm設計.md) -- Helm Chart・Vault Agent Injector
- [可観測性設計.md](../../architecture/observability/可観測性設計.md) -- OpenTelemetry・Prometheus・構造化ログ
- [認証認可設計.md](../../architecture/auth/認証認可設計.md) -- Kong ルーティング設計

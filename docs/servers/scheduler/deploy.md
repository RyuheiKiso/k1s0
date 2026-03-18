# system-scheduler-server デプロイ設計

system-scheduler-server のデプロイ仕様を定義する。概要・API 定義・アーキテクチャは [system-scheduler-server.md](server.md) を参照。

---

## デプロイ概要

| 項目 | 値 |
| --- | --- |
| サーバー名 | scheduler-server |
| Tier | system |
| 実装言語 | Rust |
| パッケージ名 | `k1s0-scheduler-server` |
| REST ポート | 8095（コンテナ内部） |
| gRPC ポート | 50051 |
| DB スキーマ | `scheduler`（PostgreSQL） |
| キャッシュ | moka v0.12（インプロセスキャッシュ） |
| メッセージング | Kafka（ジョブトリガー通知発行） |
| 分散ロック | PostgreSQL `SELECT FOR UPDATE SKIP LOCKED` |
| 配置パス | `regions/system/server/rust/scheduler/` |

> **重要**: スケジューラーサーバーは PostgreSQL 分散ロックで重複実行防止を行うため、複数レプリカでの安全な水平スケーリングが可能である。

---

## コンテナイメージ

### Dockerfile 構成

| 項目 | 詳細 |
| --- | --- |
| ビルドステージ | `rust:1.93-bookworm`（マルチステージビルド + cargo-chef） |
| ランタイムステージ | `gcr.io/distroless/cc-debian12:nonroot`（最小イメージ） |
| 追加パッケージ | `protobuf-compiler`（proto 生成）、`cmake` + `build-essential`（rdkafka ビルド） |
| libz コピー | distroless には zlib が含まれないため、ビルドステージから手動コピー |
| ビルドコマンド | `cargo build --release -p k1s0-scheduler-server`（ワークスペースから特定パッケージを指定） |
| ビルドコンテキスト | `regions/system`（`COPY . .` でシステム全体のライブラリ依存を含める） |
| 公開ポート | 8080（REST API、ホスト側 8095）、50051（gRPC） |
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
RUN cargo chef cook --release --recipe-path recipe.json -p k1s0-scheduler-server

# builder ステージ: ソースコードのビルド
FROM cook AS builder
COPY . .
RUN cargo build --release -p k1s0-scheduler-server

FROM busybox:1.36.1-musl AS busybox

# runtime ステージ: 最小実行環境
FROM gcr.io/distroless/cc-debian12:nonroot

COPY --from=builder /usr/lib/x86_64-linux-gnu/libz.so.1 /usr/lib/x86_64-linux-gnu/libz.so.1
COPY --from=builder /app/target/release/k1s0-scheduler-server /k1s0-scheduler-server
COPY --from=busybox /bin/busybox /busybox

USER nonroot:nonroot
EXPOSE 8080 50051

ENTRYPOINT ["/k1s0-scheduler-server"]
```

---

## 環境変数

config.yaml で設定する項目の一覧。Vault Agent Injector 経由でシークレットを注入する。

| 環境変数 / 設定キー | 説明 | デフォルト値 | 必須 |
| --- | --- | --- | --- |
| `app.name` | アプリケーション名 | `scheduler` | Yes |
| `app.version` | バージョン | `0.1.0` | Yes |
| `app.environment` | 実行環境（production / staging / dev） | `production` | Yes |
| `server.host` | バインドホスト | `0.0.0.0` | Yes |
| `server.port` | REST ポート | `8095` | Yes |
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
| `kafka.brokers` | Kafka ブローカーリスト | - | Yes |
| `kafka.security_protocol` | Kafka セキュリティプロトコル | `PLAINTEXT` | No |
| `kafka.topic` | ジョブトリガー通知トピック | `k1s0.system.scheduler.triggered.v1` | Yes |
| `auth.jwks_url` | JWKS エンドポイント URL | - | Yes |
| `auth.issuer` | JWT 発行者 | - | Yes |
| `auth.audience` | JWT オーディエンス | - | Yes |
| `auth.jwks_cache_ttl_secs` | JWKS キャッシュ TTL（秒） | `300` | No |

### Vault シークレットパス

| シークレット | パス |
| --- | --- |
| DB パスワード | `secret/data/k1s0/system/scheduler/database` |
| Kafka SASL | `secret/data/k1s0/system/kafka/sasl` |

---

## DB マイグレーション

ジョブ定義と実行履歴は PostgreSQL の `scheduler` スキーマに格納する。分散ロックには `SELECT FOR UPDATE SKIP LOCKED` を使用する。詳細スキーマは [database.md](database.md) 参照。

### 対象テーブル

| テーブル名 | 説明 |
| --- | --- |
| `scheduler.scheduler_jobs` | ジョブ定義（cron 式・ターゲット種別・タイムゾーン） |
| `scheduler.job_executions` | ジョブ実行履歴（状態・開始/完了日時・エラー） |

### マイグレーション実行

```bash
# sqlx-cli によるマイグレーション実行
sqlx migrate run --source regions/system/database/scheduler-db/migrations/
```

### 起動時のジョブロード

サーバー起動時に `CronSchedulerEngine` が全有効ジョブ（`status = 'active'`）を PostgreSQL からロードし、tokio による非同期タイマーを設定する。cron 式とタイムゾーンから次回実行時刻を計算し、DST（夏時間）を考慮した正確なスケジューリングを行う。

---

## ヘルスチェック

| エンドポイント | 用途 | チェック内容 |
| --- | --- | --- |
| `GET /healthz` | Liveness Probe | プロセス生存確認 |
| `GET /readyz` | Readiness Probe | PostgreSQL 接続確認 + ジョブロード完了確認 |
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

> **補足**: cron スケジューリングループは tokio タスクで非同期実行されるため CPU 使用率は通常低い。ジョブ実行時のバーストに備えて limits に余裕を持たせている。HTTP コールバック（reqwest）を使用するジョブがある場合は、同時リクエスト数に応じてメモリを調整すること。

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

> **注意**: 全レプリカが同一ジョブのスケジュールを監視するが、PostgreSQL の `SELECT FOR UPDATE SKIP LOCKED` による分散ロックで 1 レプリカのみがジョブを実行する。これにより、レプリカ数に関わらずジョブの重複実行が防止される。k1s0-distributed-lock ライブラリを使用している。

---

## テスト方針

### レイヤー別テスト

| レイヤー | テスト種別 | ツール |
| --- | --- | --- |
| domain/entity | 単体テスト | `#[cfg(test)]` + `assert!` |
| domain/service | 単体テスト（cron パース・次回実行時刻計算） | `#[cfg(test)]` + `assert!` |
| usecase | 単体テスト（モック） | `mockall` |
| adapter/handler | 統合テスト（HTTP/gRPC） | `axum::test` + `tokio::test` |
| infrastructure/persistence | 統合テスト（DB） | `testcontainers` |
| infrastructure/messaging | 統合テスト | `mockall`（Kafka プロデューサー） |

---

## Helm values

[helm設計.md](../../infrastructure/kubernetes/helm設計.md) のサーバー用 Helm Chart を使用する。スケジューラーサーバー固有の values は以下の通り。

```yaml
# values-scheduler.yaml
app:
  name: scheduler-server
  tier: system

image:
  repository: harbor.internal.example.com/k1s0/scheduler-server
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
  vault.hashicorp.com/agent-inject-secret-db-password: "secret/data/k1s0/system/scheduler/database"
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
  name: scheduler-server-config
  mountPath: /etc/app/config.yaml
```

---

## Kong ルーティング

[認証認可設計.md](../../architecture/auth/認証認可設計.md) の Kong ルーティング設計に従い、スケジューラーサーバーを Kong に登録する。

```yaml
services:
  - name: scheduler-v1
    url: http://scheduler-server.k1s0-system.svc.cluster.local:80
    routes:
      - name: scheduler-v1-jobs-route
        paths:
          - /api/v1/jobs
        strip_path: false
    plugins:
      - name: rate-limiting
        config:
          minute: 3000
          policy: redis
```

---

## 関連ドキュメント

- [system-scheduler-server.md](server.md) -- 概要・API 定義・アーキテクチャ
- [system-scheduler-database.md](database.md) -- データベーススキーマ
- [Dockerイメージ戦略.md](../../infrastructure/docker/Dockerイメージ戦略.md) -- マルチステージビルド・ベースイメージ
- [helm設計.md](../../infrastructure/kubernetes/helm設計.md) -- Helm Chart・Vault Agent Injector
- [可観測性設計.md](../../architecture/observability/可観測性設計.md) -- OpenTelemetry・Prometheus・構造化ログ
- [認証認可設計.md](../../architecture/auth/認証認可設計.md) -- Kong ルーティング設計

# system-rule-engine-server デプロイ設計

system-rule-engine-server のデプロイ仕様を定義する。概要・API 定義・アーキテクチャは [system-rule-engine-server.md](server.md) を参照。

---

## デプロイ概要

| 項目 | 値 |
| --- | --- |
| サーバー名 | rule-engine-server |
| Tier | system |
| 実装言語 | Rust |
| パッケージ名 | `k1s0-rule-engine-server` |
| REST ポート | 8111（コンテナ内部） |
| gRPC ポート | 50051 |
| DB スキーマ | `rule_engine`（PostgreSQL） |
| キャッシュ | moka v0.12（インプロセスキャッシュ、ルール定義・評価結果） |
| メッセージング | Kafka（ルール変更通知の発行 + config-server 変更通知の購読） |
| 配置パス | `regions/system/server/rust/rule-engine/` |

---

## コンテナイメージ

### Dockerfile 構成

| 項目 | 詳細 |
| --- | --- |
| ビルドステージ | `rust:1.93-bookworm`（マルチステージビルド + cargo-chef） |
| ランタイムステージ | `gcr.io/distroless/cc-debian12:nonroot`（最小イメージ） |
| 追加パッケージ | `protobuf-compiler`（proto 生成）、`cmake` + `build-essential`（rdkafka ビルド） |
| libz コピー | distroless には zlib が含まれないため、ビルドステージから手動コピー |
| ビルドコマンド | `cargo build --release -p k1s0-rule-engine-server`（ワークスペースから特定パッケージを指定） |
| ビルドコンテキスト | `regions/system`（`COPY . .` でシステム全体のライブラリ依存を含める） |
| 公開ポート | 8080（REST API、ホスト側 8111）、50051（gRPC） |
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
RUN cargo chef cook --release --recipe-path recipe.json -p k1s0-rule-engine-server

# builder ステージ: ソースコードのビルド
FROM cook AS builder
COPY . .
RUN cargo build --release -p k1s0-rule-engine-server

FROM busybox:1.36.1-musl AS busybox

# runtime ステージ: 最小実行環境
FROM gcr.io/distroless/cc-debian12:nonroot

COPY --from=builder /usr/lib/x86_64-linux-gnu/libz.so.1 /usr/lib/x86_64-linux-gnu/libz.so.1
COPY --from=builder /app/target/release/k1s0-rule-engine-server /k1s0-rule-engine-server
COPY --from=busybox /bin/busybox /busybox

USER nonroot:nonroot
EXPOSE 8080 50051

ENTRYPOINT ["/k1s0-rule-engine-server"]
```

---

## 環境変数

config.yaml で設定する項目の一覧。Vault Agent Injector 経由でシークレットを注入する。

| 環境変数 / 設定キー | 説明 | デフォルト値 | 必須 |
| --- | --- | --- | --- |
| `app.name` | アプリケーション名 | `rule-engine` | Yes |
| `app.version` | バージョン | `0.1.0` | Yes |
| `app.environment` | 実行環境（production / staging / dev） | `production` | Yes |
| `server.host` | バインドホスト | `0.0.0.0` | Yes |
| `server.port` | REST ポート | `8111` | Yes |
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
| `kafka.topic` | ルール変更通知トピック | `k1s0.system.rule_engine.rule_changed.v1` | Yes |
| `cache.max_entries` | moka キャッシュ最大エントリ数 | `100000` | No |
| `cache.ttl_seconds` | キャッシュ TTL（秒） | `60` | No |
| `audit.enabled` | 監査ログ送信の有効化 | `true` | No |
| `audit.endpoint` | audit-server エンドポイント | - | Yes（enabled 時） |
| `audit.batch_size` | 監査ログバッチサイズ | `100` | No |
| `audit.flush_interval_ms` | フラッシュ間隔（ms） | `5000` | No |
| `auth.jwks_url` | JWKS エンドポイント URL | - | Yes |
| `auth.issuer` | JWT 発行者 | - | Yes |
| `auth.audience` | JWT オーディエンス | `k1s0-api` | Yes |
| `auth.jwks_cache_ttl_secs` | JWKS キャッシュ TTL（秒） | `300` | No |

### Vault シークレットパス

| シークレット | パス |
| --- | --- |
| DB パスワード | `secret/data/k1s0/system/rule-engine/database` |
| Kafka SASL | `secret/data/k1s0/system/kafka/sasl` |

---

## DB マイグレーション

ルール定義・ルールセット・バージョン・評価ログは PostgreSQL の `rule_engine` スキーマに格納する。詳細スキーマは [database.md](database.md) 参照。

### 対象テーブル

| テーブル名 | 説明 |
| --- | --- |
| `rule_engine.rules` | ルール定義（条件式 AST + 結果データ） |
| `rule_engine.rule_sets` | ルールセット（ドメイン・評価モード・ルール ID 一覧） |
| `rule_engine.rule_set_versions` | ルールセットバージョン（公開スナップショット） |
| `rule_engine.evaluation_logs` | 評価ログ（ルールセット名・結果・入力ハッシュ） |

### マイグレーション実行

```bash
# sqlx-cli によるマイグレーション実行
sqlx migrate run --source regions/system/database/rule-engine-db/migrations/
```

---

## ヘルスチェック

| エンドポイント | 用途 | チェック内容 |
| --- | --- | --- |
| `GET /healthz` | Liveness Probe | プロセス生存確認 |
| `GET /readyz` | Readiness Probe | PostgreSQL 接続確認 |
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

> **補足**: moka インプロセスキャッシュ（最大 100,000 エントリ）を使用するため、メモリ使用量はキャッシュサイズに比例して増加する。大量のルール定義・評価結果をキャッシュする場合は memory limits の引き上げを検討すること。

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

> **注意**: 各レプリカはインプロセス moka キャッシュを独立に保持するため、スケールアウト時にはキャッシュの温まり（warm-up）に TTL 分の時間がかかる。Kafka ルール変更通知は全レプリカに配信されるため、キャッシュ無効化は全インスタンスで同時に発生する。

---

## テスト方針

### レイヤー別テスト

| レイヤー | テスト種別 | ツール |
| --- | --- | --- |
| domain/entity | 単体テスト | `#[cfg(test)]` + `assert!` |
| domain/service | 単体テスト（条件式パーサー・評価エンジン） | `#[cfg(test)]` + `assert!` |
| usecase | 単体テスト（モック） | `mockall` |
| adapter/handler | 統合テスト（HTTP/gRPC） | `axum::test` + `tokio::test` |
| infrastructure/persistence | 統合テスト（DB） | `testcontainers` |
| infrastructure/cache | 単体テスト | `tokio::test`（moka） |
| infrastructure/messaging | 統合テスト | `mockall`（Kafka プロデューサー/コンシューマー） |

---

## Helm values

[helm設計.md](../../infrastructure/kubernetes/helm設計.md) のサーバー用 Helm Chart を使用する。ルールエンジンサーバー固有の values は以下の通り。

```yaml
# values-rule-engine.yaml
app:
  name: rule-engine-server
  tier: system

image:
  repository: harbor.internal.example.com/k1s0/rule-engine-server
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
  vault.hashicorp.com/agent-inject-secret-db-password: "secret/data/k1s0/system/rule-engine/database"
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
  name: rule-engine-server-config
  mountPath: /etc/app/config.yaml
```

---

## Kong ルーティング

[認証認可設計.md](../../architecture/auth/認証認可設計.md) の Kong ルーティング設計に従い、ルールエンジンサーバーを Kong に登録する。

```yaml
services:
  - name: rule-engine-v1
    url: http://rule-engine-server.k1s0-system.svc.cluster.local:80
    routes:
      - name: rule-engine-v1-rules-route
        paths:
          - /api/v1/rules
        strip_path: false
      - name: rule-engine-v1-rule-sets-route
        paths:
          - /api/v1/rule-sets
        strip_path: false
      - name: rule-engine-v1-evaluate-route
        paths:
          - /api/v1/evaluate
        strip_path: false
      - name: rule-engine-v1-evaluation-logs-route
        paths:
          - /api/v1/evaluation-logs
        strip_path: false
    plugins:
      - name: rate-limiting
        config:
          minute: 3000
          policy: redis
```

---

## 関連ドキュメント

- [system-rule-engine-server.md](server.md) -- 概要・API 定義・アーキテクチャ
- [system-rule-engine-database.md](database.md) -- データベーススキーマ
- [Dockerイメージ戦略.md](../../infrastructure/docker/Dockerイメージ戦略.md) -- マルチステージビルド・ベースイメージ
- [helm設計.md](../../infrastructure/kubernetes/helm設計.md) -- Helm Chart・Vault Agent Injector
- [可観測性設計.md](../../architecture/observability/可観測性設計.md) -- OpenTelemetry・Prometheus・構造化ログ
- [認証認可設計.md](../../architecture/auth/認証認可設計.md) -- Kong ルーティング設計

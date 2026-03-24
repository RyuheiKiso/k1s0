# system-saga-server デプロイ設計

system-saga-server のデプロイ仕様を定義する。概要・API 定義・アーキテクチャは [system-saga-server.md](server.md) を参照。

---

## デプロイ概要

| 項目 | 値 |
| --- | --- |
| サーバー名 | saga-server |
| Tier | system |
| 実装言語 | Rust |
| パッケージ名 | `k1s0-saga-server` |
| REST ポート | 8080（コンテナ内部） |
| gRPC ポート | 50051 |
| DB スキーマ | `saga`（PostgreSQL） |
| キャッシュ | なし |
| メッセージング | Kafka（Saga 状態遷移イベント発行） |
| 配置パス | `regions/system/server/rust/saga/` |

> **重要**: Saga オーケストレーターは起動時に未完了 Saga の自動リカバリを実行する。デプロイ時のローリングアップデートでは、旧 Pod のグレースフルシャットダウンが完了してから新 Pod が起動するよう `maxUnavailable` を適切に設定すること。

---

## コンテナイメージ

### Dockerfile 構成

| 項目 | 詳細 |
| --- | --- |
| ビルドステージ | `rust:1.93-bookworm`（マルチステージビルド + cargo-chef） |
| ランタイムステージ | `gcr.io/distroless/cc-debian12:nonroot`（最小イメージ） |
| 追加パッケージ | `protobuf-compiler`（proto 生成）、`cmake` + `build-essential`（rdkafka ビルド） |
| libz コピー | distroless には zlib が含まれないため、ビルドステージから手動コピー |
| ビルドコマンド | `cargo build --release -p k1s0-saga-server`（ワークスペースから特定パッケージを指定） |
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
RUN cargo chef cook --release --recipe-path recipe.json -p k1s0-saga-server

# builder ステージ: ソースコードのビルド
FROM cook AS builder
COPY . .
RUN cargo build --release -p k1s0-saga-server

FROM busybox:1.36.1-musl AS busybox

# runtime ステージ: 最小実行環境
FROM gcr.io/distroless/cc-debian12:nonroot

COPY --from=builder /usr/lib/x86_64-linux-gnu/libz.so.1 /usr/lib/x86_64-linux-gnu/libz.so.1
COPY --from=builder /app/target/release/k1s0-saga-server /k1s0-saga-server
COPY --from=busybox /bin/busybox /busybox

USER nonroot:nonroot
EXPOSE 8080 50051

ENTRYPOINT ["/k1s0-saga-server"]
```

---

## 環境変数

config.yaml で設定する項目の一覧。Vault Agent Injector 経由でシークレットを注入する。

| 環境変数 / 設定キー | 説明 | デフォルト値 | 必須 |
| --- | --- | --- | --- |
| `app.name` | アプリケーション名 | `saga-server` | Yes |
| `app.version` | バージョン | `0.1.0` | Yes |
| `app.environment` | 実行環境（production / staging / dev） | `production` | Yes |
| `server.host` | バインドホスト | `0.0.0.0` | Yes |
| `server.port` | REST ポート | `8080` | Yes |
| `server.grpc_port` | gRPC ポート | `50051` | Yes |
| `auth.jwks_url` | JWKS エンドポイント URL | - | Yes |
| `auth.issuer` | JWT 発行者 | - | Yes |
| `auth.audience` | JWT オーディエンス | `k1s0-system` | Yes |
| `auth.jwks_cache_ttl_secs` | JWKS キャッシュ TTL（秒） | `300` | No |
| `database.host` | PostgreSQL ホスト | - | Yes |
| `database.port` | PostgreSQL ポート | `5432` | Yes |
| `database.name` | データベース名 | `k1s0_system` | Yes |
| `database.user` | DB ユーザー | `app` | Yes |
| `database.password` | DB パスワード（Vault 注入） | - | Yes |
| `database.ssl_mode` | SSL モード | `require` | No |
| `database.max_open_conns` | 最大接続数 | `25` | No |
| `database.max_idle_conns` | アイドル接続数 | `5` | No |
| `database.conn_max_lifetime` | 接続最大寿命 | `5m` | No |
| `kafka.brokers` | Kafka ブローカーリスト | - | Yes |
| `kafka.consumer_group` | Kafka コンシューマーグループ | `saga-server.default` | No |
| `kafka.security_protocol` | Kafka セキュリティプロトコル | `PLAINTEXT` | No |
| `kafka.topics.publish` | 発行トピック一覧 | `["k1s0.system.saga.state_changed.v1"]` | Yes |
| `services.<name>.host` | gRPC 呼び出し先ホスト | - | Yes（ステップ定義に応じて） |
| `services.<name>.port` | gRPC 呼び出し先ポート | `50051` | Yes（ステップ定義に応じて） |
| `saga.max_concurrent` | 同時実行 Saga 数上限 | `100` | No |
| `saga.workflow_dir` | ワークフロー YAML ディレクトリ | `workflows` | No |

### Vault シークレットパス

| シークレット | パス |
| --- | --- |
| DB パスワード | `secret/data/k1s0/system/saga/database` |
| Kafka SASL | `secret/data/k1s0/system/kafka/sasl` |

---

## DB マイグレーション

Saga 状態・ステップログ・ワークフロー定義は PostgreSQL の `saga` スキーマに格納する。詳細スキーマは [database.md](database.md) 参照。

### 対象テーブル

| テーブル名 | 説明 |
| --- | --- |
| `saga.saga_states` | Saga 実行状態（ステータス・ペイロード・相関 ID） |
| `saga.saga_step_logs` | ステップ実行ログ（アクション・結果・ペイロード） |
| `saga.workflow_definitions` | ワークフロー定義（YAML から永続化） |

### マイグレーション実行

```bash
# sqlx-cli によるマイグレーション実行
sqlx migrate run --source regions/system/database/saga-db/migrations/
```

### 起動時リカバリ

サーバー起動時に `RecoverSagasUseCase` が実行され、`status IN ('STARTED', 'RUNNING', 'COMPENSATING')` の未完了 Saga を自動検出して再開する。マイグレーション完了後に実行されるため、DB スキーマの整合性が保証される。

---

## ヘルスチェック

| エンドポイント | 用途 | チェック内容 |
| --- | --- | --- |
| `GET /healthz` | Liveness Probe | プロセス生存確認 |
| `GET /readyz` | Readiness Probe | PostgreSQL 接続確認 + ワークフロー定義ロード完了確認 |
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

> **補足**: Saga の同時実行数（`saga.max_concurrent: 100`）に応じて、各 Saga は `tokio::spawn` でバックグラウンド実行される。高負荷時にはメモリ消費が増加するため、同時実行数とメモリリミットのバランスを監視すること。

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

> **注意**: 各 Saga の実行は PostgreSQL トランザクションで状態が永続化されるため、Pod が再起動しても起動時リカバリで自動再開される。複数レプリカ間での Saga 実行の重複防止は DB レベルの排他制御で実現している。スケールイン時に実行中の Saga がある場合は、グレースフルシャットダウンでステップ完了を待ってから終了する。

---

## テスト方針

### レイヤー別テスト

| レイヤー | テスト種別 | ツール |
| --- | --- | --- |
| domain/entity | 単体テスト（状態遷移・バリデーション） | `#[cfg(test)]` + `assert!` |
| usecase | 単体テスト（モック） | `mockall` |
| adapter/handler | 統合テスト（HTTP/gRPC） | `axum::test` + `tokio::test` |
| adapter/repository | 統合テスト（DB） | `testcontainers` |
| infrastructure/grpc_caller | 単体テスト | `mockall`（gRPC 動的呼び出し） |
| infrastructure/kafka_producer | 単体テスト | `mockall`（Kafka イベント発行） |
| infrastructure/workflow_loader | 単体テスト | `tempfile`（YAML ファイルローダー） |

### テスト統計

合計 89 テスト（ユニットテスト）+ 4 統合テストファイル。詳細は [server.md](server.md) のテストセクション参照。

---

## Helm values

[helm設計.md](../../infrastructure/kubernetes/helm設計.md) のサーバー用 Helm Chart を使用する。Saga オーケストレーターサーバー固有の values は以下の通り。

```yaml
# values-saga.yaml
app:
  name: saga-server
  tier: system

image:
  repository: harbor.internal.example.com/k1s0/saga-server
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
  vault.hashicorp.com/agent-inject-secret-db-password: "secret/data/k1s0/system/saga/database"
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
  name: saga-server-config
  mountPath: /etc/app/config.yaml
```

---

## Kong ルーティング

[認証認可設計.md](../../architecture/auth/認証認可設計.md) の Kong ルーティング設計に従い、Saga サーバーを Kong に登録する。

```yaml
services:
  - name: saga-v1
    url: http://saga-server.k1s0-system.svc.cluster.local:80
    routes:
      - name: saga-v1-sagas-route
        paths:
          - /api/v1/sagas
        strip_path: false
      - name: saga-v1-workflows-route
        paths:
          - /api/v1/workflows
        strip_path: false
    plugins:
      - name: rate-limiting
        config:
          minute: 3000
          policy: redis
```

---

## 関連ドキュメント

- [system-saga-server.md](server.md) -- 概要・API 定義・アーキテクチャ
- [system-saga-database.md](database.md) -- データベーススキーマ・状態管理テーブル
- [Dockerイメージ戦略.md](../../infrastructure/docker/Dockerイメージ戦略.md) -- マルチステージビルド・ベースイメージ
- [helm設計.md](../../infrastructure/kubernetes/helm設計.md) -- Helm Chart・Vault Agent Injector
- [可観測性設計.md](../../architecture/observability/可観測性設計.md) -- OpenTelemetry・Prometheus・構造化ログ
- [認証認可設計.md](../../architecture/auth/認証認可設計.md) -- Kong ルーティング設計

# system-server デプロイ設計

> **ガイド**: テスト実装例・Dockerfile 設計背景は [deploy.guide.md](./deploy.guide.md) を参照。

system-server（認証サーバー）の DB マイグレーション・テスト・Dockerfile・Helm values を定義する。概要・API 定義・アーキテクチャは [system-server.md](../auth/server.md) を参照。

---

## データベースマイグレーション

監査ログテーブルは PostgreSQL に格納する。ユーザー情報は Keycloak が管理するため、認証サーバーの DB には監査ログのみを格納する。詳細スキーマは [system-database.md](database.md) 参照。

```sql
-- migrations/006_create_audit_logs.up.sql
-- auth-db: audit_logs テーブル作成（月次パーティショニング）

CREATE TABLE IF NOT EXISTS auth.audit_logs (
    id          UUID         NOT NULL DEFAULT gen_random_uuid(),
    user_id     TEXT,
    event_type  VARCHAR(100) NOT NULL,
    action      VARCHAR(100) NOT NULL,
    resource    VARCHAR(255),
    resource_id VARCHAR(255),
    result      VARCHAR(50)  NOT NULL DEFAULT 'SUCCESS',
    detail      JSONB,
    ip_address  TEXT,
    user_agent  TEXT,
    trace_id    VARCHAR(64),
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    PRIMARY KEY (id, created_at)
) PARTITION BY RANGE (created_at);

-- インデックス
CREATE INDEX IF NOT EXISTS idx_audit_logs_user_id_created_at
    ON auth.audit_logs (user_id, created_at);
CREATE INDEX IF NOT EXISTS idx_audit_logs_event_type_created_at
    ON auth.audit_logs (event_type, created_at);
CREATE INDEX IF NOT EXISTS idx_audit_logs_action_created_at
    ON auth.audit_logs (action, created_at);
CREATE INDEX IF NOT EXISTS idx_audit_logs_trace_id
    ON auth.audit_logs (trace_id) WHERE trace_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_audit_logs_resource
    ON auth.audit_logs (resource, resource_id) WHERE resource IS NOT NULL;
```

> **注意**: パーティショニングテーブルの PRIMARY KEY は `(id, created_at)` の複合キー。PostgreSQL のパーティショニングではパーティションキー（`created_at`）を PRIMARY KEY に含める必要がある。`user_id` は `TEXT` 型（FK なし）、`ip_address` は `TEXT` 型（INET ではなく IPv4/IPv6 文字列を柔軟に格納）。

---

## テスト方針

### レイヤー別テスト

| レイヤー | テスト種別 | Rust |
| --- | --- | --- |
| domain/service | 単体テスト | `#[cfg(test)]` + `assert!` |
| usecase | 単体テスト（モック） | `mockall` |
| adapter/handler | 統合テスト（HTTP/gRPC） | `axum::test` + `tokio::test` |
| adapter/gateway | 統合テスト | `mockall` + `wiremock` |
| infrastructure/persistence | 統合テスト（DB） | `testcontainers` |
| infrastructure/auth | 単体テスト | `tokio::test` |

> テスト実装例は [deploy.guide.md](./deploy.guide.md#テスト実装例) を参照。

---

## デプロイ

### Dockerfile

[Dockerイメージ戦略.md](../../infrastructure/docker/Dockerイメージ戦略.md) のテンプレートに従う。ビルドコンテキストは `regions/system`（ライブラリ依存解決のため）。

```dockerfile
# Build stage
# Note: build context must be ./regions/system (to include library dependencies)
FROM rust:1.88-bookworm AS builder

# Install protobuf compiler (for tonic-build in build.rs) and
# cmake + build-essential (for rdkafka cmake-build feature)
RUN apt-get update && apt-get install -y --no-install-recommends \
    protobuf-compiler \
    cmake \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the entire system directory to resolve path dependencies
COPY . .

RUN cargo build --release -p k1s0-auth-server

# Runtime stage
FROM gcr.io/distroless/cc-debian12:nonroot

COPY --from=builder /usr/lib/x86_64-linux-gnu/libz.so.1 /usr/lib/x86_64-linux-gnu/libz.so.1
COPY --from=builder /app/target/release/k1s0-auth-server /k1s0-auth-server

USER nonroot:nonroot
EXPOSE 8080 50051

ENTRYPOINT ["/k1s0-auth-server"]
```

> 構成のポイント（ステージ選定理由・libz コピー背景など）は [deploy.guide.md](./deploy.guide.md#dockerfile-構成のポイント) を参照。

### Helm values

[helm設計.md](../../infrastructure/kubernetes/helm設計.md) のサーバー用 Helm Chart を使用する。認証サーバー固有の values は以下の通り。

```yaml
# values-auth.yaml
app:
  name: auth-server
  tier: system

image:
  repository: harbor.internal.example.com/k1s0/auth-server
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
  vault.hashicorp.com/agent-inject-secret-db-password: "secret/data/k1s0/system/auth/database"
  vault.hashicorp.com/agent-inject-secret-oidc: "secret/data/k1s0/system/auth/oidc"
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
  name: auth-server-config
  mountPath: /etc/app/config.yaml
```

### Kong ルーティング

[認証認可設計.md](../../architecture/auth/認証認可設計.md) の Kong ルーティング設計に従い、認証サーバーを Kong に登録する。

```yaml
services:
  - name: auth-v1
    url: http://auth-server.k1s0-system.svc.cluster.local:80
    routes:
      - name: auth-v1-route
        paths:
          - /api/v1/auth
        strip_path: false
      - name: auth-v1-users-route
        paths:
          - /api/v1/users
        strip_path: false
      - name: auth-v1-audit-route
        paths:
          - /api/v1/audit
        strip_path: false
    plugins:
      - name: rate-limiting
        config:
          minute: 3000
          policy: redis
```

---

## 共通関連ドキュメント {#共通関連ドキュメント}

全 system-server 設計書で共通して参照されるドキュメント一覧。各 server.md の「関連ドキュメント」セクションでは本リストを参照し、サービス固有のリンクのみ追記する。

- [認証認可設計.md](../../architecture/auth/認証認可設計.md) -- Keycloak 設定・OAuth 2.0 フロー・RBAC 設計・Vault 戦略
- [API設計.md](../../architecture/api/API設計.md) -- REST / gRPC / GraphQL 設計・エラーレスポンス・バージョニング
- [可観測性設計.md](../../architecture/observability/可観測性設計.md) -- OpenTelemetry・Prometheus・構造化ログ
- [メッセージング設計.md](../../architecture/messaging/メッセージング設計.md) -- Kafka トピック設計・監査イベント配信
- [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md) -- サーバーテンプレート・クリーンアーキテクチャ・共通技術スタック
- [コーディング規約.md](../../architecture/conventions/コーディング規約.md) -- Linter・Formatter・命名規則
- [system-library-概要.md](../../libraries/_common/概要.md) -- ライブラリ一覧
- [tier-architecture.md](../../architecture/overview/tier-architecture.md) -- Tier アーキテクチャの詳細
- [helm設計.md](../../infrastructure/kubernetes/helm設計.md) -- Helm Chart・Vault Agent Injector
- [Dockerイメージ戦略.md](../../infrastructure/docker/Dockerイメージ戦略.md) -- マルチステージビルド・ベースイメージ

---

## 関連ドキュメント

- [system-server.md](../auth/server.md) -- 概要・API 定義・アーキテクチャ
- [system-server-implementation.md](implementation.md) -- Rust 実装詳細
- [Dockerイメージ戦略.md](../../infrastructure/docker/Dockerイメージ戦略.md) -- マルチステージビルド・ベースイメージ
- [helm設計.md](../../infrastructure/kubernetes/helm設計.md) -- Helm Chart・Vault Agent Injector
- [可観測性設計.md](../../architecture/observability/可観測性設計.md) -- OpenTelemetry・Prometheus・構造化ログ
- [認証認可設計.md](../../architecture/auth/認証認可設計.md) -- Kong ルーティング設計

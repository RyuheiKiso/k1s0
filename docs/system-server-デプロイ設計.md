# system-server デプロイ設計

system-server（認証サーバー）の DB マイグレーション・テスト・Dockerfile・Helm values を定義する。概要・API 定義・アーキテクチャは [system-server設計.md](system-server設計.md) を参照。

---

## データベースマイグレーション

監査ログテーブルは PostgreSQL に格納する。ユーザー情報は Keycloak が管理するため、認証サーバーの DB には監査ログのみを格納する。

```sql
-- migrations/006_create_audit_logs.up.sql
-- auth-db: audit_logs テーブル作成（月次パーティショニング）
-- 詳細スキーマは system-database設計.md 参照

CREATE TABLE IF NOT EXISTS auth.audit_logs (
    id          UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID         REFERENCES auth.users(id) ON DELETE SET NULL,
    event_type  VARCHAR(100) NOT NULL,
    action      VARCHAR(100) NOT NULL,
    resource    VARCHAR(255),
    resource_id VARCHAR(255),
    result      VARCHAR(50)  NOT NULL DEFAULT 'SUCCESS',
    detail      JSONB,
    ip_address  INET,
    user_agent  TEXT,
    trace_id    VARCHAR(64),
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW()
) PARTITION BY RANGE (created_at);

-- インデックス
CREATE INDEX IF NOT EXISTS idx_audit_logs_user_id_created_at
    ON auth.audit_logs (user_id, created_at);
CREATE INDEX IF NOT EXISTS idx_audit_logs_event_type_created_at
    ON auth.audit_logs (event_type, created_at);
CREATE INDEX IF NOT EXISTS idx_audit_logs_trace_id
    ON auth.audit_logs (trace_id) WHERE trace_id IS NOT NULL;
```

---

## テスト方針

### レイヤー別テスト

| レイヤー | テスト種別 | Rust |
| --- | --- | --- |
| domain/service | 単体テスト | `#[cfg(test)]` + `assert!` |
| usecase | 単体テスト（モック） | `mockall` |
| adapter/handler | 統合テスト（HTTP/gRPC） | `axum::test` + `tokio::test` |
| adapter/gateway | 統合テスト | `mockall` + `wiremock` |
| infra/persistence | 統合テスト（DB） | `testcontainers` |
| infra/auth | 単体テスト | `tokio::test` |

### テスト例

```rust
// src/usecase/validate_token.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::auth::MockTokenVerifier;
    use crate::infra::config::JwtConfig;

    #[tokio::test]
    async fn test_validate_token_success() {
        let mut mock_verifier = MockTokenVerifier::new();
        mock_verifier
            .expect_verify_token()
            .with(eq("valid-token"))
            .returning(|_| {
                Ok(TokenClaims {
                    sub: "user-uuid-1234".to_string(),
                    iss: "https://auth.k1s0.internal.example.com/realms/k1s0".to_string(),
                    aud: "k1s0-api".to_string(),
                    ..Default::default()
                })
            });

        let uc = ValidateTokenUseCase::new(
            Arc::new(mock_verifier),
            JwtConfig {
                issuer: "https://auth.k1s0.internal.example.com/realms/k1s0".to_string(),
                audience: "k1s0-api".to_string(),
                public_key_path: None,
            },
        );

        let claims = uc.execute("valid-token").await.unwrap();
        assert_eq!(claims.sub, "user-uuid-1234");
    }

    #[tokio::test]
    async fn test_validate_token_invalid_issuer() {
        let mut mock_verifier = MockTokenVerifier::new();
        mock_verifier
            .expect_verify_token()
            .returning(|_| {
                Ok(TokenClaims {
                    sub: "user-uuid-1234".to_string(),
                    iss: "https://evil.example.com/realms/k1s0".to_string(),
                    aud: "k1s0-api".to_string(),
                    ..Default::default()
                })
            });

        let uc = ValidateTokenUseCase::new(
            Arc::new(mock_verifier),
            JwtConfig {
                issuer: "https://auth.k1s0.internal.example.com/realms/k1s0".to_string(),
                audience: "k1s0-api".to_string(),
                public_key_path: None,
            },
        );

        let err = uc.execute("token-wrong-issuer").await.unwrap_err();
        assert!(matches!(err, AuthError::InvalidIssuer));
    }
}
```

### testcontainers による DB 統合テスト

```rust
// src/infra/persistence/audit_log_repository_test.rs
#[cfg(test)]
mod tests {
    use super::*;
    use testcontainers::{runners::AsyncRunner, GenericImage};

    #[tokio::test]
    async fn test_audit_log_create_and_search() {
        let container = GenericImage::new("postgres", "16-alpine")
            .with_env_var("POSTGRES_DB", "k1s0_system_test")
            .with_env_var("POSTGRES_PASSWORD", "test")
            .start()
            .await
            .unwrap();

        let port = container.get_host_port_ipv4(5432).await.unwrap();
        let pool = sqlx::PgPool::connect(
            &format!("postgresql://postgres:test@localhost:{}/k1s0_system_test", port),
        )
        .await
        .unwrap();

        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        let repo = AuditLogRepositoryImpl::new(pool);

        let log = AuditLog {
            id: uuid::Uuid::new_v4(),
            user_id: Some(uuid::Uuid::new_v4()),
            event_type: "LOGIN_SUCCESS".to_string(),
            action: "POST".to_string(),
            resource: Some("/api/v1/auth/token".to_string()),
            resource_id: None,
            result: "SUCCESS".to_string(),
            detail: None,
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: Some(String::new()),
            trace_id: None,
            created_at: chrono::Utc::now(),
        };

        repo.create(&log).await.unwrap();

        let (logs, count) = repo
            .search(&AuditLogSearchParams {
                user_id: Some("test-user".to_string()),
                page: 1,
                page_size: 10,
                ..Default::default()
            })
            .await
            .unwrap();

        assert_eq!(count, 1);
        assert_eq!(logs[0].event_type, "LOGIN_SUCCESS");
    }
}
```

---

## デプロイ

### Dockerfile

[Dockerイメージ戦略.md](Dockerイメージ戦略.md) のテンプレートに従う。

```dockerfile
# ---- Build ----
FROM rust:1.82-bookworm AS build
WORKDIR /src

# protoc のインストール（tonic-build に必要）
RUN apt-get update && apt-get install -y protobuf-compiler && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src
COPY . .
RUN cargo build --release

# ---- Runtime ----
FROM gcr.io/distroless/cc-debian12
COPY --from=build /src/target/release/auth-server /app
USER nonroot:nonroot
EXPOSE 8080 50051
ENTRYPOINT ["/app"]
```

### Helm values

[helm設計.md](helm設計.md) のサーバー用 Helm Chart を使用する。認証サーバー固有の values は以下の通り。

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

[認証認可設計.md](認証認可設計.md) の Kong ルーティング設計に従い、認証サーバーを Kong に登録する。

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

## 関連ドキュメント

- [system-server設計.md](system-server設計.md) -- 概要・API 定義・アーキテクチャ
- [system-server-実装設計.md](system-server-実装設計.md) -- Rust 実装詳細
- [Dockerイメージ戦略.md](Dockerイメージ戦略.md) -- マルチステージビルド・ベースイメージ
- [helm設計.md](helm設計.md) -- Helm Chart・Vault Agent Injector
- [可観測性設計.md](可観測性設計.md) -- OpenTelemetry・Prometheus・構造化ログ
- [認証認可設計.md](認証認可設計.md) -- Kong ルーティング設計

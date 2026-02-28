# system-server デプロイガイド

> **仕様**: テーブル定義・Helm values・Dockerfile は [deploy.md](./deploy.md) を参照。

---

## テスト実装例

### ユースケース単体テスト（mockall）

`ValidateTokenUseCase` の成功・失敗パターンをモックで検証する。

```rust
// src/usecase/validate_token.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::auth::MockTokenVerifier;
    use crate::infrastructure::config::JwtConfig;

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

`AuditLogRepositoryImpl` の create / search を実 PostgreSQL で検証する。

```rust
// src/infrastructure/persistence/audit_log_repository_test.rs
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

### Dockerfile 構成のポイント

| 項目 | 詳細 |
| --- | --- |
| ビルドステージ | `rust:1.88-bookworm`（マルチステージビルド） |
| ランタイムステージ | `gcr.io/distroless/cc-debian12:nonroot`（最小イメージ） |
| 追加パッケージ | `protobuf-compiler`（proto 生成）、`cmake` + `build-essential`（rdkafka ビルド） |
| libz コピー | distroless には zlib が含まれないため、ビルドステージから手動コピー |
| ビルドコマンド | `cargo build --release -p k1s0-auth-server`（ワークスペースから特定パッケージを指定） |
| ビルドコンテキスト | `regions/system`（`COPY . .` でシステム全体のライブラリ依存を含める） |
| 公開ポート | 8080（REST API）、50051（gRPC） |
| 実行ユーザー | `nonroot:nonroot`（セキュリティベストプラクティス） |

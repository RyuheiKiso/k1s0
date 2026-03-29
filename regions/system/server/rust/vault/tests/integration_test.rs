#![allow(clippy::unwrap_used)]
// Vault サーバーの統合テスト
// router 初期化の smoke test として、ヘルスチェックと認証なしアクセスを検証する

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

// Vault サーバーのクレートから必要な型をインポート
use k1s0_vault_server::adapter::handler::{router, AppState};
use k1s0_vault_server::domain::entity::access_log::SecretAccessLog;
use k1s0_vault_server::domain::entity::secret::Secret;
use k1s0_vault_server::domain::repository::{AccessLogRepository, SecretStore};
use k1s0_vault_server::infrastructure::kafka_producer::VaultEventPublisher;
use k1s0_vault_server::usecase;

// --- テスト用スタブ: SecretStore ---

/// テスト用のシークレットストア実装。全メソッドが空の結果を返す。
struct StubSecretStore;

#[async_trait]
impl SecretStore for StubSecretStore {
    async fn get(&self, _path: &str, _version: Option<i64>) -> anyhow::Result<Secret> {
        anyhow::bail!("not found")
    }
    async fn set(&self, _path: &str, _data: HashMap<String, String>) -> anyhow::Result<i64> {
        Ok(1)
    }
    async fn delete(&self, _path: &str, _versions: Vec<i64>) -> anyhow::Result<()> {
        Ok(())
    }
    async fn list(&self, _path_prefix: &str) -> anyhow::Result<Vec<String>> {
        Ok(vec![])
    }
    async fn exists(&self, _path: &str) -> anyhow::Result<bool> {
        Ok(false)
    }
}

// --- テスト用スタブ: AccessLogRepository ---

/// テスト用のアクセスログリポジトリ。全メソッドが空の結果を返す。
struct StubAccessLogRepo;

#[async_trait]
impl AccessLogRepository for StubAccessLogRepo {
    async fn record(&self, _log: &SecretAccessLog) -> anyhow::Result<()> {
        Ok(())
    }
    // LOW-12 監査対応: keyset ページネーションシグネチャに対応
    async fn list(
        &self,
        _after_id: Option<uuid::Uuid>,
        _limit: u32,
    ) -> anyhow::Result<(Vec<SecretAccessLog>, Option<uuid::Uuid>)> {
        Ok((vec![], None))
    }
}

// --- テスト用スタブ: VaultEventPublisher ---

/// テスト用のイベントパブリッシャー。全イベントを破棄する。
struct StubEventPublisher;

#[async_trait]
impl VaultEventPublisher for StubEventPublisher {
    async fn publish_secret_accessed(
        &self,
        _event: &k1s0_vault_server::infrastructure::kafka_producer::VaultAccessEvent,
    ) -> anyhow::Result<()> {
        Ok(())
    }
    async fn publish_secret_rotated(
        &self,
        _event: &k1s0_vault_server::infrastructure::kafka_producer::VaultSecretRotatedEvent,
    ) -> anyhow::Result<()> {
        Ok(())
    }
    async fn close(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

// --- テスト用アプリケーション構築 ---

/// テスト用の AppState を構築し、router を返すヘルパー関数。
/// 全リポジトリにスタブを使用し、認証・SPIFFE ともに無効化する。
fn make_test_app() -> axum::Router {
    let store: Arc<dyn SecretStore> = Arc::new(StubSecretStore);
    let audit: Arc<dyn AccessLogRepository> = Arc::new(StubAccessLogRepo);
    let publisher: Arc<dyn VaultEventPublisher> = Arc::new(StubEventPublisher);

    // ユースケースの構築
    let get_secret_uc = Arc::new(usecase::GetSecretUseCase::new(
        store.clone(),
        audit.clone(),
        publisher.clone(),
    ));
    let set_secret_uc = Arc::new(usecase::SetSecretUseCase::new(
        store.clone(),
        audit.clone(),
        publisher.clone(),
    ));
    let rotate_secret_uc = Arc::new(usecase::RotateSecretUseCase::new(
        get_secret_uc.clone(),
        set_secret_uc.clone(),
        publisher.clone(),
    ));
    let delete_secret_uc = Arc::new(usecase::DeleteSecretUseCase::new(
        store.clone(),
        audit.clone(),
        publisher.clone(),
    ));
    let list_secrets_uc = Arc::new(usecase::ListSecretsUseCase::new(store.clone()));
    let list_audit_logs_uc = Arc::new(usecase::ListAuditLogsUseCase::new(audit.clone()));

    // AppState の構築（認証なし、SPIFFE なし）
    let state = AppState {
        get_secret_uc,
        set_secret_uc,
        rotate_secret_uc,
        delete_secret_uc,
        list_secrets_uc,
        list_audit_logs_uc,
        db_pool: None,
        metrics: Arc::new(k1s0_telemetry::metrics::Metrics::new(
            "k1s0-vault-server-test",
        )),
        auth_state: None,
        spiffe_state: None,
    };

    router(state)
}

// --- ヘルスチェックテスト ---

/// /healthz と /readyz エンドポイントが 200 OK を返すことを確認する
#[tokio::test]
async fn test_healthz_and_readyz() {
    let app = make_test_app();

    // /healthz へのリクエスト
    let req = Request::builder()
        .uri("/healthz")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // /readyz へのリクエスト
    let req = Request::builder()
        .uri("/readyz")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

// --- 認証なしアクセステスト ---

/// 認証が無効な状態で保護エンドポイントにアクセスすると正常にルーティングされることを確認する。
/// auth_state が None の場合、認証ミドルウェアはスキップされる。
#[tokio::test]
async fn test_api_routes_are_reachable() {
    let app = make_test_app();

    // 認証なしモードでは /api/v1/secrets にアクセスできる
    let req = Request::builder()
        .uri("/api/v1/secrets")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    // ルーターが正常に応答すること（500 でないこと）を確認
    assert_ne!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

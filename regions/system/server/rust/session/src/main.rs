#![allow(dead_code, unused_imports)]

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use tracing::info;

mod adapter;
mod domain;
mod error;
mod infrastructure;
mod usecase;

use adapter::grpc::SessionGrpcService;
use domain::entity::session::Session;
use domain::repository::SessionRepository;
use error::SessionError;
use infrastructure::config::Config;

// --- InMemory Repository ---

struct InMemorySessionRepository {
    sessions: tokio::sync::RwLock<HashMap<String, Session>>,
}

impl InMemorySessionRepository {
    fn new() -> Self {
        Self {
            sessions: tokio::sync::RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl SessionRepository for InMemorySessionRepository {
    async fn save(&self, session: &Session) -> Result<(), SessionError> {
        let mut sessions = self.sessions.write().await;
        sessions.insert(session.id.clone(), session.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<Session>, SessionError> {
        let sessions = self.sessions.read().await;
        Ok(sessions.get(id).cloned())
    }

    async fn find_by_token(&self, token: &str) -> Result<Option<Session>, SessionError> {
        let sessions = self.sessions.read().await;
        Ok(sessions.values().find(|s| s.token == token).cloned())
    }

    async fn find_by_user_id(&self, user_id: &str) -> Result<Vec<Session>, SessionError> {
        let sessions = self.sessions.read().await;
        Ok(sessions
            .values()
            .filter(|s| s.user_id == user_id)
            .cloned()
            .collect())
    }

    async fn delete(&self, id: &str) -> Result<(), SessionError> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(id);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("info".parse().unwrap()),
        )
        .json()
        .init();

    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config/config.yaml".to_string());
    let cfg = Config::load(&config_path)?;

    info!(port = cfg.server.port, "starting session server");

    let repo: Arc<dyn SessionRepository> = Arc::new(InMemorySessionRepository::new());

    let create_uc = Arc::new(usecase::CreateSessionUseCase::new(
        repo.clone(),
        cfg.session.default_ttl_seconds,
        cfg.session.max_ttl_seconds,
    ));
    let get_uc = Arc::new(usecase::GetSessionUseCase::new(repo.clone()));
    let refresh_uc = Arc::new(usecase::RefreshSessionUseCase::new(
        repo.clone(),
        cfg.session.max_ttl_seconds,
    ));
    let revoke_uc = Arc::new(usecase::RevokeSessionUseCase::new(repo.clone()));
    let list_uc = Arc::new(usecase::ListUserSessionsUseCase::new(repo.clone()));
    let revoke_all_uc = Arc::new(usecase::RevokeAllSessionsUseCase::new(repo));

    let _grpc_svc = Arc::new(SessionGrpcService::new(
        create_uc.clone(),
        get_uc.clone(),
        refresh_uc.clone(),
        revoke_uc.clone(),
        revoke_all_uc.clone(),
        list_uc.clone(),
        cfg.session.default_ttl_seconds,
    ));

    let state = adapter::handler::session_handler::AppState {
        create_uc,
        get_uc,
        refresh_uc,
        revoke_uc,
        list_uc,
        revoke_all_uc,
    };

    let app = axum::Router::new()
        .route("/healthz", axum::routing::get(adapter::handler::health::healthz))
        .route("/readyz", axum::routing::get(adapter::handler::health::readyz))
        .route(
            "/api/v1/sessions",
            axum::routing::post(adapter::handler::session_handler::create_session),
        )
        .route(
            "/api/v1/sessions/:id",
            axum::routing::get(adapter::handler::session_handler::get_session)
                .delete(adapter::handler::session_handler::revoke_session),
        )
        .route(
            "/api/v1/sessions/:id/refresh",
            axum::routing::post(adapter::handler::session_handler::refresh_session),
        )
        .route(
            "/api/v1/users/:user_id/sessions",
            axum::routing::get(adapter::handler::session_handler::list_user_sessions)
                .delete(adapter::handler::session_handler::revoke_all_sessions),
        )
        .with_state(state);

    let grpc_addr = SocketAddr::from(([0, 0, 0, 0], 9090));
    info!("gRPC server starting on {}", grpc_addr);

    let addr = SocketAddr::from(([0, 0, 0, 0], cfg.server.port));
    info!("REST server starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

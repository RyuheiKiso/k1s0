use std::net::SocketAddr;
use std::sync::Arc;

use tracing::info;

mod adapter;
mod domain;
mod infrastructure;
mod usecase;

use adapter::handler::{self, AppState};
use infrastructure::config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Logger
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .json()
        .init();

    // Config
    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config/config.yaml".to_string());
    let cfg = Config::load(&config_path)?;

    info!(
        app_name = %cfg.app.name,
        version = %cfg.app.version,
        environment = %cfg.app.environment,
        "starting config server"
    );

    // Config repository (in-memory for dev, PostgreSQL for prod)
    let config_repo: Arc<dyn domain::repository::ConfigRepository> =
        Arc::new(InMemoryConfigRepository::new());

    // AppState
    let state = AppState::new(config_repo);

    // Router
    let app = handler::router(state);

    // Server
    let addr = SocketAddr::from(([0, 0, 0, 0], cfg.server.port));
    info!("REST server starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;

    // Graceful shutdown
    let shutdown_signal = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install CTRL+C signal handler");
        info!("shutdown signal received, starting graceful shutdown");
    };

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal)
        .await?;

    info!("config server stopped");

    Ok(())
}

// --- In-memory implementation for dev mode ---

use domain::entity::config_change_log::ConfigChangeLog;
use domain::entity::config_entry::{
    ConfigEntry, ConfigListResult, Pagination, ServiceConfigEntry, ServiceConfigResult,
};
use tokio::sync::RwLock;
use uuid::Uuid;

/// InMemoryConfigRepository は開発用のインメモリ設定リポジトリ。
struct InMemoryConfigRepository {
    entries: RwLock<Vec<ConfigEntry>>,
}

impl InMemoryConfigRepository {
    fn new() -> Self {
        Self {
            entries: RwLock::new(Vec::new()),
        }
    }
}

#[async_trait::async_trait]
impl domain::repository::ConfigRepository for InMemoryConfigRepository {
    async fn find_by_namespace_and_key(
        &self,
        namespace: &str,
        key: &str,
    ) -> anyhow::Result<Option<ConfigEntry>> {
        let entries = self.entries.read().await;
        Ok(entries
            .iter()
            .find(|e| e.namespace == namespace && e.key == key)
            .cloned())
    }

    async fn list_by_namespace(
        &self,
        namespace: &str,
        page: i32,
        page_size: i32,
        search: Option<String>,
    ) -> anyhow::Result<ConfigListResult> {
        let entries = self.entries.read().await;
        let mut filtered: Vec<_> = entries
            .iter()
            .filter(|e| {
                if e.namespace != namespace {
                    return false;
                }
                if let Some(ref s) = search {
                    if !e.key.contains(s) {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();

        let total_count = filtered.len() as i64;
        let offset = ((page - 1) * page_size) as usize;
        let limit = page_size as usize;

        filtered = filtered.into_iter().skip(offset).take(limit).collect();
        let has_next = (offset + limit) < total_count as usize;

        Ok(ConfigListResult {
            entries: filtered,
            pagination: Pagination {
                total_count,
                page,
                page_size,
                has_next,
            },
        })
    }

    async fn create(&self, entry: &ConfigEntry) -> anyhow::Result<ConfigEntry> {
        let mut entries = self.entries.write().await;
        entries.push(entry.clone());
        Ok(entry.clone())
    }

    async fn update(
        &self,
        namespace: &str,
        key: &str,
        value_json: &serde_json::Value,
        expected_version: i32,
        description: Option<String>,
        updated_by: &str,
    ) -> anyhow::Result<ConfigEntry> {
        let mut entries = self.entries.write().await;
        let entry = entries
            .iter_mut()
            .find(|e| e.namespace == namespace && e.key == key);

        match entry {
            Some(e) => {
                if e.version != expected_version {
                    return Err(anyhow::anyhow!(
                        "version conflict: current={}",
                        e.version
                    ));
                }
                e.value_json = value_json.clone();
                e.version += 1;
                if let Some(desc) = description {
                    e.description = Some(desc);
                }
                e.updated_by = updated_by.to_string();
                e.updated_at = chrono::Utc::now();
                Ok(e.clone())
            }
            None => Err(anyhow::anyhow!(
                "config not found: {}/{}",
                namespace,
                key
            )),
        }
    }

    async fn delete(&self, namespace: &str, key: &str) -> anyhow::Result<bool> {
        let mut entries = self.entries.write().await;
        let len_before = entries.len();
        entries.retain(|e| !(e.namespace == namespace && e.key == key));
        Ok(entries.len() < len_before)
    }

    async fn find_by_service_name(
        &self,
        service_name: &str,
    ) -> anyhow::Result<ServiceConfigResult> {
        let entries = self.entries.read().await;
        // サービス名からキーワードを抽出してマッチング（開発用の簡易実装）
        // 本番では service_config_mappings テーブルによる明示的マッピングを使用
        // 例: "auth-server" -> ["auth", "server"] -> namespace に含まれるかチェック
        let primary_keyword = service_name.split('-').next().unwrap_or(service_name);
        let matched: Vec<ServiceConfigEntry> = entries
            .iter()
            .filter(|e| {
                e.namespace
                    .split('.')
                    .any(|ns_part| ns_part == primary_keyword)
            })
            .map(|e| ServiceConfigEntry {
                namespace: e.namespace.clone(),
                key: e.key.clone(),
                value: e.value_json.clone(),
            })
            .collect();

        if matched.is_empty() {
            return Err(anyhow::anyhow!("service not found: {}", service_name));
        }

        Ok(ServiceConfigResult {
            service_name: service_name.to_string(),
            entries: matched,
        })
    }

    async fn record_change_log(&self, _log: &ConfigChangeLog) -> anyhow::Result<()> {
        // In-memory: ログは捨てる（開発用）
        Ok(())
    }

    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<ConfigEntry>> {
        let entries = self.entries.read().await;
        Ok(entries.iter().find(|e| e.id == *id).cloned())
    }
}

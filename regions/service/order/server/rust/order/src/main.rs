use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tracing::info;

use k1s0_order_server::adapter;
use k1s0_order_server::domain;
use k1s0_order_server::infrastructure;
use k1s0_order_server::usecase;

use adapter::handler::{self, AppState};
use infrastructure::config::{Config, DatabaseConfig};
use k1s0_order_server::MIGRATOR;
use k1s0_server_common::middleware::auth_middleware::AuthState;
use k1s0_server_common::middleware::shutdown::shutdown_signal;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Configuration
    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config/default.yaml".to_string());
    let cfg = Config::load(&config_path)?;

    // 2. Telemetry
    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-order-server".to_string(),
        version: "0.1.0".to_string(),
        tier: "service".to_string(),
        environment: cfg.app.environment.clone(),
        trace_endpoint: cfg
            .observability
            .trace
            .enabled
            .then(|| cfg.observability.trace.endpoint.clone()),
        sample_rate: cfg.observability.trace.sample_rate,
        log_level: cfg.observability.log.level.clone(),
        log_format: cfg.observability.log.format.clone(),
    };
    k1s0_telemetry::init_telemetry(&telemetry_cfg).expect("failed to init telemetry");

    info!("starting {}", cfg.app.name);

    // 3. Database
    let db_cfg = cfg
        .database
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("database configuration is required"))?;
    let db_pool = connect_database(db_cfg).await?;
    MIGRATOR.run(&db_pool).await?;
    info!(
        schema = %db_cfg.schema,
        "database connected and migrations applied"
    );

    // 4. Metrics
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("order"));

    // 5. Repository
    let order_repo: Arc<dyn domain::repository::order_repository::OrderRepository> = Arc::new(
        infrastructure::database::order_repository::OrderPostgresRepository::new(db_pool.clone()),
    );

    // 6. Kafka Producer (optional)
    let event_publisher: Arc<dyn usecase::event_publisher::OrderEventPublisher> =
        if let Some(ref kafka_cfg) = cfg.kafka {
            match infrastructure::kafka::order_producer::OrderKafkaProducer::new(kafka_cfg) {
                Ok(producer) => {
                    info!("kafka producer initialized");
                    Arc::new(producer)
                }
                Err(e) => {
                    tracing::warn!("failed to initialize kafka producer: {}", e);
                    Arc::new(usecase::event_publisher::NoopOrderEventPublisher)
                }
            }
        } else {
            Arc::new(usecase::event_publisher::NoopOrderEventPublisher)
        };

    // 7. Use Cases
    let create_order_uc = Arc::new(usecase::create_order::CreateOrderUseCase::new(
        order_repo.clone(),
        event_publisher.clone(),
    ));
    let get_order_uc = Arc::new(usecase::get_order::GetOrderUseCase::new(
        order_repo.clone(),
    ));
    let update_order_status_uc =
        Arc::new(usecase::update_order_status::UpdateOrderStatusUseCase::new(
            order_repo.clone(),
            event_publisher.clone(),
        ));
    let list_orders_uc = Arc::new(usecase::list_orders::ListOrdersUseCase::new(
        order_repo.clone(),
    ));

    // 8. Auth
    let auth_state = if let Some(ref auth_cfg) = cfg.auth {
        let verifier = Arc::new(k1s0_auth::JwksVerifier::new(
            &auth_cfg.jwks_url,
            &auth_cfg.issuer,
            &auth_cfg.audience,
            std::time::Duration::from_secs(auth_cfg.jwks_cache_ttl_secs),
        ));
        Some(AuthState { verifier })
    } else {
        None
    };

    // 9. AppState + Router
    let state = AppState {
        create_order_uc,
        get_order_uc,
        update_order_status_uc,
        list_orders_uc,
        metrics,
        auth_state: auth_state.clone(),
    };
    let app = handler::router(state);

    // 10. Start REST server
    let rest_addr: SocketAddr = format!("{}:{}", cfg.server.host, cfg.server.port).parse()?;
    info!("REST server listening on {}", rest_addr);
    let listener = tokio::net::TcpListener::bind(rest_addr).await?;

    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = shutdown_signal().await;
        })
        .await?;

    Ok(())
}

async fn connect_database(db_cfg: &DatabaseConfig) -> anyhow::Result<sqlx::PgPool> {
    let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| db_cfg.connection_url());
    let lifetime = Duration::from_secs(db_cfg.conn_max_lifetime);
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(db_cfg.max_connections)
        .min_connections(db_cfg.max_idle_conns.min(db_cfg.max_connections))
        .idle_timeout(Some(lifetime))
        .max_lifetime(Some(lifetime))
        .connect(&url)
        .await?;
    Ok(pool)
}

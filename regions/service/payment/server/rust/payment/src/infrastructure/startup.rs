use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tracing::info;

use crate::adapter;
use crate::domain;
use crate::infrastructure;
use crate::usecase;

use crate::adapter::handler::{self, AppState};
use super::config::{Config, DatabaseConfig};
use crate::MIGRATOR;
use k1s0_server_common::middleware::auth_middleware::AuthState;
use k1s0_server_common::middleware::grpc_auth::GrpcAuthLayer;
use k1s0_server_common::middleware::rbac::Tier;
use k1s0_server_common::middleware::shutdown::shutdown_signal;
use tonic::transport::Server;

pub async fn run() -> anyhow::Result<()> {
    // 1. Configuration
    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config/default.yaml".to_string());
    let cfg = Config::load(&config_path)?;

    // 2. Telemetry
    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-payment-server".to_string(),
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
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("payment"));

    // 5. Repository
    let payment_repo: Arc<dyn domain::repository::payment_repository::PaymentRepository> =
        Arc::new(
            infrastructure::database::payment_repository::PaymentPostgresRepository::new(
                db_pool.clone(),
            ),
        );

    // 6. Kafka Producer (optional) -- now used only by the OutboxPoller
    let event_publisher: Arc<dyn usecase::event_publisher::PaymentEventPublisher> =
        if let Some(ref kafka_cfg) = cfg.kafka {
            match infrastructure::kafka::payment_producer::PaymentKafkaProducer::new(kafka_cfg) {
                Ok(producer) => {
                    info!("kafka producer initialized");
                    Arc::new(producer)
                }
                Err(e) => {
                    tracing::warn!("failed to initialize kafka producer: {}", e);
                    Arc::new(usecase::event_publisher::NoopPaymentEventPublisher)
                }
            }
        } else {
            Arc::new(usecase::event_publisher::NoopPaymentEventPublisher)
        };

    // 7. Outbox Poller -- バックグラウンドタスクとして起動
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let outbox_poller = Arc::new(super::outbox_poller::OutboxPoller::new(
        payment_repo.clone(),
        event_publisher.clone(),
        Duration::from_secs(5),
        100,
    ));
    let outbox_poller_clone = outbox_poller.clone();
    let outbox_handle = tokio::spawn(async move {
        outbox_poller_clone.run(shutdown_rx).await;
    });
    info!("outbox poller started");

    // 8. Use Cases
    let initiate_payment_uc = Arc::new(usecase::initiate_payment::InitiatePaymentUseCase::new(
        payment_repo.clone(),
    ));
    let get_payment_uc = Arc::new(usecase::get_payment::GetPaymentUseCase::new(
        payment_repo.clone(),
    ));
    let list_payments_uc = Arc::new(usecase::list_payments::ListPaymentsUseCase::new(
        payment_repo.clone(),
    ));
    let complete_payment_uc = Arc::new(usecase::complete_payment::CompletePaymentUseCase::new(
        payment_repo.clone(),
    ));
    let fail_payment_uc = Arc::new(usecase::fail_payment::FailPaymentUseCase::new(
        payment_repo.clone(),
    ));
    let refund_payment_uc = Arc::new(usecase::refund_payment::RefundPaymentUseCase::new(
        payment_repo.clone(),
    ));

    // 9. Auth
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

    // 10. AppState + Router
    let state = AppState {
        initiate_payment_uc: initiate_payment_uc.clone(),
        get_payment_uc: get_payment_uc.clone(),
        list_payments_uc: list_payments_uc.clone(),
        complete_payment_uc: complete_payment_uc.clone(),
        fail_payment_uc: fail_payment_uc.clone(),
        refund_payment_uc: refund_payment_uc.clone(),
        metrics: metrics.clone(),
        auth_state: auth_state.clone(),
        db_pool: Some(db_pool.clone()),
    };
    let app = handler::router(state);

    // 11. gRPC Service
    let grpc_service = adapter::grpc::payment_grpc::PaymentGrpcService::new(
        initiate_payment_uc,
        get_payment_uc,
        list_payments_uc,
        complete_payment_uc,
        fail_payment_uc,
        refund_payment_uc,
    );
    let grpc_addr: SocketAddr = format!("{}:{}", cfg.server.host, cfg.server.grpc_port).parse()?;
    info!("gRPC server listening on {}", grpc_addr);
    let grpc_metrics = metrics.clone();
    let grpc_auth_layer = GrpcAuthLayer::new(auth_state.clone(), Tier::Service, required_action);
    let (shutdown_grpc_tx, shutdown_grpc_rx) = tokio::sync::watch::channel(false);
    let mut rest_shutdown_rx = shutdown_grpc_rx.clone();
    let mut grpc_shutdown_rx = shutdown_grpc_rx.clone();
    let grpc_future = async move {
        use crate::proto::k1s0::service::payment::v1::payment_service_server::PaymentServiceServer;

        Server::builder()
            .layer(grpc_auth_layer)
            .layer(k1s0_telemetry::GrpcMetricsLayer::new(grpc_metrics))
            .add_service(PaymentServiceServer::new(grpc_service))
            .serve_with_shutdown(grpc_addr, async move {
                let _ = grpc_shutdown_rx.changed().await;
            })
            .await
            .map_err(|e| anyhow::anyhow!("gRPC server error: {}", e))
    };

    // 12. Start REST server
    let rest_addr: SocketAddr = format!("{}:{}", cfg.server.host, cfg.server.port).parse()?;
    info!("REST server listening on {}", rest_addr);
    let listener = tokio::net::TcpListener::bind(rest_addr).await?;
    let rest_future = axum::serve(listener, app).with_graceful_shutdown(async move {
        let _ = rest_shutdown_rx.changed().await;
    });

    let shutdown_future = async move {
        shutdown_signal().await.map_err(|e| anyhow::anyhow!("{}", e))?;
        let _ = shutdown_grpc_tx.send(true);
        let _ = shutdown_tx.send(true);
        Ok::<(), anyhow::Error>(())
    };

    tokio::select! {
        result = shutdown_future => {
            result?;
        }
        result = rest_future => {
            if let Err(e) = result {
                return Err(anyhow::anyhow!("REST server error: {}", e));
            }
        }
        result = grpc_future => {
            result?;
        }
    }

    // 13. Graceful shutdown -- Outbox Poller を停止
    info!("shutting down outbox poller");
    let _ = outbox_handle.await;

    Ok(())
}

/// gRPC メソッド名 -> 必要なアクションのマッピング（payment 固有）。
fn required_action(method: &str) -> &'static str {
    match method {
        "GetPayment" | "ListPayments" => "read",
        _ => "write",
    }
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

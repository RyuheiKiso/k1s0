use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tracing::info;

use crate::adapter;
use crate::domain;
use crate::infrastructure;
use crate::usecase;

use super::config::{Config, DatabaseConfig};
use crate::adapter::handler::{self, AppState};
use crate::MIGRATOR;
use k1s0_server_common::middleware::auth_middleware::AuthState;
use k1s0_server_common::middleware::grpc_auth::GrpcAuthLayer;
use k1s0_server_common::middleware::rbac::Tier;
use k1s0_server_common::shutdown::shutdown_signal;
use anyhow::Context;
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
    // テレメトリ初期化: 失敗時はサーバーを起動しない（R-04 対応）。
    // オブザーバビリティは本番環境で必須のため、初期化失敗は即時エラーとして扱う。
    // Box<dyn Error>（非Send/Sync）は anyhow::Context が使えないため map_err を使用する。
    k1s0_telemetry::init_telemetry(&telemetry_cfg)
        .map_err(|e| anyhow::anyhow!("テレメトリ初期化に失敗しました: {}", e))?;

    info!("starting {}", cfg.app.name);

    // 3. Database
    let db_cfg = cfg
        .database
        .as_ref()
        .context("データベース設定が必要です")?;
    let db_pool = connect_database(db_cfg).await?;
    // R-05 対応: PostgreSQL advisory lock でマイグレーション競合を防止する。
    // 複数のインスタンスが同時に起動した場合も、マイグレーションが重複して実行されないようにする。
    // advisory lock はセッションレベルのため、接続終了時（クラッシュ含む）に自動解放される。
    // ロック ID 1000000003 は payment サービス専用（サービス間衝突防止のため固定値を使用）。
    {
        let mut migration_conn = db_pool.acquire().await
            .context("advisory lock 取得用接続の確保に失敗")?;
        sqlx::query("SELECT pg_advisory_lock(1000000003)")
            .execute(&mut *migration_conn)
            .await
            .context("マイグレーション用 advisory lock の取得に失敗")?;
        let migrate_result = MIGRATOR.run(&db_pool).await;
        sqlx::query("SELECT pg_advisory_unlock(1000000003)")
            .execute(&mut *migration_conn)
            .await
            .context("advisory lock の解放に失敗")?;
        migrate_result.context("マイグレーションの実行に失敗")?;
    }

    // search_path が正しく設定されていることを起動時に検証する（fail-fast）。
    // 接続 URL の options=-c search_path=<schema> が有効でない場合、
    // runtime SQL が public スキーマを参照して全 CRUD が失敗するため、ここで即座に停止する。
    let actual_search_path: String = sqlx::query_scalar("SHOW search_path")
        .fetch_one(&db_pool)
        .await
        .context("failed to verify search_path")?;
    if !actual_search_path.contains(db_cfg.schema.as_str()) {
        anyhow::bail!(
            "search_path mismatch: expected schema '{}' but got '{}'. \
             Check DATABASE_URL options=-c search_path={}",
            db_cfg.schema,
            actual_search_path,
            db_cfg.schema
        );
    }
    info!(
        schema = %db_cfg.schema,
        search_path = %actual_search_path,
        "database connected, migrations applied, search_path verified"
    );

    // 4. Metrics
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("payment"));

    // 5. Repository
    let payment_repo: Arc<dyn domain::repository::payment_repository::PaymentRepository> = Arc::new(
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

    // 7. Outbox Poller — k1s0-outbox の汎用ポーラーをバックグラウンドタスクとして起動
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let outbox_poller = Arc::new(super::outbox_poller::new_outbox_poller(
        payment_repo.clone(),
        event_publisher.clone(),
        Duration::from_secs(5),
        100,
    ));
    let outbox_shutdown_rx = shutdown_rx.clone();
    let outbox_poller_clone = outbox_poller.clone();
    let outbox_handle = tokio::spawn(async move {
        outbox_poller_clone.run(outbox_shutdown_rx).await;
    });
    info!("outbox poller started");

    // 7-b. Saga Consumer — order.created と order.cancelled を購読して決済を開始・中断する（C-001 / M-20）
    let initiate_payment_uc_for_consumer = Arc::new(usecase::initiate_payment::InitiatePaymentUseCase::new(
        payment_repo.clone(),
    ));
    let fail_payment_uc_for_consumer = Arc::new(usecase::fail_payment::FailPaymentUseCase::new(
        payment_repo.clone(),
    ));
    let consumer_handle = if let Some(ref kafka_cfg) = cfg.kafka {
        let handle_order_event_uc = Arc::new(
            usecase::handle_order_event::HandleOrderEventUseCase::new(
                initiate_payment_uc_for_consumer,
                fail_payment_uc_for_consumer,
                payment_repo.clone(),
            ),
        );
        match infrastructure::kafka::payment_consumer::PaymentKafkaConsumer::new(
            kafka_cfg,
            handle_order_event_uc,
        ) {
            Ok(consumer) => {
                let consumer_shutdown_rx = shutdown_rx.clone();
                let handle = tokio::spawn(async move {
                    consumer.run(consumer_shutdown_rx).await;
                });
                info!("payment kafka consumer started");
                Some(handle)
            }
            Err(e) => {
                tracing::warn!("failed to initialize payment kafka consumer: {}", e);
                None
            }
        }
    } else {
        None
    };

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
        let verifier = Arc::new(
            k1s0_auth::JwksVerifier::new(
                &auth_cfg.jwks_url,
                &auth_cfg.issuer,
                &auth_cfg.audience,
                std::time::Duration::from_secs(auth_cfg.jwks_cache_ttl_secs),
            )
            .expect("Failed to create JWKS verifier"),
        );
        Some(AuthState { verifier })
    } else {
        None
    };
    // 本番環境では認証設定必須チェック: auth 設定がない場合は起動を拒否する
    let auth_state = k1s0_server_common::auth::require_auth_state(
        "payment", &cfg.app.environment, auth_state
    )?;

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
    // REST router に MetricsLayer と CorrelationLayer を追加する（R-01 対応）。
    // file サーバーと同様にオブザーバビリティ・Correlation ID を全 REST エンドポイントで有効化する。
    let app = handler::router(state)
        .layer(k1s0_telemetry::MetricsLayer::new(metrics.clone()))
        .layer(k1s0_correlation::layer::CorrelationLayer::new());

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
            .context("gRPC サーバーエラー")
    };

    // 12. Start REST server
    let rest_addr: SocketAddr = format!("{}:{}", cfg.server.host, cfg.server.port).parse()?;
    info!("REST server listening on {}", rest_addr);
    let listener = tokio::net::TcpListener::bind(rest_addr).await?;
    let rest_future = axum::serve(listener, app).with_graceful_shutdown(async move {
        let _ = rest_shutdown_rx.changed().await;
    });

    let shutdown_future = async move {
        shutdown_signal()
            .await
            .map_err(|e| anyhow::anyhow!("シャットダウンシグナルの待機中にエラーが発生しました: {}", e))?;
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
                return Err(anyhow::Error::from(e).context("REST サーバーエラー"));
            }
        }
        result = grpc_future => {
            result?;
        }
    }

    // 13. Graceful shutdown -- Outbox Poller と Kafka Consumer を停止
    info!("shutting down outbox poller and kafka consumer");
    let _ = outbox_handle.await;
    if let Some(handle) = consumer_handle {
        let _ = handle.await;
    }

    // R-06 対応: テレメトリのグレースフルシャットダウンを実行する。
    // バッファに残っているトレーススパンや指標をエクスポーターにフラッシュして確実に送信する。
    k1s0_telemetry::shutdown();
    info!("telemetry flushed");

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

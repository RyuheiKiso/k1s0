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
        service_name: "k1s0-inventory-server".to_string(),
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
    k1s0_telemetry::init_telemetry(&telemetry_cfg)
        .map_err(|e| anyhow::anyhow!("テレメトリ初期化に失敗しました: {}", e))?;

    info!("starting {}", cfg.app.name);

    // 3. Database
    let db_cfg = cfg
        .database
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("database configuration is required"))?;
    let db_pool = connect_database(db_cfg).await?;
    // R-05 対応: PostgreSQL advisory lock でマイグレーション競合を防止する。
    // 複数のインスタンスが同時に起動した場合も、マイグレーションが重複して実行されないようにする。
    // advisory lock はセッションレベルのため、接続終了時（クラッシュ含む）に自動解放される。
    // ロック ID 1000000001 は inventory サービス専用（サービス間衝突防止のため固定値を使用）。
    {
        let mut migration_conn = db_pool.acquire().await
            .context("advisory lock 取得用接続の確保に失敗")?;
        sqlx::query("SELECT pg_advisory_lock(1000000001)")
            .execute(&mut *migration_conn)
            .await
            .context("マイグレーション用 advisory lock の取得に失敗")?;
        let migrate_result = MIGRATOR.run(&db_pool).await;
        sqlx::query("SELECT pg_advisory_unlock(1000000001)")
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
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("inventory"));

    // 5. Repository
    let inventory_repo: Arc<dyn domain::repository::inventory_repository::InventoryRepository> =
        Arc::new(
            infrastructure::database::inventory_repository::InventoryPostgresRepository::new(
                db_pool.clone(),
            ),
        );

    // 6. Kafka Producer (optional) — used only by the OutboxPoller
    let event_publisher: Arc<dyn usecase::event_publisher::InventoryEventPublisher> =
        if let Some(ref kafka_cfg) = cfg.kafka {
            match infrastructure::kafka::inventory_producer::InventoryKafkaProducer::new(kafka_cfg)
            {
                Ok(producer) => {
                    info!("kafka producer initialized");
                    Arc::new(producer)
                }
                Err(e) => {
                    tracing::warn!("failed to initialize kafka producer: {}", e);
                    Arc::new(usecase::event_publisher::NoopInventoryEventPublisher)
                }
            }
        } else {
            Arc::new(usecase::event_publisher::NoopInventoryEventPublisher)
        };

    // 7. Outbox Poller — k1s0-outbox の汎用ポーラーをバックグラウンドタスクとして起動
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let outbox_poller = Arc::new(super::outbox_poller::new_outbox_poller(
        inventory_repo.clone(),
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
    let reserve_stock_uc = Arc::new(usecase::reserve_stock::ReserveStockUseCase::new(
        inventory_repo.clone(),
    ));
    let release_stock_uc = Arc::new(usecase::release_stock::ReleaseStockUseCase::new(
        inventory_repo.clone(),
    ));
    let get_inventory_uc = Arc::new(usecase::get_inventory::GetInventoryUseCase::new(
        inventory_repo.clone(),
    ));
    let list_inventory_uc = Arc::new(usecase::list_inventory::ListInventoryUseCase::new(
        inventory_repo.clone(),
    ));
    let update_stock_uc = Arc::new(usecase::update_stock::UpdateStockUseCase::new(
        inventory_repo.clone(),
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
        "inventory", &cfg.app.environment, auth_state
    )?;

    // 10. AppState + Router
    let state = AppState {
        reserve_stock_uc: reserve_stock_uc.clone(),
        release_stock_uc: release_stock_uc.clone(),
        get_inventory_uc: get_inventory_uc.clone(),
        list_inventory_uc: list_inventory_uc.clone(),
        update_stock_uc: update_stock_uc.clone(),
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
    let grpc_service = adapter::grpc::inventory_grpc::InventoryGrpcService::new(
        reserve_stock_uc,
        release_stock_uc,
        get_inventory_uc,
        list_inventory_uc,
        update_stock_uc,
    );
    let grpc_addr: SocketAddr = format!("{}:{}", cfg.server.host, cfg.server.grpc_port).parse()?;
    info!("gRPC server listening on {}", grpc_addr);
    let grpc_metrics = metrics.clone();
    let grpc_auth_layer = GrpcAuthLayer::new(auth_state.clone(), Tier::Service, required_action);
    let (shutdown_grpc_tx, shutdown_grpc_rx) = tokio::sync::watch::channel(false);
    let mut rest_shutdown_rx = shutdown_grpc_rx.clone();
    let mut grpc_shutdown_rx = shutdown_grpc_rx.clone();
    let grpc_future = async move {
        use crate::proto::k1s0::service::inventory::v1::inventory_service_server::InventoryServiceServer;

        Server::builder()
            .layer(grpc_auth_layer)
            .layer(k1s0_telemetry::GrpcMetricsLayer::new(grpc_metrics))
            .add_service(InventoryServiceServer::new(grpc_service))
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
        shutdown_signal()
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
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

    // 13. Graceful shutdown — Outbox Poller を停止
    info!("shutting down outbox poller");
    let _ = outbox_handle.await;

    // R-06 対応: テレメトリのグレースフルシャットダウンを実行する。
    // バッファに残っているトレーススパンや指標をエクスポーターにフラッシュして確実に送信する。
    k1s0_telemetry::shutdown();
    info!("telemetry flushed");

    Ok(())
}

/// gRPC メソッド名 → 必要なアクションのマッピング（inventory 固有）。
fn required_action(method: &str) -> &'static str {
    match method {
        "GetInventory" | "ListInventory" => "read",
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

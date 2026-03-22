use anyhow::Context;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use tracing::info;

use k1s0_server_common::middleware::grpc_auth::GrpcAuthLayer;
use k1s0_server_common::middleware::rbac::Tier;

use super::config::Config;
use super::in_memory::{InMemoryFileMetadataRepository, InMemoryFileStorageRepository};
use super::local_fs_storage::LocalFsStorageRepository;
use super::kafka_producer::{FileEventPublisher, FileKafkaProducer, NoopFileEventPublisher};
use crate::domain::repository::{FileMetadataRepository, FileStorageRepository};
use crate::proto::k1s0::system::file::v1::file_service_server::FileServiceServer;
use crate::usecase;

pub async fn run() -> anyhow::Result<()> {
    // Telemetry
    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config/config.yaml".to_string());
    let cfg = Config::load(&config_path)?;

    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-file-server".to_string(),
        version: "0.1.0".to_string(),
        tier: "system".to_string(),
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
    k1s0_telemetry::init_telemetry(&telemetry_cfg)
        .map_err(|e| anyhow::anyhow!("テレメトリの初期化に失敗: {}", e))?;

    info!(
        app_name = %cfg.app.name,
        version = %cfg.app.version,
        environment = %cfg.app.environment,
        "starting file server"
    );

    // Metadata repository (PostgreSQL or InMemory)
    let metadata_repo: Arc<dyn FileMetadataRepository> = if let Some(ref db_cfg) = cfg.database {
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| db_cfg.url.clone());
        info!(
            schema = %db_cfg.schema,
            max_connections = db_cfg.max_connections,
            min_connections = db_cfg.min_connections,
            "initializing PostgreSQL metadata repository"
        );
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(db_cfg.max_connections)
            .min_connections(db_cfg.min_connections)
            .acquire_timeout(Duration::from_secs(db_cfg.connect_timeout_seconds))
            .connect(&database_url)
            .await?;
        Arc::new(
            super::file_metadata_postgres::FileMetadataPostgresRepository::new(
                pool,
                &db_cfg.schema,
            )?,
        )
    } else if let Ok(database_url) = std::env::var("DATABASE_URL") {
        info!("DATABASE_URL is set, using PostgreSQL metadata repository");
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(10)
            .min_connections(1)
            .acquire_timeout(Duration::from_secs(10))
            .connect(&database_url)
            .await?;
        Arc::new(super::file_metadata_postgres::FileMetadataPostgresRepository::new(pool, "file")?)
    } else {
        // infra_guard: stable サービスでは DB 設定を必須化（dev/test 以外はエラー）
        k1s0_server_common::require_infra(
            "file",
            k1s0_server_common::InfraKind::Database,
            &cfg.app.environment,
            None::<String>,
        )?;
        info!("no database configured, using in-memory metadata repository (dev/test bypass)");
        Arc::new(InMemoryFileMetadataRepository::new())
    };

    // Storage backend（ローカルFS または インメモリ）
    let storage_repo: Arc<dyn FileStorageRepository> = if let Some(ref storage_cfg) = cfg.storage {
        if storage_cfg.backend == "local" {
            let root_path = storage_cfg
                .path
                .clone()
                .unwrap_or_else(|| "/data/files".to_string());
            let base_url = storage_cfg
                .base_url
                .clone()
                .unwrap_or_else(|| {
                    format!("http://{}:{}", cfg.server.host, cfg.server.port)
                });
            info!(root_path = %root_path, "initializing local filesystem storage backend");
            Arc::new(LocalFsStorageRepository::new(
                std::path::PathBuf::from(root_path),
                base_url,
            ))
        } else {
            info!("using in-memory storage backend");
            Arc::new(InMemoryFileStorageRepository::new())
        }
    } else {
        // infra_guard: stable サービスでは Storage 設定を必須化（dev/test 以外はエラー）
        k1s0_server_common::require_infra(
            "file",
            k1s0_server_common::InfraKind::Storage,
            &cfg.app.environment,
            None::<String>,
        )?;
        info!("no storage configured, using in-memory storage backend (dev/test bypass)");
        Arc::new(InMemoryFileStorageRepository::new())
    };

    // Kafka publisher
    let publisher: Arc<dyn FileEventPublisher> = if let Some(ref kafka_cfg) = cfg.kafka {
        match FileKafkaProducer::new(kafka_cfg) {
            Ok(p) => {
                info!("Kafka file event publisher enabled");
                Arc::new(p)
            }
            Err(e) => {
                // 環境に応じてフォールバックの許否を判断する。
                // dev/test 以外では Kafka 初期化失敗時に即座にサーバー起動を中断する。
                if !k1s0_server_common::allow_in_memory_infra(&cfg.app.environment) {
                    return Err(anyhow::anyhow!(
                        "Kafka パブリッシャーの初期化に失敗しました。本番環境ではフォールバックは許可されていません: {}",
                        e
                    ));
                }
                tracing::warn!(
                    error = %e,
                    "dev/test 環境: Kafka 初期化失敗のため NoopFileEventPublisher で起動します"
                );
                Arc::new(NoopFileEventPublisher)
            }
        }
    } else {
        // infra_guard: stable サービスでは Kafka 設定を必須化（dev/test 以外はエラー）
        k1s0_server_common::require_infra(
            "file",
            k1s0_server_common::InfraKind::Kafka,
            &cfg.app.environment,
            None::<String>,
        )?;
        info!("no kafka configured, using noop publisher (dev/test bypass)");
        Arc::new(NoopFileEventPublisher)
    };

    // Use cases
    let list_files_uc = Arc::new(usecase::ListFilesUseCase::new(metadata_repo.clone()));
    let generate_upload_url_uc = Arc::new(usecase::GenerateUploadUrlUseCase::new(
        metadata_repo.clone(),
        storage_repo.clone(),
    ));
    let complete_upload_uc = Arc::new(usecase::CompleteUploadUseCase::new(
        metadata_repo.clone(),
        publisher.clone(),
    ));
    let get_file_metadata_uc =
        Arc::new(usecase::GetFileMetadataUseCase::new(metadata_repo.clone()));
    let generate_download_url_uc = Arc::new(usecase::GenerateDownloadUrlUseCase::new(
        metadata_repo.clone(),
        storage_repo.clone(),
    ));
    let delete_file_uc = Arc::new(usecase::DeleteFileUseCase::new(
        metadata_repo.clone(),
        storage_repo.clone(),
        publisher.clone(),
    ));
    let update_file_tags_uc = Arc::new(usecase::UpdateFileTagsUseCase::new(metadata_repo.clone()));

    // Metrics
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("k1s0-file-server"));

    // Token verifier (JWKS verifier if auth configured)
    let auth_state = k1s0_server_common::require_auth_state(
        "file-server",
        &cfg.app.environment,
        cfg.auth
            .as_ref()
            .map(|auth_cfg| -> anyhow::Result<_> {
                info!(jwks_url = %auth_cfg.jwks_url, "initializing JWKS verifier for file-server");
                let jwks_verifier = Arc::new(
                    k1s0_auth::JwksVerifier::new(
                        &auth_cfg.jwks_url,
                        &auth_cfg.issuer,
                        &auth_cfg.audience,
                        std::time::Duration::from_secs(auth_cfg.jwks_cache_ttl_secs),
                    )
                    .context("JWKS 検証器の作成に失敗")?,
                );
                Ok(crate::adapter::middleware::auth::AuthState {
                    verifier: jwks_verifier,
                })
            })
            .transpose()?,
    )?;

    // gRPC 認証レイヤー: メソッド名をアクション（read/write）にマッピングして RBAC チェックを行う
    let grpc_auth_layer = GrpcAuthLayer::new(auth_state.clone(), Tier::System, file_grpc_action);

    // REST app state
    let mut state = crate::adapter::handler::AppState {
        list_files_uc: list_files_uc.clone(),
        generate_upload_url_uc: generate_upload_url_uc.clone(),
        complete_upload_uc: complete_upload_uc.clone(),
        get_file_metadata_uc: get_file_metadata_uc.clone(),
        generate_download_url_uc: generate_download_url_uc.clone(),
        delete_file_uc: delete_file_uc.clone(),
        update_file_tags_uc: update_file_tags_uc.clone(),
        metrics: metrics.clone(),
        auth_state: None,
    };
    if let Some(auth_st) = auth_state {
        state = state.with_auth(auth_st);
    }

    // REST router（メトリクスレイヤーとCorrelation IDレイヤーを追加）
    let app = crate::adapter::handler::router(state)
        .layer(k1s0_telemetry::MetricsLayer::new(metrics.clone()))
        .layer(k1s0_correlation::layer::CorrelationLayer::new());

    // gRPC service
    let grpc_svc = Arc::new(crate::adapter::grpc::FileGrpcService::new(
        get_file_metadata_uc,
        list_files_uc,
        generate_upload_url_uc,
        complete_upload_uc,
        generate_download_url_uc,
        delete_file_uc,
        update_file_tags_uc,
    ));
    let tonic_svc = crate::adapter::grpc::FileServiceTonic::new(grpc_svc);

    // gRPC server
    let grpc_port = cfg.server.grpc_port;
    let grpc_addr: SocketAddr = format!("0.0.0.0:{}", grpc_port).parse()?;
    info!("gRPC server starting on {}", grpc_addr);

    // gRPC グレースフルシャットダウン用シグナル
    let grpc_shutdown = k1s0_server_common::shutdown::shutdown_signal();
    let grpc_metrics = metrics;
    let grpc_future = async move {
        tonic::transport::Server::builder()
            .layer(grpc_auth_layer)
            .layer(k1s0_telemetry::GrpcMetricsLayer::new(grpc_metrics))
            .add_service(FileServiceServer::new(tonic_svc))
            .serve_with_shutdown(grpc_addr, async move {
                let _ = grpc_shutdown.await;
            })
            .await
            .map_err(|e| anyhow::anyhow!("gRPC server error: {}", e))
    };

    // REST server
    let rest_addr: SocketAddr = format!("{}:{}", cfg.server.host, cfg.server.port).parse()?;
    info!("REST server starting on {}", rest_addr);

    let listener = tokio::net::TcpListener::bind(rest_addr).await?;
    // REST グレースフルシャットダウンを設定
    let rest_future = axum::serve(listener, app).with_graceful_shutdown(async {
        let _ = k1s0_server_common::shutdown::shutdown_signal().await;
    });

    // REST と gRPC を並行起動
    tokio::select! {
        result = rest_future => {
            if let Err(e) = result {
                tracing::error!("REST server error: {}", e);
            }
        }
        result = grpc_future => {
            if let Err(e) = result {
                tracing::error!("gRPC server error: {}", e);
            }
        }
    }

    // テレメトリのフラッシュとシャットダウン
    k1s0_telemetry::shutdown();

    Ok(())
}

/// gRPC メソッド名を RBAC アクション（read/write）にマッピングする。
/// アップロード URL 生成・アップロード完了・ファイル削除・タグ更新は write、それ以外は read とする。
fn file_grpc_action(method: &str) -> &'static str {
    match method {
        "GenerateUploadUrl" | "CompleteUpload" | "DeleteFile" | "UpdateFileTags" => "write",
        _ => "read",
    }
}

use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;

use k1s0_master_maintenance_server::adapter;
use k1s0_master_maintenance_server::domain;
use k1s0_master_maintenance_server::infrastructure;
use k1s0_master_maintenance_server::usecase;

use adapter::handler::{self, AppState};
use adapter::middleware::auth::MasterMaintenanceAuthState;
use infrastructure::config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Telemetry
    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config/config.yaml".to_string());
    let config_content = std::fs::read_to_string(&config_path)?;
    let cfg: Config = serde_yaml::from_str(&config_content)?;

    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-master-maintenance-server".to_string(),
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
    k1s0_telemetry::init_telemetry(&telemetry_cfg).expect("failed to init telemetry");

    // 2. Config
    info!("starting {}", cfg.app.name);

    // 3. Database
    let db_pool = if let Some(ref db_cfg) = cfg.database {
        let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| db_cfg.connection_url());
        let lifetime = std::time::Duration::from_secs(db_cfg.conn_max_lifetime);
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(db_cfg.max_connections)
            .min_connections(db_cfg.max_idle_conns.min(db_cfg.max_connections))
            .idle_timeout(Some(lifetime))
            .max_lifetime(Some(lifetime))
            .connect(&url)
            .await?;
        info!("database connected");
        Some(pool)
    } else {
        info!("running without database");
        None
    };

    // 4. Metrics
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("master_maintenance"));

    // 5. Repositories
    let table_repo: Arc<
        dyn domain::repository::table_definition_repository::TableDefinitionRepository,
    > = if let Some(ref pool) = db_pool {
        Arc::new(
            infrastructure::persistence::table_definition_repo_impl::TableDefinitionPostgresRepository::new(pool.clone()),
        )
    } else {
        unimplemented!("in-memory repository not yet implemented")
    };

    let column_repo: Arc<
        dyn domain::repository::column_definition_repository::ColumnDefinitionRepository,
    > = if let Some(ref pool) = db_pool {
        Arc::new(
            infrastructure::persistence::column_definition_repo_impl::ColumnDefinitionPostgresRepository::new(pool.clone()),
        )
    } else {
        unimplemented!()
    };

    let rule_repo: Arc<
        dyn domain::repository::consistency_rule_repository::ConsistencyRuleRepository,
    > = if let Some(ref pool) = db_pool {
        Arc::new(
            infrastructure::persistence::consistency_rule_repo_impl::ConsistencyRulePostgresRepository::new(pool.clone()),
        )
    } else {
        unimplemented!()
    };

    let record_repo: Arc<
        dyn domain::repository::dynamic_record_repository::DynamicRecordRepository,
    > = if let Some(ref pool) = db_pool {
        Arc::new(
            infrastructure::persistence::dynamic_record_repo_impl::DynamicRecordPostgresRepository::new(pool.clone()),
        )
    } else {
        unimplemented!()
    };

    let change_log_repo: Arc<dyn domain::repository::change_log_repository::ChangeLogRepository> =
        if let Some(ref pool) = db_pool {
            Arc::new(
                infrastructure::persistence::change_log_repo_impl::ChangeLogPostgresRepository::new(
                    pool.clone(),
                ),
            )
        } else {
            unimplemented!()
        };

    let relationship_repo: Arc<
        dyn domain::repository::table_relationship_repository::TableRelationshipRepository,
    > = if let Some(ref pool) = db_pool {
        Arc::new(
            infrastructure::persistence::table_relationship_repo_impl::TableRelationshipPostgresRepository::new(pool.clone()),
        )
    } else {
        unimplemented!()
    };

    let display_config_repo: Arc<
        dyn domain::repository::display_config_repository::DisplayConfigRepository,
    > = if let Some(ref pool) = db_pool {
        Arc::new(
            infrastructure::persistence::display_config_repo_impl::DisplayConfigPostgresRepository::new(pool.clone()),
        )
    } else {
        unimplemented!()
    };

    let import_job_repo: Arc<dyn domain::repository::import_job_repository::ImportJobRepository> =
        if let Some(ref pool) = db_pool {
            Arc::new(
                infrastructure::persistence::import_job_repo_impl::ImportJobPostgresRepository::new(
                    pool.clone(),
                ),
            )
        } else {
            unimplemented!()
        };

    // 6. Kafka Producer (optional)
    let kafka_producer = if let Some(ref kafka_cfg) = cfg.kafka {
        match infrastructure::messaging::kafka_producer::MasterMaintenanceKafkaProducer::new(
            kafka_cfg,
        ) {
            Ok(producer) => {
                info!("kafka producer initialized");
                Some(Arc::new(producer))
            }
            Err(e) => {
                tracing::warn!("failed to initialize kafka producer: {}", e);
                None
            }
        }
    } else {
        None
    };

    // 7. Rule Engine
    let rule_engine =
        Arc::new(infrastructure::rule_engine::zen_engine_adapter::ZenEngineAdapter::new());

    // 8. Use Cases
    let manage_tables_uc = Arc::new(
        usecase::manage_table_definitions::ManageTableDefinitionsUseCase::new(
            table_repo.clone(),
            column_repo.clone(),
        ),
    );
    let manage_columns_uc = Arc::new(
        usecase::manage_column_definitions::ManageColumnDefinitionsUseCase::new(
            table_repo.clone(),
            column_repo.clone(),
        ),
    );
    let crud_records_uc = Arc::new(usecase::crud_records::CrudRecordsUseCase::new(
        table_repo.clone(),
        column_repo.clone(),
        record_repo.clone(),
        change_log_repo.clone(),
    ));
    let manage_rules_uc = Arc::new(usecase::manage_rules::ManageRulesUseCase::new(
        table_repo.clone(),
        rule_repo.clone(),
    ));
    let check_consistency_uc = Arc::new(usecase::check_consistency::CheckConsistencyUseCase::new(
        table_repo.clone(),
        column_repo.clone(),
        rule_repo.clone(),
        record_repo.clone(),
        rule_engine.clone(),
    ));
    let get_audit_logs_uc = Arc::new(usecase::get_audit_logs::GetAuditLogsUseCase::new(
        change_log_repo.clone(),
    ));
    let manage_relationships_uc = Arc::new(
        usecase::manage_relationships::ManageRelationshipsUseCase::new(
            table_repo.clone(),
            relationship_repo.clone(),
            record_repo.clone(),
            column_repo.clone(),
        ),
    );
    let manage_display_configs_uc = Arc::new(
        usecase::manage_display_configs::ManageDisplayConfigsUseCase::new(
            table_repo.clone(),
            display_config_repo.clone(),
        ),
    );
    let import_export_uc = Arc::new(usecase::import_export::ImportExportUseCase::new(
        table_repo.clone(),
        column_repo.clone(),
        record_repo.clone(),
        import_job_repo.clone(),
    ));

    // 9. Auth
    let auth_state = if let Some(ref auth_cfg) = cfg.auth {
        let verifier = Arc::new(k1s0_auth::JwksVerifier::new(
            &auth_cfg.jwks_url,
            &auth_cfg.issuer,
            &auth_cfg.audience,
            std::time::Duration::from_secs(auth_cfg.jwks_cache_ttl_secs),
        ));
        Some(MasterMaintenanceAuthState { verifier })
    } else {
        None
    };

    // 10. AppState + Router
    let state = AppState {
        manage_tables_uc: manage_tables_uc.clone(),
        manage_columns_uc: manage_columns_uc.clone(),
        crud_records_uc: crud_records_uc.clone(),
        manage_rules_uc: manage_rules_uc.clone(),
        check_consistency_uc: check_consistency_uc.clone(),
        get_audit_logs_uc: get_audit_logs_uc.clone(),
        manage_relationships_uc: manage_relationships_uc.clone(),
        manage_display_configs_uc: manage_display_configs_uc.clone(),
        import_export_uc: import_export_uc.clone(),
        metrics: metrics.clone(),
        kafka_producer: kafka_producer.clone(),
        auth_state: auth_state.clone(),
    };
    let app = handler::router(state);

    // 11. gRPC Service
    use k1s0_master_maintenance_server::proto::k1s0::system::mastermaintenance::v1::master_maintenance_service_server::MasterMaintenanceServiceServer;
    let grpc_service = adapter::grpc::master_maintenance_grpc::MasterMaintenanceGrpcService::new(
        manage_tables_uc,
        manage_columns_uc,
        crud_records_uc,
        manage_rules_uc,
        check_consistency_uc,
        get_audit_logs_uc,
        manage_relationships_uc,
        manage_display_configs_uc,
        import_export_uc,
        column_repo,
        relationship_repo,
    );
    let grpc_addr: SocketAddr = format!("{}:{}", cfg.server.host, cfg.server.grpc_port).parse()?;
    info!("gRPC server listening on {}", grpc_addr);
    let grpc_metrics = metrics.clone();
    let grpc_future = async move {
        tonic::transport::Server::builder()
            .layer(k1s0_telemetry::GrpcMetricsLayer::new(grpc_metrics))
            .add_service(MasterMaintenanceServiceServer::new(grpc_service))
            .serve(grpc_addr)
            .await
            .map_err(|e| anyhow::anyhow!("gRPC server error: {}", e))
    };

    // 12. Start REST server
    let rest_addr: SocketAddr = format!("{}:{}", cfg.server.host, cfg.server.port).parse()?;
    info!("REST server listening on {}", rest_addr);
    let listener = tokio::net::TcpListener::bind(rest_addr).await?;
    let rest_future = axum::serve(listener, app);

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

    Ok(())
}

//! Local development environment management.

pub mod compose;
pub mod detect;
pub mod docker;
pub mod migration;
pub mod port;
pub mod seed;
pub mod state;
pub mod types;

use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

pub use detect::scan_dev_targets;
pub use types::*;

/// Start the local development environment.
///
/// # Errors
///
/// Returns an error when Docker is unavailable, dependency detection fails,
/// compose operations fail, or migration/seed execution fails.
pub fn execute_dev_up(config: &DevUpConfig) -> Result<()> {
    println!("ローカル開発環境を起動します...");

    docker::check_docker_available()?;

    let deps_list: Vec<detect::DetectedDependencies> = config
        .services
        .iter()
        .map(|service| detect::detect_dependencies(service))
        .collect::<Result<Vec<_>>>()?;
    let merged = detect::merge_dependencies(&deps_list);

    let ports = port::resolve_ports(&port::default_ports());

    let compose_dir = Path::new(".k1s0-dev");
    std::fs::create_dir_all(compose_dir)?;

    write_generated_files(compose_dir, &merged, &ports, config)?;

    docker::compose_up(compose_dir)?;
    println!("  コンテナを起動しました");

    docker::wait_for_healthy(compose_dir, 60)?;
    println!("  ヘルスチェック完了");

    if !merged.databases.is_empty() {
        migration::run_dev_migrations(&config.services, &ports)?;
        println!("  マイグレーションを適用しました");
    }

    seed::execute_seed(&config.services, &ports)?;
    println!("  シードデータを投入しました");

    let dev_state = build_dev_state(config, &merged, &ports);
    state::save_state(&dev_state)?;
    println!("  状態を保存しました");

    println!("\nローカル開発環境の起動が完了しました。");
    Ok(())
}

fn write_generated_files(
    compose_dir: &Path,
    merged: &DetectedDependencies,
    ports: &PortAssignments,
    config: &DevUpConfig,
) -> Result<()> {
    let compose_yaml = compose::generate_compose(merged, ports, &config.auth_mode);
    std::fs::write(compose_dir.join("docker-compose.yaml"), &compose_yaml)?;
    println!("  docker-compose.yaml を生成しました");

    if !merged.databases.is_empty() {
        let init_sql = compose::generate_init_db_sql(&merged.databases);
        std::fs::write(compose_dir.join("init-db.sql"), &init_sql)?;
        println!("  init-db.sql を生成しました");
    }

    let dev_config = compose::generate_dev_local_config(merged, ports, &config.auth_mode);
    std::fs::write(compose_dir.join("config.dev-local.yaml"), &dev_config)?;
    println!("  config.dev-local.yaml を生成しました");

    Ok(())
}

fn build_dev_state(
    config: &DevUpConfig,
    merged: &DetectedDependencies,
    ports: &PortAssignments,
) -> DevState {
    DevState {
        version: 1,
        started_at: chrono::Utc::now().to_rfc3339(),
        services: config.services.clone(),
        dependencies: build_dev_state_deps(merged, ports),
        auth_mode: auth_mode_label(&config.auth_mode).to_string(),
        migration_status: build_migration_status(&config.services, &merged.databases),
        seed_status: build_seed_status(&merged.databases),
    }
}

fn build_dev_state_deps(merged: &DetectedDependencies, ports: &PortAssignments) -> DevStateDeps {
    DevStateDeps {
        postgres: if merged.databases.is_empty() {
            None
        } else {
            Some(PostgresDep {
                port: ports.postgres,
                databases: merged.databases.iter().map(|db| db.name.clone()).collect(),
            })
        },
        kafka: merged.has_kafka.then_some(KafkaDep { port: ports.kafka }),
        redis: merged.has_redis.then_some(RedisDep { port: ports.redis }),
    }
}

fn auth_mode_label(auth_mode: &AuthMode) -> &'static str {
    match auth_mode {
        AuthMode::Skip => "skip",
        AuthMode::Keycloak => "keycloak",
    }
}

fn build_migration_status(
    services: &[String],
    databases: &[DatabaseDep],
) -> HashMap<String, MigrationStatus> {
    let total = total_migration_count(services);
    databases
        .iter()
        .map(|db| {
            (
                db.name.clone(),
                MigrationStatus {
                    applied: total,
                    total,
                },
            )
        })
        .collect()
}

fn total_migration_count(services: &[String]) -> u32 {
    services
        .iter()
        .filter_map(|service| count_sql_migrations(service))
        .sum()
}

fn count_sql_migrations(service: &str) -> Option<u32> {
    let migrations_dir = Path::new(service).join("migrations");
    if !migrations_dir.is_dir() {
        return None;
    }

    let entries = std::fs::read_dir(&migrations_dir).ok()?;
    let count = entries
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("sql"))
        .count();
    Some(u32::try_from(count).unwrap_or(u32::MAX))
}

fn build_seed_status(databases: &[DatabaseDep]) -> HashMap<String, SeedStatus> {
    databases
        .iter()
        .map(|db| (db.name.clone(), SeedStatus { applied: true }))
        .collect()
}

/// Stop the local development environment.
///
/// # Errors
///
/// Returns an error when `docker compose down` or state cleanup fails.
pub fn execute_dev_down(config: &DevDownConfig) -> Result<()> {
    println!("ローカル開発環境を停止します...");

    let compose_dir = Path::new(".k1s0-dev");
    if !compose_dir.join("docker-compose.yaml").exists() {
        println!("ローカル開発環境は起動していません。");
        return Ok(());
    }

    let remove_volumes = matches!(config.cleanup, CleanupLevel::ContainersAndVolumes);
    docker::compose_down(compose_dir, remove_volumes)?;

    let state_path = compose_dir.join("state.json");
    if state_path.exists() {
        std::fs::remove_file(&state_path)?;
    }

    if remove_volumes {
        println!("コンテナとボリュームを削除しました。");
    } else {
        println!("コンテナを停止しました。ボリュームは保持しています。");
    }

    Ok(())
}

/// Show local development environment status.
///
/// # Errors
///
/// Returns an error when `docker compose ps` fails.
pub fn execute_dev_status() -> Result<()> {
    let compose_dir = Path::new(".k1s0-dev");
    if !compose_dir.join("docker-compose.yaml").exists() {
        println!("ローカル開発環境は起動していません。");
        return Ok(());
    }

    if let Some(dev_state) = state::load_state() {
        println!("--- ローカル開発環境の状態 ---\n");
        println!("  起動日時: {}", dev_state.started_at);
        println!("  認証モード: {}", dev_state.auth_mode);
        println!("  対象サービス:");
        for service in &dev_state.services {
            println!("    - {service}");
        }
        if let Some(postgres) = &dev_state.dependencies.postgres {
            println!("  PostgreSQL: ポート {}", postgres.port);
            for db in &postgres.databases {
                println!("    - {db}");
            }
        }
        if let Some(kafka) = &dev_state.dependencies.kafka {
            println!("  Kafka: ポート {}", kafka.port);
        }
        if let Some(redis) = &dev_state.dependencies.redis {
            println!("  Redis: ポート {}", redis.port);
        }
    }

    println!("\n--- コンテナの状態 ---\n");
    let status = docker::compose_status(compose_dir)?;
    println!("{status}");

    Ok(())
}

/// Show local development environment logs.
///
/// # Errors
///
/// Returns an error when `docker compose logs` fails.
pub fn execute_dev_logs(service: Option<&str>) -> Result<()> {
    let compose_dir = Path::new(".k1s0-dev");
    if !compose_dir.join("docker-compose.yaml").exists() {
        println!("ローカル開発環境は起動していません。");
        return Ok(());
    }

    docker::compose_logs(compose_dir, service)?;
    Ok(())
}

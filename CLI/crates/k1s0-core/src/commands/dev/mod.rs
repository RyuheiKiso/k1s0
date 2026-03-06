/// ローカル開発環境コマンドのビジネスロジック。
///
/// dev up / dev down / dev status / dev logs の4つの操作を提供する。
pub mod compose;
pub mod detect;
pub mod docker;
pub mod migration;
pub mod port;
pub mod seed;
pub mod state;
pub mod types;

use anyhow::Result;
use std::path::Path;

pub use detect::scan_dev_targets;
pub use types::*;

/// dev up: ローカル開発環境を起動する。
///
/// 1. Docker が利用可能か確認
/// 2. 依存を検出・統合
/// 3. ポートを解決
/// 4. docker-compose.yaml を生成・書き出し
/// 5. init-db.sql を生成（PostgreSQL利用時）
/// 6. config.dev-local.yaml を生成
/// 7. docker compose up -d
/// 8. ヘルスチェック待機
/// 9. マイグレーション実行
/// 10. シードデータ投入
/// 11. 状態を保存
///
/// # Errors
///
/// Docker が利用不可、docker compose の起動失敗、マイグレーション失敗等の場合にエラーを返す。
pub fn execute_dev_up(config: &DevUpConfig) -> Result<()> {
    println!("ローカル開発環境を起動しています...");

    // Docker 利用可能チェック
    docker::check_docker_available()?;

    // 依存検出
    let deps_list: Vec<detect::DetectedDependencies> = config
        .services
        .iter()
        .map(|s| detect::detect_dependencies(s))
        .collect::<Result<Vec<_>>>()?;
    let merged = detect::merge_dependencies(&deps_list);

    // ポート解決
    let defaults = port::default_ports();
    let ports = port::resolve_ports(&defaults);

    // .k1s0-dev ディレクトリ作成
    let compose_dir = Path::new(".k1s0-dev");
    std::fs::create_dir_all(compose_dir)?;

    // docker-compose.yaml 生成
    let compose_yaml = compose::generate_compose(&merged, &ports, &config.auth_mode);
    std::fs::write(compose_dir.join("docker-compose.yaml"), &compose_yaml)?;
    println!("  docker-compose.yaml を生成しました");

    // init-db.sql 生成（PostgreSQL利用時）
    if !merged.databases.is_empty() {
        let init_sql = compose::generate_init_db_sql(&merged.databases);
        std::fs::write(compose_dir.join("init-db.sql"), &init_sql)?;
        println!("  init-db.sql を生成しました");
    }

    // config.dev-local.yaml 生成
    let dev_config = compose::generate_dev_local_config(&merged, &ports, &config.auth_mode);
    std::fs::write(compose_dir.join("config.dev-local.yaml"), &dev_config)?;
    println!("  config.dev-local.yaml を生成しました");

    // docker compose up
    docker::compose_up(compose_dir)?;
    println!("  コンテナを起動しました");

    // ヘルスチェック待機
    docker::wait_for_healthy(compose_dir, 60)?;
    println!("  ヘルスチェック完了");

    // マイグレーション
    if !merged.databases.is_empty() {
        migration::run_dev_migrations(&config.services, &ports)?;
        println!("  マイグレーションを実行しました");
    }

    // シードデータ
    seed::execute_seed(&config.services, &ports)?;
    println!("  シードデータを投入しました");

    // 状態保存
    let auth_mode_str = match config.auth_mode {
        AuthMode::Skip => "skip".to_string(),
        AuthMode::Keycloak => "keycloak".to_string(),
    };

    // マイグレーション状態を収集
    let mut migration_status_map = std::collections::HashMap::new();
    for db in &merged.databases {
        // 対象サービスのマイグレーションファイル数を数える
        let total = config
            .services
            .iter()
            .filter_map(|s| {
                let migrations_dir = std::path::Path::new(s).join("migrations");
                if migrations_dir.is_dir() {
                    std::fs::read_dir(&migrations_dir)
                        .ok()
                        .map(|entries| {
                            entries
                                .filter_map(|e| e.ok())
                                .filter(|e| {
                                    e.path()
                                        .extension()
                                        .and_then(|ext| ext.to_str())
                                        == Some("sql")
                                })
                                .count() as u32
                        })
                } else {
                    None
                }
            })
            .sum::<u32>();
        migration_status_map.insert(
            db.name.clone(),
            MigrationStatus {
                applied: total,
                total,
            },
        );
    }

    // シードデータ状態を収集
    let mut seed_status_map = std::collections::HashMap::new();
    for db in &merged.databases {
        seed_status_map.insert(db.name.clone(), SeedStatus { applied: true });
    }

    let dev_state = DevState {
        version: 1,
        started_at: chrono::Utc::now().to_rfc3339(),
        services: config.services.clone(),
        dependencies: DevStateDeps {
            postgres: if merged.databases.is_empty() {
                None
            } else {
                Some(PostgresDep {
                    port: ports.postgres,
                    databases: merged.databases.iter().map(|d| d.name.clone()).collect(),
                })
            },
            kafka: if merged.has_kafka {
                Some(KafkaDep { port: ports.kafka })
            } else {
                None
            },
            redis: if merged.has_redis {
                Some(RedisDep { port: ports.redis })
            } else {
                None
            },
        },
        auth_mode: auth_mode_str,
        migration_status: migration_status_map,
        seed_status: seed_status_map,
    };
    state::save_state(&dev_state)?;
    println!("  状態を保存しました");

    println!("\nローカル開発環境の起動が完了しました。");
    Ok(())
}

/// dev down: ローカル開発環境を停止する。
///
/// # Errors
///
/// docker compose の停止に失敗した場合にエラーを返す。
pub fn execute_dev_down(config: &DevDownConfig) -> Result<()> {
    println!("ローカル開発環境を停止しています...");

    let compose_dir = Path::new(".k1s0-dev");
    if !compose_dir.join("docker-compose.yaml").exists() {
        println!("ローカル開発環境は起動していません。");
        return Ok(());
    }

    let remove_volumes = matches!(config.cleanup, CleanupLevel::ContainersAndVolumes);
    docker::compose_down(compose_dir, remove_volumes)?;

    // 状態ファイル削除
    let state_path = compose_dir.join("state.json");
    if state_path.exists() {
        std::fs::remove_file(&state_path)?;
    }

    if remove_volumes {
        println!("コンテナとボリュームを削除しました。");
    } else {
        println!("コンテナを停止しました（ボリュームは保持）。");
    }

    Ok(())
}

/// dev status: ローカル開発環境の状態を表示する。
///
/// # Errors
///
/// docker compose の状態取得に失敗した場合にエラーを返す。
pub fn execute_dev_status() -> Result<()> {
    let compose_dir = Path::new(".k1s0-dev");
    if !compose_dir.join("docker-compose.yaml").exists() {
        println!("ローカル開発環境は起動していません。");
        return Ok(());
    }

    // 状態ファイルの読み込み
    if let Some(dev_state) = state::load_state() {
        println!("--- ローカル開発環境の状態 ---\n");
        println!("  起動日時: {}", dev_state.started_at);
        println!("  認証モード: {}", dev_state.auth_mode);
        println!("  対象サービス:");
        for s in &dev_state.services {
            println!("    - {s}");
        }
        if let Some(ref pg) = dev_state.dependencies.postgres {
            println!("  PostgreSQL: ポート {}", pg.port);
            for db in &pg.databases {
                println!("    - {db}");
            }
        }
        if let Some(ref kafka) = dev_state.dependencies.kafka {
            println!("  Kafka: ポート {}", kafka.port);
        }
        if let Some(ref redis) = dev_state.dependencies.redis {
            println!("  Redis: ポート {}", redis.port);
        }
    }

    // docker compose ps
    println!("\n--- コンテナの状態 ---\n");
    let status = docker::compose_status(compose_dir)?;
    println!("{status}");

    Ok(())
}

/// dev logs: ローカル開発環境のログを表示する。
///
/// # Errors
///
/// docker compose logs の実行に失敗した場合にエラーを返す。
pub fn execute_dev_logs(service: Option<&str>) -> Result<()> {
    let compose_dir = Path::new(".k1s0-dev");
    if !compose_dir.join("docker-compose.yaml").exists() {
        println!("ローカル開発環境は起動していません。");
        return Ok(());
    }

    docker::compose_logs(compose_dir, service)?;
    Ok(())
}

/// 依存検出モジュール。
///
/// 各サービスの config/config.yaml を読み込み、依存するインフラ（PostgreSQL/Kafka/Redis）を検出する。
use anyhow::Result;
use std::fs;
use std::path::Path;

use crate::config::RuntimeConfig;

use super::types::DatabaseDep;
pub use super::types::DetectedDependencies;

/// 指定サービスパスの config/config.yaml から依存を検出する。
///
/// # Errors
///
/// config.yaml の読み込み・パースに失敗した場合にエラーを返す。
/// config.yaml が存在しない場合は空の依存を返す。
pub fn detect_dependencies(service_path: &str) -> Result<DetectedDependencies> {
    detect_dependencies_at(Path::new(service_path))
}

/// 指定パスを基点にconfig/config.yamlから依存を検出する（テスト用）。
pub fn detect_dependencies_at(service_dir: &Path) -> Result<DetectedDependencies> {
    let config_path = service_dir.join("config").join("config.yaml");
    if !config_path.exists() {
        return Ok(DetectedDependencies::default());
    }

    let content = fs::read_to_string(&config_path)?;
    let config: RuntimeConfig = serde_yaml::from_str(&content)?;

    let mut deps = DetectedDependencies::default();

    // データベース検出
    if let Some(ref db) = config.database {
        let db_name = if db.name.is_empty() {
            // サービスディレクトリ名からDB名を推定
            service_dir
                .file_name()
                .and_then(|n| n.to_str())
                .map_or_else(
                    || "default_db".to_string(),
                    |n| format!("{}_db", n.replace('-', "_")),
                )
        } else {
            db.name.clone()
        };
        let service_name = service_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        deps.databases.push(DatabaseDep {
            name: db_name,
            service: service_name,
        });
    }

    // Kafka 検出
    if let Some(ref kafka) = config.kafka {
        deps.has_kafka = true;
        // publish トピックを収集
        for topic in &kafka.topics.publish {
            deps.kafka_topics.push(topic.clone());
        }
        // subscribe トピックを収集
        for topic in &kafka.topics.subscribe {
            if !deps.kafka_topics.contains(topic) {
                deps.kafka_topics.push(topic.clone());
            }
        }
    }

    // Redis 検出
    if config.redis.is_some() {
        deps.has_redis = true;
    }

    // Redis (session) 検出
    if config.redis_session.is_some() {
        deps.has_redis_session = true;
    }

    Ok(deps)
}

/// 複数サービスの依存情報を統合する。
///
/// データベース名の重複排除、Kafka トピックの重複排除を行う。
pub fn merge_dependencies(deps: &[DetectedDependencies]) -> DetectedDependencies {
    let mut merged = DetectedDependencies::default();

    for dep in deps {
        // データベース統合（名前で重複排除）
        for db in &dep.databases {
            if !merged.databases.iter().any(|d| d.name == db.name) {
                merged.databases.push(db.clone());
            }
        }

        // Kafka 統合
        if dep.has_kafka {
            merged.has_kafka = true;
        }

        // Kafka トピック統合（重複排除）
        for topic in &dep.kafka_topics {
            if !merged.kafka_topics.contains(topic) {
                merged.kafka_topics.push(topic.clone());
            }
        }

        // Redis 統合
        if dep.has_redis {
            merged.has_redis = true;
        }
        if dep.has_redis_session {
            merged.has_redis_session = true;
        }
    }

    merged
}

/// regions/ 配下のサーバーを走査して (表示名, パス) のペアを返す。
pub fn scan_dev_targets(base_dir: &Path) -> Vec<(String, String)> {
    let regions = base_dir.join("regions");
    if !regions.is_dir() {
        return Vec::new();
    }

    let mut targets = Vec::new();
    scan_dev_targets_recursive(&regions, &mut targets);
    targets.sort_by(|a, b| a.0.cmp(&b.0));
    targets
}

/// 再帰的にサーバーディレクトリを走査する。
fn scan_dev_targets_recursive(path: &Path, targets: &mut Vec<(String, String)>) {
    if !path.is_dir() {
        return;
    }

    // config/config.yaml が存在するディレクトリをサーバーとして検出
    let config_path = path.join("config").join("config.yaml");
    if config_path.exists() {
        let path_str = path.to_string_lossy().to_string();
        let normalized = path_str.replace('\\', "/");

        // 表示名を生成: 末尾のディレクトリ名
        let display_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        targets.push((display_name, normalized));
        return;
    }

    // library/ ディレクトリはスキップ
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        if name == "library" {
            return;
        }
    }

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                scan_dev_targets_recursive(&entry.path(), targets);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// config.yaml がないサービスは空の依存を返す。
    #[test]
    fn test_detect_dependencies_no_config() {
        let tmp = TempDir::new().unwrap();
        let deps = detect_dependencies_at(tmp.path()).unwrap();
        assert!(deps.databases.is_empty());
        assert!(!deps.has_kafka);
        assert!(!deps.has_redis);
        assert!(!deps.has_redis_session);
    }

    /// database セクションのみの config.yaml から PostgreSQL 依存を検出する。
    #[test]
    fn test_detect_dependencies_database_only() {
        let tmp = TempDir::new().unwrap();
        let config_dir = tmp.path().join("config");
        fs::create_dir_all(&config_dir).unwrap();
        fs::write(
            config_dir.join("config.yaml"),
            r#"
app:
  name: test-server
  tier: system
  environment: dev
database:
  host: localhost
  port: 5432
  name: test_db
  user: app
  password: ""
  ssl_mode: disable
"#,
        )
        .unwrap();

        let deps = detect_dependencies_at(tmp.path()).unwrap();
        assert_eq!(deps.databases.len(), 1);
        assert_eq!(deps.databases[0].name, "test_db");
        assert!(!deps.has_kafka);
        assert!(!deps.has_redis);
    }

    /// kafka セクションからKafka依存とトピックを検出する。
    #[test]
    fn test_detect_dependencies_kafka() {
        let tmp = TempDir::new().unwrap();
        let config_dir = tmp.path().join("config");
        fs::create_dir_all(&config_dir).unwrap();
        fs::write(
            config_dir.join("config.yaml"),
            r#"
app:
  name: test-server
  tier: system
  environment: dev
kafka:
  brokers:
    - "kafka-0:9092"
  consumer_group: test.default
  security_protocol: PLAINTEXT
  topics:
    publish:
      - "k1s0.system.test.v1"
    subscribe:
      - "k1s0.system.other.v1"
"#,
        )
        .unwrap();

        let deps = detect_dependencies_at(tmp.path()).unwrap();
        assert!(deps.has_kafka);
        assert_eq!(deps.kafka_topics.len(), 2);
        assert!(deps
            .kafka_topics
            .contains(&"k1s0.system.test.v1".to_string()));
        assert!(deps
            .kafka_topics
            .contains(&"k1s0.system.other.v1".to_string()));
    }

    /// redis / redis_session セクションから Redis 依存を検出する。
    #[test]
    fn test_detect_dependencies_redis() {
        let tmp = TempDir::new().unwrap();
        let config_dir = tmp.path().join("config");
        fs::create_dir_all(&config_dir).unwrap();
        fs::write(
            config_dir.join("config.yaml"),
            r#"
app:
  name: test-server
  tier: system
  environment: dev
redis:
  host: localhost
  port: 6379
redis_session:
  host: localhost
  port: 6380
"#,
        )
        .unwrap();

        let deps = detect_dependencies_at(tmp.path()).unwrap();
        assert!(deps.has_redis);
        assert!(deps.has_redis_session);
    }

    /// 全依存を含む config.yaml をパースできる。
    #[test]
    fn test_detect_dependencies_full() {
        let tmp = TempDir::new().unwrap();
        let config_dir = tmp.path().join("config");
        fs::create_dir_all(&config_dir).unwrap();
        fs::write(
            config_dir.join("config.yaml"),
            r#"
app:
  name: order-server
  tier: service
  environment: dev
database:
  host: localhost
  name: order_db
  user: app
kafka:
  brokers:
    - "kafka-0:9092"
  consumer_group: order.default
  security_protocol: PLAINTEXT
  topics:
    publish:
      - "k1s0.service.order.v1"
    subscribe: []
redis:
  host: localhost
  port: 6379
"#,
        )
        .unwrap();

        let deps = detect_dependencies_at(tmp.path()).unwrap();
        assert_eq!(deps.databases.len(), 1);
        assert_eq!(deps.databases[0].name, "order_db");
        assert!(deps.has_kafka);
        assert!(deps.has_redis);
        assert!(!deps.has_redis_session);
    }

    /// 複数サービスの依存を統合する（DB名重複排除）。
    #[test]
    fn test_merge_dependencies_dedup_databases() {
        let deps1 = DetectedDependencies {
            databases: vec![DatabaseDep {
                name: "shared_db".to_string(),
                service: "svc1".to_string(),
            }],
            has_kafka: true,
            kafka_topics: vec!["topic.a".to_string()],
            ..Default::default()
        };
        let deps2 = DetectedDependencies {
            databases: vec![
                DatabaseDep {
                    name: "shared_db".to_string(),
                    service: "svc2".to_string(),
                },
                DatabaseDep {
                    name: "other_db".to_string(),
                    service: "svc2".to_string(),
                },
            ],
            has_kafka: false,
            has_redis: true,
            kafka_topics: vec!["topic.a".to_string(), "topic.b".to_string()],
            ..Default::default()
        };

        let merged = merge_dependencies(&[deps1, deps2]);
        assert_eq!(merged.databases.len(), 2);
        assert!(merged.databases.iter().any(|d| d.name == "shared_db"));
        assert!(merged.databases.iter().any(|d| d.name == "other_db"));
        assert!(merged.has_kafka);
        assert!(merged.has_redis);
        assert_eq!(merged.kafka_topics.len(), 2);
    }

    /// 空の依存リストを統合すると空の結果になる。
    #[test]
    fn test_merge_dependencies_empty() {
        let merged = merge_dependencies(&[]);
        assert!(merged.databases.is_empty());
        assert!(!merged.has_kafka);
        assert!(!merged.has_redis);
    }

    /// scan_dev_targets は config/config.yaml を持つディレクトリを検出する。
    #[test]
    fn test_scan_dev_targets() {
        let tmp = TempDir::new().unwrap();

        // サーバー（config/config.yaml あり）
        let server_dir = tmp.path().join("regions/system/server/rust/auth");
        let config_dir = server_dir.join("config");
        fs::create_dir_all(&config_dir).unwrap();
        fs::write(
            config_dir.join("config.yaml"),
            "app:\n  name: auth-server\n",
        )
        .unwrap();

        // ライブラリ（スキップ対象）
        let lib_dir = tmp
            .path()
            .join("regions/system/library/rust/authlib/config");
        fs::create_dir_all(&lib_dir).unwrap();
        fs::write(lib_dir.join("config.yaml"), "app:\n  name: lib\n").unwrap();

        let targets = scan_dev_targets(tmp.path());
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].0, "auth");
    }

    /// regions/ が存在しない場合は空のリストを返す。
    #[test]
    fn test_scan_dev_targets_no_regions() {
        let tmp = TempDir::new().unwrap();
        let targets = scan_dev_targets(tmp.path());
        assert!(targets.is_empty());
    }

    /// database.name が空の場合、ディレクトリ名からDB名を推定する。
    #[test]
    fn test_detect_dependencies_empty_db_name() {
        let tmp = TempDir::new().unwrap();
        let svc_dir = tmp.path().join("my-service");
        let config_dir = svc_dir.join("config");
        fs::create_dir_all(&config_dir).unwrap();
        fs::write(
            config_dir.join("config.yaml"),
            r#"
app:
  name: my-service
  tier: system
  environment: dev
database:
  host: localhost
  name: ""
  user: app
"#,
        )
        .unwrap();

        let deps = detect_dependencies_at(&svc_dir).unwrap();
        assert_eq!(deps.databases.len(), 1);
        assert_eq!(deps.databases[0].name, "my_service_db");
    }
}

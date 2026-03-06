/// docker-compose.yaml 生成モジュール。
///
/// 検出された依存情報から docker-compose.yaml を動的に生成する。
use super::types::{AuthMode, DatabaseDep, DetectedDependencies, PortAssignments};

/// docker-compose.yaml を生成する。
///
/// 検出された依存に応じて必要なサービスのみを含む YAML を生成する。
pub fn generate_compose(
    deps: &DetectedDependencies,
    ports: &PortAssignments,
    auth: &AuthMode,
) -> String {
    let mut yaml = String::new();

    yaml.push_str("# 自動生成: k1s0 dev up\n");
    yaml.push_str("# このファイルは手動で編集しないでください\n");
    yaml.push_str("version: \"3.8\"\n\n");
    yaml.push_str("services:\n");

    // PostgreSQL
    if !deps.databases.is_empty() {
        let first_db_name = &deps.databases[0].name;
        yaml.push_str(&generate_postgres_service(ports, first_db_name));
        yaml.push_str(&generate_pgadmin_service(ports));
    }

    // Kafka (KRaft モード)
    if deps.has_kafka {
        yaml.push_str(&generate_kafka_service(ports));
        yaml.push_str(&generate_kafka_ui_service(ports));
    }

    // Redis
    if deps.has_redis {
        yaml.push_str(&generate_redis_service(ports, "redis", ports.redis));
    }

    // Redis (session)
    if deps.has_redis_session {
        yaml.push_str(&generate_redis_service(ports, "redis-session", ports.redis_session));
    }

    // Keycloak
    if *auth == AuthMode::Keycloak {
        yaml.push_str(&generate_keycloak_service(ports, !deps.databases.is_empty()));
    }

    // ボリューム定義
    yaml.push_str("\nvolumes:\n");
    if !deps.databases.is_empty() {
        yaml.push_str("  k1s0_dev_postgres_data:\n");
    }
    if deps.has_kafka {
        yaml.push_str("  k1s0_dev_kafka_data:\n");
    }
    if deps.has_redis {
        yaml.push_str("  k1s0_dev_redis_data:\n");
    }
    if deps.has_redis_session {
        yaml.push_str("  k1s0_dev_redis_session_data:\n");
    }

    yaml
}

/// PostgreSQL サービス定義を生成する。
fn generate_postgres_service(ports: &PortAssignments, first_db_name: &str) -> String {
    format!(
        r#"  postgres:
    image: postgres:17
    container_name: k1s0-dev-postgres
    environment:
      POSTGRES_USER: app
      POSTGRES_PASSWORD: password
      POSTGRES_DB: "{db_name}"
    ports:
      - "{port}:5432"
    volumes:
      - k1s0_dev_postgres_data:/var/lib/postgresql/data
      - ./init-db.sql:/docker-entrypoint-initdb.d/init.sql
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U app"]
      interval: 5s
      timeout: 5s
      retries: 5

"#,
        port = ports.postgres,
        db_name = first_db_name
    )
}

/// pgAdmin サービス定義を生成する。
fn generate_pgadmin_service(ports: &PortAssignments) -> String {
    format!(
        r#"  pgadmin:
    image: dpage/pgadmin4:latest
    container_name: k1s0-dev-pgadmin
    environment:
      PGADMIN_DEFAULT_EMAIL: admin@example.com
      PGADMIN_DEFAULT_PASSWORD: admin
      PGADMIN_CONFIG_SERVER_MODE: "False"
    ports:
      - "{port}:80"
    depends_on:
      postgres:
        condition: service_healthy

"#,
        port = ports.pgadmin
    )
}

/// Kafka サービス定義を生成する（KRaft モード）。
fn generate_kafka_service(ports: &PortAssignments) -> String {
    format!(
        r#"  kafka:
    image: confluentinc/cp-kafka:7.7.0
    container_name: k1s0-dev-kafka
    environment:
      KAFKA_NODE_ID: 1
      KAFKA_PROCESS_ROLES: broker,controller
      KAFKA_LISTENERS: PLAINTEXT://0.0.0.0:9092,CONTROLLER://0.0.0.0:9093
      KAFKA_ADVERTISED_LISTENERS: PLAINTEXT://localhost:{port}
      KAFKA_CONTROLLER_QUORUM_VOTERS: 1@localhost:9093
      KAFKA_CONTROLLER_LISTENER_NAMES: CONTROLLER
      KAFKA_LISTENER_SECURITY_PROTOCOL_MAP: PLAINTEXT:PLAINTEXT,CONTROLLER:PLAINTEXT
      KAFKA_OFFSETS_TOPIC_REPLICATION_FACTOR: 1
      KAFKA_AUTO_CREATE_TOPICS_ENABLE: "true"
      CLUSTER_ID: "k1s0-dev-cluster"
    ports:
      - "{port}:9092"
    volumes:
      - k1s0_dev_kafka_data:/var/lib/kafka/data
    healthcheck:
      test: ["CMD-SHELL", "kafka-broker-api-versions --bootstrap-server localhost:9092"]
      interval: 10s
      timeout: 10s
      retries: 5

"#,
        port = ports.kafka
    )
}

/// Kafka UI サービス定義を生成する。
fn generate_kafka_ui_service(ports: &PortAssignments) -> String {
    format!(
        r#"  kafka-ui:
    image: provectuslabs/kafka-ui:latest
    container_name: k1s0-dev-kafka-ui
    environment:
      KAFKA_CLUSTERS_0_NAME: k1s0-dev
      KAFKA_CLUSTERS_0_BOOTSTRAPSERVERS: kafka:9092
    ports:
      - "{port}:8080"
    depends_on:
      kafka:
        condition: service_healthy

"#,
        port = ports.kafka_ui
    )
}

/// Redis サービス定義を生成する。
fn generate_redis_service(
    _ports: &PortAssignments,
    name: &str,
    host_port: u16,
) -> String {
    let container_name = format!("k1s0-dev-{name}");
    let volume_name = format!("k1s0_dev_{}_data", name.replace('-', "_"));
    format!(
        r#"  {name}:
    image: redis:7
    container_name: {container_name}
    ports:
      - "{host_port}:6379"
    volumes:
      - {volume_name}:/data
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 5s
      timeout: 3s
      retries: 5

"#
    )
}

/// Keycloak サービス定義を生成する。
fn generate_keycloak_service(ports: &PortAssignments, _has_postgres: bool) -> String {
    format!(
        r#"  keycloak:
    image: quay.io/keycloak/keycloak:26.0
    container_name: k1s0-dev-keycloak
    command: start-dev --import-realm
    environment:
      KC_DB: dev-file
      KEYCLOAK_ADMIN: admin
      KEYCLOAK_ADMIN_PASSWORD: admin
    ports:
      - "{port}:8080"
    volumes:
      - ./keycloak/realm-export.json:/opt/keycloak/data/import/realm-export.json
    healthcheck:
      test: ["CMD-SHELL", "exec 3<>/dev/tcp/localhost/8080 && echo -e 'GET /health/ready HTTP/1.1\r\nHost: localhost\r\n\r\n' >&3 && cat <&3 | grep -q '200'"]
      interval: 10s
      timeout: 5s
      retries: 10

"#,
        port = ports.keycloak,
    )
}

/// 複数データベース初期化用 SQL を生成する。
///
/// PostgreSQL の docker-entrypoint-initdb.d で実行される SQL。
pub fn generate_init_db_sql(databases: &[DatabaseDep]) -> String {
    let mut sql = String::new();
    sql.push_str("-- 自動生成: k1s0 dev up\n");
    sql.push_str("-- 各サービスが使用するデータベースを作成する\n\n");

    for db in databases {
        sql.push_str(&format!(
            "-- サービス: {service}\nCREATE DATABASE \"{name}\";\n\n",
            service = db.service,
            name = db.name
        ));
    }

    // Keycloak 用 DB（常に作成しておく）
    sql.push_str("-- Keycloak 用\nCREATE DATABASE \"keycloak\";\n");

    sql
}

/// config.dev-local.yaml を生成する。
///
/// ローカル開発環境向けの設定オーバーライドファイル。
pub fn generate_dev_local_config(
    deps: &DetectedDependencies,
    ports: &PortAssignments,
    auth: &AuthMode,
) -> String {
    let mut yaml = String::new();
    yaml.push_str("# 自動生成: k1s0 dev up\n");
    yaml.push_str("# ローカル開発環境向け設定オーバーライド\n\n");

    yaml.push_str("app:\n");
    yaml.push_str("  environment: dev\n\n");

    // データベース設定
    if let Some(first_db) = deps.databases.first() {
        yaml.push_str("database:\n");
        yaml.push_str("  host: localhost\n");
        yaml.push_str(&format!("  port: {}\n", ports.postgres));
        yaml.push_str(&format!("  name: \"{}\"\n", first_db.name));
        yaml.push_str("  user: app\n");
        yaml.push_str("  password: password\n");
        yaml.push_str("  ssl_mode: disable\n\n");
    }

    // Kafka 設定
    if deps.has_kafka {
        yaml.push_str("kafka:\n");
        yaml.push_str(&format!(
            "  brokers:\n    - \"localhost:{}\"\n",
            ports.kafka
        ));
        yaml.push_str("  security_protocol: PLAINTEXT\n\n");
    }

    // Redis 設定
    if deps.has_redis {
        yaml.push_str("redis:\n");
        yaml.push_str("  host: localhost\n");
        yaml.push_str(&format!("  port: {}\n\n", ports.redis));
    }

    // Redis (session) 設定
    if deps.has_redis_session {
        yaml.push_str("redis_session:\n");
        yaml.push_str("  host: localhost\n");
        yaml.push_str(&format!("  port: {}\n\n", ports.redis_session));
    }

    // 認証設定
    match auth {
        AuthMode::Skip => {
            yaml.push_str("auth:\n");
            yaml.push_str("  skip_verification: true\n");
            yaml.push_str("  mock_claims:\n");
            yaml.push_str("    sub: \"dev-user-001\"\n");
            yaml.push_str("    realm_access:\n");
            yaml.push_str("      roles: [\"sys_admin\"]\n");
            yaml.push_str("    tier_access: [\"system\", \"business\", \"service\"]\n");
        }
        AuthMode::Keycloak => {
            yaml.push_str("auth:\n");
            yaml.push_str("  jwt:\n");
            yaml.push_str(&format!(
                "    issuer: \"http://localhost:{}/realms/k1s0\"\n",
                ports.keycloak
            ));
            yaml.push_str("    audience: k1s0-api\n");
            yaml.push_str("  oidc:\n");
            yaml.push_str(&format!(
                "    discovery_url: \"http://localhost:{}/.well-known/openid-configuration\"\n",
                ports.keycloak
            ));
            yaml.push_str("    client_id: k1s0-dev\n");
            yaml.push_str("    client_secret: \"\"\n");
        }
    }

    yaml
}

#[cfg(test)]
mod tests {
    use super::*;

    /// PostgreSQL のみの依存でdocker-compose.yamlを生成できる。
    #[test]
    fn test_generate_compose_postgres_only() {
        let deps = DetectedDependencies {
            databases: vec![DatabaseDep {
                name: "test_db".to_string(),
                service: "test-server".to_string(),
            }],
            ..Default::default()
        };
        let ports = PortAssignments {
            postgres: 5432,
            pgadmin: 5050,
            ..Default::default()
        };

        let yaml = generate_compose(&deps, &ports, &AuthMode::Skip);

        assert!(yaml.contains("postgres:"));
        assert!(yaml.contains("postgres:17"));
        assert!(yaml.contains("k1s0-dev-postgres"));
        assert!(yaml.contains("5432:5432"));
        assert!(yaml.contains("POSTGRES_DB: \"test_db\""));
        assert!(yaml.contains("k1s0_dev_postgres_data"));
        assert!(yaml.contains("pgadmin:"));
        assert!(yaml.contains("k1s0-dev-pgadmin"));
        assert!(yaml.contains("admin@example.com"));
        assert!(yaml.contains("PGADMIN_CONFIG_SERVER_MODE"));
        assert!(yaml.contains("5050:80"));
        assert!(!yaml.contains("kafka:"));
        assert!(!yaml.contains("redis:"));
        assert!(!yaml.contains("keycloak:"));
    }

    /// Kafka のみの依存でdocker-compose.yamlを生成できる。
    #[test]
    fn test_generate_compose_kafka_only() {
        let deps = DetectedDependencies {
            has_kafka: true,
            kafka_topics: vec!["test.topic".to_string()],
            ..Default::default()
        };
        let ports = PortAssignments {
            kafka: 9092,
            kafka_ui: 8081,
            ..Default::default()
        };

        let yaml = generate_compose(&deps, &ports, &AuthMode::Skip);

        assert!(yaml.contains("kafka:"));
        assert!(yaml.contains("confluentinc/cp-kafka:7.7.0"));
        assert!(yaml.contains("k1s0-dev-kafka"));
        assert!(yaml.contains("9092:9092"));
        assert!(yaml.contains("KAFKA_AUTO_CREATE_TOPICS_ENABLE"));
        assert!(yaml.contains("\"k1s0-dev-cluster\""));
        assert!(yaml.contains("k1s0_dev_kafka_data"));
        assert!(yaml.contains("kafka-ui:"));
        assert!(yaml.contains("k1s0-dev-kafka-ui"));
        assert!(yaml.contains("kafka:9092"));
        assert!(yaml.contains("8081:8080"));
        assert!(!yaml.contains("postgres:"));
    }

    /// Redis 依存を含む docker-compose.yaml を生成できる。
    #[test]
    fn test_generate_compose_redis() {
        let deps = DetectedDependencies {
            has_redis: true,
            has_redis_session: true,
            ..Default::default()
        };
        let ports = PortAssignments {
            redis: 6379,
            redis_session: 6380,
            ..Default::default()
        };

        let yaml = generate_compose(&deps, &ports, &AuthMode::Skip);

        assert!(yaml.contains("redis:"));
        assert!(yaml.contains("redis:7"));
        assert!(yaml.contains("k1s0-dev-redis"));
        assert!(yaml.contains("6379:6379"));
        assert!(yaml.contains("timeout: 3s"));
        assert!(yaml.contains("k1s0_dev_redis_data"));
        assert!(yaml.contains("redis-session:"));
        assert!(yaml.contains("k1s0-dev-redis-session"));
        assert!(yaml.contains("6380:6379"));
        assert!(yaml.contains("k1s0_dev_redis_session_data"));
    }

    /// Keycloak 認証モードで docker-compose.yaml を生成できる。
    #[test]
    fn test_generate_compose_with_keycloak() {
        let deps = DetectedDependencies {
            databases: vec![DatabaseDep {
                name: "test_db".to_string(),
                service: "test-server".to_string(),
            }],
            ..Default::default()
        };
        let ports = PortAssignments {
            postgres: 5432,
            pgadmin: 5050,
            keycloak: 8180,
            ..Default::default()
        };

        let yaml = generate_compose(&deps, &ports, &AuthMode::Keycloak);

        assert!(yaml.contains("keycloak:"));
        assert!(yaml.contains("quay.io/keycloak/keycloak:26.0"));
        assert!(yaml.contains("k1s0-dev-keycloak"));
        assert!(yaml.contains("start-dev --import-realm"));
        assert!(yaml.contains("KC_DB: dev-file"));
        assert!(yaml.contains("8180:8080"));
        assert!(yaml.contains("realm-export.json"));
        assert!(yaml.contains("healthcheck:"));
    }

    /// init-db SQL を正しく生成できる。
    #[test]
    fn test_generate_init_db_sql() {
        let databases = vec![
            DatabaseDep {
                name: "order_db".to_string(),
                service: "order-server".to_string(),
            },
            DatabaseDep {
                name: "auth_db".to_string(),
                service: "auth-server".to_string(),
            },
        ];

        let sql = generate_init_db_sql(&databases);

        assert!(sql.contains("CREATE DATABASE \"order_db\""));
        assert!(sql.contains("CREATE DATABASE \"auth_db\""));
        assert!(sql.contains("CREATE DATABASE \"keycloak\""));
        assert!(sql.contains("-- サービス: order-server"));
        assert!(sql.contains("-- サービス: auth-server"));
    }

    /// 空のデータベースリストでも keycloak DB は作成される。
    #[test]
    fn test_generate_init_db_sql_empty() {
        let sql = generate_init_db_sql(&[]);
        assert!(sql.contains("CREATE DATABASE \"keycloak\""));
    }

    /// dev-local 設定ファイルを生成できる。
    #[test]
    fn test_generate_dev_local_config_skip_auth() {
        let deps = DetectedDependencies {
            databases: vec![DatabaseDep {
                name: "test_db".to_string(),
                service: "test-server".to_string(),
            }],
            has_kafka: true,
            has_redis: true,
            ..Default::default()
        };
        let ports = PortAssignments {
            postgres: 5432,
            kafka: 9092,
            redis: 6379,
            ..Default::default()
        };

        let config = generate_dev_local_config(&deps, &ports, &AuthMode::Skip);

        assert!(config.contains("database:"));
        assert!(config.contains("port: 5432"));
        assert!(config.contains("name: \"test_db\""));
        assert!(config.contains("kafka:"));
        assert!(config.contains("localhost:9092"));
        assert!(config.contains("redis:"));
        assert!(config.contains("port: 6379"));
        assert!(config.contains("skip_verification: true"));
        assert!(config.contains("mock_claims:"));
        assert!(config.contains("sub: \"dev-user-001\""));
    }

    /// Keycloak モードの dev-local 設定ファイルを生成できる。
    #[test]
    fn test_generate_dev_local_config_keycloak_auth() {
        let deps = DetectedDependencies {
            databases: vec![DatabaseDep {
                name: "db".to_string(),
                service: "svc".to_string(),
            }],
            ..Default::default()
        };
        let ports = PortAssignments {
            postgres: 5432,
            keycloak: 8180,
            ..Default::default()
        };

        let config = generate_dev_local_config(&deps, &ports, &AuthMode::Keycloak);

        assert!(config.contains("issuer: \"http://localhost:8180/realms/k1s0\""));
        assert!(config.contains("discovery_url:"));
        assert!(config.contains("client_id: k1s0-dev"));
    }

    /// 全サービスを含む docker-compose.yaml を生成できる。
    #[test]
    fn test_generate_compose_all_services() {
        let deps = DetectedDependencies {
            databases: vec![DatabaseDep {
                name: "db".to_string(),
                service: "svc".to_string(),
            }],
            has_kafka: true,
            has_redis: true,
            has_redis_session: true,
            kafka_topics: vec!["t".to_string()],
        };
        let ports = PortAssignments {
            postgres: 5432,
            kafka: 9092,
            redis: 6379,
            redis_session: 6380,
            pgadmin: 5050,
            kafka_ui: 8081,
            keycloak: 8180,
        };

        let yaml = generate_compose(&deps, &ports, &AuthMode::Keycloak);

        // 全サービスが含まれている
        assert!(yaml.contains("postgres:"));
        assert!(yaml.contains("pgadmin:"));
        assert!(yaml.contains("kafka:"));
        assert!(yaml.contains("kafka-ui:"));
        assert!(yaml.contains("redis:"));
        assert!(yaml.contains("redis-session:"));
        assert!(yaml.contains("keycloak:"));

        // 全ボリュームが含まれている
        assert!(yaml.contains("k1s0_dev_postgres_data:"));
        assert!(yaml.contains("k1s0_dev_kafka_data:"));
        assert!(yaml.contains("k1s0_dev_redis_data:"));
        assert!(yaml.contains("k1s0_dev_redis_session_data:"));
    }
}

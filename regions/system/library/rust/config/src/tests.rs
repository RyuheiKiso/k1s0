use super::*;
use std::collections::HashMap;
use std::io::Write;
use tempfile::NamedTempFile;

const MINIMAL_CONFIG: &str = r#"
app:
  name: test-server
  version: "1.0.0"
  tier: system
  environment: dev
server:
  host: "0.0.0.0"
  port: 8080
observability:
  log:
    level: debug
    format: json
  trace:
    enabled: false
  metrics:
    enabled: false
auth:
  jwt:
    issuer: "http://localhost:8180/realms/k1s0"
    audience: "k1s0-api"
"#;

#[test]
fn test_load() {
    let mut f = NamedTempFile::new().unwrap();
    write!(f, "{}", MINIMAL_CONFIG).unwrap();
    let cfg = load(f.path().to_str().unwrap(), None).unwrap();
    assert_eq!(cfg.app.name, "test-server");
    assert_eq!(cfg.server.port, 8080);
}

#[test]
fn test_load_file_not_found() {
    let result = load("/nonexistent/config.yaml", None);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("failed to read file"));
}

#[test]
fn test_load_invalid_yaml() {
    let mut f = NamedTempFile::new().unwrap();
    write!(f, "invalid: [yaml: broken").unwrap();
    let result = load(f.path().to_str().unwrap(), None);
    assert!(result.is_err());
}

#[test]
fn test_load_with_env_override() {
    let mut base = NamedTempFile::new().unwrap();
    write!(base, "{}", MINIMAL_CONFIG).unwrap();

    let mut env = NamedTempFile::new().unwrap();
    write!(
        env,
        r#"
app:
  environment: staging
server:
  port: 9090
observability:
  log:
    level: info
"#
    )
    .unwrap();

    let cfg = load(
        base.path().to_str().unwrap(),
        Some(env.path().to_str().unwrap()),
    )
    .unwrap();
    assert_eq!(cfg.app.environment, "staging");
    assert_eq!(cfg.server.port, 9090);
    assert_eq!(cfg.app.name, "test-server"); // base value preserved
}

#[test]
fn test_validate_valid_config() {
    let mut f = NamedTempFile::new().unwrap();
    write!(f, "{}", MINIMAL_CONFIG).unwrap();
    let cfg = load(f.path().to_str().unwrap(), None).unwrap();
    assert!(validate(&cfg).is_ok());
}

#[test]
fn test_validate_missing_name() {
    let cfg = Config {
        app: AppConfig {
            name: "".into(),
            version: "1.0".into(),
            tier: "system".into(),
            environment: "dev".into(),
        },
        server: ServerConfig {
            host: "0.0.0.0".into(),
            port: 8080,
            read_timeout: None,
            write_timeout: None,
            shutdown_timeout: None,
        },
        observability: ObservabilityConfig {
            log: LogConfig {
                level: "info".into(),
                format: "json".into(),
            },
            trace: TraceConfig {
                enabled: false,
                endpoint: None,
                sample_rate: None,
            },
            metrics: MetricsConfig {
                enabled: false,
                path: None,
            },
        },
        auth: AuthConfig {
            jwt: JwtConfig {
                issuer: "x".into(),
                audience: "x".into(),
                public_key_path: None,
            },
            oidc: None,
        },
        grpc: None,
        database: None,
        kafka: None,
        redis: None,
        redis_session: None,
    };
    assert!(validate(&cfg).is_err());
}

#[test]
fn test_validate_invalid_tier() {
    let cfg = Config {
        app: AppConfig {
            name: "test".into(),
            version: "1.0".into(),
            tier: "invalid".into(),
            environment: "dev".into(),
        },
        server: ServerConfig {
            host: "0.0.0.0".into(),
            port: 8080,
            read_timeout: None,
            write_timeout: None,
            shutdown_timeout: None,
        },
        observability: ObservabilityConfig {
            log: LogConfig {
                level: "info".into(),
                format: "json".into(),
            },
            trace: TraceConfig {
                enabled: false,
                endpoint: None,
                sample_rate: None,
            },
            metrics: MetricsConfig {
                enabled: false,
                path: None,
            },
        },
        auth: AuthConfig {
            jwt: JwtConfig {
                issuer: "x".into(),
                audience: "x".into(),
                public_key_path: None,
            },
            oidc: None,
        },
        grpc: None,
        database: None,
        kafka: None,
        redis: None,
        redis_session: None,
    };
    assert!(validate(&cfg).is_err());
}

#[test]
fn test_validate_invalid_environment() {
    let cfg = Config {
        app: AppConfig {
            name: "test".into(),
            version: "1.0".into(),
            tier: "system".into(),
            environment: "invalid".into(),
        },
        server: ServerConfig {
            host: "0.0.0.0".into(),
            port: 8080,
            read_timeout: None,
            write_timeout: None,
            shutdown_timeout: None,
        },
        observability: ObservabilityConfig {
            log: LogConfig {
                level: "info".into(),
                format: "json".into(),
            },
            trace: TraceConfig {
                enabled: false,
                endpoint: None,
                sample_rate: None,
            },
            metrics: MetricsConfig {
                enabled: false,
                path: None,
            },
        },
        auth: AuthConfig {
            jwt: JwtConfig {
                issuer: "x".into(),
                audience: "x".into(),
                public_key_path: None,
            },
            oidc: None,
        },
        grpc: None,
        database: None,
        kafka: None,
        redis: None,
        redis_session: None,
    };
    assert!(validate(&cfg).is_err());
}

#[test]
fn test_validate_zero_port() {
    let cfg = Config {
        app: AppConfig {
            name: "test".into(),
            version: "1.0".into(),
            tier: "system".into(),
            environment: "dev".into(),
        },
        server: ServerConfig {
            host: "0.0.0.0".into(),
            port: 0,
            read_timeout: None,
            write_timeout: None,
            shutdown_timeout: None,
        },
        observability: ObservabilityConfig {
            log: LogConfig {
                level: "info".into(),
                format: "json".into(),
            },
            trace: TraceConfig {
                enabled: false,
                endpoint: None,
                sample_rate: None,
            },
            metrics: MetricsConfig {
                enabled: false,
                path: None,
            },
        },
        auth: AuthConfig {
            jwt: JwtConfig {
                issuer: "x".into(),
                audience: "x".into(),
                public_key_path: None,
            },
            oidc: None,
        },
        grpc: None,
        database: None,
        kafka: None,
        redis: None,
        redis_session: None,
    };
    assert!(validate(&cfg).is_err());
}

#[test]
fn test_merge_vault_secrets_database() {
    let mut f = NamedTempFile::new().unwrap();
    write!(
        f,
        r#"
app:
  name: test
  version: "1.0"
  tier: system
  environment: dev
server:
  host: "0.0.0.0"
  port: 8080
database:
  host: localhost
  port: 5432
  name: test_db
  user: app
  password: ""
observability:
  log:
    level: info
    format: json
  trace:
    enabled: false
  metrics:
    enabled: false
auth:
  jwt:
    issuer: "http://localhost"
    audience: "test"
"#
    )
    .unwrap();

    let mut cfg = load(f.path().to_str().unwrap(), None).unwrap();
    let mut secrets = HashMap::new();
    secrets.insert("database.password".to_string(), "secret123".to_string());
    merge_vault_secrets(&mut cfg, &secrets);
    assert_eq!(cfg.database.unwrap().password, "secret123");
}

#[test]
fn test_merge_vault_secrets_redis() {
    let mut f = NamedTempFile::new().unwrap();
    write!(
        f,
        r#"
app:
  name: test
  version: "1.0"
  tier: system
  environment: dev
server:
  host: "0.0.0.0"
  port: 8080
redis:
  host: localhost
  port: 6379
observability:
  log:
    level: info
    format: json
  trace:
    enabled: false
  metrics:
    enabled: false
auth:
  jwt:
    issuer: "http://localhost"
    audience: "test"
"#
    )
    .unwrap();

    let mut cfg = load(f.path().to_str().unwrap(), None).unwrap();
    let mut secrets = HashMap::new();
    secrets.insert("redis.password".to_string(), "redis-secret".to_string());
    merge_vault_secrets(&mut cfg, &secrets);
    assert_eq!(
        cfg.redis.unwrap().password,
        Some("redis-secret".to_string())
    );
}

#[test]
fn test_merge_vault_secrets_oidc() {
    let mut f = NamedTempFile::new().unwrap();
    write!(
        f,
        r#"
app:
  name: test
  version: "1.0"
  tier: system
  environment: dev
server:
  host: "0.0.0.0"
  port: 8080
observability:
  log:
    level: info
    format: json
  trace:
    enabled: false
  metrics:
    enabled: false
auth:
  jwt:
    issuer: "http://localhost"
    audience: "test"
  oidc:
    discovery_url: "http://localhost/.well-known"
    client_id: "test"
    redirect_uri: "http://localhost/callback"
    scopes: ["openid"]
    jwks_uri: "http://localhost/jwks"
"#
    )
    .unwrap();

    let mut cfg = load(f.path().to_str().unwrap(), None).unwrap();
    let mut secrets = HashMap::new();
    secrets.insert(
        "auth.oidc.client_secret".to_string(),
        "oidc-secret".to_string(),
    );
    merge_vault_secrets(&mut cfg, &secrets);
    assert_eq!(
        cfg.auth.oidc.unwrap().client_secret,
        Some("oidc-secret".to_string())
    );
}

#[test]
fn test_merge_vault_secrets_nil_fields() {
    let mut f = NamedTempFile::new().unwrap();
    write!(f, "{}", MINIMAL_CONFIG).unwrap();
    let mut cfg = load(f.path().to_str().unwrap(), None).unwrap();
    let mut secrets = HashMap::new();
    secrets.insert("database.password".to_string(), "secret".to_string());
    secrets.insert("redis.password".to_string(), "secret".to_string());
    secrets.insert(
        "auth.oidc.client_secret".to_string(),
        "secret".to_string(),
    );
    // Should not panic when optional fields are None
    merge_vault_secrets(&mut cfg, &secrets);
    assert!(cfg.database.is_none());
    assert!(cfg.redis.is_none());
    assert!(cfg.auth.oidc.is_none());
}

#[test]
fn test_load_full_config() {
    let mut f = NamedTempFile::new().unwrap();
    write!(
        f,
        r#"
app:
  name: order-server
  version: "1.0.0"
  tier: service
  environment: dev
server:
  host: "0.0.0.0"
  port: 8080
  read_timeout: "30s"
  write_timeout: "30s"
  shutdown_timeout: "10s"
grpc:
  port: 50051
  max_recv_msg_size: 4194304
database:
  host: "localhost"
  port: 5432
  name: "order_db"
  user: "app"
  password: ""
  ssl_mode: "disable"
  max_open_conns: 25
  max_idle_conns: 5
  conn_max_lifetime: "5m"
kafka:
  brokers:
    - "localhost:9092"
  consumer_group: "order-server.default"
  security_protocol: "PLAINTEXT"
  topics:
    publish:
      - "k1s0.service.order.created.v1"
    subscribe:
      - "k1s0.service.payment.completed.v1"
redis:
  host: "localhost"
  port: 6379
  db: 0
  pool_size: 10
observability:
  log:
    level: info
    format: json
  trace:
    enabled: true
    endpoint: "localhost:4317"
    sample_rate: 1.0
  metrics:
    enabled: true
    path: "/metrics"
auth:
  jwt:
    issuer: "http://localhost:8180/realms/k1s0"
    audience: "k1s0-api"
  oidc:
    discovery_url: "http://localhost:8180/realms/k1s0/.well-known/openid-configuration"
    client_id: "k1s0-bff"
    redirect_uri: "http://localhost:3000/callback"
    scopes:
      - "openid"
      - "profile"
    jwks_uri: "http://localhost:8180/realms/k1s0/protocol/openid-connect/certs"
    jwks_cache_ttl: "10m"
"#
    )
    .unwrap();

    let cfg = load(f.path().to_str().unwrap(), None).unwrap();
    assert_eq!(cfg.app.name, "order-server");
    assert_eq!(cfg.app.tier, "service");
    assert!(cfg.grpc.is_some());
    assert_eq!(cfg.grpc.as_ref().unwrap().port, 50051);
    assert!(cfg.database.is_some());
    assert_eq!(cfg.database.as_ref().unwrap().name, "order_db");
    assert!(cfg.kafka.is_some());
    assert_eq!(cfg.kafka.as_ref().unwrap().security_protocol, "PLAINTEXT");
    assert!(cfg.redis.is_some());
    assert_eq!(cfg.redis.as_ref().unwrap().port, 6379);
    assert!(cfg.auth.oidc.is_some());
    assert_eq!(cfg.auth.oidc.as_ref().unwrap().client_id, "k1s0-bff");

    assert!(validate(&cfg).is_ok());
}

use std::collections::HashMap;

use crate::vault::merge_vault_secrets;
use crate::types::*;

fn minimal_config() -> Config {
    Config {
        app: AppConfig {
            name: "test".into(),
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
        grpc: None,
        database: None,
        kafka: None,
        redis: None,
        redis_session: None,
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
                issuer: "http://localhost".into(),
                audience: "test".into(),
                public_key_path: None,
            },
            oidc: None,
        },
    }
}

#[test]
fn test_vault_database_password_merge() {
    let mut cfg = minimal_config();
    cfg.database = Some(DatabaseConfig {
        host: "localhost".into(),
        port: 5432,
        name: "test_db".into(),
        user: "app".into(),
        password: "old".into(),
        ssl_mode: None,
        max_open_conns: None,
        max_idle_conns: None,
        conn_max_lifetime: None,
    });
    let secrets = HashMap::from([("database.password".into(), "vault-db-pass".into())]);
    merge_vault_secrets(&mut cfg, &secrets);
    assert_eq!(cfg.database.unwrap().password, "vault-db-pass");
}

#[test]
fn test_vault_redis_password_merge() {
    let mut cfg = minimal_config();
    cfg.redis = Some(RedisConfig {
        host: "localhost".into(),
        port: 6379,
        password: Some("old".into()),
        db: None,
        pool_size: None,
    });
    let secrets = HashMap::from([("redis.password".into(), "vault-redis-pass".into())]);
    merge_vault_secrets(&mut cfg, &secrets);
    assert_eq!(
        cfg.redis.unwrap().password,
        Some("vault-redis-pass".into())
    );
}

#[test]
fn test_vault_kafka_sasl_merge() {
    let mut cfg = minimal_config();
    cfg.kafka = Some(KafkaConfig {
        brokers: vec!["localhost:9092".into()],
        consumer_group: "test.default".into(),
        security_protocol: "SASL_SSL".into(),
        sasl: Some(KafkaSaslConfig {
            mechanism: "SCRAM-SHA-512".into(),
            username: "".into(),
            password: "".into(),
        }),
        tls: None,
        topics: KafkaTopics {
            publish: vec![],
            subscribe: vec![],
        },
    });
    let secrets = HashMap::from([
        ("kafka.sasl.username".into(), "vault-kafka-user".into()),
        ("kafka.sasl.password".into(), "vault-kafka-pass".into()),
    ]);
    merge_vault_secrets(&mut cfg, &secrets);
    let sasl = cfg.kafka.unwrap().sasl.unwrap();
    assert_eq!(sasl.username, "vault-kafka-user");
    assert_eq!(sasl.password, "vault-kafka-pass");
}

#[test]
fn test_vault_redis_session_password_merge() {
    let mut cfg = minimal_config();
    cfg.redis_session = Some(RedisConfig {
        host: "localhost".into(),
        port: 6380,
        password: None,
        db: None,
        pool_size: None,
    });
    let secrets = HashMap::from([("redis_session.password".into(), "vault-session-pass".into())]);
    merge_vault_secrets(&mut cfg, &secrets);
    assert_eq!(
        cfg.redis_session.unwrap().password,
        Some("vault-session-pass".into())
    );
}

#[test]
fn test_vault_oidc_client_secret_merge() {
    let mut cfg = minimal_config();
    cfg.auth.oidc = Some(OidcConfig {
        discovery_url: "http://localhost/.well-known".into(),
        client_id: "test".into(),
        client_secret: None,
        redirect_uri: "http://localhost/callback".into(),
        scopes: vec!["openid".into()],
        jwks_uri: "http://localhost/jwks".into(),
        jwks_cache_ttl: None,
    });
    let secrets =
        HashMap::from([("auth.oidc.client_secret".into(), "vault-oidc-secret".into())]);
    merge_vault_secrets(&mut cfg, &secrets);
    assert_eq!(
        cfg.auth.oidc.unwrap().client_secret,
        Some("vault-oidc-secret".into())
    );
}

#[test]
fn test_vault_empty_secrets_no_change() {
    let mut cfg = minimal_config();
    cfg.database = Some(DatabaseConfig {
        host: "localhost".into(),
        port: 5432,
        name: "test_db".into(),
        user: "app".into(),
        password: "original".into(),
        ssl_mode: None,
        max_open_conns: None,
        max_idle_conns: None,
        conn_max_lifetime: None,
    });
    cfg.redis = Some(RedisConfig {
        host: "localhost".into(),
        port: 6379,
        password: Some("original".into()),
        db: None,
        pool_size: None,
    });
    let secrets: HashMap<String, String> = HashMap::new();
    merge_vault_secrets(&mut cfg, &secrets);
    assert_eq!(cfg.database.as_ref().unwrap().password, "original");
    assert_eq!(
        cfg.redis.as_ref().unwrap().password,
        Some("original".into())
    );
}

#[test]
fn test_vault_nil_sections_safe() {
    let mut cfg = minimal_config();
    // All optional sections are None
    let secrets = HashMap::from([
        ("database.password".into(), "secret".into()),
        ("redis.password".into(), "secret".into()),
        ("kafka.sasl.username".into(), "user".into()),
        ("kafka.sasl.password".into(), "pass".into()),
        ("redis_session.password".into(), "secret".into()),
        ("auth.oidc.client_secret".into(), "secret".into()),
    ]);
    // Should not panic
    merge_vault_secrets(&mut cfg, &secrets);
    assert!(cfg.database.is_none());
    assert!(cfg.redis.is_none());
    assert!(cfg.kafka.is_none());
    assert!(cfg.redis_session.is_none());
    assert!(cfg.auth.oidc.is_none());
}

#[test]
fn test_vault_partial_secrets() {
    let mut cfg = minimal_config();
    cfg.database = Some(DatabaseConfig {
        host: "localhost".into(),
        port: 5432,
        name: "test_db".into(),
        user: "app".into(),
        password: "old-db".into(),
        ssl_mode: None,
        max_open_conns: None,
        max_idle_conns: None,
        conn_max_lifetime: None,
    });
    cfg.redis = Some(RedisConfig {
        host: "localhost".into(),
        port: 6379,
        password: Some("old-redis".into()),
        db: None,
        pool_size: None,
    });
    cfg.auth.oidc = Some(OidcConfig {
        discovery_url: "http://localhost/.well-known".into(),
        client_id: "test".into(),
        client_secret: Some("old-oidc".into()),
        redirect_uri: "http://localhost/callback".into(),
        scopes: vec!["openid".into()],
        jwks_uri: "http://localhost/jwks".into(),
        jwks_cache_ttl: None,
    });
    // Only database.password is provided
    let secrets = HashMap::from([("database.password".into(), "new-db".into())]);
    merge_vault_secrets(&mut cfg, &secrets);
    assert_eq!(cfg.database.as_ref().unwrap().password, "new-db");
    assert_eq!(
        cfg.redis.as_ref().unwrap().password,
        Some("old-redis".into())
    );
    assert_eq!(
        cfg.auth.oidc.as_ref().unwrap().client_secret,
        Some("old-oidc".into())
    );
}

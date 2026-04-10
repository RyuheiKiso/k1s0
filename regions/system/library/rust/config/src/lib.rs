mod merge;
mod vault;

use thiserror::Error;

pub use merge::merge_yaml;
pub use vault::merge_vault_secrets;

mod types;
pub use types::*;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("failed to read file: {0}")]
    ReadFile(#[from] std::io::Error),
    #[error("failed to parse YAML: {0}")]
    ParseYaml(#[from] serde_yaml::Error),
    #[error("validation error: {0}")]
    Validation(String),
}

/// YAML を読み込み Config `を返す。env_path` があればマージする。
pub fn load(base_path: &str, env_path: Option<&str>) -> Result<Config, ConfigError> {
    let base = std::fs::read_to_string(base_path)?;
    let mut config: Config = serde_yaml::from_str(&base)?;

    if let Some(env) = env_path {
        let env_data = std::fs::read_to_string(env)?;
        let env_config: serde_yaml::Value = serde_yaml::from_str(&env_data)?;
        let mut base_value: serde_yaml::Value = serde_yaml::from_str(&base)?;
        merge_yaml(&mut base_value, &env_config);
        config = serde_yaml::from_value(base_value)?;
    }

    Ok(config)
}

/// 設定値のバリデーション。
// バリデーション項目が多いため行数が多くなる
#[allow(clippy::too_many_lines)]
pub fn validate(config: &Config) -> Result<(), ConfigError> {
    require_non_empty("app.name", &config.app.name)?;
    require_non_empty("app.version", &config.app.version)?;
    validate_one_of(
        "app.tier",
        &config.app.tier,
        &["system", "business", "service"],
    )?;
    validate_one_of(
        "app.environment",
        &config.app.environment,
        &["dev", "staging", "prod"],
    )?;

    require_non_empty("server.host", &config.server.host)?;
    validate_port("server.port", config.server.port)?;

    if let Some(grpc) = &config.grpc {
        validate_port("grpc.port", grpc.port)?;
        if let Some(max_recv_msg_size) = grpc.max_recv_msg_size {
            ensure(max_recv_msg_size > 0, "grpc.max_recv_msg_size must be > 0")?;
        }
    }

    if let Some(database) = &config.database {
        require_non_empty("database.host", &database.host)?;
        validate_port("database.port", database.port)?;
        require_non_empty("database.name", &database.name)?;
        require_non_empty("database.user", &database.user)?;
        if let Some(ssl_mode) = &database.ssl_mode {
            validate_one_of(
                "database.ssl_mode",
                ssl_mode,
                &["disable", "require", "verify-full"],
            )?;
        }
        if let Some(max_open_conns) = database.max_open_conns {
            ensure(max_open_conns > 0, "database.max_open_conns must be > 0")?;
        }
        if let Some(max_idle_conns) = database.max_idle_conns {
            ensure(max_idle_conns > 0, "database.max_idle_conns must be > 0")?;
            if let Some(max_open_conns) = database.max_open_conns {
                ensure(
                    max_idle_conns <= max_open_conns,
                    "database.max_idle_conns must be <= database.max_open_conns",
                )?;
            }
        }
    }

    if let Some(kafka) = &config.kafka {
        ensure(!kafka.brokers.is_empty(), "kafka.brokers is required")?;
        for broker in &kafka.brokers {
            ensure(
                !broker.trim().is_empty(),
                "kafka.brokers must not contain empty values",
            )?;
        }
        require_non_empty("kafka.consumer_group", &kafka.consumer_group)?;
        validate_one_of(
            "kafka.security_protocol",
            &kafka.security_protocol,
            &["PLAINTEXT", "SASL_SSL"],
        )?;
        ensure(
            !(kafka.topics.publish.is_empty() && kafka.topics.subscribe.is_empty()),
            "kafka.topics.publish or kafka.topics.subscribe must have at least one topic",
        )?;
        for topic in &kafka.topics.publish {
            ensure(
                !topic.trim().is_empty(),
                "kafka.topics.publish must not contain empty topic names",
            )?;
        }
        for topic in &kafka.topics.subscribe {
            ensure(
                !topic.trim().is_empty(),
                "kafka.topics.subscribe must not contain empty topic names",
            )?;
        }

        if kafka.security_protocol == "SASL_SSL" {
            ensure(
                kafka.sasl.is_some(),
                "kafka.sasl is required when kafka.security_protocol is SASL_SSL",
            )?;
        }
        if let Some(sasl) = &kafka.sasl {
            validate_one_of(
                "kafka.sasl.mechanism",
                &sasl.mechanism,
                &["SCRAM-SHA-512", "PLAIN"],
            )?;
            require_non_empty("kafka.sasl.username", &sasl.username)?;
            require_non_empty("kafka.sasl.password", &sasl.password)?;
        }
        if let Some(tls) = &kafka.tls {
            if let Some(ca_cert_path) = &tls.ca_cert_path {
                ensure(
                    !ca_cert_path.trim().is_empty(),
                    "kafka.tls.ca_cert_path must not be empty",
                )?;
            }
        }
    }

    if let Some(redis) = &config.redis {
        require_non_empty("redis.host", &redis.host)?;
        validate_port("redis.port", redis.port)?;
        if let Some(pool_size) = redis.pool_size {
            ensure(pool_size > 0, "redis.pool_size must be > 0")?;
        }
    }
    if let Some(redis_session) = &config.redis_session {
        require_non_empty("redis_session.host", &redis_session.host)?;
        validate_port("redis_session.port", redis_session.port)?;
        if let Some(pool_size) = redis_session.pool_size {
            ensure(pool_size > 0, "redis_session.pool_size must be > 0")?;
        }
    }

    validate_one_of(
        "observability.log.level",
        &config.observability.log.level,
        &["debug", "info", "warn", "error"],
    )?;
    validate_one_of(
        "observability.log.format",
        &config.observability.log.format,
        &["json", "text"],
    )?;

    if config.observability.trace.enabled {
        let endpoint = config
            .observability
            .trace
            .endpoint
            .as_deref()
            .ok_or_else(|| {
                ConfigError::Validation(
                    "observability.trace.endpoint is required when trace is enabled".into(),
                )
            })?;
        ensure(
            !endpoint.trim().is_empty(),
            "observability.trace.endpoint must not be empty when trace is enabled",
        )?;
    }
    if let Some(sample_rate) = config.observability.trace.sample_rate {
        ensure(
            (0.0..=1.0).contains(&sample_rate),
            "observability.trace.sample_rate must be between 0.0 and 1.0",
        )?;
    }

    if config.observability.metrics.enabled {
        let path = config
            .observability
            .metrics
            .path
            .as_deref()
            .ok_or_else(|| {
                ConfigError::Validation(
                    "observability.metrics.path is required when metrics is enabled".into(),
                )
            })?;
        ensure(
            !path.trim().is_empty(),
            "observability.metrics.path must not be empty when metrics is enabled",
        )?;
        ensure(
            path.starts_with('/'),
            "observability.metrics.path must start with '/'",
        )?;
    }

    require_non_empty("auth.jwt.issuer", &config.auth.jwt.issuer)?;
    require_non_empty("auth.jwt.audience", &config.auth.jwt.audience)?;

    if let Some(oidc) = &config.auth.oidc {
        require_non_empty("auth.oidc.discovery_url", &oidc.discovery_url)?;
        require_non_empty("auth.oidc.client_id", &oidc.client_id)?;
        require_non_empty("auth.oidc.redirect_uri", &oidc.redirect_uri)?;
        require_non_empty("auth.oidc.jwks_uri", &oidc.jwks_uri)?;
        ensure(
            !oidc.scopes.is_empty(),
            "auth.oidc.scopes must not be empty",
        )?;
        for scope in &oidc.scopes {
            ensure(
                !scope.trim().is_empty(),
                "auth.oidc.scopes must not contain empty values",
            )?;
        }
    }

    Ok(())
}

fn require_non_empty(field: &str, value: &str) -> Result<(), ConfigError> {
    ensure(!value.trim().is_empty(), format!("{field} is required"))
}

fn validate_port(field: &str, port: u16) -> Result<(), ConfigError> {
    ensure(port > 0, format!("{field} must be between 1 and 65535"))
}

fn validate_one_of(field: &str, value: &str, allowed: &[&str]) -> Result<(), ConfigError> {
    let is_valid = allowed.iter().any(|v| v == &value);
    ensure(
        is_valid,
        format!("{field} must be one of: {}", allowed.join(", ")),
    )
}

fn ensure(condition: bool, message: impl Into<String>) -> Result<(), ConfigError> {
    if condition {
        Ok(())
    } else {
        Err(ConfigError::Validation(message.into()))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests;
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod vault_test;

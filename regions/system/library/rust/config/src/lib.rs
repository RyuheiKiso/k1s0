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

/// YAML を読み込み Config を返す。env_path があればマージする。
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
pub fn validate(config: &Config) -> Result<(), ConfigError> {
    if config.app.name.is_empty() {
        return Err(ConfigError::Validation("app.name is required".into()));
    }
    if config.app.version.is_empty() {
        return Err(ConfigError::Validation("app.version is required".into()));
    }
    if !["system", "business", "service"].contains(&config.app.tier.as_str()) {
        return Err(ConfigError::Validation(
            "app.tier must be system, business, or service".into(),
        ));
    }
    if !["dev", "staging", "prod"].contains(&config.app.environment.as_str()) {
        return Err(ConfigError::Validation(
            "app.environment must be dev, staging, or prod".into(),
        ));
    }
    if config.server.host.is_empty() {
        return Err(ConfigError::Validation("server.host is required".into()));
    }
    if config.server.port == 0 {
        return Err(ConfigError::Validation("server.port must be > 0".into()));
    }
    if config.auth.jwt.issuer.is_empty() {
        return Err(ConfigError::Validation(
            "auth.jwt.issuer is required".into(),
        ));
    }
    if config.auth.jwt.audience.is_empty() {
        return Err(ConfigError::Validation(
            "auth.jwt.audience is required".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod vault_test;

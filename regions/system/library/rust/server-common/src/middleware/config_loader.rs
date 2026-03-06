use serde::de::DeserializeOwned;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("config file not found: {0}")]
    NotFound(String),
    #[error("config parse error: {0}")]
    ParseError(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub fn load_config<T: DeserializeOwned>(path: &str) -> Result<T, ConfigError> {
    let content =
        std::fs::read_to_string(path).map_err(|_| ConfigError::NotFound(path.to_string()))?;

    serde_yaml::from_str(&content).map_err(|e| ConfigError::ParseError(e.to_string()))
}

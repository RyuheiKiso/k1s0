use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::ComponentError;

/// ComponentsConfig はビルディングブロック群の設定を表す。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentsConfig {
    pub components: Vec<ComponentConfig>,
}

/// ComponentConfig は個々のビルディングブロックの設定を表す。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub component_type: String,
    pub version: Option<String>,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl ComponentsConfig {
    /// YAML 文字列からパースする。
    pub fn from_yaml(yaml: &str) -> Result<Self, ComponentError> {
        serde_yaml::from_str(yaml).map_err(|e| ComponentError::Config(e.to_string()))
    }

    /// YAML ファイルから読み込む。
    pub fn from_file(path: &std::path::Path) -> Result<Self, ComponentError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ComponentError::Config(format!("ファイル読み込みエラー: {e}")))?;
        Self::from_yaml(&content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_yaml_valid() {
        let yaml = r#"
components:
  - name: redis-store
    type: statestore
    version: "1.0"
    metadata:
      host: localhost
      port: "6379"
  - name: kafka-pubsub
    type: pubsub
"#;
        let config = ComponentsConfig::from_yaml(yaml).unwrap();
        assert_eq!(config.components.len(), 2);
        assert_eq!(config.components[0].name, "redis-store");
        assert_eq!(config.components[0].component_type, "statestore");
        assert_eq!(config.components[0].version.as_deref(), Some("1.0"));
        assert_eq!(
            config.components[0].metadata.get("host").unwrap(),
            "localhost"
        );
        assert_eq!(config.components[1].name, "kafka-pubsub");
        assert!(config.components[1].version.is_none());
        assert!(config.components[1].metadata.is_empty());
    }

    #[test]
    fn test_from_yaml_invalid() {
        let yaml = "not: valid: yaml: [";
        let result = ComponentsConfig::from_yaml(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_from_file_not_found() {
        let result = ComponentsConfig::from_file(std::path::Path::new("/nonexistent/path.yaml"));
        assert!(result.is_err());
    }
}

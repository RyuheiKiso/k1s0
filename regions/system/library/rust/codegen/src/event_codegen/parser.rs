use std::fs;
use std::path::Path;

use crate::error::CodegenError;

use super::config::EventConfig;

/// Parse an `events.yaml` file into an `EventConfig`.
pub fn parse_event_config(path: &Path) -> Result<EventConfig, CodegenError> {
    let content = fs::read_to_string(path).map_err(|e| CodegenError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;

    parse_event_config_str(&content)
}

/// Parse an `events.yaml` string into an `EventConfig`.
pub fn parse_event_config_str(yaml: &str) -> Result<EventConfig, CodegenError> {
    serde_yaml::from_str(yaml).map_err(|e| {
        CodegenError::Validation(format!("failed to parse events.yaml: {e}"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_yaml() {
        let yaml = r#"
domain: accounting
tier: business
service_name: domain-master
language: rust
events:
  - name: master-item.created
    version: 1
    partition_key: item_id
    schema:
      fields:
        - name: item_id
          type: string
          number: 1
"#;
        let config = parse_event_config_str(yaml).unwrap();
        assert_eq!(config.domain, "accounting");
        assert_eq!(config.events.len(), 1);
    }

    #[test]
    fn parse_invalid_yaml() {
        let yaml = "not: valid: yaml: [";
        let result = parse_event_config_str(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn parse_missing_required_field() {
        let yaml = r#"
domain: accounting
tier: business
"#;
        let result = parse_event_config_str(yaml);
        assert!(result.is_err());
    }
}

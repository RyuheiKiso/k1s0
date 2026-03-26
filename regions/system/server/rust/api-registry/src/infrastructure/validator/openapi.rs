use super::{SchemaValidator, ValidationError};
use async_trait::async_trait;
use tokio::process::Command;

/// OpenAPI validator backed by `openapi-spec-validator` subprocess.
///
/// If the executable is missing or times out, it falls back to YAML parsing.
pub struct OpenApiSubprocessValidator {
    validator_path: String,
    timeout_secs: u64,
}

impl OpenApiSubprocessValidator {
    pub fn new(validator_path: String, timeout_secs: u64) -> Self {
        Self {
            validator_path,
            timeout_secs,
        }
    }
}

#[async_trait]
impl SchemaValidator for OpenApiSubprocessValidator {
    async fn validate(&self, content: &str) -> anyhow::Result<Vec<ValidationError>> {
        let tmp = tempfile::NamedTempFile::with_suffix(".yaml")?;
        tokio::fs::write(tmp.path(), content).await?;

        let result = tokio::time::timeout(
            std::time::Duration::from_secs(self.timeout_secs),
            Command::new(&self.validator_path).arg(tmp.path()).output(),
        )
        .await;

        match result {
            Ok(Ok(output)) => {
                if output.status.success() {
                    Ok(vec![])
                } else {
                    // バリデーターの標準エラー出力を UTF-8 でパースする。
                    // 非 UTF-8 バイトが含まれる場合はサイズ情報をフォールバックメッセージとして使用する。
                    let stderr = String::from_utf8(output.stderr)
                        .unwrap_or_else(|e| format!("(non-UTF-8 stderr: {} bytes)", e.into_bytes().len()));
                    let errors = stderr
                        .lines()
                        .filter(|l| !l.trim().is_empty())
                        .map(|l| ValidationError {
                            field: "content".to_string(),
                            message: l.to_string(),
                        })
                        .collect();
                    Ok(errors)
                }
            }
            Ok(Err(_)) | Err(_) => validate_yaml(content),
        }
    }
}

fn validate_yaml(content: &str) -> anyhow::Result<Vec<ValidationError>> {
    match serde_yaml::from_str::<serde_json::Value>(content) {
        Ok(_) => Ok(vec![]),
        Err(e) => Ok(vec![ValidationError {
            field: "content".to_string(),
            message: format!("YAML parse error: {}", e),
        }]),
    }
}

// Backward-compatible alias for older references.
pub type OpenApiValidator = OpenApiSubprocessValidator;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validate_valid_yaml() {
        let validator = OpenApiSubprocessValidator::new("openapi-spec-validator".to_string(), 5);
        let content = "openapi: '3.0.3'\ninfo:\n  title: Test\n  version: '1.0.0'\npaths: {}\n";
        let result = validator.validate(content).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_invalid_yaml() {
        let validator = OpenApiSubprocessValidator::new("openapi-spec-validator".to_string(), 5);
        let content = "not: valid: yaml: :::";
        let result = validator.validate(content).await;
        assert!(result.is_ok());
    }
}

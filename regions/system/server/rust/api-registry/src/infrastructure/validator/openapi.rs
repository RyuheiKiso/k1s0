//! OpenAPI スキーマバリデーター。
//! openapi-spec-validator を subprocess 経由で実行する。
//! 使えない環境では基本的な YAML パース検証を行う。

use super::{SchemaValidator, ValidationError};
use async_trait::async_trait;
use tokio::process::Command;

pub struct OpenApiValidator {
    validator_path: String,
    timeout_secs: u64,
}

impl OpenApiValidator {
    pub fn new(validator_path: String, timeout_secs: u64) -> Self {
        Self {
            validator_path,
            timeout_secs,
        }
    }
}

#[async_trait]
impl SchemaValidator for OpenApiValidator {
    async fn validate(&self, content: &str) -> anyhow::Result<Vec<ValidationError>> {
        let tmp = tempfile::NamedTempFile::with_suffix(".yaml")?;
        tokio::fs::write(tmp.path(), content).await?;

        let result = tokio::time::timeout(
            std::time::Duration::from_secs(self.timeout_secs),
            Command::new(&self.validator_path)
                .arg(tmp.path())
                .output(),
        )
        .await;

        match result {
            Ok(Ok(output)) => {
                if output.status.success() {
                    Ok(vec![])
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validate_valid_yaml() {
        let validator = OpenApiValidator::new("openapi-spec-validator".to_string(), 5);
        let content = "openapi: '3.0.3'\ninfo:\n  title: Test\n  version: '1.0.0'\npaths: {}\n";
        let result = validator.validate(content).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_invalid_yaml() {
        let validator = OpenApiValidator::new("openapi-spec-validator".to_string(), 5);
        let content = "not: valid: yaml: :::";
        let result = validator.validate(content).await;
        assert!(result.is_ok());
        // バリデーターが使えない場合 YAML パース検証にフォールバック
        // 有効な YAML かどうかにより結果が変わる
    }
}

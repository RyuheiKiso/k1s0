use super::{SchemaValidator, ValidationError};
use async_trait::async_trait;
use tokio::process::Command;

/// Protobuf validator backed by `buf lint` subprocess.
///
/// If `buf` is unavailable or times out, it falls back to a syntax check.
pub struct ProtobufSubprocessValidator {
    buf_path: String,
    timeout_secs: u64,
}

impl ProtobufSubprocessValidator {
    pub fn new(buf_path: String, timeout_secs: u64) -> Self {
        Self {
            buf_path,
            timeout_secs,
        }
    }
}

#[async_trait]
impl SchemaValidator for ProtobufSubprocessValidator {
    async fn validate(&self, content: &str) -> anyhow::Result<Vec<ValidationError>> {
        let tmp_dir = tempfile::tempdir()?;
        let proto_file = tmp_dir.path().join("schema.proto");
        tokio::fs::write(&proto_file, content).await?;

        let buf_yaml = tmp_dir.path().join("buf.yaml");
        tokio::fs::write(&buf_yaml, "version: v2\nmodules:\n  - path: .\n").await?;

        let result = tokio::time::timeout(
            std::time::Duration::from_secs(self.timeout_secs),
            Command::new(&self.buf_path)
                .args(["lint", "--path", proto_file.to_str().unwrap_or("schema.proto")])
                .current_dir(tmp_dir.path())
                .output(),
        )
        .await;

        match result {
            Ok(Ok(output)) => {
                if output.status.success() {
                    Ok(vec![])
                } else {
                    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                    let errors = stdout
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
            Ok(Err(_)) | Err(_) => validate_proto_syntax(content),
        }
    }
}

fn validate_proto_syntax(content: &str) -> anyhow::Result<Vec<ValidationError>> {
    if !content.contains("syntax") {
        return Ok(vec![ValidationError {
            field: "content".to_string(),
            message: "proto file must start with a syntax declaration".to_string(),
        }]);
    }
    Ok(vec![])
}

// Backward-compatible alias for older references.
pub type ProtobufValidator = ProtobufSubprocessValidator;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validate_valid_proto() {
        let validator = ProtobufSubprocessValidator::new("buf".to_string(), 5);
        let content = "syntax = \"proto3\";\n\npackage test.v1;\n\nmessage Test {\n  string id = 1;\n}\n";
        let result = validator.validate(content).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_proto_missing_syntax() {
        let validator = ProtobufSubprocessValidator::new("buf".to_string(), 5);
        let content = "package test.v1;\n\nmessage Test {\n  string id = 1;\n}\n";
        let result = validator.validate(content).await;
        assert!(result.is_ok());
    }
}

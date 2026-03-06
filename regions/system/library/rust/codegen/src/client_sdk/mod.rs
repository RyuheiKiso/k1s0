pub mod grpc_generator;
pub mod http_generator;
pub mod mock_generator;
pub mod resilient_generator;
pub mod trait_generator;

use std::path::Path;

use crate::error::CodegenError;

/// Client SDK生成の入力パラメータ
#[derive(Debug, Clone)]
pub struct ClientSdkConfig {
    /// サービス名 (e.g., "UserProfile")
    pub service_name: String,
    /// パッケージ名 (e.g., "k1s0-user-profile-client")
    pub package_name: String,
    /// メソッド定義
    pub methods: Vec<ClientMethod>,
    /// 型定義
    pub types: Vec<ClientType>,
}

#[derive(Debug, Clone)]
pub struct ClientMethod {
    pub name: String,
    pub request_type: String,
    pub response_type: String,
}

#[derive(Debug, Clone)]
pub struct ClientType {
    pub name: String,
    pub fields: Vec<ClientField>,
}

#[derive(Debug, Clone)]
pub struct ClientField {
    pub name: String,
    pub field_type: String,
}

/// Client SDKの全ファイルを生成する
pub fn generate_client_sdk(
    config: &ClientSdkConfig,
    output_dir: &Path,
) -> Result<Vec<std::path::PathBuf>, CodegenError> {
    let mut created = Vec::new();

    // Cargo.toml
    let cargo = generate_cargo_toml(config);
    let cargo_path = output_dir.join("Cargo.toml");
    write_if_not_exists(&cargo_path, &cargo, &mut created)?;

    // src/lib.rs
    let lib_rs = generate_lib_rs(config);
    let lib_path = output_dir.join("src/lib.rs");
    write_if_not_exists(&lib_path, &lib_rs, &mut created)?;

    // src/client.rs (trait)
    let client_trait = trait_generator::generate(config);
    let trait_path = output_dir.join("src/client.rs");
    write_if_not_exists(&trait_path, &client_trait, &mut created)?;

    // src/grpc.rs
    let grpc = grpc_generator::generate(config);
    let grpc_path = output_dir.join("src/grpc.rs");
    write_if_not_exists(&grpc_path, &grpc, &mut created)?;

    // src/http.rs
    let http = http_generator::generate(config);
    let http_path = output_dir.join("src/http.rs");
    write_if_not_exists(&http_path, &http, &mut created)?;

    // src/mock.rs
    let mock = mock_generator::generate(config);
    let mock_path = output_dir.join("src/mock.rs");
    write_if_not_exists(&mock_path, &mock, &mut created)?;

    // src/resilient.rs
    let resilient = resilient_generator::generate(config);
    let resilient_path = output_dir.join("src/resilient.rs");
    write_if_not_exists(&resilient_path, &resilient, &mut created)?;

    // src/error.rs
    let error = generate_error_rs(config);
    let error_path = output_dir.join("src/error.rs");
    write_if_not_exists(&error_path, &error, &mut created)?;

    // src/types.rs
    let types = generate_types_rs(config);
    let types_path = output_dir.join("src/types.rs");
    write_if_not_exists(&types_path, &types, &mut created)?;

    Ok(created)
}

fn write_if_not_exists(
    path: &Path,
    content: &str,
    created: &mut Vec<std::path::PathBuf>,
) -> Result<(), CodegenError> {
    if path.exists() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| CodegenError::Io {
            path: parent.to_path_buf(),
            source: e,
        })?;
    }
    std::fs::write(path, content).map_err(|e| CodegenError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;
    created.push(path.to_path_buf());
    Ok(())
}

fn generate_cargo_toml(config: &ClientSdkConfig) -> String {
    format!(
        r#"[package]
name = "{package_name}"
version = "0.1.0"
edition = "2021"

[dependencies]
async-trait = "0.1"
thiserror = "2"
serde = {{ version = "1", features = ["derive"] }}

[dependencies.tonic]
version = "0.12"
optional = true

[dependencies.reqwest]
version = "0.12"
features = ["json"]
optional = true

[dependencies.mockall]
version = "0.13"
optional = true

[dependencies.tokio]
version = "1"
features = ["time"]
optional = true

[features]
default = ["grpc"]
grpc = ["dep:tonic"]
http = ["dep:reqwest"]
mock = ["dep:mockall"]
resilient = ["dep:tokio"]

[dev-dependencies]
tokio = {{ version = "1", features = ["macros", "rt-multi-thread"] }}
"#,
        package_name = config.package_name,
    )
}

fn generate_lib_rs(config: &ClientSdkConfig) -> String {
    format!(
        r#"pub mod client;
pub mod error;
pub mod types;

#[cfg(feature = "grpc")]
pub mod grpc;

#[cfg(feature = "http")]
pub mod http;

#[cfg(feature = "mock")]
pub mod mock;

#[cfg(feature = "resilient")]
pub mod resilient;

pub use client::{service_name}Client;
pub use error::ClientError;
pub use types::*;
"#,
        service_name = config.service_name,
    )
}

fn generate_error_rs(config: &ClientSdkConfig) -> String {
    let _ = config;
    r#"use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("transport error: {0}")]
    Transport(String),

    #[error("request failed with status {status}: {message}")]
    Request { status: u32, message: String },

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("timeout after {0:?}")]
    Timeout(std::time::Duration),

    #[error("circuit breaker open")]
    CircuitBreakerOpen,
}
"#
    .to_string()
}

fn generate_types_rs(config: &ClientSdkConfig) -> String {
    let mut out = String::from("use serde::{Deserialize, Serialize};\n\n");

    for ty in &config.types {
        out.push_str(&format!(
            "#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct {} {{\n",
            ty.name
        ));
        for field in &ty.fields {
            out.push_str(&format!("    pub {}: {},\n", field.name, field.field_type));
        }
        out.push_str("}\n\n");
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_config() -> ClientSdkConfig {
        ClientSdkConfig {
            service_name: "UserProfile".to_string(),
            package_name: "k1s0-user-profile-client".to_string(),
            methods: vec![
                ClientMethod {
                    name: "get_user".to_string(),
                    request_type: "GetUserRequest".to_string(),
                    response_type: "GetUserResponse".to_string(),
                },
                ClientMethod {
                    name: "create_user".to_string(),
                    request_type: "CreateUserRequest".to_string(),
                    response_type: "CreateUserResponse".to_string(),
                },
            ],
            types: vec![
                ClientType {
                    name: "GetUserRequest".to_string(),
                    fields: vec![ClientField {
                        name: "user_id".to_string(),
                        field_type: "String".to_string(),
                    }],
                },
                ClientType {
                    name: "GetUserResponse".to_string(),
                    fields: vec![
                        ClientField {
                            name: "user_id".to_string(),
                            field_type: "String".to_string(),
                        },
                        ClientField {
                            name: "name".to_string(),
                            field_type: "String".to_string(),
                        },
                    ],
                },
            ],
        }
    }

    #[test]
    fn test_generate_cargo_toml() {
        let config = sample_config();
        let cargo = generate_cargo_toml(&config);
        assert!(cargo.contains("k1s0-user-profile-client"));
        assert!(cargo.contains("async-trait"));
        assert!(cargo.contains("[features]"));
        assert!(cargo.contains("grpc"));
    }

    #[test]
    fn test_generate_lib_rs() {
        let config = sample_config();
        let lib = generate_lib_rs(&config);
        assert!(lib.contains("pub mod client;"));
        assert!(lib.contains("UserProfileClient"));
    }

    #[test]
    fn test_generate_error_rs() {
        let config = sample_config();
        let error = generate_error_rs(&config);
        assert!(error.contains("ClientError"));
        assert!(error.contains("Transport"));
        assert!(error.contains("CircuitBreakerOpen"));
    }

    #[test]
    fn test_generate_types_rs() {
        let config = sample_config();
        let types = generate_types_rs(&config);
        assert!(types.contains("pub struct GetUserRequest"));
        assert!(types.contains("pub user_id: String"));
        assert!(types.contains("Serialize, Deserialize"));
    }

    #[test]
    fn test_trait_generator() {
        let config = sample_config();
        let output = trait_generator::generate(&config);
        assert!(output.contains("UserProfileClient"));
        assert!(output.contains("async fn get_user"));
        assert!(output.contains("async fn create_user"));
        assert!(output.contains("async_trait"));
    }

    #[test]
    fn test_grpc_generator() {
        let config = sample_config();
        let output = grpc_generator::generate(&config);
        assert!(output.contains("GrpcUserProfileClient"));
        assert!(output.contains("tonic"));
        assert!(output.contains("Channel"));
    }

    #[test]
    fn test_http_generator() {
        let config = sample_config();
        let output = http_generator::generate(&config);
        assert!(output.contains("HttpUserProfileClient"));
        assert!(output.contains("reqwest"));
    }

    #[test]
    fn test_mock_generator() {
        let config = sample_config();
        let output = mock_generator::generate(&config);
        assert!(output.contains("MockUserProfileClient"));
        assert!(output.contains("mockall"));
    }

    #[test]
    fn test_resilient_generator() {
        let config = sample_config();
        let output = resilient_generator::generate(&config);
        assert!(output.contains("ResilientUserProfileClient"));
        assert!(output.contains("max_retries"));
        assert!(output.contains("circuit_breaker"));
    }

    #[test]
    fn test_generate_client_sdk_creates_files() {
        let config = sample_config();
        let tmp = std::env::temp_dir().join("k1s0_client_sdk_test");
        let _ = std::fs::remove_dir_all(&tmp);

        let created = generate_client_sdk(&config, &tmp).unwrap();
        assert!(!created.is_empty());
        assert!(tmp.join("Cargo.toml").exists());
        assert!(tmp.join("src/lib.rs").exists());
        assert!(tmp.join("src/client.rs").exists());
        assert!(tmp.join("src/grpc.rs").exists());
        assert!(tmp.join("src/http.rs").exists());
        assert!(tmp.join("src/mock.rs").exists());
        assert!(tmp.join("src/resilient.rs").exists());
        assert!(tmp.join("src/error.rs").exists());
        assert!(tmp.join("src/types.rs").exists());

        // idempotency: running again should create no new files
        let created2 = generate_client_sdk(&config, &tmp).unwrap();
        assert!(created2.is_empty());

        let _ = std::fs::remove_dir_all(&tmp);
    }
}

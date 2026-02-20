//! k1s0-schemaregistry: Confluent Schema Registry クライアントライブラリ。
//!
//! Kafka トピックに対する Protobuf スキーマの登録・取得・互換性検証を提供する。
//!
//! # 使用例
//!
//! ```rust,no_run
//! use k1s0_schemaregistry::{
//!     HttpSchemaRegistryClient, SchemaRegistryClient, SchemaRegistryConfig, SchemaType,
//! };
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = SchemaRegistryConfig::new("http://schema-registry:8081");
//!     let client = HttpSchemaRegistryClient::new(config)?;
//!
//!     let topic = "k1s0.system.auth.user-created.v1";
//!     let subject = SchemaRegistryConfig::subject_name(topic);
//!
//!     let schema_id = client
//!         .register_schema(&subject, r#"syntax = "proto3"; message UserCreated {}"#, SchemaType::Protobuf)
//!         .await?;
//!
//!     println!("Registered schema id={}", schema_id);
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod config;
pub mod error;
pub mod schema;

pub use client::{HttpSchemaRegistryClient, SchemaRegistryClient};
pub use config::{CompatibilityMode, SchemaRegistryConfig};
pub use error::SchemaRegistryError;
pub use schema::{RegisteredSchema, SchemaType};

#[cfg(feature = "mock")]
pub use client::MockSchemaRegistryClient;

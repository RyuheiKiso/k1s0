pub mod config;
pub mod context;
pub mod error;
pub mod generator;
pub mod naming;
pub mod path;
mod templates;
pub mod validator;

#[cfg(feature = "event-codegen")]
pub mod event_codegen;

#[cfg(feature = "client-sdk")]
pub mod client_sdk;

#[cfg(feature = "cargo-update")]
pub mod cargo_updater;
#[cfg(feature = "proto")]
pub mod proto_parser;

pub use config::{ApiStyle, DatabaseType, ScaffoldConfig, Tier};
pub use error::CodegenError;
#[cfg(feature = "event-codegen")]
pub use event_codegen::{generate_events, EventConfig, EventGenerateResult};
pub use generator::{generate, GenerateResult};
pub use path::build_output_path;
#[cfg(feature = "proto")]
pub use proto_parser::ProtoService;
pub use validator::ValidationResult;

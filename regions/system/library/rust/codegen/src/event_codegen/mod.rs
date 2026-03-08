pub mod config;
pub mod generator;
pub mod migration_generator;
pub mod parser;
pub mod proto_generator;
pub mod rust_generator;
pub mod validator;

pub use config::EventConfig;
pub use generator::{generate_events, EventGenerateResult};
pub use parser::{parse_event_config, parse_event_config_str};
pub use validator::validate_event_config;

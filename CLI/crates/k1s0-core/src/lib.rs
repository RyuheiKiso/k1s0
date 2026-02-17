// k1s0-core: CLI と GUI で共有するビジネスロジック

pub mod commands;
pub mod config;
pub mod progress;
pub mod template;
pub mod validation;

pub use config::{load_config, CliConfig};
pub use validation::validate_name;
pub use template::context::{build_context, ProjectContext, TemplateContext, TemplateContextBuilder};
pub use template::TemplateEngine;

use tera::Context;

use crate::config::{ApiStyle, DatabaseType, ScaffoldConfig};
use crate::naming;

/// Build a Tera context from a ScaffoldConfig.
pub fn build_context(config: &ScaffoldConfig) -> Context {
    let mut ctx = Context::new();

    // Names
    ctx.insert("name", &config.name);
    ctx.insert("name_snake", &naming::to_snake(&config.name));
    ctx.insert("name_pascal", &naming::to_pascal(&config.name));
    ctx.insert("name_camel", &naming::to_camel(&config.name));

    // Tier
    ctx.insert("tier", config.tier.as_str());

    // API style flags
    ctx.insert("has_rest", &config.has_rest());
    ctx.insert("has_grpc", &config.has_grpc());
    ctx.insert(
        "api_style",
        match config.api_style {
            ApiStyle::Rest => "rest",
            ApiStyle::Grpc => "grpc",
            ApiStyle::Both => "both",
        },
    );

    // Database flags
    ctx.insert("has_database", &config.has_database());
    ctx.insert(
        "database_type",
        match config.database {
            DatabaseType::Postgres => "postgres",
            DatabaseType::None => "none",
        },
    );

    // Description
    ctx.insert("description", &config.description);

    ctx
}

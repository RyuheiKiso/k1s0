use std::fs;
use std::path::{Path, PathBuf};

use tera::{Context, Tera};

use crate::config::ScaffoldConfig;
use crate::context::build_context;
use crate::error::CodegenError;
use crate::naming;
use crate::templates::create_tera_engine;

/// Result of a `generate()` call.
#[derive(Debug)]
pub struct GenerateResult {
    /// Files that were created.
    pub created: Vec<PathBuf>,
    /// Files that were skipped (already existed).
    pub skipped: Vec<PathBuf>,
}

/// Generate a full server scaffold from the given config.
///
/// The output root directory must be provided (the server directory itself).
/// Files that already exist are skipped (idempotent).
pub fn generate(
    config: &ScaffoldConfig,
    output_dir: &Path,
) -> Result<GenerateResult, CodegenError> {
    config.validate()?;

    let tera = create_tera_engine().map_err(|e| CodegenError::Template {
        template: "engine_init".into(),
        source: e,
    })?;
    let ctx = build_context(config);

    let mut result = GenerateResult {
        created: Vec::new(),
        skipped: Vec::new(),
    };

    // Always-generated files
    let always_files: Vec<(&str, &str)> = vec![
        ("cargo_toml", "Cargo.toml"),
        ("main_rs", "src/main.rs"),
        ("lib_rs", "src/lib.rs"),
        ("error_rs", "src/error.rs"),
        ("config_yaml", "config/config.yaml"),
        ("readme", "README.md"),
        ("handler_rs", "src/adapter/handler/mod.rs"),
        ("config_rs", "src/infrastructure/config.rs"),
    ];

    for (template, rel_path) in &always_files {
        render_file(&tera, template, &ctx, output_dir, rel_path, &mut result)?;
    }

    // mod.rs files for directory structure
    let mod_files: Vec<(&str, &[&str])> = vec![
        ("src/adapter/mod.rs", &["handler"]),
        ("src/domain/mod.rs", &["entity", "repository", "service"]),
        ("src/domain/entity/mod.rs", &[]),
        ("src/domain/repository/mod.rs", &[]),
        ("src/domain/service/mod.rs", &[]),
        ("src/usecase/mod.rs", &[]),
        ("src/infrastructure/mod.rs", &["config"]),
    ];

    for (rel_path, children) in &mod_files {
        let mut mod_ctx = ctx.clone();
        let children_vec: Vec<String> = children.iter().map(std::string::ToString::to_string).collect();
        mod_ctx.insert("children", &children_vec);
        render_file(&tera, "mod_rs", &mod_ctx, output_dir, rel_path, &mut result)?;
    }

    // gRPC-only files
    if config.has_grpc() {
        render_file(&tera, "build_rs", &ctx, output_dir, "build.rs", &mut result)?;

        let proto_path = format!(
            "api/proto/k1s0/{}/{}/v1/{}.proto",
            config.tier.as_str(),
            naming::to_snake(&config.name),
            naming::to_snake(&config.name),
        );
        render_file(&tera, "proto", &ctx, output_dir, &proto_path, &mut result)?;

        // .gitkeep for src/proto/
        let gitkeep = output_dir.join("src/proto/.gitkeep");
        ensure_parent(&gitkeep)?;
        if gitkeep.exists() {
            result.skipped.push(gitkeep);
        } else {
            fs::write(&gitkeep, "").map_err(|e| CodegenError::Io {
                path: gitkeep.clone(),
                source: e,
            })?;
            result.created.push(gitkeep);
        }
    }

    // Database-only files
    if config.has_database() {
        render_file(
            &tera,
            "migration_up",
            &ctx,
            output_dir,
            "migrations/001_initial.up.sql",
            &mut result,
        )?;
        render_file(
            &tera,
            "migration_down",
            &ctx,
            output_dir,
            "migrations/001_initial.down.sql",
            &mut result,
        )?;
    }

    Ok(result)
}

fn render_file(
    tera: &Tera,
    template_name: &str,
    ctx: &Context,
    output_dir: &Path,
    rel_path: &str,
    result: &mut GenerateResult,
) -> Result<(), CodegenError> {
    let full_path = output_dir.join(rel_path);

    if full_path.exists() {
        result.skipped.push(full_path);
        return Ok(());
    }

    let rendered = tera
        .render(template_name, ctx)
        .map_err(|e| CodegenError::Template {
            template: template_name.to_string(),
            source: e,
        })?;

    ensure_parent(&full_path)?;
    fs::write(&full_path, rendered).map_err(|e| CodegenError::Io {
        path: full_path.clone(),
        source: e,
    })?;

    result.created.push(full_path);
    Ok(())
}

fn ensure_parent(path: &Path) -> Result<(), CodegenError> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| CodegenError::Io {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }
    }
    Ok(())
}

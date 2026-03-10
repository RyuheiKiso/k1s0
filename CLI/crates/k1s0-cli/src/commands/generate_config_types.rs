use anyhow::Result;
use dialoguer::{Input, MultiSelect};
use std::path::{Path, PathBuf};

use crate::prompt;
use k1s0_core::commands::generate::config_types::{
    load_validated_schema_from_file, push_config_schema, write_generated_types_to_targets,
    GeneratedTypesTarget,
};

pub fn run() -> Result<()> {
    println!("\n--- Generate Config Types ---\n");

    let schema_path: String = Input::with_theme(&prompt::theme())
        .with_prompt("config-schema.yaml path")
        .default("config-schema.yaml".to_string())
        .interact_text()?;
    let schema = load_validated_schema_from_file(Path::new(&schema_path))
        .map_err(|error| anyhow::anyhow!("{schema_path}: {error}"))?;

    let selections = MultiSelect::with_theme(&prompt::theme())
        .with_prompt("Select generation targets")
        .items(&["React (TypeScript)", "Flutter (Dart)"])
        .interact()?;
    if selections.is_empty() {
        return Ok(());
    }

    let mut output_dirs: Vec<(String, PathBuf)> = Vec::new();
    if selections.contains(&0) {
        let output_dir: String = Input::with_theme(&prompt::theme())
            .with_prompt("React output directory")
            .default("src/config/__generated__".to_string())
            .interact_text()?;
        output_dirs.push(("typescript".to_string(), PathBuf::from(output_dir)));
    }
    if selections.contains(&1) {
        let output_dir: String = Input::with_theme(&prompt::theme())
            .with_prompt("Flutter output directory")
            .default("lib/config/__generated__".to_string())
            .interact_text()?;
        output_dirs.push(("dart".to_string(), PathBuf::from(output_dir)));
    }

    let push = match prompt::yes_no_prompt("Push schema to config server?")? {
        Some(value) => value,
        None => return Ok(()),
    };

    let server_url = if push {
        Some(
            Input::with_theme(&prompt::theme())
                .with_prompt("config server URL")
                .default("http://localhost:8080".to_string())
                .interact_text()?,
        )
    } else {
        None
    };

    println!("\n[Summary]");
    println!("  Schema: {schema_path} ({})", schema.service);
    if let Some(url) = &server_url {
        println!("  Push:   {url}");
    }
    for (target, output_dir) in &output_dirs {
        let file_name = match target.as_str() {
            "typescript" => "config-types.ts",
            "dart" => "config_types.dart",
            _ => continue,
        };
        println!("  {target}: {}", output_dir.join(file_name).display());
    }

    if prompt::confirm_prompt()? != prompt::ConfirmResult::Yes {
        println!("Cancelled.");
        return Ok(());
    }

    if let Some(url) = &server_url {
        let token = std::env::var("K1S0_TOKEN")
            .map_err(|_| anyhow::anyhow!("K1S0_TOKEN is required when push is enabled"))?;
        println!("\nPushing schema...");
        push_config_schema(&schema, url, &token).map_err(|error| anyhow::anyhow!("{error}"))?;
        println!(
            "  OK schema registered: {} ({} categories, {} fields)",
            schema.service,
            schema.categories.len(),
            schema
                .categories
                .iter()
                .map(|category| category.fields.len())
                .sum::<usize>()
        );
    }

    let target_specs = output_dirs
        .iter()
        .map(|(target, output_dir)| GeneratedTypesTarget {
            target: target.as_str(),
            output_dir: output_dir.as_path(),
        })
        .collect::<Vec<_>>();

    println!("\nGenerating type definitions...");
    let generated = write_generated_types_to_targets(&schema, &target_specs)
        .map_err(|error| anyhow::anyhow!("{error}"))?;
    for path in &generated {
        println!("  OK {}", path.display());
    }

    println!("\nConfig types generated successfully.");
    Ok(())
}

mod cli;
mod domain;
mod application;
mod infrastructure;

use clap::Parser;

use cli::{Cli, Commands, NewSubcommand};
use application::new_project::{NewProjectArgs, NewProjectUseCase};
use domain::model::ProjectType;
use infrastructure::generator::FsGenerator;
use infrastructure::prompt::DialoguerPrompt;

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Some(Commands::New { subcommand }) => run_new(subcommand),
        None => run_new(None),
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn run_new(subcommand: Option<NewSubcommand>) -> Result<(), String> {
    let prompt = DialoguerPrompt::new();
    let generator = FsGenerator::new();
    let use_case = NewProjectUseCase::new(prompt, generator);

    let args = match subcommand {
        Some(NewSubcommand::Frontend { template, name, path, yes }) => NewProjectArgs {
            project_type: Some(ProjectType::Frontend),
            template,
            name,
            db: None,
            path,
            yes,
        },
        Some(NewSubcommand::Backend { template, name, db, path, yes }) => NewProjectArgs {
            project_type: Some(ProjectType::Backend),
            template,
            name,
            db,
            path,
            yes,
        },
        None => NewProjectArgs {
            project_type: None,
            template: None,
            name: None,
            db: None,
            path: None,
            yes: false,
        },
    };

    let config = use_case.execute(args)?;
    println!("Project '{}' created at {}", config.name, config.path.display());
    Ok(())
}

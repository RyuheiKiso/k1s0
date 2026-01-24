//! k1s0 CLI - 雛形生成・導入・アップグレード支援ツール

use clap::Parser;
use k1s0_cli::{Cli, Commands};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init(args) => k1s0_cli::commands::init::execute(args),
        Commands::NewFeature(args) => k1s0_cli::commands::new_feature::execute(args),
        Commands::Lint(args) => k1s0_cli::commands::lint::execute(args),
        Commands::Upgrade(args) => k1s0_cli::commands::upgrade::execute(args),
        Commands::Completions(args) => k1s0_cli::commands::completions::execute(args),
    }
}

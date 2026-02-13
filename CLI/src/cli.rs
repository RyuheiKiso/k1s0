use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "k1s0", version, about = "k1s0 project generator CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Create a new project
    New {
        #[command(subcommand)]
        subcommand: Option<NewSubcommand>,
    },
}

#[derive(Subcommand, Debug)]
pub enum NewSubcommand {
    /// Create a new frontend project
    Frontend {
        /// Template to use (react, flutter)
        #[arg(short, long)]
        template: Option<String>,

        /// Project name
        #[arg(short, long)]
        name: Option<String>,

        /// Output path
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Skip confirmation prompts
        #[arg(short, long, default_value_t = false)]
        yes: bool,
    },
    /// Create a new backend project
    Backend {
        /// Template to use (rust, go)
        #[arg(short, long)]
        template: Option<String>,

        /// Project name
        #[arg(short, long)]
        name: Option<String>,

        /// Database to use (postgresql, none)
        #[arg(short, long)]
        db: Option<String>,

        /// Output path
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Skip confirmation prompts
        #[arg(short, long, default_value_t = false)]
        yes: bool,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_parse_frontend_react() {
        let cli = Cli::parse_from([
            "k1s0", "new", "frontend", "--template", "react", "--name", "my-app",
        ]);
        match cli.command {
            Some(Commands::New { subcommand: Some(NewSubcommand::Frontend { template, name, .. }) }) => {
                assert_eq!(template.unwrap(), "react");
                assert_eq!(name.unwrap(), "my-app");
            }
            _ => panic!("Expected frontend subcommand"),
        }
    }

    #[test]
    fn test_parse_backend_rust_with_db() {
        let cli = Cli::parse_from([
            "k1s0", "new", "backend", "--template", "rust", "--name", "my-service", "--db", "postgresql",
        ]);
        match cli.command {
            Some(Commands::New { subcommand: Some(NewSubcommand::Backend { template, name, db, .. }) }) => {
                assert_eq!(template.unwrap(), "rust");
                assert_eq!(name.unwrap(), "my-service");
                assert_eq!(db.unwrap(), "postgresql");
            }
            _ => panic!("Expected backend subcommand"),
        }
    }

    #[test]
    fn test_parse_yes_flag() {
        let cli = Cli::parse_from([
            "k1s0", "new", "frontend", "--template", "react", "--name", "app", "--yes",
        ]);
        match cli.command {
            Some(Commands::New { subcommand: Some(NewSubcommand::Frontend { yes, .. }) }) => {
                assert!(yes);
            }
            _ => panic!("Expected frontend subcommand"),
        }
    }

    #[test]
    fn test_parse_path_option() {
        let cli = Cli::parse_from([
            "k1s0", "new", "frontend", "--template", "react", "--name", "app", "--path", "/tmp/out",
        ]);
        match cli.command {
            Some(Commands::New { subcommand: Some(NewSubcommand::Frontend { path, .. }) }) => {
                assert_eq!(path.unwrap(), PathBuf::from("/tmp/out"));
            }
            _ => panic!("Expected frontend subcommand"),
        }
    }

    #[test]
    fn test_parse_no_template_interactive_mode() {
        let cli = Cli::parse_from(["k1s0", "new", "frontend"]);
        match cli.command {
            Some(Commands::New { subcommand: Some(NewSubcommand::Frontend { template, name, .. }) }) => {
                assert!(template.is_none());
                assert!(name.is_none());
            }
            _ => panic!("Expected frontend subcommand"),
        }
    }

    #[test]
    fn test_parse_no_args_is_none() {
        let cli = Cli::parse_from(["k1s0"]);
        assert!(cli.command.is_none());
    }

    #[test]
    fn test_parse_new_without_subcommand() {
        let cli = Cli::parse_from(["k1s0", "new"]);
        match cli.command {
            Some(Commands::New { subcommand }) => {
                assert!(subcommand.is_none());
            }
            _ => panic!("Expected New command"),
        }
    }

    #[test]
    fn test_parse_invalid_command_fails() {
        let result = Cli::try_parse_from(["k1s0", "invalid"]);
        assert!(result.is_err());
    }
}

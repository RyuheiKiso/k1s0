//! k1s0 CLI - 雛形生成・導入・アップグレード支援ツール

use clap::Parser;
use k1s0_cli::{init_output, output, Cli, Commands, ExitCode};

fn main() -> std::process::ExitCode {
    let cli = Cli::parse();

    // 出力設定を初期化
    init_output(cli.output_config());

    // コマンドを実行
    let result = match cli.command {
        Commands::Init(args) => k1s0_cli::commands::init::execute(args),
        Commands::NewFeature(args) => k1s0_cli::commands::new_feature::execute(args),
        Commands::NewScreen(args) => k1s0_cli::commands::new_screen::execute(args),
        Commands::Lint(args) => k1s0_cli::commands::lint::execute(args),
        Commands::Upgrade(args) => k1s0_cli::commands::upgrade::execute(args),
        Commands::Registry(args) => k1s0_cli::commands::registry::execute(args),
        Commands::Completions(args) => k1s0_cli::commands::completions::execute(args),
    };

    // 結果を処理
    match result {
        Ok(()) => ExitCode::Success.into(),
        Err(e) => {
            output().error(&e);
            e.exit_code().into()
        }
    }
}

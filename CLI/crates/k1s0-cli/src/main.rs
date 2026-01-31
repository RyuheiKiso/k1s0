//! k1s0 CLI - 雛形生成・導入・アップグレード支援ツール

use clap::Parser;
use k1s0_cli::output::output;
use k1s0_cli::prompts::command_select::{select_command, SelectedCommand};
use k1s0_cli::prompts::is_interactive;
use k1s0_cli::{init_output, Cli, Commands, ExitCode, OutputConfig, OutputMode};

fn main() -> std::process::ExitCode {
    // 引数が1つ（プログラム名のみ）かつ TTY の場合、対話モードへ
    if std::env::args().len() == 1 && is_interactive() {
        return run_interactive_mode();
    }

    // 通常のコマンド実行
    let cli = Cli::parse();

    // 出力設定を初期化
    init_output(cli.output_config());

    // コマンドを実行
    let result = match cli.command {
        Commands::Init(args) => k1s0_cli::commands::init::execute(args),
        Commands::NewDomain(args) => k1s0_cli::commands::new_domain::execute(args),
        Commands::NewFeature(args) => k1s0_cli::commands::new_feature::execute(args),
        Commands::NewScreen(args) => k1s0_cli::commands::new_screen::execute(args),
        Commands::Lint(args) => k1s0_cli::commands::lint::execute(args),
        Commands::Upgrade(args) => k1s0_cli::commands::upgrade::execute(args),
        Commands::Registry(args) => k1s0_cli::commands::registry::execute(args),
        Commands::Completions(args) => k1s0_cli::commands::completions::execute(args),
        Commands::Doctor(args) => k1s0_cli::commands::doctor::execute(args),
        Commands::DomainVersion(args) => k1s0_cli::commands::domain_version::execute(args),
        Commands::DomainList(args) => k1s0_cli::commands::domain_list::execute(args),
        Commands::DomainDependents(args) => k1s0_cli::commands::domain_dependents::execute(args),
        Commands::DomainImpact(args) => k1s0_cli::commands::domain_impact::execute(args),
        Commands::FeatureUpdateDomain(args) => {
            k1s0_cli::commands::feature_update_domain::execute(args)
        }
        Commands::DomainCatalog(args) => k1s0_cli::commands::domain_catalog::execute(args),
        Commands::DomainGraph(args) => k1s0_cli::commands::domain_graph::execute(args),
        Commands::Docker(args) => k1s0_cli::commands::docker::execute(args),
        Commands::Playground(args) => k1s0_cli::commands::playground::execute(args),
        Commands::Migrate(args) => k1s0_cli::commands::migrate::execute(args),
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

/// 対話モードでコマンドを選択して実行
fn run_interactive_mode() -> std::process::ExitCode {
    // デフォルトの出力設定を初期化（カラー有効、Human モード）
    init_output(OutputConfig {
        mode: OutputMode::Human,
        color: true,
        verbose: false,
    });

    // バナーを表示
    print_banner();

    // コマンドを選択
    let selected = match select_command() {
        Ok(cmd) => cmd,
        Err(e) => {
            output().error(&e);
            return e.exit_code().into();
        }
    };

    // 選択されたコマンドを実行
    execute_selected_command(selected)
}

/// k1s0 バナーを表示
fn print_banner() {
    let out = output();
    out.newline();
    out.header("k1s0 - 高速な開発サイクルを実現する統合開発基盤");
    out.newline();
}

/// 選択されたコマンドを実行
fn execute_selected_command(selected: SelectedCommand) -> std::process::ExitCode {
    let result = match selected {
        SelectedCommand::NewFeature => {
            // 対話モードで継続（引数なしで execute を呼び出す）
            let args = k1s0_cli::commands::new_feature::NewFeatureArgs {
                service_type: None,
                name: None,
                domain: None,
                output: None,
                force: false,
                with_grpc: false,
                with_rest: false,
                with_db: false,
                interactive: true, // 対話モードを強制
                yes: false,
                skip_doctor: false,
            };
            k1s0_cli::commands::new_feature::execute(args)
        }
        SelectedCommand::NewDomain => {
            // 対話モードで継続
            let args = k1s0_cli::commands::new_domain::NewDomainArgs {
                domain_type: None,
                name: None,
                output: None,
                force: false,
                interactive: true,
                with_events: false,
                with_repository: true,
                version: "0.1.0".to_string(),
                yes: false,
            };
            k1s0_cli::commands::new_domain::execute(args)
        }
        SelectedCommand::NewScreen => {
            // 対話モードで継続
            let args = k1s0_cli::commands::new_screen::NewScreenArgs {
                frontend_type: k1s0_cli::commands::new_screen::FrontendType::React, // デフォルト値（対話で選択可能）
                screen_id: None,
                title: None,
                feature_dir: None,
                with_menu: false,
                path: None,
                permissions: None,
                flags: None,
                force: false,
                interactive: true,
                yes: false,
            };
            k1s0_cli::commands::new_screen::execute(args)
        }
        SelectedCommand::Init => {
            // 対話モードで継続
            let args = k1s0_cli::commands::init::InitArgs {
                path: ".".to_string(),
                force: false,
                template_source: "local".to_string(),
                interactive: true,
                skip_doctor: false,
            };
            k1s0_cli::commands::init::execute(args)
        }
        SelectedCommand::Lint => {
            // ヘルプメッセージを表示（非対話コマンド）
            show_command_help("lint")
        }
        SelectedCommand::Upgrade => {
            // ヘルプメッセージを表示（非対話コマンド）
            show_command_help("upgrade")
        }
        SelectedCommand::Domain => {
            // ヘルプメッセージを表示（非対話コマンド）
            show_domain_help()
        }
        SelectedCommand::Completions => {
            // ヘルプメッセージを表示（非対話コマンド）
            show_command_help("completions")
        }
        SelectedCommand::Playground => {
            // ヘルプメッセージを表示（非対話コマンド）
            show_command_help("playground")
        }
        SelectedCommand::Migrate => {
            // ヘルプメッセージを表示（非対話コマンド）
            show_command_help("migrate")
        }
    };

    match result {
        Ok(()) => ExitCode::Success.into(),
        Err(e) => {
            output().error(&e);
            e.exit_code().into()
        }
    }
}

/// 指定されたコマンドのヘルプを表示
fn show_command_help(command: &str) -> k1s0_cli::Result<()> {
    let out = output();
    out.info(&format!(
        "'{command}' コマンドはオプションが必要です。ヘルプを表示します:"
    ));
    out.newline();

    // コマンドのヘルプを表示するために引数を再構築して parse する
    // （エラーが発生するとヘルプが表示される）
    let args = vec!["k1s0", command, "--help"];
    let _ = Cli::try_parse_from(args);

    Ok(())
}

/// domain サブコマンドのヘルプを表示
fn show_domain_help() -> k1s0_cli::Result<()> {
    let out = output();
    out.info("'domain' コマンドにはサブコマンドが必要です:");
    out.newline();
    out.list_item("domain-list", "全 domain の一覧表示");
    out.list_item("domain-catalog", "domain カタログ（依存状況付き一覧）");
    out.list_item("domain-version", "domain バージョンの表示・更新");
    out.list_item("domain-dependents", "domain に依存する feature の一覧表示");
    out.list_item("domain-impact", "domain バージョンアップの影響分析");
    out.list_item("domain-graph", "domain 依存グラフの出力");
    out.newline();
    out.hint("例: k1s0 domain-list");

    Ok(())
}

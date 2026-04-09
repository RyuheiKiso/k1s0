mod commands;
mod config;
mod prompt;
mod template;

use anyhow::Result;
use clap::{Parser, Subcommand};
use dialoguer::{theme::ColorfulTheme, Select};
use std::path::PathBuf;

// ============================================================================
// clap 構造体定義
// ============================================================================

/// k1s0 プロジェクト管理 CLI のトップレベルコマンド構造体。
/// サブコマンドが省略された場合は対話メニューを表示する。
#[derive(Parser)]
#[command(name = "k1s0", about = "k1s0 プロジェクト管理 CLI", version)]
struct Cli {
    /// TTY なし環境（CI/CD）向け非インタラクティブモード。
    /// 対話プロンプトをスキップしてデフォルト値を使用するか、即時エラー終了する。
    #[arg(long, global = true, env = "K1S0_NON_INTERACTIVE")]
    non_interactive: bool,

    /// --non-interactive の短縮エイリアス。CI での利用を想定する。
    #[arg(long, global = true)]
    yes: bool,

    /// サブコマンド（省略時は対話メニューを表示）
    #[command(subcommand)]
    command: Option<Commands>,
}

/// 利用可能なサブコマンド一覧。
#[derive(Subcommand)]
enum Commands {
    /// プロジェクトを初期化する
    Init {
        /// プロジェクト名（--non-interactive 時は必須）
        #[arg(long, value_name = "NAME")]
        name: Option<String>,
    },
    /// ひな形を生成する
    Generate,
    /// ビルドを実行する
    Build,
    /// テストを実行する
    Test,
    /// デプロイを実行する
    Deploy,
    /// ローカル開発環境を操作する
    Dev,
    /// マイグレーションを管理する
    Migrate,
    /// 設定スキーマ型を生成する
    ConfigTypes,
    /// ナビゲーション型を生成する
    Navigation,
    /// イベントコードを生成する
    Events,
    /// バリデーションを実行する
    Validate {
        /// バリデーション対象ファイルのパス（省略時はインタラクティブモード）
        #[arg(long, value_name = "FILE")]
        file: Option<std::path::PathBuf>,
        /// バリデーション種別: "config-schema" | "navigation"（--file と併用時に必須）
        #[arg(long = "type", value_name = "TYPE")]
        validate_type: Option<String>,
    },
    /// 依存関係マップを表示する
    Deps {
        /// 解析スコープ: "all"（デフォルト）| "tier" | "services"
        #[arg(long, value_name = "SCOPE")]
        scope: Option<String>,
        /// Tier 指定（--scope tier 時に必須）: "system" | "business" | "service"
        #[arg(long, value_name = "TIER")]
        tier: Option<String>,
        /// 出力形式: "terminal"（デフォルト）| "mermaid" | "both"
        #[arg(long, value_name = "OUTPUT")]
        output: Option<String>,
        /// Mermaid 出力先ファイルパス（--output mermaid / both 時に必須）
        #[arg(long, value_name = "PATH")]
        output_path: Option<std::path::PathBuf>,
    },
    /// テンプレートマイグレーションを実行する
    TemplateMigrate,
    /// 開発環境を診断する
    Doctor,
}

// ============================================================================
// カテゴリ別メニュー定数
// ============================================================================

/// メインメニューのカテゴリ選択肢。
const MENU_CATEGORIES: &[&str] = &[
    "よく使う操作 >",
    "コード生成 >",
    "プロジェクト管理 >",
    "運用 >",
    "終了",
];

/// よく使う操作カテゴリのサブメニュー選択肢。
/// プロジェクト初期化は新規参加者が最初に使う操作のため先頭に配置する。
const MENU_FREQUENT: &[&str] = &[
    "プロジェクト初期化",
    "ひな形生成",
    "ビルド",
    "テスト実行",
    "ローカル開発",
    "← 戻る",
];

/// コード生成カテゴリのサブメニュー選択肢。
const MENU_CODEGEN: &[&str] = &[
    "設定スキーマ型生成",
    "ナビゲーション型生成",
    "イベントコード生成",
    "← 戻る",
];

/// プロジェクト管理カテゴリのサブメニュー選択肢。
/// LOW-4 監査対応: 「プロジェクト初期化」は `MENU_FREQUENT` にも存在するため重複を削除する。
const MENU_PROJECT: &[&str] = &[
    "バリデーション",
    "依存関係マップ",
    "テンプレートマイグレーション",
    "← 戻る",
];

/// 運用カテゴリのサブメニュー選択肢。
const MENU_OPS: &[&str] = &["デプロイ", "マイグレーション管理", "← 戻る"];

// ============================================================================
// エントリポイント
// ============================================================================

#[allow(clippy::too_many_lines)]
fn main() {
    // Ctrl+C でパニックせずに終了するためのハンドラを設定する
    ctrlc_handler();

    // コマンドライン引数を解析する（引数なしなら対話メニューへ）
    let cli = Cli::parse();

    // --non-interactive / --yes フラグまたは TTY なし環境の場合は非インタラクティブモードとする。
    // K1S0_NON_INTERACTIVE 環境変数でも制御できる（clap の env = "K1S0_NON_INTERACTIVE" で自動取得）。
    let non_interactive =
        cli.non_interactive || cli.yes || !std::io::IsTerminal::is_terminal(&std::io::stdin());

    // 非インタラクティブモードをプロンプトモジュールへ伝達する
    prompt::set_non_interactive(non_interactive);

    // サブコマンドが指定された場合は非対話モードで実行する
    if let Some(command) = cli.command {
        // 設定ファイルを読み込む（失敗時はデフォルト設定を使用する）
        let _cli_config = match config::load_config_with_vault("k1s0.yaml") {
            Ok(config) => config,
            Err(e) => {
                eprintln!("設定ファイルの読み込みに失敗しました: {e}");
                eprintln!("デフォルト設定を使用します。");
                config::CliConfig::default()
            }
        };

        // 各サブコマンドを対話モードと同じ run() 関数に委譲する
        let result = match command {
            // LOW-001 対応: --name が指定された場合は非インタラクティブモードで直接実行する
            Commands::Init { name } => {
                if non_interactive {
                    commands::init::run_non_interactive(name)
                } else {
                    commands::init::run()
                }
            }
            Commands::Generate => commands::generate::run(),
            Commands::Build => commands::build::run(),
            Commands::Test => commands::test_cmd::run(),
            Commands::Deploy => commands::deploy::run(),
            Commands::Dev => commands::dev::run(),
            Commands::Migrate => commands::migrate::run(),
            Commands::ConfigTypes => commands::generate_config_types::run(),
            Commands::Navigation => commands::generate_navigation::run(),
            Commands::Events => commands::generate_events::run(),
            Commands::Validate {
                file,
                validate_type,
            } => commands::validate::run_with_args(file, validate_type),
            // MED-008/HIGH-008 監査対応: --scope 等のフラグが指定された場合、または非インタラクティブモードの場合は
            // 対話プロンプトをスキップして直接実行する。フラグなし TTY 環境では従来の対話フローを使用する。
            Commands::Deps {
                scope,
                tier,
                output,
                output_path,
            } => {
                if non_interactive || scope.is_some() || output.is_some() {
                    commands::deps::run_non_interactive(scope, tier, output, output_path)
                } else {
                    commands::deps::run()
                }
            }
            Commands::TemplateMigrate => commands::template_migrate::run(),
            Commands::Doctor => {
                // 開発環境診断スクリプトを実行する
                println!("開発環境を診断しています...");
                // CLI-02 監査対応: CWD 相対パスではなく優先順位付きのパス解決を使用する
                match find_doctor_script() {
                    Some(script_path) => {
                        // Windows 環境では bash が PATH に存在しない場合がある（HIGH-5 監査対応）。
                        // bash が利用可能か確認し、利用不可の場合は sh (Git for Windows 等) にフォールバックする。
                        // L-13 監査対応: exit status を無視せず、非0の場合はエラーを伝播する。
                        let shell = if cfg!(target_os = "windows") {
                            // bash の存在確認（Git for Windows / WSL 等でインストールされている場合は使用可能）
                            let bash_available = std::process::Command::new("bash")
                                .arg("--version")
                                .output()
                                .map(|o| o.status.success())
                                .unwrap_or(false);
                            if bash_available {
                                "bash"
                            } else {
                                "sh"
                            }
                        } else {
                            "bash"
                        };
                        // HIGH-003 監査対応: Windows パスを bash 互換パスに変換する
                        let bash_script = to_bash_path(&script_path);
                        match std::process::Command::new(shell).arg(&bash_script).status() {
                            Err(e) => Err(anyhow::anyhow!(
                                "doctor.sh の実行に失敗しました（シェル: {shell}）: {e}"
                            )),
                            Ok(s) if !s.success() => {
                                let code = s.code().unwrap_or(-1);
                                Err(anyhow::anyhow!(
                                    "doctor.sh が終了コード {code} で失敗しました"
                                ))
                            }
                            Ok(_) => Ok(()),
                        }
                    }
                    None => {
                        // M-16 監査対応: doctor.sh が見つからない場合はエラーを返す
                        Err(anyhow::anyhow!(
                            "doctor.sh が見つかりません。\n\
                            ヒント: K1S0_ROOT 環境変数にリポジトリルートのパスを設定してください。\n\
                            例: export K1S0_ROOT=/path/to/k1s0"
                        ))
                    }
                }
            }
        };

        // エラーが発生した場合は標準エラーへ出力して終了コード1で終了する
        if let Err(e) = result {
            eprintln!("エラー: {e}");
            std::process::exit(1);
        }
        return;
    }

    // 非インタラクティブモードでサブコマンドなしの場合はエラー終了する。
    // TTY なし環境でメニューを表示しようとしても操作できないためである。
    if non_interactive {
        eprintln!("エラー: 非インタラクティブモードではサブコマンドが必要です。");
        eprintln!("使用例: k1s0 doctor");
        eprintln!("       k1s0 --help でコマンド一覧を確認してください。");
        std::process::exit(1);
    }

    // 引数なし: 設定ファイルを読み込んで対話メニューを表示する
    let cli_config = match config::load_config_with_vault("k1s0.yaml") {
        Ok(config) => config,
        Err(e) => {
            eprintln!("設定ファイルの読み込みに失敗しました: {e}");
            eprintln!("デフォルト設定を使用します。");
            config::CliConfig::default()
        }
    };

    // 対話メニューのメインループ
    loop {
        match show_main_menu(&cli_config) {
            Ok(should_exit) => {
                if should_exit {
                    println!("終了します。");
                    break;
                }
            }
            Err(e) => {
                let msg = format!("{e}");
                if msg.contains("interrupted") {
                    // メインメニューで Ctrl+C → 終了
                    println!("\n終了します。");
                    break;
                }
                // その他のエラーはメインメニューに戻る
                eprintln!("エラーが発生しました: {e}");
            }
        }
    }
}

// ============================================================================
// ユーティリティ
// ============================================================================

/// Ctrl+C のグローバルハンドラを設定する。
/// dialoguer が Ctrl+C を処理するため、ここでは最低限のフォールバックのみ。
fn ctrlc_handler() {
    let _ = ctrlc::set_handler(|| {
        // dialoguer の interact_opt が None を返すので、
        // ここでは何もしない（二重終了を防ぐ）。
    });
}

/// CLI-02 監査対応: doctor.sh のパスを優先順位に従って解決する。
///
/// 探索順序:
/// 1. `K1S0_ROOT` 環境変数が設定されている場合は `$K1S0_ROOT/scripts/doctor.sh` を使用する
/// 2. 実行ファイルのパスから上位ディレクトリを遡り `scripts/doctor.sh` を探す
/// 3. 現在のワーキングディレクトリから `scripts/doctor.sh` を探す（フォールバック）
fn find_doctor_script() -> Option<PathBuf> {
    // 優先度1: K1S0_ROOT 環境変数が設定されている場合はそのパスを優先する
    // CLI-MED-001 監査対応: canonicalize でシンボリックリンクと相対パスを解決し、
    // パストラバーサル攻撃を防止する。
    // L-001 監査対応: `canonicalize().ok()` + `if let Some` を `if let Ok` に変更する。
    if let Ok(root) = std::env::var("K1S0_ROOT") {
        if let Ok(canonical) = std::path::PathBuf::from(&root).canonicalize() {
            let path = canonical.join("scripts").join("doctor.sh");
            if path.exists() {
                return Some(path);
            }
        }
    }

    // 優先度2: 実行ファイルの位置から上位ディレクトリを遡って探す
    // インストール先が bin/ 配下の場合にリポジトリルートを特定できる
    if let Ok(exe) = std::env::current_exe() {
        let mut dir = exe.parent();
        while let Some(d) = dir {
            let path = d.join("scripts").join("doctor.sh");
            if path.exists() {
                return Some(path);
            }
            dir = d.parent();
        }
    }

    // 優先度3: 現在のワーキングディレクトリからの相対パス（後方互換フォールバック）
    let path = PathBuf::from("scripts/doctor.sh");
    if path.exists() {
        return Some(path);
    }

    None
}

/// Windows 環境でのみ有効。PathBuf を MSYS/Git Bash が解釈できる Unix 形式に変換する。
/// 例: C:\work\github\k1s0\scripts\doctor.sh → /c/work/github/k1s0/scripts/doctor.sh
/// \\?\ プレフィックス（canonicalize が付与する）も除去する。
#[cfg(target_os = "windows")]
fn to_bash_path(p: &std::path::Path) -> String {
    let s = p.to_string_lossy();
    // \\?\ プレフィックスを除去する
    let s = s.strip_prefix(r"\\?\").unwrap_or(&s);
    // ドライブレターを変換する: C:\ → /c/
    if s.len() >= 3 && s.as_bytes()[1] == b':' {
        let drive = s.as_bytes()[0].to_ascii_lowercase() as char;
        let rest = s[2..].replace('\\', "/");
        format!("/{drive}{rest}")
    } else {
        s.replace('\\', "/")
    }
}

/// Windows 以外の環境では PathBuf を文字列にそのまま変換する。
#[cfg(not(target_os = "windows"))]
fn to_bash_path(p: &std::path::Path) -> String {
    p.to_string_lossy().to_string()
}

// ============================================================================
// 対話メニュー
// ============================================================================

/// メインメニューを表示し、カテゴリを選択させる。
/// 終了が選択された場合は Ok(true) を返す。
fn show_main_menu(_cli_config: &config::CliConfig) -> Result<bool> {
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("操作を選択してください")
        .items(MENU_CATEGORIES)
        .default(0)
        .interact_opt()?;

    match selection {
        // Ctrl+C が押された場合 (None) → 終了
        None => Ok(true),
        Some(index) => match index {
            // よく使う操作カテゴリを表示する
            0 => show_submenu_frequent(),
            // コード生成カテゴリを表示する
            1 => show_submenu_codegen(),
            // プロジェクト管理カテゴリを表示する
            2 => show_submenu_project(),
            // 運用カテゴリを表示する
            3 => show_submenu_ops(),
            // 終了を選択した
            4 => Ok(true),
            _ => unreachable!(),
        },
    }
}

/// よく使う操作カテゴリのサブメニューを表示する。
/// 「← 戻る」が選択された場合は Ok(false) を返してトップに戻る。
fn show_submenu_frequent() -> Result<bool> {
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("よく使う操作")
        .items(MENU_FREQUENT)
        .default(0)
        .interact_opt()?;

    match selection {
        // Ctrl+C → 終了
        None => Ok(true),
        Some(index) => {
            match index {
                // プロジェクト初期化を実行する
                0 => {
                    if let Err(e) = commands::init::run() {
                        eprintln!("プロジェクト初期化エラー: {e}");
                    }
                }
                // ひな形生成を実行する
                1 => {
                    if let Err(e) = commands::generate::run() {
                        eprintln!("ひな形生成エラー: {e}");
                    }
                }
                // ビルドを実行する
                2 => {
                    if let Err(e) = commands::build::run() {
                        eprintln!("ビルドエラー: {e}");
                    }
                }
                // テストを実行する
                3 => {
                    if let Err(e) = commands::test_cmd::run() {
                        eprintln!("テスト実行エラー: {e}");
                    }
                }
                // ローカル開発を実行する
                4 => {
                    if let Err(e) = commands::dev::run() {
                        eprintln!("ローカル開発エラー: {e}");
                    }
                }
                // 戻る → トップメニューへ
                5 => return Ok(false),
                _ => unreachable!(),
            }
            Ok(false)
        }
    }
}

/// コード生成カテゴリのサブメニューを表示する。
/// 「← 戻る」が選択された場合は Ok(false) を返してトップに戻る。
fn show_submenu_codegen() -> Result<bool> {
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("コード生成")
        .items(MENU_CODEGEN)
        .default(0)
        .interact_opt()?;

    match selection {
        // Ctrl+C → 終了
        None => Ok(true),
        Some(index) => {
            match index {
                // 設定スキーマ型を生成する
                0 => {
                    if let Err(e) = commands::generate_config_types::run() {
                        eprintln!("設定スキーマ型生成エラー: {e}");
                    }
                }
                // ナビゲーション型を生成する
                1 => {
                    if let Err(e) = commands::generate_navigation::run() {
                        eprintln!("ナビゲーション型生成エラー: {e}");
                    }
                }
                // イベントコードを生成する
                2 => {
                    if let Err(e) = commands::generate_events::run() {
                        eprintln!("イベントコード生成エラー: {e}");
                    }
                }
                // 戻る → トップメニューへ
                3 => return Ok(false),
                _ => unreachable!(),
            }
            Ok(false)
        }
    }
}

/// プロジェクト管理カテゴリのサブメニューを表示する。
/// 「← 戻る」が選択された場合は Ok(false) を返してトップに戻る。
fn show_submenu_project() -> Result<bool> {
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("プロジェクト管理")
        .items(MENU_PROJECT)
        .default(0)
        .interact_opt()?;

    match selection {
        // Ctrl+C → 終了
        None => Ok(true),
        Some(index) => {
            match index {
                // バリデーションを実行する
                // LOW-4 監査対応: 「プロジェクト初期化」を MENU_FREQUENT との重複として削除したためインデックスが変わった
                0 => {
                    if let Err(e) = commands::validate::run() {
                        eprintln!("バリデーションエラー: {e}");
                    }
                }
                // 依存関係マップを表示する
                1 => {
                    if let Err(e) = commands::deps::run() {
                        eprintln!("依存関係マップエラー: {e}");
                    }
                }
                // テンプレートマイグレーションを実行する
                2 => {
                    if let Err(e) = commands::template_migrate::run() {
                        eprintln!("テンプレートマイグレーションエラー: {e}");
                    }
                }
                // 戻る → トップメニューへ
                3 => return Ok(false),
                _ => unreachable!(),
            }
            Ok(false)
        }
    }
}

/// 運用カテゴリのサブメニューを表示する。
/// 「← 戻る」が選択された場合は Ok(false) を返してトップに戻る。
fn show_submenu_ops() -> Result<bool> {
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("運用")
        .items(MENU_OPS)
        .default(0)
        .interact_opt()?;

    match selection {
        // Ctrl+C → 終了
        None => Ok(true),
        Some(index) => {
            match index {
                // デプロイを実行する
                0 => {
                    if let Err(e) = commands::deploy::run() {
                        eprintln!("デプロイエラー: {e}");
                    }
                }
                // マイグレーション管理を実行する
                1 => {
                    if let Err(e) = commands::migrate::run() {
                        eprintln!("マイグレーション管理エラー: {e}");
                    }
                }
                // 戻る → トップメニューへ
                2 => return Ok(false),
                _ => unreachable!(),
            }
            Ok(false)
        }
    }
}

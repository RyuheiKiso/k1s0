// 本ファイルは k1s0-scaffold CLI のエントリポイント。
// 設計正典: docs/05_実装/20_コード生成設計/30_Scaffold_CLI/01_Scaffold_CLI設計.md
//
// CLI サブコマンド契約（IMP-CODEGEN-SCF-030 / 運用 30_Scaffold_CLI運用.md）:
//   scaffold list                      ... テンプレ一覧
//   scaffold new <template> [opts]     ... 雛形を生成
//   scaffold new <template> --dry-run  ... 生成せず差分のみ stdout に出力（CI / golden test 用）

use clap::{Parser, Subcommand};
use std::path::PathBuf;

// CLI のルート定義（clap derive）。
#[derive(Parser, Debug)]
#[command(
    name = "k1s0-scaffold",
    version,
    about = "k1s0 Backstage Software Template 互換 Scaffold CLI",
    long_about = None,
)]
struct Cli {
    // テンプレート探索ルート（既定: リポジトリ root の src/tier{2,3}/templates/）。
    // CI / 開発時に明示指定する用途で公開する。
    #[arg(long, env = "K1S0_SCAFFOLD_TEMPLATES_DIR", global = true)]
    templates_dir: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

// 3 サブコマンド（IMP-CODEGEN-SCF-030 で固定）。
#[derive(Subcommand, Debug)]
enum Commands {
    // 利用可能テンプレート一覧を表示する。
    List,
    // 新規サービスを生成する。
    New {
        // テンプレート名（例: tier2-go-service）
        template: String,
        // サービス名（kebab-case、^[a-z][a-z0-9-]+$）
        #[arg(long)]
        name: Option<String>,
        // 所有チーム（@k1s0/<team> 形式）
        #[arg(long)]
        owner: Option<String>,
        // 所属サブシステム
        #[arg(long, default_value = "k1s0")]
        system: String,
        // .NET ルート名前空間（tier2-dotnet-service の場合のみ必須）
        #[arg(long)]
        namespace: Option<String>,
        // 出力先ディレクトリ（既定: カレント）
        #[arg(long, short = 'o', default_value = ".")]
        out: PathBuf,
        // 入力 JSON（CI / golden test で使用、対話入力をスキップ）
        #[arg(long)]
        input: Option<PathBuf>,
        // 生成せず差分のみ stdout に出力する
        #[arg(long)]
        dry_run: bool,
        // 概要説明（任意）
        #[arg(long)]
        description: Option<String>,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    // テンプレート root を解決する（明示指定がなければ git rev-parse --show-toplevel で探す）。
    let templates_root = match cli.templates_dir {
        Some(p) => p,
        None => k1s0_scaffold::resolve_templates_root()?,
    };

    match cli.command {
        Commands::List => {
            // 全テンプレートを列挙して表形式で出力する。
            for tpl in k1s0_scaffold::list_templates(&templates_root)? {
                println!(
                    "{name:30}  {tier:8}  {language:12}  {desc}",
                    name = tpl.name,
                    tier = tpl.tier.unwrap_or_default(),
                    language = tpl.language.unwrap_or_default(),
                    desc = tpl.description.unwrap_or_default(),
                );
            }
        }
        Commands::New {
            template,
            name,
            owner,
            system,
            namespace,
            out,
            input,
            dry_run,
            description,
        } => {
            // 入力 JSON が指定されていればそれを優先採用、そうでなければ CLI flags から構築する。
            let values = if let Some(input_path) = input {
                k1s0_scaffold::load_values_from_json(&input_path)?
            } else {
                k1s0_scaffold::ScaffoldValues {
                    name: name.ok_or_else(|| {
                        anyhow::anyhow!("--name か --input が必須")
                    })?,
                    owner: owner.ok_or_else(|| {
                        anyhow::anyhow!("--owner か --input が必須")
                    })?,
                    system,
                    namespace,
                    description,
                }
            };
            // engine を呼び出して生成（または diff 出力）。
            k1s0_scaffold::scaffold(&templates_root, &template, &values, &out, dry_run)?;
        }
    }
    Ok(())
}

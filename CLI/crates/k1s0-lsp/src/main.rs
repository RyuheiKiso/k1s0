//! k1s0 Language Server Protocol 実行可能ファイル

use clap::Parser;

/// k1s0 Language Server
#[derive(Parser, Debug)]
#[command(name = "k1s0-lsp")]
#[command(about = "k1s0 Language Server Protocol implementation")]
struct Args {
    /// stdio モードで起動
    #[arg(long)]
    stdio: bool,

    /// TCP モードで起動
    #[arg(long)]
    tcp: bool,

    /// TCP ポート番号
    #[arg(long, default_value = "9257")]
    port: u16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if args.tcp {
        eprintln!("Starting k1s0-lsp in TCP mode on port {}", args.port);
        k1s0_lsp::run_tcp(args.port).await
    } else {
        // デフォルトは stdio モード
        k1s0_lsp::run_stdio().await
    }
}

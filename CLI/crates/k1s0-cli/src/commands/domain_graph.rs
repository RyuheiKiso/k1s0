//! `k1s0 domain-graph` コマンド
//!
//! ドメイン依存グラフの可視化と循環依存検出。

use clap::{Args, ValueEnum};

use k1s0_generator::domain::graph::DomainGraph;
use k1s0_generator::domain::scanner::{scan_domains, scan_features};

use crate::error::{CliError, Result};
use crate::output::output;

/// グラフ出力フォーマット
#[derive(ValueEnum, Clone, Debug, Default)]
pub enum GraphFormat {
    /// Mermaid 形式
    #[default]
    Mermaid,
    /// DOT（Graphviz）形式
    Dot,
}

/// `k1s0 domain-graph` の引数
#[derive(Args, Debug)]
#[command(after_long_help = r#"例:
  k1s0 domain-graph --format mermaid
  k1s0 domain-graph --detect-cycles --root user-management
"#)]
pub struct DomainGraphArgs {
    /// 出力フォーマット
    #[arg(short, long, value_enum, default_value = "mermaid")]
    pub format: GraphFormat,

    /// 部分グラフのルートノード
    #[arg(short, long)]
    pub root: Option<String>,

    /// 循環依存を検出する
    #[arg(long, default_value = "false")]
    pub detect_cycles: bool,
}

/// `k1s0 domain-graph` を実行する
pub fn execute(args: DomainGraphArgs) -> Result<()> {
    let out = output();

    let root_dir = std::env::current_dir().map_err(|e| {
        CliError::io(format!("カレントディレクトリの取得に失敗: {}", e))
    })?;

    let domains = scan_domains(&root_dir).map_err(|e| {
        CliError::internal(format!("ドメインの走査に失敗: {}", e))
    })?;

    let features = scan_features(&root_dir).map_err(|e| {
        CliError::internal(format!("feature の走査に失敗: {}", e))
    })?;

    let graph = DomainGraph::from_domains(&domains, &features);

    // 循環依存検出
    if args.detect_cycles {
        let cycles = graph.detect_cycles();
        if cycles.is_empty() {
            out.success("循環依存は検出されませんでした");
        } else {
            out.warning(&format!("{} 件の循環依存を検出しました:", cycles.len()));
            out.newline();
            for (i, cycle) in cycles.iter().enumerate() {
                out.list_item(
                    &format!("cycle {}", i + 1),
                    &cycle.join(" -> "),
                );
            }
            return Err(CliError::config(
                "循環依存が検出されました（K043 違反）".to_string(),
            ));
        }
        return Ok(());
    }

    // グラフ出力
    let graph_to_render = if let Some(ref root_name) = args.root {
        graph.subgraph(root_name).map_err(|e| {
            CliError::config(format!("部分グラフの構築に失敗: {}", e))
        })?
    } else {
        graph
    };

    let output_str = match args.format {
        GraphFormat::Mermaid => graph_to_render.to_mermaid(),
        GraphFormat::Dot => graph_to_render.to_dot(),
    };

    println!("{}", output_str);

    Ok(())
}

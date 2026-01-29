//! `k1s0 domain-catalog` コマンド
//!
//! ドメイン一覧と依存状況の詳細表示。

use clap::Args;

use k1s0_generator::domain::catalog::{build_catalog, format_json, format_table};
use k1s0_generator::domain::scanner::{scan_domains, scan_features};

use crate::error::{CliError, Result};
use crate::output::{output, SuccessOutput};

/// `k1s0 domain-catalog` の引数
#[derive(Args, Debug)]
#[command(after_long_help = r#"例:
  k1s0 domain-catalog --language rust --include-deprecated
"#)]
pub struct DomainCatalogArgs {
    /// 言語でフィルタ（rust, go, typescript, dart）
    #[arg(short, long)]
    pub language: Option<String>,

    /// 非推奨ドメインを含める
    #[arg(long, default_value = "false")]
    pub include_deprecated: bool,
}

/// `k1s0 domain-catalog` を実行する
pub fn execute(args: DomainCatalogArgs) -> Result<()> {
    let out = output();

    let root = std::env::current_dir().map_err(|e| {
        CliError::io(format!("カレントディレクトリの取得に失敗: {}", e))
    })?;

    let mut domains = scan_domains(&root).map_err(|e| {
        CliError::internal(format!("ドメインの走査に失敗: {}", e))
    })?;

    // フィルタ
    if let Some(ref lang) = args.language {
        domains.retain(|d| d.language == *lang);
    }
    if !args.include_deprecated {
        domains.retain(|d| d.deprecated.is_none());
    }

    let features = scan_features(&root).map_err(|e| {
        CliError::internal(format!("feature の走査に失敗: {}", e))
    })?;

    let catalog = build_catalog(&domains, &features);

    // JSON 出力
    if out.is_json_mode() {
        let json_str = format_json(&catalog);
        // serde_json で直接出力
        out.print_json(&SuccessOutput::new(
            serde_json::from_str::<serde_json::Value>(&json_str)
                .unwrap_or(serde_json::Value::Null),
        ));
        return Ok(());
    }

    // Human 出力
    out.header("k1s0 domain catalog");
    out.newline();

    if catalog.domains.is_empty() {
        out.info("domain が見つかりませんでした");
        out.hint("'k1s0 new-domain' で新しい domain を作成してください");
        return Ok(());
    }

    let table = format_table(&catalog);
    println!("{}", table);

    out.newline();
    out.success(&format!(
        "{} domain(s) found",
        catalog.summary.total
    ));

    Ok(())
}

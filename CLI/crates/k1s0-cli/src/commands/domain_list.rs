//! `k1s0 domain list` コマンド
//!
//! 全ての domain を一覧表示する。

use std::path::PathBuf;

use clap::Args;
use serde::Serialize;

use k1s0_generator::manifest::Manifest;

use crate::error::{CliError, Result};
use crate::output::{output, SuccessOutput};

/// domain 情報
#[derive(Debug, Clone, Serialize)]
pub struct DomainInfo {
    /// domain 名
    pub name: String,
    /// バージョン
    pub version: String,
    /// タイプ（backend-rust, backend-go, frontend-react, frontend-flutter）
    #[serde(rename = "type")]
    pub domain_type: String,
    /// パス
    pub path: String,
    /// 言語
    pub language: String,
}

/// domain 一覧出力（JSON 用）
#[derive(Debug, Serialize)]
pub struct DomainListOutput {
    /// domain の総数
    pub total: usize,
    /// domain リスト
    pub domains: Vec<DomainInfo>,
}

/// `k1s0 domain list` の引数
#[derive(Args, Debug)]
pub struct DomainListArgs {
    /// タイプでフィルタ（backend-rust, backend-go, frontend-react, frontend-flutter）
    #[arg(short = 't', long = "type")]
    pub filter_type: Option<String>,
}

/// `k1s0 domain list` を実行する
pub fn execute(args: DomainListArgs) -> Result<()> {
    let out = output();

    // 全ての domain を収集
    let mut domains = collect_all_domains()?;

    // タイプでフィルタ
    if let Some(ref filter_type) = args.filter_type {
        domains.retain(|d| d.domain_type == *filter_type);
    }

    // JSON 出力モードの場合
    if out.is_json_mode() {
        let output_data = DomainListOutput {
            total: domains.len(),
            domains,
        };
        out.print_json(&SuccessOutput::new(output_data));
        return Ok(());
    }

    // Human 出力
    out.header("k1s0 domain list");
    out.newline();

    if domains.is_empty() {
        out.info("domain が見つかりませんでした");
        out.hint("'k1s0 new-domain' で新しい domain を作成してください");
        return Ok(());
    }

    out.list_item("total", &domains.len().to_string());
    out.newline();

    // テーブル形式で表示
    println!("{:<25} {:<12} {:<20} {:<40}", "NAME", "VERSION", "TYPE", "PATH");
    println!("{}", "-".repeat(100));

    for domain in &domains {
        println!(
            "{:<25} {:<12} {:<20} {:<40}",
            domain.name, domain.version, domain.domain_type, domain.path
        );
    }

    out.newline();
    out.success(&format!("{} domain(s) found", domains.len()));

    Ok(())
}

/// 全ての domain を収集する
fn collect_all_domains() -> Result<Vec<DomainInfo>> {
    let domain_bases = [
        ("domain/backend/rust", "backend-rust", "rust"),
        ("domain/backend/go", "backend-go", "go"),
        ("domain/frontend/react", "frontend-react", "typescript"),
        ("domain/frontend/flutter", "frontend-flutter", "dart"),
    ];

    let mut domains = Vec::new();

    for (base_path, domain_type, language) in &domain_bases {
        let base = PathBuf::from(base_path);
        if !base.exists() {
            continue;
        }

        // ディレクトリ内の各 domain を走査
        let entries = std::fs::read_dir(&base).map_err(|e| {
            CliError::io(format!("ディレクトリの読み込みに失敗: {}: {}", base_path, e))
        })?;

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let manifest_path = path.join(".k1s0/manifest.json");
            if !manifest_path.exists() {
                continue;
            }

            // manifest.json を読み込んでバージョンを取得
            let manifest = Manifest::load(&manifest_path).ok();
            let version = manifest
                .as_ref()
                .map(|m| m.template.version.clone())
                .unwrap_or_else(|| "unknown".to_string());

            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            domains.push(DomainInfo {
                name,
                version,
                domain_type: domain_type.to_string(),
                path: path.display().to_string(),
                language: language.to_string(),
            });
        }
    }

    // 名前でソート
    domains.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(domains)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_info_serialization() {
        let info = DomainInfo {
            name: "test-domain".to_string(),
            version: "0.1.0".to_string(),
            domain_type: "backend-rust".to_string(),
            path: "domain/backend/rust/test-domain".to_string(),
            language: "rust".to_string(),
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("test-domain"));
        assert!(json.contains("0.1.0"));
        assert!(json.contains("backend-rust"));
    }

    #[test]
    fn test_domain_list_output_serialization() {
        let output = DomainListOutput {
            total: 1,
            domains: vec![DomainInfo {
                name: "test-domain".to_string(),
                version: "0.1.0".to_string(),
                domain_type: "backend-rust".to_string(),
                path: "domain/backend/rust/test-domain".to_string(),
                language: "rust".to_string(),
            }],
        };

        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"total\":1"));
        assert!(json.contains("\"domains\":["));
    }
}

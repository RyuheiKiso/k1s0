//! `k1s0 domain dependents` コマンド
//!
//! 指定した domain に依存する feature を一覧表示する。

use std::path::PathBuf;

use clap::Args;
use serde::Serialize;

use k1s0_generator::manifest::Manifest;

use crate::error::{CliError, Result};
use crate::output::{output, SuccessOutput};

/// 依存 feature 情報
#[derive(Debug, Clone, Serialize)]
pub struct DependentFeature {
    /// feature 名
    pub name: String,
    /// feature タイプ
    #[serde(rename = "type")]
    pub feature_type: String,
    /// domain バージョン制約
    pub domain_version: String,
    /// パス
    pub path: String,
}

/// 依存 feature 一覧出力（JSON 用）
#[derive(Debug, Serialize)]
pub struct DependentsOutput {
    /// 対象の domain 名
    pub domain: String,
    /// 依存 feature の総数
    pub total: usize,
    /// 依存 feature リスト
    pub dependents: Vec<DependentFeature>,
}

/// `k1s0 domain dependents` の引数
#[derive(Args, Debug)]
pub struct DomainDependentsArgs {
    /// domain 名
    #[arg(short, long)]
    pub name: String,
}

/// `k1s0 domain dependents` を実行する
pub fn execute(args: DomainDependentsArgs) -> Result<()> {
    let out = output();

    // domain の存在チェック
    if !domain_exists(&args.name) {
        return Err(CliError::config(format!(
            "domain '{}' が見つかりません",
            args.name
        ))
        .with_hint("'k1s0 domain list' で利用可能な domain を確認してください"));
    }

    // 依存している feature を収集
    let dependents = collect_dependents(&args.name)?;

    // JSON 出力モードの場合
    if out.is_json_mode() {
        let output_data = DependentsOutput {
            domain: args.name.clone(),
            total: dependents.len(),
            dependents,
        };
        out.print_json(&SuccessOutput::new(output_data));
        return Ok(());
    }

    // Human 出力
    out.header("k1s0 domain dependents");
    out.newline();

    out.list_item("domain", &args.name);
    out.list_item("total_dependents", &dependents.len().to_string());
    out.newline();

    if dependents.is_empty() {
        out.info(&format!("domain '{}' に依存する feature はありません", args.name));
        return Ok(());
    }

    // テーブル形式で表示
    println!("{:<25} {:<20} {:<15} {:<40}", "NAME", "TYPE", "VERSION", "PATH");
    println!("{}", "-".repeat(100));

    for dep in &dependents {
        println!(
            "{:<25} {:<20} {:<15} {:<40}",
            dep.name, dep.feature_type, dep.domain_version, dep.path
        );
    }

    out.newline();
    out.success(&format!(
        "domain '{}' に {} feature(s) が依存しています",
        args.name,
        dependents.len()
    ));

    Ok(())
}

/// domain が存在するかチェック
fn domain_exists(domain_name: &str) -> bool {
    let domain_bases = [
        "domain/backend/rust",
        "domain/backend/go",
        "domain/frontend/react",
        "domain/frontend/flutter",
    ];

    for base in &domain_bases {
        let path = PathBuf::from(format!("{}/{}", base, domain_name));
        if path.exists() && path.join(".k1s0/manifest.json").exists() {
            return true;
        }
    }

    false
}

/// 指定 domain に依存する feature を収集
fn collect_dependents(domain_name: &str) -> Result<Vec<DependentFeature>> {
    let feature_bases = [
        ("feature/backend/rust", "backend-rust"),
        ("feature/backend/go", "backend-go"),
        ("feature/frontend/react", "frontend-react"),
        ("feature/frontend/flutter", "frontend-flutter"),
    ];

    let mut dependents = Vec::new();

    for (base_path, feature_type) in &feature_bases {
        let base = PathBuf::from(base_path);
        if !base.exists() {
            continue;
        }

        // ディレクトリ内の各 feature を走査
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

            // manifest.json を読み込んで domain 依存をチェック
            let manifest = match Manifest::load(&manifest_path) {
                Ok(m) => m,
                Err(_) => continue,
            };

            // domain 依存をチェック（新形式: dependencies.domain は HashMap<String, String>）
            if let Some(deps) = &manifest.dependencies {
                if let Some(domain_deps) = &deps.domain {
                    if let Some(version_constraint) = domain_deps.get(domain_name) {
                        let name = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown")
                            .to_string();

                        dependents.push(DependentFeature {
                            name,
                            feature_type: feature_type.to_string(),
                            domain_version: version_constraint.clone(),
                            path: path.display().to_string(),
                        });
                    }
                }
            }

            // 旧形式: manifest.domain フィールドでもチェック
            if manifest.domain.as_ref() == Some(&domain_name.to_string()) {
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                // 既に追加されていない場合のみ追加
                if !dependents.iter().any(|d| d.name == name && d.path == path.display().to_string()) {
                    dependents.push(DependentFeature {
                        name,
                        feature_type: feature_type.to_string(),
                        domain_version: manifest.domain_version.clone().unwrap_or_else(|| "^0.1.0".to_string()),
                        path: path.display().to_string(),
                    });
                }
            }
        }
    }

    // 名前でソート
    dependents.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(dependents)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependent_feature_serialization() {
        let feature = DependentFeature {
            name: "test-feature".to_string(),
            feature_type: "backend-rust".to_string(),
            domain_version: "^0.1.0".to_string(),
            path: "feature/backend/rust/test-feature".to_string(),
        };

        let json = serde_json::to_string(&feature).unwrap();
        assert!(json.contains("test-feature"));
        assert!(json.contains("^0.1.0"));
    }

    #[test]
    fn test_dependents_output_serialization() {
        let output = DependentsOutput {
            domain: "test-domain".to_string(),
            total: 1,
            dependents: vec![DependentFeature {
                name: "test-feature".to_string(),
                feature_type: "backend-rust".to_string(),
                domain_version: "^0.1.0".to_string(),
                path: "feature/backend/rust/test-feature".to_string(),
            }],
        };

        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"domain\":\"test-domain\""));
        assert!(json.contains("\"total\":1"));
    }
}

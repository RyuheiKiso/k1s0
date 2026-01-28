//! `k1s0 domain impact` コマンド
//!
//! domain のバージョンアップによる影響を分析する。

use std::path::PathBuf;

use clap::Args;
use serde::Serialize;

use k1s0_generator::manifest::Manifest;

use crate::error::{CliError, Result};
use crate::output::{output, SuccessOutput};

/// 影響を受ける feature 情報
#[derive(Debug, Clone, Serialize)]
pub struct ImpactedFeature {
    /// feature 名
    pub name: String,
    /// feature タイプ
    #[serde(rename = "type")]
    pub feature_type: String,
    /// 現在の domain バージョン制約
    pub current_constraint: String,
    /// 影響度（compatible / breaking）
    pub impact: ImpactLevel,
    /// パス
    pub path: String,
}

/// 影響度
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ImpactLevel {
    /// 互換性あり（制約を満たす）
    Compatible,
    /// 破壊的変更（制約を満たさない）
    Breaking,
}

impl std::fmt::Display for ImpactLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImpactLevel::Compatible => write!(f, "compatible"),
            ImpactLevel::Breaking => write!(f, "BREAKING"),
        }
    }
}

/// 影響分析出力（JSON 用）
#[derive(Debug, Serialize)]
pub struct ImpactOutput {
    /// 対象の domain 名
    pub domain: String,
    /// 現在のバージョン
    pub current_version: String,
    /// 目標バージョン
    pub target_version: String,
    /// 影響を受ける feature の総数
    pub total_impacted: usize,
    /// 破壊的変更となる feature の数
    pub breaking_count: usize,
    /// 影響を受ける feature リスト
    pub impacted: Vec<ImpactedFeature>,
}

/// `k1s0 domain impact` の引数
#[derive(Args, Debug)]
pub struct DomainImpactArgs {
    /// domain 名
    #[arg(short, long)]
    pub name: String,

    /// 目標バージョン
    #[arg(long)]
    pub to: String,
}

/// `k1s0 domain impact` を実行する
pub fn execute(args: DomainImpactArgs) -> Result<()> {
    let out = output();

    // domain の存在チェックとバージョン取得
    let (domain_path, current_version) = find_domain_and_version(&args.name)?;

    // 目標バージョンのバリデーション
    validate_version(&args.to)?;

    // 影響を受ける feature を分析
    let impacted = analyze_impact(&args.name, &args.to)?;

    let breaking_count = impacted.iter().filter(|f| f.impact == ImpactLevel::Breaking).count();

    // JSON 出力モードの場合
    if out.is_json_mode() {
        let output_data = ImpactOutput {
            domain: args.name.clone(),
            current_version: current_version.clone(),
            target_version: args.to.clone(),
            total_impacted: impacted.len(),
            breaking_count,
            impacted,
        };
        out.print_json(&SuccessOutput::new(output_data));
        return Ok(());
    }

    // Human 出力
    out.header("k1s0 domain impact");
    out.newline();

    out.list_item("domain", &args.name);
    out.list_item("path", &domain_path.display().to_string());
    out.list_item("current_version", &current_version);
    out.list_item("target_version", &args.to);
    out.newline();

    if impacted.is_empty() {
        out.info(&format!("domain '{}' に依存する feature はありません", args.name));
        out.success("影響なし");
        return Ok(());
    }

    out.list_item("total_impacted", &impacted.len().to_string());
    out.list_item("breaking_count", &breaking_count.to_string());
    out.newline();

    // テーブル形式で表示
    println!("{:<25} {:<20} {:<15} {:<12}", "NAME", "TYPE", "CONSTRAINT", "IMPACT");
    println!("{}", "-".repeat(75));

    for feature in &impacted {
        let impact_display = match feature.impact {
            ImpactLevel::Compatible => "OK",
            ImpactLevel::Breaking => "BREAKING",
        };
        println!(
            "{:<25} {:<20} {:<15} {:<12}",
            feature.name, feature.feature_type, feature.current_constraint, impact_display
        );
    }

    out.newline();

    if breaking_count > 0 {
        out.warning(&format!(
            "{} feature(s) に破壊的変更の影響があります",
            breaking_count
        ));
        out.hint("'k1s0 feature update-domain' で各 feature の依存バージョンを更新してください");
    } else {
        out.success("全ての feature は新しいバージョンと互換性があります");
    }

    Ok(())
}

/// domain を探してバージョンを取得
fn find_domain_and_version(domain_name: &str) -> Result<(PathBuf, String)> {
    let domain_bases = [
        "domain/backend/rust",
        "domain/backend/go",
        "domain/frontend/react",
        "domain/frontend/flutter",
    ];

    for base in &domain_bases {
        let path = PathBuf::from(format!("{}/{}", base, domain_name));
        let manifest_path = path.join(".k1s0/manifest.json");

        if manifest_path.exists() {
            let manifest = Manifest::load(&manifest_path).map_err(|e| {
                CliError::config(format!("manifest.json の読み込みに失敗: {}", e))
            })?;
            return Ok((path, manifest.template.version));
        }
    }

    Err(CliError::config(format!(
        "domain '{}' が見つかりません",
        domain_name
    ))
    .with_hint("'k1s0 domain list' で利用可能な domain を確認してください"))
}

/// バージョン形式のバリデーション
fn validate_version(version: &str) -> Result<()> {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() < 3 {
        return Err(CliError::usage(format!(
            "無効なバージョン形式: {} (x.y.z 形式が必要)",
            version
        )));
    }

    for (i, part) in parts.iter().take(3).enumerate() {
        let clean_part = part.split('-').next().unwrap_or(part);
        if clean_part.parse::<u32>().is_err() {
            return Err(CliError::usage(format!(
                "無効なバージョン番号: {} (位置 {})",
                part, i + 1
            )));
        }
    }

    Ok(())
}

/// 影響を分析
fn analyze_impact(domain_name: &str, target_version: &str) -> Result<Vec<ImpactedFeature>> {
    let feature_bases = [
        ("feature/backend/rust", "backend-rust"),
        ("feature/backend/go", "backend-go"),
        ("feature/frontend/react", "frontend-react"),
        ("feature/frontend/flutter", "frontend-flutter"),
    ];

    let mut impacted = Vec::new();

    for (base_path, feature_type) in &feature_bases {
        let base = PathBuf::from(base_path);
        if !base.exists() {
            continue;
        }

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

            let manifest = match Manifest::load(&manifest_path) {
                Ok(m) => m,
                Err(_) => continue,
            };

            // domain 依存をチェック
            if let Some(deps) = &manifest.dependencies {
                if let Some(domain_dep) = &deps.domain {
                    if domain_dep.name == domain_name {
                        let name = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown")
                            .to_string();

                        let impact = check_version_compatibility(&domain_dep.version, target_version);

                        impacted.push(ImpactedFeature {
                            name,
                            feature_type: feature_type.to_string(),
                            current_constraint: domain_dep.version.clone(),
                            impact,
                            path: path.display().to_string(),
                        });
                    }
                }
            }
        }
    }

    // 名前でソート
    impacted.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(impacted)
}

/// バージョン制約と目標バージョンの互換性をチェック
fn check_version_compatibility(constraint: &str, target_version: &str) -> ImpactLevel {
    // ^x.y.z 形式の制約を解析
    let constraint_version = constraint.trim_start_matches('^');

    let constraint_parts: Vec<u32> = constraint_version
        .split('.')
        .filter_map(|p| p.split('-').next().and_then(|s| s.parse().ok()))
        .collect();

    let target_parts: Vec<u32> = target_version
        .split('.')
        .filter_map(|p| p.split('-').next().and_then(|s| s.parse().ok()))
        .collect();

    if constraint_parts.len() < 2 || target_parts.len() < 2 {
        return ImpactLevel::Breaking;
    }

    // ^x.y.z は x.y.0 <= version < (x+1).0.0 を許容
    // ただし x が 0 の場合は 0.y.0 <= version < 0.(y+1).0 を許容

    let (constraint_major, constraint_minor) = (constraint_parts[0], constraint_parts[1]);
    let (target_major, target_minor) = (target_parts[0], target_parts[1]);

    if constraint_major == 0 {
        // 0.x.y の場合、マイナーバージョンが互換性の境界
        if target_major == 0 && target_minor == constraint_minor {
            ImpactLevel::Compatible
        } else {
            ImpactLevel::Breaking
        }
    } else {
        // 1.x.y 以上の場合、メジャーバージョンが互換性の境界
        if target_major == constraint_major && target_minor >= constraint_minor {
            ImpactLevel::Compatible
        } else {
            ImpactLevel::Breaking
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_version_compatibility_compatible() {
        // 0.x.y 系
        assert_eq!(check_version_compatibility("^0.1.0", "0.1.0"), ImpactLevel::Compatible);
        assert_eq!(check_version_compatibility("^0.1.0", "0.1.5"), ImpactLevel::Compatible);

        // 1.x.y 系
        assert_eq!(check_version_compatibility("^1.0.0", "1.0.0"), ImpactLevel::Compatible);
        assert_eq!(check_version_compatibility("^1.0.0", "1.5.0"), ImpactLevel::Compatible);
        assert_eq!(check_version_compatibility("^1.2.0", "1.2.5"), ImpactLevel::Compatible);
        assert_eq!(check_version_compatibility("^1.2.0", "1.3.0"), ImpactLevel::Compatible);
    }

    #[test]
    fn test_check_version_compatibility_breaking() {
        // 0.x.y 系（マイナーバージョン違い）
        assert_eq!(check_version_compatibility("^0.1.0", "0.2.0"), ImpactLevel::Breaking);
        assert_eq!(check_version_compatibility("^0.1.0", "1.0.0"), ImpactLevel::Breaking);

        // 1.x.y 系（メジャーバージョン違い）
        assert_eq!(check_version_compatibility("^1.0.0", "2.0.0"), ImpactLevel::Breaking);
        assert_eq!(check_version_compatibility("^1.2.0", "1.1.0"), ImpactLevel::Breaking);
    }

    #[test]
    fn test_validate_version_valid() {
        assert!(validate_version("1.0.0").is_ok());
        assert!(validate_version("0.1.0").is_ok());
        assert!(validate_version("1.2.3-alpha").is_ok());
    }

    #[test]
    fn test_validate_version_invalid() {
        assert!(validate_version("invalid").is_err());
        assert!(validate_version("1.2").is_err());
        assert!(validate_version("a.b.c").is_err());
    }
}

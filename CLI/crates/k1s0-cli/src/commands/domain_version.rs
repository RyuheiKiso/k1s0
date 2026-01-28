//! `k1s0 domain version` コマンド
//!
//! domain のバージョンを更新する。

use std::path::PathBuf;

use chrono::Utc;
use clap::{Args, ValueEnum};

use k1s0_generator::manifest::Manifest;

use crate::error::{CliError, Result};
use crate::output::output;

/// バージョンバンプの種類
#[derive(ValueEnum, Clone, Debug, Copy)]
pub enum BumpType {
    /// メジャーバージョン (1.0.0 -> 2.0.0)
    Major,
    /// マイナーバージョン (1.0.0 -> 1.1.0)
    Minor,
    /// パッチバージョン (1.0.0 -> 1.0.1)
    Patch,
}

impl std::fmt::Display for BumpType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BumpType::Major => write!(f, "major"),
            BumpType::Minor => write!(f, "minor"),
            BumpType::Patch => write!(f, "patch"),
        }
    }
}

/// `k1s0 domain version` の引数
#[derive(Args, Debug)]
pub struct DomainVersionArgs {
    /// domain 名
    #[arg(short, long)]
    pub name: String,

    /// バージョンバンプの種類
    #[arg(short, long, value_enum)]
    pub bump: BumpType,

    /// 破壊的変更の説明（major bump 時に推奨）
    #[arg(long)]
    pub breaking: Option<String>,

    /// 変更の説明
    #[arg(short, long)]
    pub message: Option<String>,
}

/// `k1s0 domain version` を実行する
pub fn execute(args: DomainVersionArgs) -> Result<()> {
    let out = output();

    out.header("k1s0 domain version");
    out.newline();

    // domain ディレクトリを探す
    let domain_path = find_domain_path(&args.name)?;
    out.list_item("domain", &args.name);
    out.list_item("path", &domain_path.display().to_string());

    // manifest.json を読み込む
    let manifest_path = domain_path.join(".k1s0/manifest.json");
    if !manifest_path.exists() {
        return Err(CliError::config(format!(
            "domain '{}' の manifest.json が見つかりません",
            args.name
        ))
        .with_target(manifest_path.display().to_string()));
    }

    let mut manifest = Manifest::load(&manifest_path).map_err(|e| {
        CliError::config(format!("manifest.json の読み込みに失敗: {}", e))
    })?;

    // 現在のバージョンを取得
    let current_version = get_current_version(&manifest);
    out.list_item("current_version", &current_version);

    // 新しいバージョンを計算
    let new_version = bump_version(&current_version, args.bump)?;
    out.list_item("new_version", &new_version);
    out.list_item("bump", &args.bump.to_string());

    out.newline();

    // manifest.json を更新
    update_manifest_version(&mut manifest, &new_version);
    manifest.save(&manifest_path).map_err(|e| {
        CliError::io(format!("manifest.json の保存に失敗: {}", e))
    })?;
    out.file_modified(".k1s0/manifest.json");

    // CHANGELOG.md を更新
    let changelog_path = domain_path.join("CHANGELOG.md");
    update_changelog(
        &changelog_path,
        &new_version,
        args.bump,
        args.breaking.as_deref(),
        args.message.as_deref(),
    )?;
    if changelog_path.exists() {
        out.file_modified("CHANGELOG.md");
    } else {
        out.file_added("CHANGELOG.md");
    }

    out.newline();
    out.success(&format!(
        "domain '{}' のバージョンを {} から {} に更新しました",
        args.name, current_version, new_version
    ));

    Ok(())
}

/// domain ディレクトリを探す
fn find_domain_path(domain_name: &str) -> Result<PathBuf> {
    let domain_bases = [
        "domain/backend/rust",
        "domain/backend/go",
        "domain/frontend/react",
        "domain/frontend/flutter",
    ];

    for base in &domain_bases {
        let path = PathBuf::from(format!("{}/{}", base, domain_name));
        if path.exists() {
            return Ok(path);
        }
    }

    Err(CliError::config(format!(
        "domain '{}' が見つかりません",
        domain_name
    ))
    .with_hint("'k1s0 domain list' で利用可能な domain を確認してください"))
}

/// manifest から現在のバージョンを取得
fn get_current_version(manifest: &Manifest) -> String {
    // まず template.version を使用
    manifest.template.version.clone()
}

/// バージョンをバンプする
fn bump_version(version: &str, bump_type: BumpType) -> Result<String> {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() < 3 {
        return Err(CliError::config(format!(
            "無効なバージョン形式: {} (x.y.z 形式が必要)",
            version
        )));
    }

    let major: u32 = parts[0].parse().map_err(|_| {
        CliError::config(format!("無効なメジャーバージョン: {}", parts[0]))
    })?;
    let minor: u32 = parts[1].parse().map_err(|_| {
        CliError::config(format!("無効なマイナーバージョン: {}", parts[1]))
    })?;
    let patch: u32 = parts[2].split('-').next().unwrap_or("0").parse().map_err(|_| {
        CliError::config(format!("無効なパッチバージョン: {}", parts[2]))
    })?;

    let (new_major, new_minor, new_patch) = match bump_type {
        BumpType::Major => (major + 1, 0, 0),
        BumpType::Minor => (major, minor + 1, 0),
        BumpType::Patch => (major, minor, patch + 1),
    };

    Ok(format!("{}.{}.{}", new_major, new_minor, new_patch))
}

/// manifest のバージョンを更新
fn update_manifest_version(manifest: &mut Manifest, new_version: &str) {
    manifest.template.version = new_version.to_string();
}

/// CHANGELOG.md を更新
fn update_changelog(
    path: &PathBuf,
    new_version: &str,
    bump_type: BumpType,
    breaking: Option<&str>,
    message: Option<&str>,
) -> Result<()> {
    let date = Utc::now().format("%Y-%m-%d").to_string();

    let mut new_entry = format!("\n## [{}] - {}\n\n", new_version, date);

    // 破壊的変更がある場合
    if let Some(breaking_msg) = breaking {
        new_entry.push_str("### BREAKING CHANGES\n\n");
        new_entry.push_str(&format!("- {}\n\n", breaking_msg));
    }

    // セクションを追加
    let section = match bump_type {
        BumpType::Major => "Changed",
        BumpType::Minor => "Added",
        BumpType::Patch => "Fixed",
    };
    new_entry.push_str(&format!("### {}\n\n", section));

    if let Some(msg) = message {
        new_entry.push_str(&format!("- {}\n", msg));
    } else {
        new_entry.push_str(&format!("- Version bump ({})\n", bump_type));
    }

    // CHANGELOG.md の内容を更新
    let existing_content = if path.exists() {
        std::fs::read_to_string(path).unwrap_or_default()
    } else {
        "# Changelog\n\nAll notable changes to this project will be documented in this file.\n\n\
         The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),\n\
         and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).\n".to_string()
    };

    // ヘッダー部分と本文を分離
    let (header, body) = if let Some(pos) = existing_content.find("\n## ") {
        (&existing_content[..pos], &existing_content[pos..])
    } else {
        (existing_content.as_str(), "")
    };

    let new_content = format!("{}{}{}", header, new_entry, body);
    std::fs::write(path, new_content).map_err(|e| {
        CliError::io(format!("CHANGELOG.md の書き込みに失敗: {}", e))
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bump_version_major() {
        assert_eq!(bump_version("1.2.3", BumpType::Major).unwrap(), "2.0.0");
        assert_eq!(bump_version("0.1.0", BumpType::Major).unwrap(), "1.0.0");
    }

    #[test]
    fn test_bump_version_minor() {
        assert_eq!(bump_version("1.2.3", BumpType::Minor).unwrap(), "1.3.0");
        assert_eq!(bump_version("0.1.0", BumpType::Minor).unwrap(), "0.2.0");
    }

    #[test]
    fn test_bump_version_patch() {
        assert_eq!(bump_version("1.2.3", BumpType::Patch).unwrap(), "1.2.4");
        assert_eq!(bump_version("0.1.0", BumpType::Patch).unwrap(), "0.1.1");
    }

    #[test]
    fn test_bump_version_with_prerelease() {
        assert_eq!(bump_version("1.2.3-alpha", BumpType::Patch).unwrap(), "1.2.4");
    }

    #[test]
    fn test_bump_version_invalid() {
        assert!(bump_version("invalid", BumpType::Patch).is_err());
        assert!(bump_version("1.2", BumpType::Patch).is_err());
    }
}

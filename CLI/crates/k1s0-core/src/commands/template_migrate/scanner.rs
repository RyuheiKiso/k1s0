use anyhow::Result;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use super::parser;
use super::types::TemplateManifest;

/// ルートディレクトリ以下の .k1s0-template.yaml を走査し、
/// マイグレーション対象の一覧を返す。
///
/// # Errors
///
/// マニフェストの読み込みに失敗した場合にエラーを返す。
pub fn scan_targets(root: &Path) -> Result<Vec<(PathBuf, TemplateManifest)>> {
    let mut targets = Vec::new();

    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_name() == ".k1s0-template.yaml" {
            let manifest = parser::parse_manifest(entry.path())?;
            let project_dir = entry.path().parent().unwrap_or(root).to_path_buf();
            targets.push((project_dir, manifest));
        }
    }

    Ok(targets)
}

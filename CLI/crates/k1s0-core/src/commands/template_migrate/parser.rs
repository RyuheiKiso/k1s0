use anyhow::Result;
use std::path::Path;

use super::types::TemplateManifest;

/// .k1s0-template.yaml を読み込み、マニフェストを返す。
///
/// # Errors
///
/// ファイルの読み込みまたはパースに失敗した場合にエラーを返す。
pub fn parse_manifest(path: &Path) -> Result<TemplateManifest> {
    let content = std::fs::read_to_string(path)?;
    let manifest: TemplateManifest = serde_yaml::from_str(&content)?;
    Ok(manifest)
}

// 本ファイルは k1s0-scaffold engine の library API。
// CLI（src/main.rs）と Backstage UI 経路（custom action k1s0:scaffold-engine）の両方が
// 本ライブラリを呼ぶことで、生成結果のバイト一致を保証する（IMP-DEV-SO-035）。
//
// 設計正典: docs/05_実装/20_コード生成設計/30_Scaffold_CLI/01_Scaffold_CLI設計.md
#![deny(missing_debug_implementations)]
#![deny(unsafe_code)]

// 公開モジュール（外部から呼ばれる API）。
pub mod engine;
pub mod error;
pub mod template;

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub use engine::{render_skeleton, scaffold};
pub use error::ScaffoldError;
pub use template::{TemplateManifest, TemplateMetadata};

// scaffold 引数（CLI から / Backstage custom action から渡される共通入力）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaffoldValues {
    // 生成サービス名（kebab-case）
    pub name: String,
    // 所有チーム（@k1s0/<team>）
    pub owner: String,
    // 所属サブシステム
    pub system: String,
    // .NET ルート名前空間（tier2-dotnet-service の場合のみ必要）
    #[serde(default)]
    pub namespace: Option<String>,
    // 概要説明（catalog-info.yaml の description に入る）
    #[serde(default)]
    pub description: Option<String>,
}

// list_templates の返却値（1 テンプレート 1 行の概要）。
#[derive(Debug, Clone)]
pub struct TemplateSummary {
    pub name: String,
    pub description: Option<String>,
    pub tier: Option<String>,
    pub language: Option<String>,
}

// 入力 JSON の deserialize（CI / golden test 用）。
pub fn load_values_from_json(path: &Path) -> Result<ScaffoldValues, ScaffoldError> {
    let raw = std::fs::read_to_string(path)
        .map_err(|e| ScaffoldError::Io(format!("read {}: {}", path.display(), e)))?;
    let v: ScaffoldValues = serde_json::from_str(&raw)
        .map_err(|e| ScaffoldError::Parse(format!("invalid input json: {}", e)))?;
    Ok(v)
}

// テンプレート一覧を列挙する（src/tier{2,3}/templates/<template-name>/template.yaml）。
pub fn list_templates(templates_root: &Path) -> Result<Vec<TemplateSummary>, ScaffoldError> {
    let mut out = Vec::new();
    // tier2 / tier3 の 2 階層を走査する。
    for tier_dir in ["tier2/templates", "tier3/templates"] {
        let dir = templates_root.join(tier_dir);
        if !dir.is_dir() {
            continue;
        }
        for entry in std::fs::read_dir(&dir)
            .map_err(|e| ScaffoldError::Io(format!("read_dir {}: {}", dir.display(), e)))?
        {
            let entry = entry.map_err(|e| ScaffoldError::Io(format!("read_dir entry: {}", e)))?;
            let template_yaml = entry.path().join("template.yaml");
            if !template_yaml.is_file() {
                continue;
            }
            // template.yaml をパース。失敗したら read 段階で報告して continue。
            let manifest = template::load(&template_yaml)?;
            // values から tier / language を抜き取る（fetch ステップ values 経由）。
            let (tier, language) = manifest.fetch_step_tier_language();
            out.push(TemplateSummary {
                name: manifest.metadata.name,
                description: manifest.metadata.description,
                tier,
                language,
            });
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

// テンプレート root を git toplevel から解決する。
pub fn resolve_templates_root() -> Result<PathBuf, ScaffoldError> {
    let out = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .map_err(|e| ScaffoldError::Io(format!("git rev-parse: {}", e)))?;
    if !out.status.success() {
        return Err(ScaffoldError::Io(format!(
            "git rev-parse failed: {}",
            String::from_utf8_lossy(&out.stderr)
        )));
    }
    let toplevel = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    let root = PathBuf::from(toplevel).join("src");
    Ok(root)
}

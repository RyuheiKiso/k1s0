// 本ファイルは Backstage Software Template v1beta3 の template.yaml を読込・解釈する。
//
// 互換対象は spec.parameters / spec.steps[?action=fetch:template, input.values] のみ。
// publish:github 等の他 action は CLI 経路では無視（Backstage UI 経路では Backstage が処理する）。

use crate::error::ScaffoldError;
use serde::{Deserialize, Serialize};
use std::path::Path;

// template.yaml のルート。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TemplateManifest {
    // apiVersion: scaffolder.backstage.io/v1beta3 を想定。
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub kind: String,
    pub metadata: TemplateMetadata,
    pub spec: TemplateSpec,
}

// metadata セクション。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TemplateMetadata {
    pub name: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub annotations: serde_yaml::Mapping,
}

// spec セクション。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TemplateSpec {
    #[serde(default)]
    pub owner: Option<String>,
    #[serde(default, rename = "type")]
    pub kind_type: Option<String>,
    #[serde(default)]
    pub parameters: Vec<serde_yaml::Value>,
    #[serde(default)]
    pub steps: Vec<TemplateStep>,
}

// steps[i]。fetch:template ステップのみ CLI 経路で参照する。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TemplateStep {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    pub action: String,
    #[serde(default)]
    pub input: serde_yaml::Mapping,
}

// template.yaml を読込・パースする。
pub fn load(path: &Path) -> Result<TemplateManifest, ScaffoldError> {
    let raw = std::fs::read_to_string(path)
        .map_err(|e| ScaffoldError::Io(format!("read {}: {}", path.display(), e)))?;
    let manifest: TemplateManifest = serde_yaml::from_str(&raw)
        .map_err(|e| ScaffoldError::Parse(format!("invalid template.yaml {}: {}", path.display(), e)))?;
    Ok(manifest)
}

impl TemplateManifest {
    // fetch:template ステップの input.values から (tier, language) を抜き取る。
    // 値が `${{ parameters.xxx }}` のテンプレ参照のままなら None を返す。
    pub fn fetch_step_tier_language(&self) -> (Option<String>, Option<String>) {
        let fetch = self
            .spec
            .steps
            .iter()
            .find(|s| s.action == "fetch:template");
        let Some(fetch) = fetch else {
            return (None, None);
        };
        let values = fetch
            .input
            .get(&serde_yaml::Value::String("values".to_owned()));
        let Some(serde_yaml::Value::Mapping(values)) = values else {
            return (None, None);
        };
        let pick = |k: &str| -> Option<String> {
            let v = values.get(serde_yaml::Value::String(k.to_owned()))?;
            // string 型かつ `${{` で始まらない場合のみ採用。
            if let serde_yaml::Value::String(s) = v {
                if s.starts_with("${{") {
                    None
                } else {
                    Some(s.clone())
                }
            } else {
                None
            }
        };
        (pick("tier"), pick("language"))
    }

    // skeleton 配下のテンプレ展開時に Handlebars に渡す追加 values（fetch step が固定値で
    // 持つ tier / language 等）を抜き取る。
    pub fn fetch_step_static_values(&self) -> serde_yaml::Mapping {
        let mut out = serde_yaml::Mapping::new();
        let Some(fetch) = self
            .spec
            .steps
            .iter()
            .find(|s| s.action == "fetch:template")
        else {
            return out;
        };
        let Some(serde_yaml::Value::Mapping(values)) = fetch
            .input
            .get(&serde_yaml::Value::String("values".to_owned()))
        else {
            return out;
        };
        for (k, v) in values {
            if let serde_yaml::Value::String(s) = v {
                if !s.starts_with("${{") {
                    out.insert(k.clone(), v.clone());
                }
            }
        }
        out
    }
}

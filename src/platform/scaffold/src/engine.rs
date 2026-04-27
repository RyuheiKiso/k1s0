// 本ファイルは Handlebars + walkdir を使った skeleton 展開エンジン。
//
// 動作:
//   1. <templates_root>/tier{2,3}/templates/<template>/template.yaml を読込
//   2. ScaffoldValues + template.yaml の固定 values をマージして Handlebars context を構築
//   3. <templates_root>/.../<template>/skeleton/ 以下を再帰走査
//   4. 各 path / file content を Handlebars でレンダリング、`.hbs` 拡張子を取り除いて出力
//   5. dry_run=true なら ファイル出力せず stdout に diff のみ出力

use crate::error::ScaffoldError;
use crate::template;
use crate::ScaffoldValues;
use handlebars::Handlebars;
use serde_json::{Map, Value};
use std::fs;
use std::path::{Path, PathBuf};

// scaffold は新規サービス雛形を生成する高位 API。
pub fn scaffold(
    templates_root: &Path,
    template_name: &str,
    values: &ScaffoldValues,
    out_dir: &Path,
    dry_run: bool,
) -> Result<(), ScaffoldError> {
    // テンプレ root を tier2 / tier3 のどちらに置かれているか自動判定する。
    let template_dir = locate_template(templates_root, template_name)?;
    let manifest = template::load(&template_dir.join("template.yaml"))?;

    // Handlebars context を構築（ScaffoldValues + template.yaml 固定 values）。
    let context = build_context(values, &manifest)?;

    // skeleton 配下を再帰展開する。
    let skeleton = template_dir.join("skeleton");
    if !skeleton.is_dir() {
        return Err(ScaffoldError::Validation(format!(
            "skeleton/ ディレクトリが見つからない: {}",
            skeleton.display()
        )));
    }
    render_skeleton(&skeleton, &context, out_dir, dry_run)
}

// 指定された template_name（template.yaml の metadata.name）を tier2 / tier3 配下から
// 走査して探す。ディレクトリ名と metadata.name は別物（IMP-CODEGEN-SCF-031〜034 で
// 例えばディレクトリ go-service / metadata.name tier2-go-service の組合せ）。
fn locate_template(templates_root: &Path, template_name: &str) -> Result<PathBuf, ScaffoldError> {
    for tier_dir in ["tier2/templates", "tier3/templates"] {
        let dir = templates_root.join(tier_dir);
        if !dir.is_dir() {
            continue;
        }
        for entry in std::fs::read_dir(&dir)
            .map_err(|e| ScaffoldError::Io(format!("read_dir {}: {}", dir.display(), e)))?
        {
            let entry = entry
                .map_err(|e| ScaffoldError::Io(format!("read_dir entry: {}", e)))?;
            let template_yaml = entry.path().join("template.yaml");
            if !template_yaml.is_file() {
                continue;
            }
            let manifest = crate::template::load(&template_yaml)?;
            if manifest.metadata.name == template_name {
                return Ok(entry.path());
            }
        }
    }
    Err(ScaffoldError::Validation(format!(
        "テンプレート '{}' が見つからない（{}/tier{{2,3}}/templates/ 配下の template.yaml metadata.name を走査）",
        template_name,
        templates_root.display(),
    )))
}

// ScaffoldValues + template.yaml の固定 values をマージして Handlebars 用 context を作る。
fn build_context(
    values: &ScaffoldValues,
    manifest: &template::TemplateManifest,
) -> Result<Value, ScaffoldError> {
    let mut map = Map::new();
    // user-supplied values
    map.insert("name".to_owned(), Value::String(values.name.clone()));
    map.insert("owner".to_owned(), Value::String(values.owner.clone()));
    map.insert("system".to_owned(), Value::String(values.system.clone()));
    if let Some(ns) = &values.namespace {
        map.insert("namespace".to_owned(), Value::String(ns.clone()));
    }
    if let Some(d) = &values.description {
        map.insert("description".to_owned(), Value::String(d.clone()));
    }
    // template.yaml の固定 values（tier / language 等）を上書きせずにマージ。
    for (k, v) in manifest.fetch_step_static_values() {
        let serde_yaml::Value::String(key) = k else {
            continue;
        };
        let serde_yaml::Value::String(val) = v else {
            continue;
        };
        map.entry(key).or_insert(Value::String(val));
    }
    // 必須フィールド検証
    validate(&map, &manifest.metadata.name)?;
    Ok(Value::Object(map))
}

// 必須フィールドが揃っているか検証する。
fn validate(map: &Map<String, Value>, template_name: &str) -> Result<(), ScaffoldError> {
    for required in ["name", "owner"] {
        if !map.contains_key(required) {
            return Err(ScaffoldError::Validation(format!(
                "{} は必須: template={}",
                required, template_name
            )));
        }
    }
    // tier2-dotnet-service は namespace が必須
    if template_name == "tier2-dotnet-service" && !map.contains_key("namespace") {
        return Err(ScaffoldError::Validation(
            "tier2-dotnet-service は --namespace が必須".to_owned(),
        ));
    }
    Ok(())
}

// skeleton 配下のファイル / ディレクトリを Handlebars で展開して out_dir に書き出す。
// dry_run=true の場合はファイル出力せず stdout に出力ファイル一覧と先頭差分のみを出す。
pub fn render_skeleton(
    skeleton: &Path,
    context: &Value,
    out_dir: &Path,
    dry_run: bool,
) -> Result<(), ScaffoldError> {
    let mut hb = Handlebars::new();
    // strict mode で未定義変数を即時エラー化する（テンプレ間違いの早期検出）。
    hb.set_strict_mode(true);

    for entry in walkdir::WalkDir::new(skeleton).into_iter() {
        let entry = entry.map_err(|e| ScaffoldError::Io(format!("walkdir: {}", e)))?;
        let path = entry.path();
        if path == skeleton {
            continue;
        }
        // 相対パス（skeleton 配下）を取得し、Handlebars でレンダ。
        let rel = path
            .strip_prefix(skeleton)
            .map_err(|e| ScaffoldError::Io(format!("strip_prefix: {}", e)))?;
        let rendered_path_str = hb
            .render_template(&rel.to_string_lossy(), context)
            .map_err(|e| {
                ScaffoldError::Render(format!("path render '{}': {}", rel.display(), e))
            })?;
        // `.hbs` 拡張子を除去する。
        let rendered_path = strip_hbs_suffix(&rendered_path_str);
        let dest = out_dir.join(rendered_path);

        if entry.file_type().is_dir() {
            if dry_run {
                println!("[dry-run] mkdir   {}", dest.display());
            } else {
                fs::create_dir_all(&dest)
                    .map_err(|e| ScaffoldError::Io(format!("create_dir_all: {}", e)))?;
            }
            continue;
        }

        // 通常ファイルは内容を Handlebars でレンダリングして書き出す。
        let raw = fs::read_to_string(path)
            .map_err(|e| ScaffoldError::Io(format!("read {}: {}", path.display(), e)))?;
        let rendered = hb
            .render_template(&raw, context)
            .map_err(|e| {
                ScaffoldError::Render(format!("content render '{}': {}", rel.display(), e))
            })?;

        if dry_run {
            println!("[dry-run] write   {} ({} bytes)", dest.display(), rendered.len());
        } else {
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| ScaffoldError::Io(format!("create_dir_all: {}", e)))?;
            }
            fs::write(&dest, rendered)
                .map_err(|e| ScaffoldError::Io(format!("write {}: {}", dest.display(), e)))?;
        }
    }
    Ok(())
}

// `<basename>.hbs` から `.hbs` を除去する。サブパスは触らない。
fn strip_hbs_suffix(rel: &str) -> String {
    if let Some(stripped) = rel.strip_suffix(".hbs") {
        stripped.to_owned()
    } else {
        rel.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_hbs() {
        assert_eq!(strip_hbs_suffix("a/b.go.hbs"), "a/b.go");
        assert_eq!(strip_hbs_suffix("c.txt"), "c.txt");
    }
}

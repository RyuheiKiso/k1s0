use std::path::Path;

use toml_edit::{value, Array, DocumentMut, InlineTable, Item, Table};

use crate::error::CodegenError;

#[derive(Debug, Clone)]
pub struct Dependency {
    pub name: String,
    pub version: Option<String>,
    pub path: Option<String>,
    pub features: Vec<String>,
    pub optional: bool,
}

pub fn add_dependency(cargo_toml_path: &Path, dep: &Dependency) -> Result<bool, CodegenError> {
    let content = std::fs::read_to_string(cargo_toml_path).map_err(|e| CodegenError::Io {
        path: cargo_toml_path.to_path_buf(),
        source: e,
    })?;

    let mut doc: DocumentMut = content
        .parse()
        .map_err(|e: toml_edit::TomlError| CodegenError::CargoUpdate(e.to_string()))?;

    if !doc.contains_key("dependencies") {
        doc["dependencies"] = Item::Table(Table::new());
    }

    let deps = doc["dependencies"]
        .as_table_mut()
        .ok_or_else(|| CodegenError::CargoUpdate("[dependencies] is not a table".into()))?;

    if deps.contains_key(&dep.name) {
        return Ok(false);
    }

    let needs_table = dep.path.is_some() || !dep.features.is_empty() || dep.optional;

    if needs_table {
        let mut inline = InlineTable::new();
        if let Some(ref ver) = dep.version {
            inline.insert("version", ver.as_str().into());
        }
        if let Some(ref path) = dep.path {
            inline.insert("path", path.as_str().into());
        }
        if !dep.features.is_empty() {
            let mut arr = Array::new();
            for f in &dep.features {
                arr.push(f.as_str());
            }
            inline.insert("features", toml_edit::Value::Array(arr));
        }
        if dep.optional {
            inline.insert("optional", true.into());
        }
        deps.insert(
            &dep.name,
            toml_edit::Item::Value(toml_edit::Value::InlineTable(inline)),
        );
    } else if let Some(ref ver) = dep.version {
        deps.insert(&dep.name, value(ver.as_str()));
    } else {
        return Err(CodegenError::CargoUpdate(
            "dependency must have version or path".into(),
        ));
    }

    std::fs::write(cargo_toml_path, doc.to_string()).map_err(|e| CodegenError::Io {
        path: cargo_toml_path.to_path_buf(),
        source: e,
    })?;

    Ok(true)
}

pub fn add_feature(
    cargo_toml_path: &Path,
    feature_name: &str,
    deps: &[&str],
) -> Result<bool, CodegenError> {
    let content = std::fs::read_to_string(cargo_toml_path).map_err(|e| CodegenError::Io {
        path: cargo_toml_path.to_path_buf(),
        source: e,
    })?;

    let mut doc: DocumentMut = content
        .parse()
        .map_err(|e: toml_edit::TomlError| CodegenError::CargoUpdate(e.to_string()))?;

    if !doc.contains_key("features") {
        doc["features"] = Item::Table(Table::new());
    }

    let features = doc["features"]
        .as_table_mut()
        .ok_or_else(|| CodegenError::CargoUpdate("[features] is not a table".into()))?;

    if features.contains_key(feature_name) {
        return Ok(false);
    }

    let mut arr = Array::new();
    for d in deps {
        arr.push(*d);
    }
    features.insert(feature_name, value(arr));

    std::fs::write(cargo_toml_path, doc.to_string()).map_err(|e| CodegenError::Io {
        path: cargo_toml_path.to_path_buf(),
        source: e,
    })?;

    Ok(true)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn write_temp_cargo(dir: &Path, content: &str) -> std::path::PathBuf {
        let path = dir.join("Cargo.toml");
        std::fs::write(&path, content).unwrap();
        path
    }

    // シンプルな依存クレートを Cargo.toml に追加できることを確認する。
    #[test]
    fn add_simple_dependency() {
        let tmp = tempfile::tempdir().unwrap();
        let cargo = write_temp_cargo(tmp.path(), "[package]\nname = \"test\"\n\n[dependencies]\n");
        let dep = Dependency {
            name: "serde".into(),
            version: Some("1".into()),
            path: None,
            features: vec![],
            optional: false,
        };
        let changed = add_dependency(&cargo, &dep).unwrap();
        assert!(changed);
        let content = std::fs::read_to_string(&cargo).unwrap();
        assert!(content.contains("serde"));
    }

    // 既存の依存クレートを追加しようとした場合にスキップされることを確認する。
    #[test]
    fn skip_existing_dependency() {
        let tmp = tempfile::tempdir().unwrap();
        let cargo = write_temp_cargo(
            tmp.path(),
            "[package]\nname = \"test\"\n\n[dependencies]\nserde = \"1\"\n",
        );
        let dep = Dependency {
            name: "serde".into(),
            version: Some("2".into()),
            path: None,
            features: vec![],
            optional: false,
        };
        let changed = add_dependency(&cargo, &dep).unwrap();
        assert!(!changed);
    }

    // フィーチャー付きの依存クレートを Cargo.toml に追加できることを確認する。
    #[test]
    fn add_dependency_with_features() {
        let tmp = tempfile::tempdir().unwrap();
        let cargo = write_temp_cargo(tmp.path(), "[package]\nname = \"test\"\n\n[dependencies]\n");
        let dep = Dependency {
            name: "serde".into(),
            version: Some("1".into()),
            path: None,
            features: vec!["derive".into()],
            optional: false,
        };
        let changed = add_dependency(&cargo, &dep).unwrap();
        assert!(changed);
        let content = std::fs::read_to_string(&cargo).unwrap();
        assert!(content.contains("derive"));
    }

    // パス指定の依存クレートを Cargo.toml に追加できることを確認する。
    #[test]
    fn add_path_dependency() {
        let tmp = tempfile::tempdir().unwrap();
        let cargo = write_temp_cargo(tmp.path(), "[package]\nname = \"test\"\n\n[dependencies]\n");
        let dep = Dependency {
            name: "my-lib".into(),
            version: None,
            path: Some("../my-lib".into()),
            features: vec![],
            optional: false,
        };
        let changed = add_dependency(&cargo, &dep).unwrap();
        assert!(changed);
        let content = std::fs::read_to_string(&cargo).unwrap();
        assert!(content.contains("../my-lib"));
    }

    // 新しいフィーチャーを Cargo.toml に追加できることを確認する。
    #[test]
    fn add_feature_new() {
        let tmp = tempfile::tempdir().unwrap();
        let cargo = write_temp_cargo(tmp.path(), "[package]\nname = \"test\"\n\n[dependencies]\n");
        let changed = add_feature(&cargo, "full", &["dep:serde", "dep:tokio"]).unwrap();
        assert!(changed);
        let content = std::fs::read_to_string(&cargo).unwrap();
        assert!(content.contains("full"));
        assert!(content.contains("dep:serde"));
    }

    // 既存のフィーチャーを追加しようとした場合にスキップされることを確認する。
    #[test]
    fn skip_existing_feature() {
        let tmp = tempfile::tempdir().unwrap();
        let cargo = write_temp_cargo(
            tmp.path(),
            "[package]\nname = \"test\"\n\n[features]\nfull = []\n",
        );
        let changed = add_feature(&cargo, "full", &["dep:serde"]).unwrap();
        assert!(!changed);
    }

    // [dependencies] セクションがない場合に自動生成されることを確認する。
    #[test]
    fn add_dependency_creates_dependencies_section() {
        let tmp = tempfile::tempdir().unwrap();
        let cargo = write_temp_cargo(tmp.path(), "[package]\nname = \"test\"\n");
        let dep = Dependency {
            name: "tokio".into(),
            version: Some("1".into()),
            path: None,
            features: vec![],
            optional: false,
        };
        let changed = add_dependency(&cargo, &dep).unwrap();
        assert!(changed);
        let content = std::fs::read_to_string(&cargo).unwrap();
        assert!(content.contains("[dependencies]"));
        assert!(content.contains("tokio"));
    }
}

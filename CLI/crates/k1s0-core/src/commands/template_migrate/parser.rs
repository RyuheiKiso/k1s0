use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

use super::types::{normalize_path, MergeStrategy, TemplateCustomizations, TemplateManifest};

pub const CURRENT_TEMPLATE_VERSION: &str = "1.5.0";
pub const MANIFEST_FILE_NAME: &str = ".k1s0-template.yaml";
pub const STATE_DIR_NAME: &str = ".k1s0-template";
pub const SNAPSHOT_DIR_NAME: &str = "base";
pub const BACKUP_DIR_NAME: &str = ".k1s0-backup";

/// .k1s0-template.yaml を読み込み、マニフェストを返す。
///
/// # Errors
///
/// ファイルの読み込みまたはパースに失敗した場合にエラーを返す。
pub fn parse_manifest(path: &Path) -> Result<TemplateManifest> {
    let content = fs::read_to_string(path)?;
    match serde_yaml::from_str::<TemplateManifest>(&content) {
        Ok(manifest) => Ok(manifest),
        Err(_) => {
            let legacy: LegacyTemplateManifest = serde_yaml::from_str(&content)?;
            Ok(legacy.into_manifest(path))
        }
    }
}

/// マニフェストを保存する。
///
/// # Errors
///
/// マニフェストのシリアライズまたはファイル書き込みに失敗した場合にエラーを返す。
pub fn write_manifest(project_dir: &Path, manifest: &TemplateManifest) -> Result<()> {
    let manifest_path = manifest_path(project_dir);
    let yaml = serde_yaml::to_string(manifest)?;
    fs::write(&manifest_path, yaml)?;
    Ok(())
}

/// プロジェクト配下のテンプレート由来ファイルからチェックサムを計算する。
///
/// # Errors
///
/// ファイル読み込みに失敗した場合にエラーを返す。
pub fn compute_checksum(root: &Path, files: &[PathBuf]) -> Result<String> {
    let mut relative_paths: Vec<PathBuf> = files
        .iter()
        .map(|path| {
            path.strip_prefix(root)
                .map(Path::to_path_buf)
                .with_context(|| {
                    format!(
                        "generated file is not inside root: {} (root: {})",
                        path.display(),
                        root.display()
                    )
                })
        })
        .collect::<Result<_>>()?;
    relative_paths.sort();

    let mut hasher = Sha256::new();
    for relative in relative_paths {
        hasher.update(normalize_path(&relative).as_bytes());
        hasher.update([0]);
        hasher.update(fs::read(root.join(&relative))?);
        hasher.update([0]);
    }

    Ok(format!("sha256:{:x}", hasher.finalize()))
}

/// スナップショット用ディレクトリを返す。
pub fn snapshot_dir(project_dir: &Path, checksum: &str) -> PathBuf {
    state_dir(project_dir)
        .join(SNAPSHOT_DIR_NAME)
        .join(checksum.trim_start_matches("sha256:"))
}

/// プロジェクト配下のメタデータ以外のファイルを列挙する。
///
/// # Errors
///
/// 走査中のパス計算に失敗した場合にエラーを返す。
pub fn collect_project_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in WalkDir::new(root)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        let relative = entry.path().strip_prefix(root)?;
        if is_reserved_path(relative) {
            continue;
        }
        files.push(entry.into_path());
    }

    files.sort();
    Ok(files)
}

/// 相対パス一覧を別ディレクトリにスナップショットとして保存する。
///
/// # Errors
///
/// ディレクトリ作成またはファイルコピーに失敗した場合にエラーを返す。
pub fn write_snapshot(root: &Path, files: &[PathBuf], destination: &Path) -> Result<()> {
    if destination.exists() {
        fs::remove_dir_all(destination)?;
    }
    fs::create_dir_all(destination)?;

    for file in files {
        let relative = file
            .strip_prefix(root)
            .with_context(|| format!("file {} is not under {}", file.display(), root.display()))?;
        let target = destination.join(relative);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(file, target)?;
    }

    Ok(())
}

/// ルートディレクトリ配下の内容を完全コピーする。
///
/// # Errors
///
/// ファイルコピーに失敗した場合にエラーを返す。
pub fn copy_tree(source: &Path, destination: &Path) -> Result<()> {
    if destination.exists() {
        fs::remove_dir_all(destination)?;
    }
    fs::create_dir_all(destination)?;

    for entry in WalkDir::new(source)
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        let relative = entry.path().strip_prefix(source)?;
        if relative.as_os_str().is_empty() {
            continue;
        }

        let target = destination.join(relative);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target)?;
            continue;
        }

        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(entry.path(), &target)?;
    }

    Ok(())
}

/// バックアップ・マニフェスト更新時に除外する予約パスか判定する。
pub fn is_reserved_path(path: &Path) -> bool {
    let normalized = normalize_path(path);
    normalized == MANIFEST_FILE_NAME
        || normalized.starts_with(&format!("{STATE_DIR_NAME}/"))
        || normalized.starts_with(&format!("{BACKUP_DIR_NAME}/"))
}

/// マニフェストのファイルパスを返す。
pub fn manifest_path(project_dir: &Path) -> PathBuf {
    project_dir.join(MANIFEST_FILE_NAME)
}

/// テンプレート状態ディレクトリを返す。
pub fn state_dir(project_dir: &Path) -> PathBuf {
    project_dir.join(STATE_DIR_NAME)
}

#[derive(Debug, Deserialize)]
struct LegacyTemplateManifest {
    template_type: String,
    language: String,
    version: String,
    checksum: String,
    #[serde(default)]
    customization: LegacyTemplateCustomization,
}

#[derive(Debug, Default, Deserialize)]
struct LegacyTemplateCustomization {
    #[serde(default)]
    ignore_paths: Vec<String>,
    #[serde(default)]
    merge_strategy: std::collections::BTreeMap<String, MergeStrategy>,
}

impl LegacyTemplateManifest {
    fn into_manifest(self, path: &Path) -> TemplateManifest {
        let module_name = path
            .parent()
            .and_then(Path::file_name)
            .and_then(|value| value.to_str())
            .unwrap_or("module");

        TemplateManifest {
            api_version: "k1s0/v1".to_string(),
            kind: "TemplateInstance".to_string(),
            metadata: super::types::TemplateMetadata {
                name: format!("{module_name}-{}", self.template_type),
                generated_at: chrono::Utc::now().to_rfc3339(),
                generated_by: "legacy-import".to_string(),
            },
            spec: super::types::TemplateSpec {
                template: super::types::TemplateDescriptor {
                    template_type: self.template_type,
                    language: self.language,
                    version: self.version,
                    checksum: self.checksum,
                },
                parameters: super::types::TemplateParameters {
                    module_name: Some(module_name.to_string()),
                    ..super::types::TemplateParameters::default()
                },
                customizations: TemplateCustomizations {
                    ignore_paths: self.customization.ignore_paths,
                    merge_strategy: self.customization.merge_strategy,
                },
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn parse_legacy_manifest() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join(MANIFEST_FILE_NAME);
        fs::write(
            &path,
            r#"
template_type: server
language: rust
version: 1.2.0
checksum: sha256:abc
customization:
  ignore_paths:
    - src/domain/**
"#,
        )
        .unwrap();

        let manifest = parse_manifest(&path).unwrap();
        assert_eq!(manifest.template_type(), "server");
        assert_eq!(manifest.language(), "rust");
        assert_eq!(manifest.version(), "1.2.0");
        assert_eq!(
            manifest.spec.customizations.ignore_paths,
            vec!["src/domain/**"]
        );
    }

    #[test]
    fn compute_checksum_is_stable() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("main.rs"), "fn main() {}\n").unwrap();
        fs::write(
            temp.path().join("Cargo.toml"),
            "[package]\nname = \"demo\"\n",
        )
        .unwrap();

        let files = collect_project_files(temp.path()).unwrap();
        let first = compute_checksum(temp.path(), &files).unwrap();
        let second = compute_checksum(temp.path(), &files).unwrap();

        assert_eq!(first, second);
        assert!(first.starts_with("sha256:"));
    }

    #[test]
    fn collect_project_files_excludes_reserved_paths() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join(MANIFEST_FILE_NAME), "manifest").unwrap();
        fs::create_dir_all(temp.path().join(STATE_DIR_NAME).join("base")).unwrap();
        fs::write(temp.path().join(STATE_DIR_NAME).join("base").join("x"), "x").unwrap();
        fs::create_dir_all(temp.path().join("src")).unwrap();
        fs::write(temp.path().join("src").join("main.rs"), "fn main() {}\n").unwrap();

        let files = collect_project_files(temp.path()).unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("src\\main.rs") || files[0].ends_with("src/main.rs"));
    }

    #[test]
    fn write_snapshot_copies_relative_files() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("main.rs"), "fn main() {}\n").unwrap();
        let files = collect_project_files(temp.path()).unwrap();
        let destination = temp.path().join("snapshot");

        write_snapshot(temp.path(), &files, &destination).unwrap();

        assert!(destination.join("src/main.rs").is_file());
    }
}

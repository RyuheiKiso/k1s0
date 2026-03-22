use anyhow::Result;
use std::path::Path;
use walkdir::WalkDir;

use super::parser::{parse_manifest, CURRENT_TEMPLATE_VERSION, MANIFEST_FILE_NAME};
use super::types::MigrationTarget;

/// ルートディレクトリ以下の .k1s0-template.yaml を走査し、
/// マイグレーション対象の一覧を返す。
///
/// # Errors
///
/// マニフェストの読み込みに失敗した場合にエラーを返す。
pub fn scan_targets(root: &Path) -> Result<Vec<MigrationTarget>> {
    let mut targets = Vec::new();

    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
    {
        if entry.file_name() != MANIFEST_FILE_NAME {
            continue;
        }

        let manifest = parse_manifest(entry.path())?;
        let project_dir = entry.path().parent().unwrap_or(root).to_path_buf();
        targets.push(MigrationTarget {
            path: project_dir,
            manifest,
            available_version: CURRENT_TEMPLATE_VERSION.to_string(),
        });
    }

    targets.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(targets)
}

// テストコードでは unwrap() の使用を許可する
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn scan_targets_returns_manifest_targets() {
        let temp = TempDir::new().unwrap();
        let project = temp.path().join("regions/service/task/server/rust");
        fs::create_dir_all(&project).unwrap();
        fs::write(
            project.join(MANIFEST_FILE_NAME),
            r#"
apiVersion: k1s0/v1
kind: TemplateInstance
metadata:
  name: task-server
  generatedAt: "2026-03-12T00:00:00Z"
  generatedBy: k1s0-cli@0.1.0
spec:
  template:
    type: server
    language: rust
    version: "1.2.0"
    checksum: sha256:abc
  parameters:
    tier: service
    placement: task
    serviceName: task
    moduleName: task
    apiStyles: [rest]
"#,
        )
        .unwrap();

        let targets = scan_targets(temp.path()).unwrap();

        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].path, project);
        assert_eq!(targets[0].manifest.version(), "1.2.0");
        assert_eq!(targets[0].available_version, CURRENT_TEMPLATE_VERSION);
    }
}

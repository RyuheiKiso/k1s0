use anyhow::{anyhow, Result};
use chrono::Utc;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

use super::parser::{
    collect_project_files, compute_checksum, snapshot_dir, write_manifest, write_snapshot,
    BACKUP_DIR_NAME,
};
use super::planner::render_latest_template_tree;
use super::types::{ChangeType, MergeResult, MigrationPlan};

/// マイグレーション計画を実行する。
///
/// 1. フルバックアップを作成
/// 2. 変更を適用
/// 3. 最新テンプレートのスナップショットを保存
/// 4. マニフェストのバージョンとチェックサムを更新
///
/// # Errors
///
/// ファイル操作またはテンプレート再レンダリングに失敗した場合にエラーを返す。
pub fn execute_migration(plan: &MigrationPlan) -> Result<()> {
    if plan.has_conflicts() {
        return Err(anyhow!(
            "cannot execute template migration while unresolved conflicts remain"
        ));
    }

    let backup_dir = plan
        .target
        .path
        .join(BACKUP_DIR_NAME)
        .join(Utc::now().format("%Y%m%d_%H%M%S").to_string());

    create_full_backup(&plan.target.path, &backup_dir)?;

    for change in &plan.changes {
        apply_change(&plan.target.path, change)?;
    }

    let rendered = render_latest_template_tree(&plan.target)?;
    let rendered_files = collect_project_files(&rendered.output_path)?;
    let checksum = compute_checksum(&rendered.output_path, &rendered_files)?;
    let snapshot_path = snapshot_dir(&plan.target.path, &checksum);
    write_snapshot(&rendered.output_path, &rendered_files, &snapshot_path)?;

    let mut manifest = plan.target.manifest.clone();
    manifest.update_template_state(&plan.target.available_version, &checksum);
    write_manifest(&plan.target.path, &manifest)?;

    Ok(())
}

fn create_full_backup(project_dir: &Path, backup_dir: &Path) -> Result<()> {
    fs::create_dir_all(backup_dir)?;
    let destination = backup_dir.join("project");
    if destination.exists() {
        fs::remove_dir_all(&destination)?;
    }
    fs::create_dir_all(&destination)?;

    for entry in WalkDir::new(project_dir)
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        let relative = entry.path().strip_prefix(project_dir)?;
        if relative.as_os_str().is_empty() || relative.starts_with(BACKUP_DIR_NAME) {
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
        fs::copy(entry.path(), target)?;
    }

    Ok(())
}

fn apply_change(project_dir: &Path, change: &super::types::FileChange) -> Result<()> {
    let target = project_dir.join(&change.path);
    match change.change_type {
        ChangeType::Added | ChangeType::Modified => {
            if let MergeResult::Clean(content) = &change.merge_result {
                if let Some(parent) = target.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(target, content)?;
            }
        }
        ChangeType::Deleted => {
            if target.exists() {
                fs::remove_file(target)?;
            }
        }
        ChangeType::Skipped => {}
    }
    Ok(())
}

/// バックアップの実体ディレクトリを返す。
pub fn backup_project_dir(backup_dir: &Path) -> std::path::PathBuf {
    backup_dir.join("project")
}

// テストコードでは unwrap() の使用を許可する
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::commands::generate::paths::build_output_path;
    use crate::commands::generate::template::{render_scaffold_preview, resolve_template_dir};
    use crate::commands::generate::types::{
        ApiStyle, DetailConfig, GenerateConfig, Kind, LangFw, Language, Tier,
    };
    use crate::commands::template_migrate::types::{FileChange, MergeStrategy};
    use crate::commands::template_migrate::{parser, planner, rollback, scanner};
    use crate::config::CliConfig;
    use std::path::PathBuf;
    use tempfile::{Builder, TempDir};

    fn replace_line_starting_with(content: &str, prefix: &str, replacement: &str) -> String {
        let mut found = false;
        let updated = content
            .split_inclusive('\n')
            .map(|line| {
                let body = line.strip_suffix('\n').unwrap_or(line);
                if body.starts_with(prefix) {
                    found = true;
                    if line.ends_with('\n') {
                        format!("{replacement}\n")
                    } else {
                        replacement.to_string()
                    }
                } else {
                    line.to_string()
                }
            })
            .collect::<String>();
        assert!(found, "line starting with {prefix} was not found");
        updated
    }

    #[test]
    fn backup_project_dir_points_to_project_snapshot() {
        let backup = PathBuf::from(".k1s0-backup/20260312_120000");
        assert_eq!(backup_project_dir(&backup), backup.join("project"));
    }

    #[test]
    fn apply_change_writes_clean_content() {
        let temp = TempDir::new().unwrap();
        let change = FileChange {
            path: PathBuf::from("src/main.rs"),
            change_type: ChangeType::Added,
            merge_strategy: MergeStrategy::Template,
            merge_result: MergeResult::Clean("fn main() {}\n".to_string()),
        };

        apply_change(temp.path(), &change).unwrap();

        assert_eq!(
            fs::read_to_string(temp.path().join("src/main.rs")).unwrap(),
            "fn main() {}\n"
        );
    }

    #[test]
    fn execute_and_rollback_template_migration_end_to_end() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .unwrap();
        let temp = Builder::new()
            .prefix("template-migrate-e2e-")
            .tempdir_in(repo_root)
            .unwrap();
        fs::create_dir_all(temp.path().join("infra/helm/services")).unwrap();
        fs::create_dir_all(temp.path().join("regions")).unwrap();

        let config = GenerateConfig {
            kind: Kind::Server,
            tier: Tier::Service,
            placement: Some("order".to_string()),
            lang_fw: LangFw::Language(Language::Rust),
            detail: DetailConfig {
                name: Some("order".to_string()),
                api_styles: vec![ApiStyle::Rest],
                db: None,
                kafka: false,
                redis: false,
                bff_language: None,
            },
        };

        let template_dir = resolve_template_dir(temp.path());
        let project_dir =
            render_scaffold_preview(&config, temp.path(), &CliConfig::default(), &template_dir)
                .unwrap();
        let generated_files = parser::collect_project_files(&project_dir).unwrap();
        let checksum = parser::compute_checksum(&project_dir, &generated_files).unwrap();
        let manifest =
            crate::commands::template_migrate::types::TemplateManifest::from_generate_config(
                &config,
                &CliConfig::default(),
                parser::CURRENT_TEMPLATE_VERSION,
                &checksum,
            );
        parser::write_snapshot(
            &project_dir,
            &generated_files,
            &parser::snapshot_dir(&project_dir, &checksum),
        )
        .unwrap();
        parser::write_manifest(&project_dir, &manifest).unwrap();

        let project_dir = build_output_path(&config, temp.path());
        let mut manifest = parser::parse_manifest(&parser::manifest_path(&project_dir)).unwrap();
        let current_snapshot = parser::snapshot_dir(&project_dir, manifest.checksum());

        let legacy_snapshot = temp.path().join("legacy-snapshot");
        parser::copy_tree(&current_snapshot, &legacy_snapshot).unwrap();

        let cargo_toml_snapshot = legacy_snapshot.join("Cargo.toml");
        let snapshot_content = fs::read_to_string(&cargo_toml_snapshot).unwrap();
        fs::write(
            &cargo_toml_snapshot,
            snapshot_content.replace("name = \"order\"", "name = \"legacy-order\""),
        )
        .unwrap();

        let legacy_files = parser::collect_project_files(&legacy_snapshot).unwrap();
        let legacy_checksum = parser::compute_checksum(&legacy_snapshot, &legacy_files).unwrap();
        let legacy_snapshot_dir = parser::snapshot_dir(&project_dir, &legacy_checksum);
        parser::write_snapshot(&legacy_snapshot, &legacy_files, &legacy_snapshot_dir).unwrap();
        manifest.update_template_state("1.2.0", &legacy_checksum);
        parser::write_manifest(&project_dir, &manifest).unwrap();

        let cargo_toml_project = project_dir.join("Cargo.toml");
        let project_content = fs::read_to_string(&cargo_toml_project).unwrap();
        fs::write(
            &cargo_toml_project,
            project_content
                .replace("name = \"order\"", "name = \"user-order\"")
                .replace("tower = \"0.5\"", "tower = \"0.4\""),
        )
        .unwrap();

        let target = scanner::scan_targets(temp.path()).unwrap().pop().unwrap();
        let mut plan = planner::build_plan(&target).unwrap();
        assert!(plan.has_conflicts());

        for change in &mut plan.changes {
            if let MergeResult::Conflict(hunks) = &change.merge_result {
                change.merge_result = MergeResult::Clean(hunks[0].theirs.clone());
            }
        }

        execute_migration(&plan).unwrap();

        let updated_manifest =
            parser::parse_manifest(&parser::manifest_path(&project_dir)).unwrap();
        assert_eq!(updated_manifest.version(), plan.target.available_version);
        assert!(parser::snapshot_dir(&project_dir, updated_manifest.checksum()).exists());

        let backups = rollback::list_backups(&project_dir).unwrap();
        assert_eq!(backups.len(), 1);

        rollback::rollback(
            &project_dir,
            &rollback::backup_dir(&project_dir, &backups[0]),
        )
        .unwrap();
        let rolled_back = fs::read_to_string(project_dir.join("Cargo.toml")).unwrap();
        assert!(rolled_back.contains("name = \"user-order\""));
    }

    #[test]
    fn execute_template_migration_preserves_non_overlapping_user_changes() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .unwrap();
        let temp = Builder::new()
            .prefix("template-migrate-merge-")
            .tempdir_in(repo_root)
            .unwrap();
        fs::create_dir_all(temp.path().join("infra/helm/services")).unwrap();
        fs::create_dir_all(temp.path().join("regions")).unwrap();

        let config = GenerateConfig {
            kind: Kind::Server,
            tier: Tier::Service,
            placement: Some("order".to_string()),
            lang_fw: LangFw::Language(Language::Rust),
            detail: DetailConfig {
                name: Some("order".to_string()),
                api_styles: vec![ApiStyle::Rest],
                db: None,
                kafka: false,
                redis: false,
                bff_language: None,
            },
        };

        let template_dir = resolve_template_dir(temp.path());
        let project_dir =
            render_scaffold_preview(&config, temp.path(), &CliConfig::default(), &template_dir)
                .unwrap();
        let generated_files = parser::collect_project_files(&project_dir).unwrap();
        let checksum = parser::compute_checksum(&project_dir, &generated_files).unwrap();
        let manifest =
            crate::commands::template_migrate::types::TemplateManifest::from_generate_config(
                &config,
                &CliConfig::default(),
                parser::CURRENT_TEMPLATE_VERSION,
                &checksum,
            );
        parser::write_snapshot(
            &project_dir,
            &generated_files,
            &parser::snapshot_dir(&project_dir, &checksum),
        )
        .unwrap();
        parser::write_manifest(&project_dir, &manifest).unwrap();

        let project_dir = build_output_path(&config, temp.path());
        let mut manifest = parser::parse_manifest(&parser::manifest_path(&project_dir)).unwrap();
        let current_snapshot = parser::snapshot_dir(&project_dir, manifest.checksum());

        let legacy_snapshot = temp.path().join("legacy-merge-snapshot");
        parser::copy_tree(&current_snapshot, &legacy_snapshot).unwrap();

        let cargo_toml_snapshot = legacy_snapshot.join("Cargo.toml");
        let snapshot_content = fs::read_to_string(&cargo_toml_snapshot).unwrap();
        fs::write(
            &cargo_toml_snapshot,
            replace_line_starting_with(&snapshot_content, "tower = ", "tower = \"0.4\""),
        )
        .unwrap();

        let legacy_files = parser::collect_project_files(&legacy_snapshot).unwrap();
        let legacy_checksum = parser::compute_checksum(&legacy_snapshot, &legacy_files).unwrap();
        let legacy_snapshot_dir = parser::snapshot_dir(&project_dir, &legacy_checksum);
        parser::write_snapshot(&legacy_snapshot, &legacy_files, &legacy_snapshot_dir).unwrap();
        manifest.update_template_state("1.2.0", &legacy_checksum);
        parser::write_manifest(&project_dir, &manifest).unwrap();

        let cargo_toml_project = project_dir.join("Cargo.toml");
        let project_content = fs::read_to_string(&cargo_toml_project).unwrap();
        fs::write(
            &cargo_toml_project,
            replace_line_starting_with(
                &replace_line_starting_with(&project_content, "name = ", "name = \"user-order\""),
                "tower = ",
                "tower = \"0.4\"",
            ),
        )
        .unwrap();

        let target = scanner::scan_targets(temp.path()).unwrap().pop().unwrap();
        let plan = planner::build_plan(&target).unwrap();
        assert!(!plan.has_conflicts());

        let cargo_change = plan
            .changes
            .iter()
            .find(|change| change.path == Path::new("Cargo.toml"))
            .unwrap();
        match &cargo_change.merge_result {
            MergeResult::Clean(content) => {
                assert!(content.contains("name = \"user-order\""));
                assert!(content.contains("tower = \"0.5\""));
            }
            other => panic!("expected clean merge result, got {other:?}"),
        }

        execute_migration(&plan).unwrap();

        let migrated = fs::read_to_string(project_dir.join("Cargo.toml")).unwrap();
        assert!(migrated.contains("name = \"user-order\""));
        assert!(migrated.contains("tower = \"0.5\""));
        assert!(!migrated.contains("tower = \"0.4\""));
    }
}

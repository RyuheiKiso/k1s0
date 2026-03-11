use anyhow::Result;
use chrono::Utc;
use std::path::Path;

use super::types::{ChangeType, FileChange, MergeResult, MigrationPlan};

/// マイグレーション計画を実行する。
///
/// 1. バックアップを作成
/// 2. 変更を適用
/// 3. マニフェストのバージョンを更新
///
/// # Errors
///
/// ファイル操作に失敗した場合にエラーを返す。
pub fn execute_migration(plan: &MigrationPlan) -> Result<()> {
    let backup_dir = plan
        .target
        .path
        .join(".k1s0-backup")
        .join(Utc::now().format("%Y%m%d_%H%M%S").to_string());

    // バックアップ作成
    create_backup(&plan.target.path, &backup_dir, &plan.changes)?;

    // 変更適用
    for change in &plan.changes {
        apply_change(&plan.target.path, change)?;
    }

    // マニフェストのバージョン更新
    update_manifest(&plan.target.path, &plan.target.available_version)?;

    Ok(())
}

/// 変更対象ファイルのバックアップを作成する。
fn create_backup(project_dir: &Path, backup_dir: &Path, changes: &[FileChange]) -> Result<()> {
    std::fs::create_dir_all(backup_dir)?;
    for change in changes {
        let src = project_dir.join(&change.path);
        if src.exists() {
            let dst = backup_dir.join(&change.path);
            if let Some(parent) = dst.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(&src, &dst)?;
        }
    }
    Ok(())
}

/// 個々のファイル変更を適用する。
fn apply_change(project_dir: &Path, change: &FileChange) -> Result<()> {
    let target = project_dir.join(&change.path);
    match &change.change_type {
        ChangeType::Added | ChangeType::Modified => {
            if let MergeResult::Clean(content) = &change.merge_result {
                if let Some(parent) = target.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&target, content)?;
            }
        }
        ChangeType::Deleted => {
            if target.exists() {
                std::fs::remove_file(&target)?;
            }
        }
    }
    Ok(())
}

/// マニフェストのバージョンを更新する。
fn update_manifest(project_dir: &Path, new_version: &str) -> Result<()> {
    let manifest_path = project_dir.join(".k1s0-template.yaml");
    let content = std::fs::read_to_string(&manifest_path)?;
    let mut manifest: super::types::TemplateManifest = serde_yaml::from_str(&content)?;
    manifest.version = new_version.to_string();
    let yaml = serde_yaml::to_string(&manifest)?;
    std::fs::write(&manifest_path, yaml)?;
    Ok(())
}

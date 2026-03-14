use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use regex::Regex;
use tempfile::TempDir;

use crate::commands::generate::template::{render_scaffold_preview, resolve_template_dir};

use super::differ::three_way_merge;
use super::parser::{collect_project_files, compute_checksum, snapshot_dir};
use super::types::{
    normalize_path, ChangeType, FileChange, MergeResult, MergeStrategy, MigrationPlan,
    MigrationTarget, TemplateCustomizations,
};

/// 最新テンプレートを再レンダリングした結果。
pub struct RenderedTemplateTree {
    _temp_dir: TempDir,
    pub output_path: PathBuf,
    pub checksum: String,
}

/// マイグレーション計画を構築する。
///
/// # Errors
///
/// ベーススナップショットが存在しない場合、またはテンプレート再レンダリングに失敗した場合にエラーを返す。
pub fn build_plan(target: &MigrationTarget) -> Result<MigrationPlan> {
    let base_root = snapshot_dir(&target.path, target.manifest.checksum());
    if !base_root.exists() {
        return Err(anyhow!(
            "base snapshot not found for {}. regenerate the project or restore .k1s0-template/base/{}",
            target.path.display(),
            target.manifest.checksum().trim_start_matches("sha256:")
        ));
    }

    let rendered = render_latest_template_tree(target)?;
    let base_files = collect_relative_files(&base_root)?;
    let ours_files = collect_relative_files(&target.path)?;
    let theirs_files = collect_relative_files(&rendered.output_path)?;

    let paths: BTreeSet<PathBuf> = base_files
        .into_iter()
        .chain(ours_files)
        .chain(theirs_files)
        .collect();

    let mut changes = Vec::new();
    for path in paths {
        let base = read_optional(&base_root, &path)?;
        let ours = read_optional(&target.path, &path)?;
        let theirs = read_optional(&rendered.output_path, &path)?;

        if base == theirs {
            continue;
        }

        let merge_strategy = resolve_merge_strategy(&target.manifest.spec.customizations, &path);
        if path_is_ignored(&target.manifest.spec.customizations, &path) {
            changes.push(FileChange {
                path,
                change_type: ChangeType::Skipped,
                merge_strategy,
                merge_result: MergeResult::NoChange,
            });
            continue;
        }

        if let Some(change) = plan_change(
            path,
            base.as_deref(),
            ours.as_deref(),
            theirs.as_deref(),
            merge_strategy,
        ) {
            changes.push(change);
        }
    }

    Ok(MigrationPlan {
        target: target.clone(),
        changes,
    })
}

/// 最新テンプレートを一時ディレクトリへレンダリングする。
///
/// # Errors
///
/// テンプレート再レンダリングに失敗した場合にエラーを返す。
pub fn render_latest_template_tree(target: &MigrationTarget) -> Result<RenderedTemplateTree> {
    let template_root = find_template_workspace_root(&target.path).ok_or_else(|| {
        anyhow!(
            "template directory was not found from {}",
            target.path.display()
        )
    })?;
    let template_dir = resolve_template_dir(&template_root);
    if !template_dir.exists() {
        return Err(anyhow!(
            "template directory does not exist: {}",
            template_dir.display()
        ));
    }

    let (config, cli_config) = target.manifest.to_generate_context()?;
    let temp_dir = TempDir::new()?;
    let output_path =
        render_scaffold_preview(&config, temp_dir.path(), &cli_config, &template_dir)?;
    let files = collect_project_files(&output_path)?;
    let checksum = compute_checksum(&output_path, &files)?;

    Ok(RenderedTemplateTree {
        _temp_dir: temp_dir,
        output_path,
        checksum,
    })
}

fn collect_relative_files(root: &Path) -> Result<Vec<PathBuf>> {
    collect_project_files(root)?
        .into_iter()
        .map(|path| {
            path.strip_prefix(root)
                .map(Path::to_path_buf)
                .with_context(|| {
                    format!(
                        "failed to strip prefix {} from {}",
                        root.display(),
                        path.display()
                    )
                })
        })
        .collect()
}

fn read_optional(root: &Path, relative: &Path) -> Result<Option<String>> {
    let target = root.join(relative);
    if !target.exists() {
        return Ok(None);
    }
    let bytes = fs::read(&target)?;
    String::from_utf8(bytes).map(Some).map_err(|_| {
        anyhow!(
            "non-utf8 file is not supported for template migration: {}",
            target.display()
        )
    })
}

fn plan_change(
    path: PathBuf,
    base: Option<&str>,
    ours: Option<&str>,
    theirs: Option<&str>,
    merge_strategy: MergeStrategy,
) -> Option<FileChange> {
    let desired_exists = theirs.is_some();
    let current_exists = ours.is_some();

    if !desired_exists && !current_exists {
        return None;
    }

    let change_type = match (current_exists, desired_exists) {
        (false, true) => ChangeType::Added,
        (true, false) => ChangeType::Deleted,
        _ => ChangeType::Modified,
    };

    let merge_result = match merge_strategy {
        MergeStrategy::User => MergeResult::NoChange,
        MergeStrategy::Template => {
            if !desired_exists {
                MergeResult::Clean(String::new())
            } else if ours == theirs {
                MergeResult::NoChange
            } else {
                MergeResult::Clean(theirs.unwrap_or_default().to_string())
            }
        }
        MergeStrategy::Merge | MergeStrategy::Ask => {
            let merged = three_way_merge(
                base.unwrap_or_default(),
                ours.unwrap_or_default(),
                theirs.unwrap_or_default(),
            );
            if desired_exists {
                merged
            } else {
                match merged {
                    MergeResult::NoChange => MergeResult::NoChange,
                    MergeResult::Clean(content) if content.is_empty() => {
                        MergeResult::Clean(content)
                    }
                    other => other,
                }
            }
        }
    };

    if is_effectively_no_change(&merge_result, ours, desired_exists) {
        return None;
    }

    Some(FileChange {
        path,
        change_type,
        merge_strategy,
        merge_result,
    })
}

fn is_effectively_no_change(
    merge_result: &MergeResult,
    ours: Option<&str>,
    desired_exists: bool,
) -> bool {
    match merge_result {
        MergeResult::NoChange => true,
        MergeResult::Clean(content) => {
            if !desired_exists {
                ours.is_none()
            } else {
                ours == Some(content.as_str())
            }
        }
        MergeResult::Conflict(_) => false,
    }
}

fn find_template_workspace_root(project_dir: &Path) -> Option<PathBuf> {
    project_dir.ancestors().find_map(|ancestor| {
        let candidate = resolve_template_dir(ancestor);
        if candidate.exists() {
            Some(ancestor.to_path_buf())
        } else {
            None
        }
    })
}

fn path_is_ignored(customizations: &TemplateCustomizations, path: &Path) -> bool {
    let normalized = normalize_path(path);
    customizations
        .ignore_paths
        .iter()
        .any(|pattern| pattern_matches(pattern, &normalized))
}

fn resolve_merge_strategy(customizations: &TemplateCustomizations, path: &Path) -> MergeStrategy {
    let normalized = normalize_path(path);
    customizations
        .merge_strategy
        .iter()
        .filter(|(pattern, _)| pattern_matches(pattern, &normalized))
        .max_by_key(|(pattern, _)| (usize::from(pattern.as_str() == normalized), pattern.len()))
        .map(|(_, strategy)| strategy.clone())
        .unwrap_or(MergeStrategy::Ask)
}

fn pattern_matches(pattern: &str, normalized_path: &str) -> bool {
    if pattern == normalized_path {
        return true;
    }

    glob_regex(pattern)
        .map(|regex| regex.is_match(normalized_path))
        .unwrap_or(false)
}

fn glob_regex(pattern: &str) -> Result<Regex> {
    let mut regex = String::from("^");
    let mut chars = pattern.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '*' => {
                if chars.peek() == Some(&'*') {
                    chars.next();
                    regex.push_str(".*");
                } else {
                    regex.push_str("[^/]*");
                }
            }
            '?' => regex.push_str("[^/]"),
            '/' => regex.push('/'),
            _ => regex.push_str(&regex::escape(&ch.to_string())),
        }
    }

    regex.push('$');
    Regex::new(&regex).map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn pattern_matches_supports_double_star() {
        assert!(pattern_matches("src/domain/**", "src/domain/model/user.rs"));
        assert!(pattern_matches("Cargo.toml", "Cargo.toml"));
        assert!(!pattern_matches("src/domain/**", "src/adapter/http.rs"));
    }

    #[test]
    fn plan_change_prefers_template_for_new_file() {
        let change = plan_change(
            PathBuf::from("src/main.rs"),
            None,
            None,
            Some("fn main() {}\n"),
            MergeStrategy::Template,
        )
        .unwrap();

        assert_eq!(change.change_type, ChangeType::Added);
        assert!(matches!(change.merge_result, MergeResult::Clean(_)));
    }

    #[test]
    fn plan_change_marks_conflict_on_parallel_updates() {
        let change = plan_change(
            PathBuf::from("Cargo.toml"),
            Some("[package]\nname = \"demo\"\n"),
            Some("[package]\nname = \"demo-user\"\n"),
            Some("[package]\nname = \"demo-template\"\n"),
            MergeStrategy::Ask,
        )
        .unwrap();

        assert!(matches!(change.merge_result, MergeResult::Conflict(_)));
    }

    #[test]
    fn resolve_merge_strategy_prefers_exact_match() {
        let customizations = TemplateCustomizations {
            ignore_paths: Vec::new(),
            merge_strategy: BTreeMap::from([
                ("src/**".to_string(), MergeStrategy::User),
                ("src/main.rs".to_string(), MergeStrategy::Template),
            ]),
        };

        let strategy = resolve_merge_strategy(&customizations, Path::new("src/main.rs"));
        assert_eq!(strategy, MergeStrategy::Template);
    }
}

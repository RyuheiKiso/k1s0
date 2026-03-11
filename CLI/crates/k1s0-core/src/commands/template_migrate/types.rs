use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// テンプレートマニフェスト (.k1s0-template.yaml)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateManifest {
    pub template_type: String,
    pub language: String,
    pub version: String,
    pub checksum: String,
    #[serde(default)]
    pub customization: CustomizationRules,
}

/// カスタマイズルール。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CustomizationRules {
    #[serde(default)]
    pub ignore_paths: Vec<String>,
    #[serde(default)]
    pub merge_strategy: HashMap<String, MergeStrategy>,
}

/// マージ戦略。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MergeStrategy {
    Merge,
    Template,
    User,
    Ask,
}

impl Default for MergeStrategy {
    fn default() -> Self {
        Self::Merge
    }
}

/// マイグレーション対象。
#[derive(Debug, Clone)]
pub struct MigrationTarget {
    pub path: PathBuf,
    pub manifest: TemplateManifest,
    pub available_version: String,
}

/// マイグレーション計画。
#[derive(Debug, Clone)]
pub struct MigrationPlan {
    pub target: MigrationTarget,
    pub changes: Vec<FileChange>,
}

impl MigrationPlan {
    /// コンフリクトを含むか判定する。
    pub fn has_conflicts(&self) -> bool {
        self.changes
            .iter()
            .any(|c| matches!(c.merge_result, MergeResult::Conflict(_)))
    }
}

/// ファイル変更。
#[derive(Debug, Clone)]
pub struct FileChange {
    pub path: PathBuf,
    pub change_type: ChangeType,
    pub merge_result: MergeResult,
}

/// 変更種別。
#[derive(Debug, Clone)]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
}

/// マージ結果。
#[derive(Debug, Clone)]
pub enum MergeResult {
    Clean(String),
    Conflict(Vec<ConflictHunk>),
    NoChange,
}

/// コンフリクトハンク。
#[derive(Debug, Clone)]
pub struct ConflictHunk {
    pub base: String,
    pub ours: String,
    pub theirs: String,
}

/// コンフリクト解決方針。
#[derive(Debug, Clone)]
pub enum ConflictResolution {
    UseTemplate,
    UseUser,
    Skip,
}

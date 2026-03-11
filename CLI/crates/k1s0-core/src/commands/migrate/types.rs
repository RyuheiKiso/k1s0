/// マイグレーション管理の型定義。
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// マイグレーション操作の種類。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MigrateOperation {
    /// 新規マイグレーションファイルの作成
    Create,
    /// マイグレーションの適用（up）
    Up,
    /// マイグレーションのロールバック（down）
    Down,
    /// マイグレーション状態の確認
    Status,
    /// マイグレーションの修復
    Repair,
}

impl MigrateOperation {
    /// 操作の日本語ラベルを返す。
    pub fn label(&self) -> &'static str {
        match self {
            MigrateOperation::Create => "新規作成",
            MigrateOperation::Up => "適用 (up)",
            MigrateOperation::Down => "ロールバック (down)",
            MigrateOperation::Status => "状態確認",
            MigrateOperation::Repair => "修復",
        }
    }
}

/// 全操作のラベル一覧。
pub const OPERATION_LABELS: &[&str] = &[
    "新規作成",
    "適用 (up)",
    "ロールバック (down)",
    "状態確認",
    "修復",
];

/// インデックスから操作を取得する。
pub const ALL_OPERATIONS: &[MigrateOperation] = &[
    MigrateOperation::Create,
    MigrateOperation::Up,
    MigrateOperation::Down,
    MigrateOperation::Status,
    MigrateOperation::Repair,
];

/// サーバーの言語。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    Rust,
    Go,
}

impl Language {
    /// 言語のラベルを返す。
    pub fn label(&self) -> &'static str {
        match self {
            Language::Rust => "Rust (sqlx-cli)",
            Language::Go => "Go (golang-migrate)",
        }
    }
}

/// マイグレーション対象のサービス。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrateTarget {
    /// サービス名（表示用）
    pub service_name: String,
    /// ティア（system / business / service）
    pub tier: String,
    /// 言語（Rust / Go）
    pub language: Language,
    /// migrations/ ディレクトリのパス
    pub migrations_dir: PathBuf,
    /// データベース名
    pub db_name: String,
}

impl MigrateTarget {
    /// 表示用のラベルを返す。
    pub fn display_label(&self) -> String {
        format!(
            "{} ({}/{}) [{}]",
            self.service_name,
            self.tier,
            self.language.label(),
            self.db_name
        )
    }
}

/// マイグレーションファイルの方向。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    Up,
    Down,
}

/// マイグレーションファイル。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationFile {
    /// 連番（3桁ゼロ埋め）
    pub number: u32,
    /// 説明（英小文字・数字・アンダースコア）
    pub description: String,
    /// 方向（up / down）
    pub direction: Direction,
    /// ファイルパス
    pub path: PathBuf,
}

/// マイグレーション作成設定。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrateCreateConfig {
    /// 対象サービス
    pub target: MigrateTarget,
    /// マイグレーション名
    pub migration_name: String,
}

/// マイグレーション適用範囲。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MigrateRange {
    /// すべてのマイグレーションを適用
    All,
    /// 指定バージョンまで適用
    UpTo(u32),
    /// 指定件数ぶんだけ進める/戻す
    Steps(u32),
}

/// DB接続先。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DbConnection {
    /// ローカル開発環境
    LocalDev,
    /// カスタム接続文字列
    Custom(String),
}

/// マイグレーション適用設定。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrateUpConfig {
    /// 対象サービス
    pub target: MigrateTarget,
    /// 適用範囲
    pub range: MigrateRange,
    /// DB接続先
    pub connection: DbConnection,
}

/// マイグレーションロールバック設定。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrateDownConfig {
    /// 対象サービス
    pub target: MigrateTarget,
    /// ロールバック範囲
    pub range: MigrateRange,
    /// DB接続先
    pub connection: DbConnection,
}

/// 修復操作の種類。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepairOperation {
    /// ダーティフラグのクリア
    ClearDirty,
    /// バージョンの強制設定
    ForceVersion(u32),
}

/// マイグレーション状態。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationStatus {
    /// 連番
    pub number: u32,
    /// 説明
    pub description: String,
    /// 適用済みかどうか
    pub applied: bool,
    /// 適用日時
    pub applied_at: Option<String>,
}

/// CI整合性チェック結果。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MigrateCiResult {
    /// エラー一覧
    pub errors: Vec<String>,
    /// 警告一覧
    pub warnings: Vec<String>,
}

impl MigrateCiResult {
    /// エラーがあるかどうかを返す。
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// 警告があるかどうかを返す。
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// 結果が正常（エラーも警告もない）かどうかを返す。
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty() && self.warnings.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrate_operation_labels() {
        assert_eq!(MigrateOperation::Create.label(), "新規作成");
        assert_eq!(MigrateOperation::Up.label(), "適用 (up)");
        assert_eq!(MigrateOperation::Down.label(), "ロールバック (down)");
        assert_eq!(MigrateOperation::Status.label(), "状態確認");
        assert_eq!(MigrateOperation::Repair.label(), "修復");
    }

    #[test]
    fn test_language_labels() {
        assert_eq!(Language::Rust.label(), "Rust (sqlx-cli)");
        assert_eq!(Language::Go.label(), "Go (golang-migrate)");
    }

    #[test]
    fn test_migrate_target_display_label() {
        let target = MigrateTarget {
            service_name: "auth".to_string(),
            tier: "system".to_string(),
            language: Language::Rust,
            migrations_dir: PathBuf::from("regions/system/server/rust/auth/migrations"),
            db_name: "auth_db".to_string(),
        };
        assert_eq!(
            target.display_label(),
            "auth (system/Rust (sqlx-cli)) [auth_db]"
        );
    }

    #[test]
    fn test_migrate_ci_result_default() {
        let result = MigrateCiResult::default();
        assert!(!result.has_errors());
        assert!(!result.has_warnings());
        assert!(result.is_ok());
    }

    #[test]
    fn test_migrate_ci_result_with_errors() {
        let result = MigrateCiResult {
            errors: vec!["エラー1".to_string()],
            warnings: vec![],
        };
        assert!(result.has_errors());
        assert!(!result.has_warnings());
        assert!(!result.is_ok());
    }

    #[test]
    fn test_migrate_ci_result_with_warnings() {
        let result = MigrateCiResult {
            errors: vec![],
            warnings: vec!["警告1".to_string()],
        };
        assert!(!result.has_errors());
        assert!(result.has_warnings());
        assert!(!result.is_ok());
    }

    #[test]
    fn test_operation_labels_constant() {
        assert_eq!(OPERATION_LABELS.len(), 5);
        assert_eq!(ALL_OPERATIONS.len(), 5);
    }

    #[test]
    fn test_db_connection_variants() {
        let local = DbConnection::LocalDev;
        assert_eq!(local, DbConnection::LocalDev);

        let custom = DbConnection::Custom("postgres://localhost/test".to_string());
        assert_ne!(custom, DbConnection::LocalDev);
    }

    #[test]
    fn test_migrate_range_variants() {
        let all = MigrateRange::All;
        assert_eq!(all, MigrateRange::All);

        let up_to = MigrateRange::UpTo(5);
        assert_eq!(up_to, MigrateRange::UpTo(5));
        assert_ne!(up_to, MigrateRange::All);

        let steps = MigrateRange::Steps(1);
        assert_eq!(steps, MigrateRange::Steps(1));
        assert_ne!(steps, MigrateRange::All);
    }

    #[test]
    fn test_repair_operation_variants() {
        let clear = RepairOperation::ClearDirty;
        assert_eq!(clear, RepairOperation::ClearDirty);

        let force = RepairOperation::ForceVersion(3);
        assert_eq!(force, RepairOperation::ForceVersion(3));
    }
}

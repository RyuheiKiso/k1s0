//! upgrade ロジック
//!
//! テンプレートの更新チェックと適用を提供する。
//! - Phase 32: upgrade --check (差分提示)
//! - Phase 33: upgrade (managed領域の適用)

use std::path::{Path, PathBuf};

use crate::diff::{calculate_diff_with_conflicts, format_diff, DiffKind, DiffResult, FileDiff};
use crate::fingerprint::{calculate_file_checksum, calculate_fingerprint};
use crate::manifest::Manifest;
use crate::{Error, Result};

/// バージョン変更の種類
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionChange {
    /// メジャーバージョン更新 (1.x.x -> 2.x.x)
    Major,
    /// マイナーバージョン更新 (x.1.x -> x.2.x)
    Minor,
    /// パッチバージョン更新 (x.x.1 -> x.x.2)
    Patch,
    /// 変更なし
    None,
}

impl VersionChange {
    /// バージョン文字列を比較して変更の種類を判定する
    pub fn from_versions(old: &str, new: &str) -> Self {
        let old_parts: Vec<u32> = old
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();
        let new_parts: Vec<u32> = new
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();

        let old_major = old_parts.first().copied().unwrap_or(0);
        let old_minor = old_parts.get(1).copied().unwrap_or(0);
        let old_patch = old_parts.get(2).copied().unwrap_or(0);

        let new_major = new_parts.first().copied().unwrap_or(0);
        let new_minor = new_parts.get(1).copied().unwrap_or(0);
        let new_patch = new_parts.get(2).copied().unwrap_or(0);

        if new_major > old_major {
            VersionChange::Major
        } else if new_minor > old_minor {
            VersionChange::Minor
        } else if new_patch > old_patch {
            VersionChange::Patch
        } else {
            VersionChange::None
        }
    }

    /// 表示用ラベル
    pub fn label(&self) -> &'static str {
        match self {
            VersionChange::Major => "MAJOR (破壊的変更の可能性)",
            VersionChange::Minor => "MINOR (後方互換性あり)",
            VersionChange::Patch => "PATCH (バグ修正)",
            VersionChange::None => "変更なし",
        }
    }
}

/// アップグレードチェックの結果
#[derive(Debug)]
pub struct UpgradeCheckResult {
    /// 現在のテンプレートバージョン
    pub current_version: String,
    /// 新しいテンプレートバージョン
    pub new_version: String,
    /// バージョン変更の種類
    pub version_change: VersionChange,
    /// 現在の fingerprint
    pub current_fingerprint: String,
    /// 新しい fingerprint
    pub new_fingerprint: String,
    /// 差分結果
    pub diff: DiffResult,
    /// managed 領域の差分
    pub managed_diff: DiffResult,
    /// protected 領域の差分
    pub protected_diff: DiffResult,
    /// MAJOR 変更時の ADR ファイル存在
    pub has_upgrade_adr: bool,
    /// UPGRADE.md の存在
    pub has_upgrade_md: bool,
    /// 更新が必要かどうか
    pub needs_upgrade: bool,
    /// 衝突があるかどうか
    pub has_conflicts: bool,
}

impl UpgradeCheckResult {
    /// サマリーを取得
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();

        parts.push(format!(
            "バージョン: {} -> {} ({})",
            self.current_version, self.new_version, self.version_change.label()
        ));

        if self.needs_upgrade {
            parts.push(format!("変更: {}", self.diff.summary()));
            parts.push(format!("  - managed: {}", self.managed_diff.summary()));
            parts.push(format!("  - protected: {}", self.protected_diff.summary()));
        } else {
            parts.push("変更なし（最新の状態です）".to_string());
        }

        if self.has_conflicts {
            parts.push(format!(
                "⚠️ 衝突: {} ファイル（手動解決が必要）",
                self.diff.conflicts.len()
            ));
        }

        if self.version_change == VersionChange::Major {
            if self.has_upgrade_adr {
                parts.push("✓ ADR/UPGRADE.md が存在します".to_string());
            } else {
                parts.push(
                    "⚠️ MAJOR変更ですが ADR/UPGRADE.md が見つかりません".to_string(),
                );
            }
        }

        parts.join("\n")
    }

    /// upgrade --check の出力用フォーマット
    pub fn format_check_output(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!(
            "┌─────────────────────────────────────────────────────────────────────────┐\n"
        ));
        output.push_str(&format!(
            "│ k1s0 upgrade --check                                                    │\n"
        ));
        output.push_str(&format!(
            "├─────────────────────────────────────────────────────────────────────────┤\n"
        ));
        output.push_str(&format!(
            "│ テンプレート: {} -> {}                                 \n",
            self.current_version, self.new_version
        ));
        output.push_str(&format!(
            "│ 変更種別: {}                                          \n",
            self.version_change.label()
        ));
        output.push_str(&format!(
            "└─────────────────────────────────────────────────────────────────────────┘\n"
        ));
        output.push('\n');

        if !self.needs_upgrade {
            output.push_str("✓ 最新の状態です。更新は不要です。\n");
            return output;
        }

        // Managed 領域
        if self.managed_diff.has_changes() {
            output.push_str("━━━ Managed 領域（自動更新対象）━━━\n");
            output.push_str(&format_diff(&self.managed_diff));
        }

        // Protected 領域
        if self.protected_diff.has_changes() {
            output.push_str("━━━ Protected 領域（手動更新が必要）━━━\n");
            output.push_str(&format_diff(&self.protected_diff));
        }

        // 衝突
        if self.has_conflicts {
            output.push_str("━━━ 衝突（手動解決が必要）━━━\n");
            for conflict in &self.diff.conflicts {
                output.push_str(&format!(
                    "  ! {} (expected: {}, actual: {})\n",
                    conflict.path,
                    conflict
                        .expected_checksum
                        .as_deref()
                        .unwrap_or("?"),
                    conflict.actual_checksum.as_deref().unwrap_or("?")
                ));
            }
            output.push('\n');
        }

        // MAJOR 変更の警告
        if self.version_change == VersionChange::Major {
            output.push_str("━━━ MAJOR バージョン変更 ━━━\n");
            if self.has_upgrade_adr || self.has_upgrade_md {
                output.push_str("✓ 移行ガイドが存在します:\n");
                if self.has_upgrade_adr {
                    output.push_str("  - ADR/UPGRADE.md\n");
                }
                if self.has_upgrade_md {
                    output.push_str("  - UPGRADE.md\n");
                }
            } else {
                output.push_str("⚠️ 移行ガイドが見つかりません。\n");
                output.push_str("  MAJOR バージョン変更では ADR/UPGRADE.md を確認してください。\n");
            }
            output.push('\n');
        }

        // 次のアクション
        output.push_str("━━━ 次のステップ ━━━\n");
        if self.has_conflicts {
            output.push_str(
                "1. 衝突を解決してください（手動で変更をマージするか、チェックサムを更新）\n",
            );
            output.push_str("2. k1s0 upgrade を再実行\n");
        } else {
            output.push_str("k1s0 upgrade を実行して変更を適用できます\n");
            if !self.protected_diff.has_changes() {
                output.push_str("  (--managed-only オプションで managed 領域のみ更新)\n");
            }
        }

        output
    }
}

/// アップグレード適用の結果
#[derive(Debug)]
pub struct UpgradeApplyResult {
    /// 適用されたファイル
    pub applied: Vec<String>,
    /// スキップされたファイル（protected）
    pub skipped: Vec<String>,
    /// バックアップされたファイル
    pub backed_up: Vec<String>,
    /// 衝突したファイル
    pub conflicts: Vec<String>,
}

/// アップグレードチェックを実行する
///
/// # Arguments
/// * `service_path` - サービスディレクトリへのパス
/// * `template_path` - テンプレートディレクトリへのパス（省略時は manifest から取得）
///
/// # Returns
/// * `UpgradeCheckResult` - チェック結果
pub fn check_upgrade<P: AsRef<Path>>(
    service_path: P,
    template_path: Option<&Path>,
) -> Result<UpgradeCheckResult> {
    let service_path = service_path.as_ref();
    let manifest_path = service_path.join(".k1s0/manifest.json");

    // manifest を読み込む
    let manifest = Manifest::load(&manifest_path)?;

    // テンプレートパスを決定
    let template_dir = if let Some(path) = template_path {
        path.to_path_buf()
    } else {
        // manifest からテンプレートパスを取得
        // ルートからの相対パスを解決
        let cli_root = find_cli_root(service_path)?;
        cli_root.join(&manifest.template.path)
    };

    if !template_dir.exists() {
        return Err(Error::TemplateNotFound(template_dir.display().to_string()));
    }

    // 新しい fingerprint を計算
    let new_fingerprint = calculate_fingerprint(&template_dir)?;

    // バージョン情報（現時点では fingerprint ベース）
    let current_version = manifest.template.version.clone();
    let new_version = get_template_version(&template_dir).unwrap_or_else(|_| current_version.clone());
    let version_change = VersionChange::from_versions(&current_version, &new_version);

    // fingerprint が同じなら更新不要
    if new_fingerprint == manifest.template.fingerprint {
        return Ok(UpgradeCheckResult {
            current_version,
            new_version,
            version_change: VersionChange::None,
            current_fingerprint: manifest.template.fingerprint.clone(),
            new_fingerprint,
            diff: DiffResult::default(),
            managed_diff: DiffResult::default(),
            protected_diff: DiffResult::default(),
            has_upgrade_adr: false,
            has_upgrade_md: false,
            needs_upgrade: false,
            has_conflicts: false,
        });
    }

    // checksums を (path, checksum) のベクタに変換
    let checksums: Vec<(String, String)> = manifest
        .checksums
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    // 差分を計算
    let diff = calculate_diff_with_conflicts(service_path, &template_dir, &checksums)?;

    // managed/protected に分類
    let managed_diff = filter_diff_by_paths(&diff, &manifest.managed_paths, true);
    let protected_diff = filter_diff_by_paths(&diff, &manifest.protected_paths, true);

    // ADR/UPGRADE.md の存在確認
    let has_upgrade_adr = template_dir.join("ADR/UPGRADE.md").exists()
        || template_dir.join("adr/UPGRADE.md").exists()
        || template_dir.join("docs/ADR/UPGRADE.md").exists();
    let has_upgrade_md = template_dir.join("UPGRADE.md").exists();

    Ok(UpgradeCheckResult {
        current_version,
        new_version,
        version_change,
        current_fingerprint: manifest.template.fingerprint.clone(),
        new_fingerprint,
        has_conflicts: diff.has_conflicts(),
        diff,
        managed_diff,
        protected_diff,
        has_upgrade_adr,
        has_upgrade_md,
        needs_upgrade: true,
    })
}

/// アップグレードを適用する
///
/// # Arguments
/// * `service_path` - サービスディレクトリへのパス
/// * `check_result` - upgrade --check の結果
/// * `managed_only` - managed 領域のみ更新
/// * `create_backup` - バックアップを作成
///
/// # Returns
/// * `UpgradeApplyResult` - 適用結果
pub fn apply_upgrade<P: AsRef<Path>>(
    service_path: P,
    check_result: &UpgradeCheckResult,
    managed_only: bool,
    create_backup: bool,
) -> Result<UpgradeApplyResult> {
    let service_path = service_path.as_ref();
    let manifest_path = service_path.join(".k1s0/manifest.json");

    // 衝突がある場合は適用しない
    if check_result.has_conflicts {
        return Err(Error::FileConflict(format!(
            "{} ファイルに衝突があります。手動で解決してください。",
            check_result.diff.conflicts.len()
        )));
    }

    let mut result = UpgradeApplyResult {
        applied: Vec::new(),
        skipped: Vec::new(),
        backed_up: Vec::new(),
        conflicts: Vec::new(),
    };

    let manifest = Manifest::load(&manifest_path)?;

    // テンプレートパスを決定
    let cli_root = find_cli_root(service_path)?;
    let template_dir = cli_root.join(&manifest.template.path);

    // managed 領域の変更を適用
    for diff in check_result.managed_diff.all_changes() {
        let target_path = service_path.join(&diff.path);

        // バックアップを作成
        if create_backup && target_path.exists() {
            let backup_path = target_path.with_extension("bak");
            std::fs::copy(&target_path, &backup_path)?;
            result.backed_up.push(diff.path.clone());
        }

        // 変更を適用
        match diff.kind {
            DiffKind::Added | DiffKind::Modified => {
                let source_path = template_dir.join(&diff.path);
                if let Some(parent) = target_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::copy(&source_path, &target_path)?;
                result.applied.push(diff.path.clone());
            }
            DiffKind::Removed => {
                if target_path.exists() {
                    std::fs::remove_file(&target_path)?;
                    result.applied.push(diff.path.clone());
                }
            }
            _ => {}
        }
    }

    // protected 領域は managed_only が false の場合のみ
    if !managed_only {
        for diff in check_result.protected_diff.all_changes() {
            // protected 領域は差分を提示するだけでスキップ
            result.skipped.push(diff.path.clone());
        }
    }

    // manifest を更新
    let mut updated_manifest = manifest.clone();
    updated_manifest.template.fingerprint = check_result.new_fingerprint.clone();
    updated_manifest.template.version = check_result.new_version.clone();
    updated_manifest.generated_at = chrono::Utc::now().to_rfc3339();

    // checksums を更新
    for path in &result.applied {
        let file_path = service_path.join(path);
        if file_path.exists() {
            let checksum = calculate_file_checksum(&file_path)?;
            updated_manifest.checksums.insert(path.clone(), checksum);
        } else {
            updated_manifest.checksums.remove(path);
        }
    }

    updated_manifest.save(&manifest_path)?;

    Ok(result)
}

/// CLI ルートディレクトリを見つける
fn find_cli_root(start_path: &Path) -> Result<PathBuf> {
    let mut current = start_path.to_path_buf();

    loop {
        // CLI/templates が存在するか確認
        let cli_dir = current.join("CLI");
        if cli_dir.exists() && cli_dir.join("templates").exists() {
            return Ok(current);
        }

        // k1s0 リポジトリのルートを確認
        if current.join(".git").exists() {
            if current.join("CLI/templates").exists() {
                return Ok(current);
            }
        }

        // 親ディレクトリへ
        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        } else {
            break;
        }
    }

    Err(Error::Other(
        "k1s0 CLI ルートディレクトリが見つかりません".to_string(),
    ))
}

/// テンプレートのバージョンを取得
fn get_template_version(template_dir: &Path) -> Result<String> {
    // template.yaml からバージョンを読み取る
    let template_yaml = template_dir.join("template.yaml");
    if template_yaml.exists() {
        let content = std::fs::read_to_string(&template_yaml)?;
        let yaml: serde_yaml::Value = serde_yaml::from_str(&content)?;
        if let Some(version) = yaml.get("version").and_then(|v| v.as_str()) {
            return Ok(version.to_string());
        }
    }

    // Cargo.toml からバージョンを読み取る（fallback）
    let cargo_toml = template_dir.join("Cargo.toml");
    if cargo_toml.exists() {
        let content = std::fs::read_to_string(&cargo_toml)?;
        for line in content.lines() {
            if line.starts_with("version = ") {
                let version = line
                    .trim_start_matches("version = ")
                    .trim_matches('"')
                    .trim_matches('\'');
                return Ok(version.to_string());
            }
        }
    }

    Err(Error::Other("テンプレートバージョンが見つかりません".to_string()))
}

/// DiffResult をパスでフィルタ
/// マイグレーション結果
#[derive(Debug)]
pub struct MigrationResult {
    /// 適用したマイグレーション
    pub applied: Vec<String>,
    /// スキップしたマイグレーション（既に適用済み）
    pub skipped: Vec<String>,
    /// 失敗したマイグレーション
    pub failed: Vec<(String, String)>,
    /// 環境
    pub env: String,
}

impl MigrationResult {
    /// 新しい結果を作成
    pub fn new(env: impl Into<String>) -> Self {
        Self {
            applied: Vec::new(),
            skipped: Vec::new(),
            failed: Vec::new(),
            env: env.into(),
        }
    }

    /// 成功かどうか
    pub fn is_success(&self) -> bool {
        self.failed.is_empty()
    }

    /// サマリーを取得
    pub fn summary(&self) -> String {
        format!(
            "適用: {}, スキップ: {}, 失敗: {}",
            self.applied.len(),
            self.skipped.len(),
            self.failed.len()
        )
    }
}

/// マイグレーションファイル
#[derive(Debug, Clone)]
pub struct MigrationFile {
    /// ファイル名
    pub name: String,
    /// ファイルパス
    pub path: PathBuf,
    /// バージョン番号
    pub version: u64,
    /// 方向（up/down）
    pub direction: MigrationDirection,
}

/// マイグレーション方向
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationDirection {
    Up,
    Down,
}

/// マイグレーション設定
#[derive(Debug, Clone)]
pub struct MigrationConfig {
    /// データベース接続文字列
    pub database_url: Option<String>,
    /// 環境名
    pub env: String,
    /// マイグレーションディレクトリ
    pub migrations_dir: PathBuf,
    /// ドライラン（実際には実行しない）
    pub dry_run: bool,
}

impl MigrationConfig {
    /// 環境変数から設定を読み込む
    pub fn from_env(env: &str, service_path: &Path) -> Self {
        let database_url = std::env::var("DATABASE_URL").ok()
            .or_else(|| std::env::var(&format!("DATABASE_URL_{}", env.to_uppercase())).ok());

        let migrations_dir = service_path.join("migrations");

        Self {
            database_url,
            env: env.to_string(),
            migrations_dir,
            dry_run: false,
        }
    }
}

/// マイグレーションを適用する
///
/// # Arguments
/// * `service_path` - サービスディレクトリへのパス
/// * `env` - 環境名（dev/stg/prod）
/// * `dry_run` - ドライラン（実際には実行しない）
///
/// # Returns
/// * `MigrationResult` - マイグレーション結果
pub fn apply_migrations<P: AsRef<Path>>(
    service_path: P,
    env: &str,
    dry_run: bool,
) -> Result<MigrationResult> {
    let service_path = service_path.as_ref();
    let mut result = MigrationResult::new(env);

    // dev 環境のみ許可
    if env != "dev" && env != "development" && env != "local" {
        return Err(Error::Other(format!(
            "自動マイグレーションは dev 環境でのみ実行可能です（現在: {}）",
            env
        )));
    }

    // マイグレーションディレクトリを探す
    let migrations_dir = find_migrations_dir(service_path)?;

    // マイグレーションファイルを収集
    let migrations = collect_migrations(&migrations_dir)?;

    if migrations.is_empty() {
        return Ok(result);
    }

    // 設定を取得
    let config = MigrationConfig::from_env(env, service_path);

    // データベースURLが設定されていない場合
    if config.database_url.is_none() {
        // ドライランまたはファイル一覧の表示のみ
        for migration in &migrations {
            if migration.direction == MigrationDirection::Up {
                if dry_run {
                    result.applied.push(migration.name.clone());
                } else {
                    result.skipped.push(format!(
                        "{} (DATABASE_URL が未設定)",
                        migration.name
                    ));
                }
            }
        }
        return Ok(result);
    }

    // マイグレーションを実行
    for migration in &migrations {
        if migration.direction != MigrationDirection::Up {
            continue;
        }

        if dry_run {
            result.applied.push(migration.name.clone());
            continue;
        }

        // SQLファイルを読み込む
        let sql = match std::fs::read_to_string(&migration.path) {
            Ok(content) => content,
            Err(e) => {
                result.failed.push((migration.name.clone(), e.to_string()));
                continue;
            }
        };

        // 実際のDB実行はここで行う（将来的にsqlxを使用）
        // 現時点ではファイル内容を表示するのみ
        println!("-- Migration: {}", migration.name);
        println!("-- File: {}", migration.path.display());
        println!("{}", sql);
        println!("-- End of migration\n");

        result.applied.push(migration.name.clone());
    }

    Ok(result)
}

/// マイグレーションディレクトリを探す
fn find_migrations_dir(service_path: &Path) -> Result<PathBuf> {
    // 優先順位:
    // 1. migrations/
    // 2. db/migrations/
    // 3. database/migrations/
    let candidates = [
        service_path.join("migrations"),
        service_path.join("db/migrations"),
        service_path.join("database/migrations"),
    ];

    for candidate in &candidates {
        if candidate.exists() && candidate.is_dir() {
            return Ok(candidate.clone());
        }
    }

    // migrations ディレクトリが存在しない場合は作成可能なパスを返す
    Ok(candidates[0].clone())
}

/// マイグレーションファイルを収集
fn collect_migrations(migrations_dir: &Path) -> Result<Vec<MigrationFile>> {
    if !migrations_dir.exists() {
        return Ok(Vec::new());
    }

    let mut migrations = Vec::new();

    for entry in std::fs::read_dir(migrations_dir)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let file_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name.to_string(),
            None => continue,
        };

        // ファイル名パターン: NNNN_name.up.sql または NNNN_name.down.sql
        if !file_name.ends_with(".sql") {
            continue;
        }

        let direction = if file_name.contains(".up.") {
            MigrationDirection::Up
        } else if file_name.contains(".down.") {
            MigrationDirection::Down
        } else {
            // .up/.down がない場合は Up として扱う
            MigrationDirection::Up
        };

        // バージョン番号を抽出
        let version = file_name
            .split('_')
            .next()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        migrations.push(MigrationFile {
            name: file_name,
            path,
            version,
            direction,
        });
    }

    // バージョン順にソート
    migrations.sort_by(|a, b| a.version.cmp(&b.version));

    Ok(migrations)
}

/// 保留中のマイグレーションを一覧表示
pub fn list_pending_migrations<P: AsRef<Path>>(service_path: P) -> Result<Vec<MigrationFile>> {
    let service_path = service_path.as_ref();
    let migrations_dir = find_migrations_dir(service_path)?;
    let migrations = collect_migrations(&migrations_dir)?;

    // Up マイグレーションのみをフィルタ
    Ok(migrations
        .into_iter()
        .filter(|m| m.direction == MigrationDirection::Up)
        .collect())
}

/// DiffResult をパスでフィルタ
fn filter_diff_by_paths(diff: &DiffResult, paths: &[String], include: bool) -> DiffResult {
    let filter_fn = |file_diff: &FileDiff| -> bool {
        let matches = paths.iter().any(|pattern| {
            if pattern.ends_with('/') {
                let prefix = pattern.trim_end_matches('/');
                file_diff.path.starts_with(prefix)
            } else {
                file_diff.path == *pattern
            }
        });
        if include {
            matches
        } else {
            !matches
        }
    };

    DiffResult {
        added: diff.added.iter().filter(|d| filter_fn(d)).cloned().collect(),
        removed: diff.removed.iter().filter(|d| filter_fn(d)).cloned().collect(),
        modified: diff.modified.iter().filter(|d| filter_fn(d)).cloned().collect(),
        conflicts: diff.conflicts.iter().filter(|d| filter_fn(d)).cloned().collect(),
        unchanged: diff.unchanged.iter().filter(|d| filter_fn(d)).cloned().collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_change_major() {
        assert_eq!(
            VersionChange::from_versions("1.0.0", "2.0.0"),
            VersionChange::Major
        );
        assert_eq!(
            VersionChange::from_versions("0.1.0", "1.0.0"),
            VersionChange::Major
        );
    }

    #[test]
    fn test_version_change_minor() {
        assert_eq!(
            VersionChange::from_versions("1.0.0", "1.1.0"),
            VersionChange::Minor
        );
        assert_eq!(
            VersionChange::from_versions("1.2.0", "1.3.0"),
            VersionChange::Minor
        );
    }

    #[test]
    fn test_version_change_patch() {
        assert_eq!(
            VersionChange::from_versions("1.0.0", "1.0.1"),
            VersionChange::Patch
        );
        assert_eq!(
            VersionChange::from_versions("1.2.3", "1.2.4"),
            VersionChange::Patch
        );
    }

    #[test]
    fn test_version_change_none() {
        assert_eq!(
            VersionChange::from_versions("1.0.0", "1.0.0"),
            VersionChange::None
        );
        // ダウングレードは None として扱う
        assert_eq!(
            VersionChange::from_versions("2.0.0", "1.0.0"),
            VersionChange::None
        );
    }

    #[test]
    fn test_filter_diff_by_paths() {
        let mut diff = DiffResult::default();
        diff.added.push(FileDiff::added("deploy/base/deployment.yaml", None));
        diff.added.push(FileDiff::added("src/main.rs", None));
        diff.modified.push(FileDiff::modified("deploy/overlays/dev.yaml", None, None));
        diff.modified.push(FileDiff::modified("src/lib.rs", None, None));

        let managed_paths = vec!["deploy/".to_string()];
        let filtered = filter_diff_by_paths(&diff, &managed_paths, true);

        assert_eq!(filtered.added.len(), 1);
        assert_eq!(filtered.added[0].path, "deploy/base/deployment.yaml");
        assert_eq!(filtered.modified.len(), 1);
        assert_eq!(filtered.modified[0].path, "deploy/overlays/dev.yaml");
    }
}

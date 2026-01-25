//! lint ルールと検査
//!
//! k1s0 規約に対する検査を提供する。
//!
//! # ルール ID
//!
//! - `K001`: manifest.json が存在しない
//! - `K002`: manifest.json の必須キーが不足
//! - `K003`: manifest.json の値が不正
//! - `K010`: 必須ディレクトリが存在しない
//! - `K011`: 必須ファイルが存在しない

use std::path::{Path, PathBuf};

use crate::manifest::Manifest;

/// ルール ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuleId {
    /// manifest.json が存在しない
    ManifestNotFound,
    /// manifest.json の必須キーが不足
    ManifestMissingKey,
    /// manifest.json の値が不正
    ManifestInvalidValue,
    /// 必須ディレクトリが存在しない
    RequiredDirMissing,
    /// 必須ファイルが存在しない
    RequiredFileMissing,
}

impl RuleId {
    /// ルール ID を文字列として取得
    pub fn as_str(&self) -> &'static str {
        match self {
            RuleId::ManifestNotFound => "K001",
            RuleId::ManifestMissingKey => "K002",
            RuleId::ManifestInvalidValue => "K003",
            RuleId::RequiredDirMissing => "K010",
            RuleId::RequiredFileMissing => "K011",
        }
    }

    /// ルールの説明
    pub fn description(&self) -> &'static str {
        match self {
            RuleId::ManifestNotFound => "manifest.json が存在しません",
            RuleId::ManifestMissingKey => "manifest.json の必須キーが不足しています",
            RuleId::ManifestInvalidValue => "manifest.json の値が不正です",
            RuleId::RequiredDirMissing => "必須ディレクトリが存在しません",
            RuleId::RequiredFileMissing => "必須ファイルが存在しません",
        }
    }
}

impl std::fmt::Display for RuleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// 重要度
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// エラー（lint 失敗）
    Error,
    /// 警告（lint 成功だが注意）
    Warning,
}

impl Severity {
    /// 重要度を文字列として取得
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
        }
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// 違反
#[derive(Debug, Clone)]
pub struct Violation {
    /// ルール ID
    pub rule: RuleId,
    /// 重要度
    pub severity: Severity,
    /// メッセージ
    pub message: String,
    /// 対象パス
    pub path: Option<String>,
    /// 行番号
    pub line: Option<usize>,
    /// ヒント
    pub hint: Option<String>,
}

impl Violation {
    /// 新しい違反を作成
    pub fn new(rule: RuleId, severity: Severity, message: impl Into<String>) -> Self {
        Self {
            rule,
            severity,
            message: message.into(),
            path: None,
            line: None,
            hint: None,
        }
    }

    /// パスを設定
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// 行番号を設定
    pub fn with_line(mut self, line: usize) -> Self {
        self.line = Some(line);
        self
    }

    /// ヒントを設定
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    /// エラーかどうか
    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }
}

/// lint 結果
#[derive(Debug, Clone)]
pub struct LintResult {
    /// 検査したパス
    pub path: PathBuf,
    /// 違反リスト
    pub violations: Vec<Violation>,
}

impl LintResult {
    /// 新しい結果を作成
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            violations: Vec::new(),
        }
    }

    /// 違反を追加
    pub fn add_violation(&mut self, violation: Violation) {
        self.violations.push(violation);
    }

    /// エラーの数
    pub fn error_count(&self) -> usize {
        self.violations.iter().filter(|v| v.is_error()).count()
    }

    /// 警告の数
    pub fn warning_count(&self) -> usize {
        self.violations
            .iter()
            .filter(|v| !v.is_error())
            .count()
    }

    /// 成功かどうか（エラーがないか）
    pub fn is_success(&self) -> bool {
        self.error_count() == 0
    }
}

/// lint 設定
#[derive(Debug, Clone)]
pub struct LintConfig {
    /// 実行するルール（None の場合は全て）
    pub rules: Option<Vec<String>>,
    /// 除外するルール
    pub exclude_rules: Vec<String>,
    /// 警告をエラーとして扱う
    pub strict: bool,
}

impl Default for LintConfig {
    fn default() -> Self {
        Self {
            rules: None,
            exclude_rules: Vec::new(),
            strict: false,
        }
    }
}

/// サービスの種別ごとの必須ファイル定義
#[derive(Debug, Clone)]
pub struct RequiredFiles {
    /// 必須ディレクトリ
    pub directories: Vec<&'static str>,
    /// 必須ファイル
    pub files: Vec<&'static str>,
}

impl RequiredFiles {
    /// backend-rust の必須ファイル
    pub fn backend_rust() -> Self {
        Self {
            directories: vec![
                "src/domain",
                "src/application",
                "src/infrastructure",
                "src/presentation",
                "config",
                "deploy/base",
                "deploy/overlays/dev",
                "deploy/overlays/stg",
                "deploy/overlays/prod",
            ],
            files: vec![
                "Cargo.toml",
                "README.md",
                "src/main.rs",
                "src/domain/mod.rs",
                "src/application/mod.rs",
                "src/infrastructure/mod.rs",
                "src/presentation/mod.rs",
                "config/default.yaml",
                "config/dev.yaml",
                "config/stg.yaml",
                "config/prod.yaml",
                "buf.yaml",
            ],
        }
    }

    /// backend-go の必須ファイル
    pub fn backend_go() -> Self {
        Self {
            directories: vec![
                "internal/domain",
                "internal/application",
                "internal/infrastructure",
                "internal/presentation",
                "config",
            ],
            files: vec![
                "go.mod",
                "README.md",
                "cmd/main.go",
                "config/default.yaml",
            ],
        }
    }

    /// frontend-react の必須ファイル
    pub fn frontend_react() -> Self {
        Self {
            directories: vec![
                "src/domain",
                "src/application",
                "src/infrastructure",
                "src/presentation",
                "src/pages",
                "src/components/layout",
                "config",
            ],
            files: vec![
                "package.json",
                "README.md",
                "src/main.tsx",
                "src/App.tsx",
                "config/default.yaml",
            ],
        }
    }

    /// frontend-flutter の必須ファイル
    pub fn frontend_flutter() -> Self {
        Self {
            directories: vec![
                "lib/src/domain",
                "lib/src/application",
                "lib/src/infrastructure",
                "lib/src/presentation",
                "config",
            ],
            files: vec![
                "pubspec.yaml",
                "README.md",
                "lib/main.dart",
                "config/default.yaml",
            ],
        }
    }

    /// テンプレート名から必須ファイルを取得
    pub fn from_template_name(name: &str) -> Option<Self> {
        match name {
            "backend-rust" => Some(Self::backend_rust()),
            "backend-go" => Some(Self::backend_go()),
            "frontend-react" => Some(Self::frontend_react()),
            "frontend-flutter" => Some(Self::frontend_flutter()),
            _ => None,
        }
    }
}

/// linter
pub struct Linter {
    /// 設定
    config: LintConfig,
}

impl Linter {
    /// 新しい linter を作成
    pub fn new(config: LintConfig) -> Self {
        Self { config }
    }

    /// デフォルト設定で作成
    pub fn default_linter() -> Self {
        Self::new(LintConfig::default())
    }

    /// ルールがスキップされるかどうか
    fn is_rule_skipped(&self, rule: RuleId) -> bool {
        let rule_id = rule.as_str();

        // 除外ルールに含まれている場合はスキップ
        if self.config.exclude_rules.iter().any(|r| r == rule_id) {
            return true;
        }

        // 実行ルールが指定されている場合、含まれていなければスキップ
        if let Some(rules) = &self.config.rules {
            if !rules.iter().any(|r| r == rule_id) {
                return true;
            }
        }

        false
    }

    /// ディレクトリを検査する
    pub fn lint<P: AsRef<Path>>(&self, path: P) -> LintResult {
        let path = path.as_ref().to_path_buf();
        let mut result = LintResult::new(path.clone());

        // manifest の検査
        self.check_manifest(&path, &mut result);

        // 必須ファイルの検査（manifest から情報を取得）
        self.check_required_files(&path, &mut result);

        // strict モードの場合、警告をエラーに昇格
        if self.config.strict {
            for v in &mut result.violations {
                if v.severity == Severity::Warning {
                    v.severity = Severity::Error;
                }
            }
        }

        result
    }

    /// manifest を検査する
    fn check_manifest(&self, path: &Path, result: &mut LintResult) {
        let manifest_path = path.join(".k1s0/manifest.json");

        // K001: manifest.json が存在しない
        if !self.is_rule_skipped(RuleId::ManifestNotFound) {
            if !manifest_path.exists() {
                result.add_violation(
                    Violation::new(
                        RuleId::ManifestNotFound,
                        Severity::Error,
                        "manifest.json が見つかりません",
                    )
                    .with_path(".k1s0/manifest.json")
                    .with_hint("k1s0 new-feature で生成したプロジェクトか確認してください"),
                );
                return; // manifest がない場合は以降の検査をスキップ
            }
        }

        // manifest を読み込む
        let manifest = match Manifest::load(&manifest_path) {
            Ok(m) => m,
            Err(e) => {
                result.add_violation(
                    Violation::new(
                        RuleId::ManifestInvalidValue,
                        Severity::Error,
                        format!("manifest.json の読み込みに失敗: {}", e),
                    )
                    .with_path(".k1s0/manifest.json"),
                );
                return;
            }
        };

        // K002: 必須キーの検査
        if !self.is_rule_skipped(RuleId::ManifestMissingKey) {
            self.check_manifest_required_keys(&manifest, result);
        }

        // K003: 値の妥当性検査
        if !self.is_rule_skipped(RuleId::ManifestInvalidValue) {
            self.check_manifest_values(&manifest, result);
        }
    }

    /// manifest の必須キーを検査
    fn check_manifest_required_keys(&self, manifest: &Manifest, result: &mut LintResult) {
        // k1s0_version
        if manifest.k1s0_version.is_empty() {
            result.add_violation(
                Violation::new(
                    RuleId::ManifestMissingKey,
                    Severity::Error,
                    "k1s0_version が空です",
                )
                .with_path(".k1s0/manifest.json")
                .with_hint("k1s0 のバージョンを指定してください"),
            );
        }

        // template.name
        if manifest.template.name.is_empty() {
            result.add_violation(
                Violation::new(
                    RuleId::ManifestMissingKey,
                    Severity::Error,
                    "template.name が空です",
                )
                .with_path(".k1s0/manifest.json"),
            );
        }

        // template.version
        if manifest.template.version.is_empty() {
            result.add_violation(
                Violation::new(
                    RuleId::ManifestMissingKey,
                    Severity::Error,
                    "template.version が空です",
                )
                .with_path(".k1s0/manifest.json"),
            );
        }

        // template.fingerprint
        if manifest.template.fingerprint.is_empty() {
            result.add_violation(
                Violation::new(
                    RuleId::ManifestMissingKey,
                    Severity::Error,
                    "template.fingerprint が空です",
                )
                .with_path(".k1s0/manifest.json"),
            );
        }

        // service.service_name
        if manifest.service.service_name.is_empty() {
            result.add_violation(
                Violation::new(
                    RuleId::ManifestMissingKey,
                    Severity::Error,
                    "service.service_name が空です",
                )
                .with_path(".k1s0/manifest.json"),
            );
        }

        // service.language
        if manifest.service.language.is_empty() {
            result.add_violation(
                Violation::new(
                    RuleId::ManifestMissingKey,
                    Severity::Error,
                    "service.language が空です",
                )
                .with_path(".k1s0/manifest.json"),
            );
        }

        // managed_paths
        if manifest.managed_paths.is_empty() {
            result.add_violation(
                Violation::new(
                    RuleId::ManifestMissingKey,
                    Severity::Warning,
                    "managed_paths が空です",
                )
                .with_path(".k1s0/manifest.json")
                .with_hint("upgrade で自動更新されるパスを指定してください"),
            );
        }

        // protected_paths
        if manifest.protected_paths.is_empty() {
            result.add_violation(
                Violation::new(
                    RuleId::ManifestMissingKey,
                    Severity::Warning,
                    "protected_paths が空です",
                )
                .with_path(".k1s0/manifest.json")
                .with_hint("upgrade で保護されるパスを指定してください"),
            );
        }
    }

    /// manifest の値を検査
    fn check_manifest_values(&self, manifest: &Manifest, result: &mut LintResult) {
        // service.language の妥当性
        let valid_languages = ["rust", "go", "typescript", "dart"];
        if !valid_languages.contains(&manifest.service.language.as_str()) {
            result.add_violation(
                Violation::new(
                    RuleId::ManifestInvalidValue,
                    Severity::Error,
                    format!(
                        "service.language '{}' は不正です。有効な値: {}",
                        manifest.service.language,
                        valid_languages.join(", ")
                    ),
                )
                .with_path(".k1s0/manifest.json"),
            );
        }

        // service.service_type の妥当性
        let valid_types = ["backend", "frontend", "bff"];
        if !valid_types.contains(&manifest.service.service_type.as_str()) {
            result.add_violation(
                Violation::new(
                    RuleId::ManifestInvalidValue,
                    Severity::Error,
                    format!(
                        "service.type '{}' は不正です。有効な値: {}",
                        manifest.service.service_type,
                        valid_types.join(", ")
                    ),
                )
                .with_path(".k1s0/manifest.json"),
            );
        }

        // template.name の妥当性
        let valid_templates = [
            "backend-rust",
            "backend-go",
            "frontend-react",
            "frontend-flutter",
        ];
        if !valid_templates.contains(&manifest.template.name.as_str()) {
            result.add_violation(
                Violation::new(
                    RuleId::ManifestInvalidValue,
                    Severity::Warning,
                    format!(
                        "template.name '{}' は標準テンプレートではありません",
                        manifest.template.name
                    ),
                )
                .with_path(".k1s0/manifest.json"),
            );
        }
    }

    /// 必須ファイルを検査
    fn check_required_files(&self, path: &Path, result: &mut LintResult) {
        // manifest から template.name を取得
        let manifest_path = path.join(".k1s0/manifest.json");
        let manifest = match Manifest::load(&manifest_path) {
            Ok(m) => m,
            Err(_) => return, // manifest がない場合はスキップ（既にエラー報告済み）
        };

        // template.name から必須ファイルを取得
        let required = match RequiredFiles::from_template_name(&manifest.template.name) {
            Some(r) => r,
            None => return, // 不明なテンプレートの場合はスキップ
        };

        // K010: 必須ディレクトリの検査
        if !self.is_rule_skipped(RuleId::RequiredDirMissing) {
            for dir in &required.directories {
                let dir_path = path.join(dir);
                if !dir_path.exists() || !dir_path.is_dir() {
                    result.add_violation(
                        Violation::new(
                            RuleId::RequiredDirMissing,
                            Severity::Error,
                            format!("必須ディレクトリ '{}' が存在しません", dir),
                        )
                        .with_path(*dir),
                    );
                }
            }
        }

        // K011: 必須ファイルの検査
        if !self.is_rule_skipped(RuleId::RequiredFileMissing) {
            for file in &required.files {
                let file_path = path.join(file);
                if !file_path.exists() || !file_path.is_file() {
                    result.add_violation(
                        Violation::new(
                            RuleId::RequiredFileMissing,
                            Severity::Error,
                            format!("必須ファイル '{}' が存在しません", file),
                        )
                        .with_path(*file),
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_manifest(dir: &Path, template_name: &str) {
        let k1s0_dir = dir.join(".k1s0");
        fs::create_dir_all(&k1s0_dir).unwrap();

        let manifest = crate::manifest::Manifest {
            schema_version: "1.0.0".to_string(),
            k1s0_version: "0.1.0".to_string(),
            template: crate::manifest::TemplateInfo {
                name: template_name.to_string(),
                version: "0.1.0".to_string(),
                source: "local".to_string(),
                path: format!("CLI/templates/{}/feature", template_name),
                revision: None,
                fingerprint: "abcd1234".to_string(),
            },
            service: crate::manifest::ServiceInfo {
                service_name: "test-service".to_string(),
                language: "rust".to_string(),
                service_type: "backend".to_string(),
                framework: None,
            },
            generated_at: "2026-01-26T10:00:00Z".to_string(),
            managed_paths: vec!["deploy/".to_string()],
            protected_paths: vec!["src/domain/".to_string()],
            update_policy: std::collections::HashMap::new(),
            checksums: std::collections::HashMap::new(),
            dependencies: None,
        };

        manifest.save(k1s0_dir.join("manifest.json")).unwrap();
    }

    fn create_backend_rust_structure(dir: &Path) {
        // ディレクトリ作成
        for d in &[
            "src/domain",
            "src/application",
            "src/infrastructure",
            "src/presentation",
            "config",
            "deploy/base",
            "deploy/overlays/dev",
            "deploy/overlays/stg",
            "deploy/overlays/prod",
        ] {
            fs::create_dir_all(dir.join(d)).unwrap();
        }

        // ファイル作成
        for f in &[
            "Cargo.toml",
            "README.md",
            "src/main.rs",
            "src/domain/mod.rs",
            "src/application/mod.rs",
            "src/infrastructure/mod.rs",
            "src/presentation/mod.rs",
            "config/default.yaml",
            "config/dev.yaml",
            "config/stg.yaml",
            "config/prod.yaml",
            "buf.yaml",
        ] {
            fs::write(dir.join(f), "").unwrap();
        }
    }

    #[test]
    fn test_lint_success() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // 完全な構造を作成
        create_test_manifest(path, "backend-rust");
        create_backend_rust_structure(path);

        let linter = Linter::default_linter();
        let result = linter.lint(path);

        assert!(result.is_success(), "Expected success, got {:?}", result.violations);
        assert_eq!(result.error_count(), 0);
    }

    #[test]
    fn test_lint_manifest_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        let linter = Linter::default_linter();
        let result = linter.lint(path);

        assert!(!result.is_success());
        assert!(result.violations.iter().any(|v| v.rule == RuleId::ManifestNotFound));
    }

    #[test]
    fn test_lint_required_file_missing() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // manifest だけ作成（必須ファイルなし）
        create_test_manifest(path, "backend-rust");

        let linter = Linter::default_linter();
        let result = linter.lint(path);

        assert!(!result.is_success());
        assert!(result.violations.iter().any(|v| v.rule == RuleId::RequiredFileMissing));
        assert!(result.violations.iter().any(|v| v.rule == RuleId::RequiredDirMissing));
    }

    #[test]
    fn test_lint_with_exclude_rules() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // manifest だけ作成（必須ファイルなし）
        create_test_manifest(path, "backend-rust");

        let config = LintConfig {
            rules: None,
            exclude_rules: vec!["K010".to_string(), "K011".to_string()],
            strict: false,
        };
        let linter = Linter::new(config);
        let result = linter.lint(path);

        // 必須ファイル/ディレクトリのチェックはスキップされる
        assert!(result.is_success());
    }

    #[test]
    fn test_lint_with_specific_rules() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // manifest だけ作成（必須ファイルなし）
        create_test_manifest(path, "backend-rust");

        let config = LintConfig {
            rules: Some(vec!["K001".to_string()]), // manifest 存在チェックのみ
            exclude_rules: vec![],
            strict: false,
        };
        let linter = Linter::new(config);
        let result = linter.lint(path);

        // manifest は存在するので成功
        assert!(result.is_success());
    }

    #[test]
    fn test_lint_strict_mode() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // 完全な構造を作成
        create_test_manifest(path, "backend-rust");
        create_backend_rust_structure(path);

        // manifest の managed_paths を空にする
        let manifest_path = path.join(".k1s0/manifest.json");
        let mut manifest = crate::manifest::Manifest::load(&manifest_path).unwrap();
        manifest.managed_paths = vec![];
        manifest.save(&manifest_path).unwrap();

        // 通常モード
        let linter = Linter::default_linter();
        let result = linter.lint(path);
        assert!(result.is_success()); // 警告のみなので成功

        // strict モード
        let config = LintConfig {
            rules: None,
            exclude_rules: vec![],
            strict: true,
        };
        let linter = Linter::new(config);
        let result = linter.lint(path);
        assert!(!result.is_success()); // 警告がエラーに昇格
    }

    #[test]
    fn test_rule_id_display() {
        assert_eq!(RuleId::ManifestNotFound.as_str(), "K001");
        assert_eq!(RuleId::ManifestMissingKey.as_str(), "K002");
        assert_eq!(RuleId::ManifestInvalidValue.as_str(), "K003");
        assert_eq!(RuleId::RequiredDirMissing.as_str(), "K010");
        assert_eq!(RuleId::RequiredFileMissing.as_str(), "K011");
    }

    #[test]
    fn test_required_files_from_template_name() {
        assert!(RequiredFiles::from_template_name("backend-rust").is_some());
        assert!(RequiredFiles::from_template_name("backend-go").is_some());
        assert!(RequiredFiles::from_template_name("frontend-react").is_some());
        assert!(RequiredFiles::from_template_name("frontend-flutter").is_some());
        assert!(RequiredFiles::from_template_name("unknown").is_none());
    }
}

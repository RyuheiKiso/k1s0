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
//! - `K020`: 環境変数参照の禁止
//! - `K021`: config YAML への機密直書き禁止
//! - `K022`: Clean Architecture 依存方向違反

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
    /// 環境変数参照の禁止
    EnvVarUsage,
    /// config YAML への機密直書き禁止
    SecretInConfig,
    /// Clean Architecture 依存方向違反
    DependencyDirection,
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
            RuleId::EnvVarUsage => "K020",
            RuleId::SecretInConfig => "K021",
            RuleId::DependencyDirection => "K022",
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
            RuleId::EnvVarUsage => "環境変数の参照は禁止されています",
            RuleId::SecretInConfig => "config YAML に機密情報が直接書かれています",
            RuleId::DependencyDirection => "Clean Architecture の依存方向に違反しています",
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
    /// 環境変数参照を許可するファイルパス（glob パターン）
    pub env_var_allowlist: Vec<String>,
}

impl Default for LintConfig {
    fn default() -> Self {
        Self {
            rules: None,
            exclude_rules: Vec::new(),
            strict: false,
            env_var_allowlist: Vec::new(),
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

        // K020: 環境変数参照の検査
        if !self.is_rule_skipped(RuleId::EnvVarUsage) {
            self.check_env_var_usage(&path, &mut result);
        }

        // K021: config YAML への機密直書き検査
        if !self.is_rule_skipped(RuleId::SecretInConfig) {
            self.check_secret_in_config(&path, &mut result);
        }

        // K022: Clean Architecture 依存方向検査
        if !self.is_rule_skipped(RuleId::DependencyDirection) {
            self.check_dependency_direction(&path, &mut result);
        }

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

    /// 環境変数参照を検査（K020）
    fn check_env_var_usage(&self, path: &Path, result: &mut LintResult) {
        // manifest から言語を取得
        let manifest_path = path.join(".k1s0/manifest.json");
        let manifest = match Manifest::load(&manifest_path) {
            Ok(m) => m,
            Err(_) => return, // manifest がない場合はスキップ
        };

        // 言語に応じたソースディレクトリとパターンを決定
        let (src_dir, patterns) = match manifest.service.language.as_str() {
            "rust" => ("src", EnvVarPatterns::rust()),
            "go" => ("internal", EnvVarPatterns::go()),
            "typescript" => ("src", EnvVarPatterns::typescript()),
            "dart" => ("lib", EnvVarPatterns::dart()),
            _ => return, // 不明な言語の場合はスキップ
        };

        let src_path = path.join(src_dir);
        if !src_path.exists() {
            return;
        }

        // ソースファイルを走査
        self.scan_directory_for_env_vars(&src_path, path, &patterns, result);
    }

    /// ディレクトリを再帰的に走査して環境変数参照を検出
    fn scan_directory_for_env_vars(
        &self,
        dir: &Path,
        base_path: &Path,
        patterns: &EnvVarPatterns,
        result: &mut LintResult,
    ) {
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                // 再帰的に走査
                self.scan_directory_for_env_vars(&entry_path, base_path, patterns, result);
            } else if entry_path.is_file() {
                // ファイルの拡張子をチェック
                let ext = entry_path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if patterns.file_extensions.contains(&ext) {
                    self.check_file_for_env_vars(&entry_path, base_path, patterns, result);
                }
            }
        }
    }

    /// ファイル内の環境変数参照を検査
    fn check_file_for_env_vars(
        &self,
        file_path: &Path,
        base_path: &Path,
        patterns: &EnvVarPatterns,
        result: &mut LintResult,
    ) {
        // allowlist チェック
        let relative_path = file_path
            .strip_prefix(base_path)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| file_path.to_string_lossy().to_string());

        if self.is_path_in_allowlist(&relative_path) {
            return;
        }

        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => return,
        };

        for (line_num, line) in content.lines().enumerate() {
            for pattern in &patterns.patterns {
                if line.contains(pattern.pattern) {
                    result.add_violation(
                        Violation::new(
                            RuleId::EnvVarUsage,
                            Severity::Error,
                            format!("環境変数参照 '{}' が検出されました", pattern.pattern),
                        )
                        .with_path(&relative_path)
                        .with_line(line_num + 1)
                        .with_hint(&pattern.hint),
                    );
                }
            }
        }
    }

    /// パスが allowlist に含まれるかチェック
    fn is_path_in_allowlist(&self, path: &str) -> bool {
        // パス区切り文字を統一（Windows 対応）
        let normalized_path = path.replace('\\', "/");

        for pattern in &self.config.env_var_allowlist {
            let normalized_pattern = pattern.replace('\\', "/");

            // 単純なワイルドカードマッチング
            if normalized_pattern.ends_with('*') {
                let prefix = &normalized_pattern[..normalized_pattern.len() - 1];
                if normalized_path.starts_with(prefix) {
                    return true;
                }
            } else if normalized_pattern == normalized_path {
                return true;
            }
        }
        false
    }

    /// config YAML への機密直書きを検査（K021）
    fn check_secret_in_config(&self, path: &Path, result: &mut LintResult) {
        let config_dir = path.join("config");
        if !config_dir.exists() || !config_dir.is_dir() {
            return;
        }

        // config ディレクトリ内の YAML ファイルを検査
        let entries = match std::fs::read_dir(&config_dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_file() {
                let ext = entry_path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if ext == "yaml" || ext == "yml" {
                    self.check_yaml_for_secrets(&entry_path, path, result);
                }
            }
        }
    }

    /// YAML ファイル内の機密直書きを検査
    fn check_yaml_for_secrets(&self, yaml_path: &Path, base_path: &Path, result: &mut LintResult) {
        let content = match std::fs::read_to_string(yaml_path) {
            Ok(c) => c,
            Err(_) => return,
        };

        let relative_path = yaml_path
            .strip_prefix(base_path)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| yaml_path.to_string_lossy().to_string());

        // 機密キーのパターン
        let secret_key_patterns = SecretKeyPatterns::default();

        for (line_num, line) in content.lines().enumerate() {
            // コメント行はスキップ
            let trimmed = line.trim();
            if trimmed.starts_with('#') || trimmed.is_empty() {
                continue;
            }

            // キー: 値 の形式をパース
            if let Some((key, value)) = parse_yaml_line(line) {
                // キーが機密パターンにマッチするかチェック
                if let Some(pattern) = secret_key_patterns.matches_secret_key(&key) {
                    // *_file サフィックスを持つキーは許可
                    if key.ends_with("_file") || key.ends_with("_path") || key.ends_with("_ref") {
                        continue;
                    }

                    // 値が空、null、参照形式でない場合はエラー
                    let value_trimmed = value.trim();
                    if !value_trimmed.is_empty()
                        && value_trimmed != "null"
                        && value_trimmed != "~"
                        && !value_trimmed.starts_with("${")
                        && !value_trimmed.starts_with("{{")
                    {
                        result.add_violation(
                            Violation::new(
                                RuleId::SecretInConfig,
                                Severity::Error,
                                format!(
                                    "機密キー '{}' に値が直接設定されています",
                                    key
                                ),
                            )
                            .with_path(&relative_path)
                            .with_line(line_num + 1)
                            .with_hint(format!(
                                "機密情報は直接書かず、'{}_file' キーでファイルパスを参照してください。例: {}_file: /var/run/secrets/k1s0/{}",
                                pattern, pattern, pattern
                            )),
                        );
                    }
                }
            }
        }
    }

    /// Clean Architecture 依存方向を検査（K022）
    fn check_dependency_direction(&self, path: &Path, result: &mut LintResult) {
        // manifest から言語を取得
        let manifest_path = path.join(".k1s0/manifest.json");
        let manifest = match Manifest::load(&manifest_path) {
            Ok(m) => m,
            Err(_) => return, // manifest がない場合はスキップ
        };

        // 言語に応じたソースディレクトリとパターンを決定
        let (src_dir, rules) = match manifest.service.language.as_str() {
            "rust" => ("src", DependencyRules::rust()),
            "go" => ("internal", DependencyRules::go()),
            "typescript" => ("src", DependencyRules::typescript()),
            "dart" => ("lib/src", DependencyRules::dart()),
            _ => return, // 不明な言語の場合はスキップ
        };

        let src_path = path.join(src_dir);
        if !src_path.exists() {
            return;
        }

        // 各層のディレクトリを検査
        for layer in &["domain", "application"] {
            let layer_path = src_path.join(layer);
            if layer_path.exists() && layer_path.is_dir() {
                self.scan_layer_for_violations(&layer_path, path, layer, &rules, result);
            }
        }
    }

    /// 特定の層のディレクトリを走査して依存方向違反を検出
    fn scan_layer_for_violations(
        &self,
        dir: &Path,
        base_path: &Path,
        layer: &str,
        rules: &DependencyRules,
        result: &mut LintResult,
    ) {
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                // 再帰的に走査
                self.scan_layer_for_violations(&entry_path, base_path, layer, rules, result);
            } else if entry_path.is_file() {
                // ファイルの拡張子をチェック
                let ext = entry_path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if rules.file_extensions.contains(&ext) {
                    self.check_file_for_violations(&entry_path, base_path, layer, rules, result);
                }
            }
        }
    }

    /// ファイル内の依存方向違反を検査
    fn check_file_for_violations(
        &self,
        file_path: &Path,
        base_path: &Path,
        layer: &str,
        rules: &DependencyRules,
        result: &mut LintResult,
    ) {
        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => return,
        };

        let relative_path = file_path
            .strip_prefix(base_path)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| file_path.to_string_lossy().to_string());

        // 層に応じた禁止パターンを取得
        let forbidden_layers = match layer {
            "domain" => vec!["application", "infrastructure", "presentation"],
            "application" => vec!["infrastructure", "presentation"],
            _ => return,
        };

        for (line_num, line) in content.lines().enumerate() {
            // コメント行はスキップ
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with("#") || trimmed.starts_with("/*") {
                continue;
            }

            // 各禁止層へのインポートをチェック
            for forbidden in &forbidden_layers {
                for pattern in &rules.import_patterns {
                    let forbidden_pattern = pattern.replace("{layer}", forbidden);
                    if line.contains(&forbidden_pattern) {
                        result.add_violation(
                            Violation::new(
                                RuleId::DependencyDirection,
                                Severity::Error,
                                format!(
                                    "{} 層から {} 層への依存が検出されました",
                                    layer, forbidden
                                ),
                            )
                            .with_path(&relative_path)
                            .with_line(line_num + 1)
                            .with_hint(format!(
                                "Clean Architecture では {} 層は {} 層に依存できません。依存関係を逆転させてください。",
                                layer, forbidden
                            )),
                        );
                    }
                }
            }
        }
    }
}

/// 依存方向ルールの定義
#[derive(Debug, Clone)]
pub struct DependencyRules {
    /// 対象ファイルの拡張子
    pub file_extensions: Vec<&'static str>,
    /// import パターン（{layer} はプレースホルダ）
    pub import_patterns: Vec<String>,
}

impl DependencyRules {
    /// Rust の依存方向ルール
    pub fn rust() -> Self {
        Self {
            file_extensions: vec!["rs"],
            import_patterns: vec![
                "use crate::{layer}".to_string(),
                "crate::{layer}::".to_string(),
                "super::super::{layer}".to_string(),
            ],
        }
    }

    /// Go の依存方向ルール
    pub fn go() -> Self {
        Self {
            file_extensions: vec!["go"],
            import_patterns: vec![
                "\"internal/{layer}".to_string(),
                "/internal/{layer}".to_string(),
            ],
        }
    }

    /// TypeScript の依存方向ルール
    pub fn typescript() -> Self {
        Self {
            file_extensions: vec!["ts", "tsx", "js", "jsx"],
            import_patterns: vec![
                "from '../{layer}".to_string(),
                "from \"../{layer}".to_string(),
                "from '../../{layer}".to_string(),
                "from \"../../{layer}".to_string(),
                "from '@/{layer}".to_string(),
                "from \"@/{layer}".to_string(),
            ],
        }
    }

    /// Dart の依存方向ルール
    pub fn dart() -> Self {
        Self {
            file_extensions: vec!["dart"],
            import_patterns: vec![
                "import 'package:".to_string() + "{layer}",
                "import '../{layer}".to_string(),
                "import '../../{layer}".to_string(),
            ],
        }
    }
}

/// YAML の行をパースしてキーと値を取得
fn parse_yaml_line(line: &str) -> Option<(String, String)> {
    // インデントを除去
    let trimmed = line.trim_start();

    // コメントや空行はスキップ
    if trimmed.starts_with('#') || trimmed.is_empty() || trimmed.starts_with('-') {
        return None;
    }

    // キー: 値 の形式を探す
    if let Some(colon_pos) = trimmed.find(':') {
        let key = trimmed[..colon_pos].trim();
        let value = if colon_pos + 1 < trimmed.len() {
            trimmed[colon_pos + 1..].trim()
        } else {
            ""
        };

        // キーが空でなければ返す
        if !key.is_empty() {
            return Some((key.to_string(), value.to_string()));
        }
    }

    None
}

/// 機密キーのパターン定義
#[derive(Debug, Clone)]
pub struct SecretKeyPatterns {
    /// 機密キーのパターン（部分一致）
    pub patterns: Vec<&'static str>,
}

impl Default for SecretKeyPatterns {
    fn default() -> Self {
        Self {
            patterns: vec![
                "password",
                "passwd",
                "secret",
                "token",
                "api_key",
                "apikey",
                "api-key",
                "private_key",
                "privatekey",
                "private-key",
                "credential",
                "auth_key",
                "authkey",
                "auth-key",
                "access_key",
                "accesskey",
                "access-key",
                "secret_key",
                "secretkey",
                "secret-key",
                "encryption_key",
                "encryptionkey",
                "encryption-key",
                "signing_key",
                "signingkey",
                "signing-key",
                "client_secret",
                "clientsecret",
                "client-secret",
            ],
        }
    }
}

impl SecretKeyPatterns {
    /// キーが機密パターンにマッチするかチェック
    /// マッチした場合はパターン名を返す
    pub fn matches_secret_key(&self, key: &str) -> Option<&'static str> {
        let key_lower = key.to_lowercase();
        for pattern in &self.patterns {
            if key_lower.contains(pattern) {
                return Some(pattern);
            }
        }
        None
    }
}

/// 環境変数パターンの定義
#[derive(Debug, Clone)]
pub struct EnvVarPattern {
    /// 検出対象のパターン文字列
    pub pattern: &'static str,
    /// 検出時に表示するヒント
    pub hint: String,
}

/// 言語ごとの環境変数パターン
#[derive(Debug, Clone)]
pub struct EnvVarPatterns {
    /// 対象ファイルの拡張子
    pub file_extensions: Vec<&'static str>,
    /// 検出パターン
    pub patterns: Vec<EnvVarPattern>,
}

impl EnvVarPatterns {
    /// Rust の環境変数パターン
    pub fn rust() -> Self {
        Self {
            file_extensions: vec!["rs"],
            patterns: vec![
                EnvVarPattern {
                    pattern: "std::env::var",
                    hint: "config/{env}.yaml で設定を管理してください。framework の config モジュールを使用してください。".to_string(),
                },
                EnvVarPattern {
                    pattern: "std::env::var_os",
                    hint: "config/{env}.yaml で設定を管理してください。framework の config モジュールを使用してください。".to_string(),
                },
                EnvVarPattern {
                    pattern: "std::env::vars",
                    hint: "config/{env}.yaml で設定を管理してください。framework の config モジュールを使用してください。".to_string(),
                },
                EnvVarPattern {
                    pattern: "std::env::vars_os",
                    hint: "config/{env}.yaml で設定を管理してください。framework の config モジュールを使用してください。".to_string(),
                },
                EnvVarPattern {
                    pattern: "std::env::set_var",
                    hint: "環境変数の設定は禁止されています。config/{env}.yaml で設定を管理してください。".to_string(),
                },
                EnvVarPattern {
                    pattern: "std::env::remove_var",
                    hint: "環境変数の削除は禁止されています。".to_string(),
                },
                EnvVarPattern {
                    pattern: "env::var(",
                    hint: "config/{env}.yaml で設定を管理してください。framework の config モジュールを使用してください。".to_string(),
                },
                EnvVarPattern {
                    pattern: "env::var_os(",
                    hint: "config/{env}.yaml で設定を管理してください。framework の config モジュールを使用してください。".to_string(),
                },
                EnvVarPattern {
                    pattern: "env::vars(",
                    hint: "config/{env}.yaml で設定を管理してください。framework の config モジュールを使用してください。".to_string(),
                },
                EnvVarPattern {
                    pattern: "env::set_var(",
                    hint: "環境変数の設定は禁止されています。config/{env}.yaml で設定を管理してください。".to_string(),
                },
                EnvVarPattern {
                    pattern: "env::remove_var(",
                    hint: "環境変数の削除は禁止されています。".to_string(),
                },
                EnvVarPattern {
                    pattern: "dotenv",
                    hint: "dotenv の使用は禁止されています。config/{env}.yaml で設定を管理してください。".to_string(),
                },
                EnvVarPattern {
                    pattern: "dotenvy",
                    hint: "dotenvy の使用は禁止されています。config/{env}.yaml で設定を管理してください。".to_string(),
                },
            ],
        }
    }

    /// Go の環境変数パターン
    pub fn go() -> Self {
        Self {
            file_extensions: vec!["go"],
            patterns: vec![
                EnvVarPattern {
                    pattern: "os.Getenv",
                    hint: "config/{env}.yaml で設定を管理してください。framework の config パッケージを使用してください。".to_string(),
                },
                EnvVarPattern {
                    pattern: "os.LookupEnv",
                    hint: "config/{env}.yaml で設定を管理してください。framework の config パッケージを使用してください。".to_string(),
                },
                EnvVarPattern {
                    pattern: "os.Setenv",
                    hint: "環境変数の設定は禁止されています。config/{env}.yaml で設定を管理してください。".to_string(),
                },
                EnvVarPattern {
                    pattern: "os.Unsetenv",
                    hint: "環境変数の削除は禁止されています。".to_string(),
                },
                EnvVarPattern {
                    pattern: "os.Environ",
                    hint: "環境変数の一覧取得は禁止されています。config/{env}.yaml で設定を管理してください。".to_string(),
                },
                EnvVarPattern {
                    pattern: "godotenv",
                    hint: "godotenv の使用は禁止されています。config/{env}.yaml で設定を管理してください。".to_string(),
                },
            ],
        }
    }

    /// TypeScript の環境変数パターン
    pub fn typescript() -> Self {
        Self {
            file_extensions: vec!["ts", "tsx", "js", "jsx"],
            patterns: vec![
                EnvVarPattern {
                    pattern: "process.env",
                    hint: "config/{env}.yaml で設定を管理してください。framework の config モジュールを使用してください。".to_string(),
                },
                EnvVarPattern {
                    pattern: "import.meta.env",
                    hint: "config/{env}.yaml で設定を管理してください。framework の config モジュールを使用してください。".to_string(),
                },
                EnvVarPattern {
                    pattern: "dotenv",
                    hint: "dotenv の使用は禁止されています。config/{env}.yaml で設定を管理してください。".to_string(),
                },
            ],
        }
    }

    /// Dart の環境変数パターン
    pub fn dart() -> Self {
        Self {
            file_extensions: vec!["dart"],
            patterns: vec![
                EnvVarPattern {
                    pattern: "Platform.environment",
                    hint: "config/{env}.yaml で設定を管理してください。framework の config モジュールを使用してください。".to_string(),
                },
                EnvVarPattern {
                    pattern: "fromEnvironment",
                    hint: "コンパイル時環境変数の使用は禁止されています。config/{env}.yaml で設定を管理してください。".to_string(),
                },
                EnvVarPattern {
                    pattern: "flutter_dotenv",
                    hint: "flutter_dotenv の使用は禁止されています。config/{env}.yaml で設定を管理してください。".to_string(),
                },
            ],
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
            env_var_allowlist: vec![],
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
            env_var_allowlist: vec![],
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
            env_var_allowlist: vec![],
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

    #[test]
    fn test_rule_id_k020() {
        assert_eq!(RuleId::EnvVarUsage.as_str(), "K020");
        assert_eq!(
            RuleId::EnvVarUsage.description(),
            "環境変数の参照は禁止されています"
        );
    }

    #[test]
    fn test_lint_env_var_usage_detected() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // 完全な構造を作成
        create_test_manifest(path, "backend-rust");
        create_backend_rust_structure(path);

        // 環境変数を使用するコードを追加
        let bad_code = r#"
use std::env;

fn main() {
    let value = std::env::var("MY_VAR").unwrap();
    println!("{}", value);
}
"#;
        fs::write(path.join("src/main.rs"), bad_code).unwrap();

        let linter = Linter::default_linter();
        let result = linter.lint(path);

        // K020 の違反が検出される
        assert!(
            result.violations.iter().any(|v| v.rule == RuleId::EnvVarUsage),
            "Expected K020 violation, got {:?}",
            result.violations
        );

        // ヒントが含まれている
        let env_var_violation = result
            .violations
            .iter()
            .find(|v| v.rule == RuleId::EnvVarUsage)
            .unwrap();
        assert!(env_var_violation.hint.is_some());
        assert!(env_var_violation.hint.as_ref().unwrap().contains("config"));
    }

    #[test]
    fn test_lint_env_var_usage_not_detected_when_clean() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // 完全な構造を作成
        create_test_manifest(path, "backend-rust");
        create_backend_rust_structure(path);

        // 環境変数を使用しないコードを追加
        let clean_code = r#"
fn main() {
    println!("Hello, world!");
}
"#;
        fs::write(path.join("src/main.rs"), clean_code).unwrap();

        let linter = Linter::default_linter();
        let result = linter.lint(path);

        // K020 の違反が検出されない
        assert!(
            !result.violations.iter().any(|v| v.rule == RuleId::EnvVarUsage),
            "Unexpected K020 violation: {:?}",
            result.violations
        );
    }

    #[test]
    fn test_lint_env_var_allowlist() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // 完全な構造を作成
        create_test_manifest(path, "backend-rust");
        create_backend_rust_structure(path);

        // 環境変数を使用するコードを追加
        let bad_code = r#"
use std::env;

fn main() {
    let value = std::env::var("MY_VAR").unwrap();
    println!("{}", value);
}
"#;
        fs::write(path.join("src/main.rs"), bad_code).unwrap();

        // allowlist に追加
        let config = LintConfig {
            rules: None,
            exclude_rules: vec![],
            strict: false,
            env_var_allowlist: vec!["src/main.rs".to_string()],
        };
        let linter = Linter::new(config);
        let result = linter.lint(path);

        // allowlist に含まれているので K020 の違反が検出されない
        assert!(
            !result.violations.iter().any(|v| v.rule == RuleId::EnvVarUsage),
            "Unexpected K020 violation (should be allowlisted): {:?}",
            result.violations
        );
    }

    #[test]
    fn test_lint_env_var_allowlist_wildcard() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // 完全な構造を作成
        create_test_manifest(path, "backend-rust");
        create_backend_rust_structure(path);

        // 環境変数を使用するコードを追加
        let bad_code = r#"
fn get_config() {
    let value = env::var("CONFIG_VAR").unwrap();
}
"#;
        fs::write(path.join("src/infrastructure/mod.rs"), bad_code).unwrap();

        // ワイルドカード allowlist
        let config = LintConfig {
            rules: None,
            exclude_rules: vec![],
            strict: false,
            env_var_allowlist: vec!["src/infrastructure/*".to_string()],
        };
        let linter = Linter::new(config);
        let result = linter.lint(path);

        // allowlist に含まれているので K020 の違反が検出されない
        assert!(
            !result.violations.iter().any(|v| v.rule == RuleId::EnvVarUsage),
            "Unexpected K020 violation (should be allowlisted): {:?}",
            result.violations
        );
    }

    #[test]
    fn test_lint_env_var_dotenv_detected() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // 完全な構造を作成
        create_test_manifest(path, "backend-rust");
        create_backend_rust_structure(path);

        // dotenv を使用するコードを追加
        let bad_code = r#"
use dotenv::dotenv;

fn main() {
    dotenv().ok();
}
"#;
        fs::write(path.join("src/main.rs"), bad_code).unwrap();

        let linter = Linter::default_linter();
        let result = linter.lint(path);

        // dotenv が検出される
        let dotenv_violation = result
            .violations
            .iter()
            .find(|v| v.rule == RuleId::EnvVarUsage && v.message.contains("dotenv"));
        assert!(
            dotenv_violation.is_some(),
            "Expected dotenv violation, got {:?}",
            result.violations
        );
    }

    #[test]
    fn test_lint_env_var_line_number() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // 完全な構造を作成
        create_test_manifest(path, "backend-rust");
        create_backend_rust_structure(path);

        // 環境変数を使用するコードを追加（3行目）
        let bad_code = "fn main() {\n    // comment\n    let x = std::env::var(\"X\").unwrap();\n}\n";
        fs::write(path.join("src/main.rs"), bad_code).unwrap();

        let linter = Linter::default_linter();
        let result = linter.lint(path);

        // K020 の違反が検出され、行番号が正しい
        let env_var_violation = result
            .violations
            .iter()
            .find(|v| v.rule == RuleId::EnvVarUsage)
            .unwrap();
        assert_eq!(env_var_violation.line, Some(3));
    }

    #[test]
    fn test_lint_env_var_exclude_rule() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // 完全な構造を作成
        create_test_manifest(path, "backend-rust");
        create_backend_rust_structure(path);

        // 環境変数を使用するコードを追加
        let bad_code = r#"
fn main() {
    let value = std::env::var("MY_VAR").unwrap();
}
"#;
        fs::write(path.join("src/main.rs"), bad_code).unwrap();

        // K020 を除外
        let config = LintConfig {
            rules: None,
            exclude_rules: vec!["K020".to_string()],
            strict: false,
            env_var_allowlist: vec![],
        };
        let linter = Linter::new(config);
        let result = linter.lint(path);

        // K020 の違反が検出されない
        assert!(
            !result.violations.iter().any(|v| v.rule == RuleId::EnvVarUsage),
            "Unexpected K020 violation (rule should be excluded): {:?}",
            result.violations
        );
    }

    #[test]
    fn test_env_var_patterns_rust() {
        let patterns = EnvVarPatterns::rust();
        assert!(patterns.file_extensions.contains(&"rs"));
        assert!(patterns.patterns.iter().any(|p| p.pattern == "std::env::var"));
        assert!(patterns.patterns.iter().any(|p| p.pattern == "dotenv"));
    }

    #[test]
    fn test_env_var_patterns_go() {
        let patterns = EnvVarPatterns::go();
        assert!(patterns.file_extensions.contains(&"go"));
        assert!(patterns.patterns.iter().any(|p| p.pattern == "os.Getenv"));
    }

    #[test]
    fn test_env_var_patterns_typescript() {
        let patterns = EnvVarPatterns::typescript();
        assert!(patterns.file_extensions.contains(&"ts"));
        assert!(patterns.file_extensions.contains(&"tsx"));
        assert!(patterns.patterns.iter().any(|p| p.pattern == "process.env"));
    }

    #[test]
    fn test_env_var_patterns_dart() {
        let patterns = EnvVarPatterns::dart();
        assert!(patterns.file_extensions.contains(&"dart"));
        assert!(patterns.patterns.iter().any(|p| p.pattern == "Platform.environment"));
    }

    // K021: config YAML への機密直書き禁止のテスト

    #[test]
    fn test_rule_id_k021() {
        assert_eq!(RuleId::SecretInConfig.as_str(), "K021");
        assert_eq!(
            RuleId::SecretInConfig.description(),
            "config YAML に機密情報が直接書かれています"
        );
    }

    #[test]
    fn test_lint_secret_in_config_detected() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // 完全な構造を作成
        create_test_manifest(path, "backend-rust");
        create_backend_rust_structure(path);

        // 機密情報を直接書いた config を作成
        let bad_config = r#"
db:
  host: localhost
  port: 5432
  user: myuser
  password: mysecretpassword123
"#;
        fs::write(path.join("config/default.yaml"), bad_config).unwrap();

        let linter = Linter::default_linter();
        let result = linter.lint(path);

        // K021 の違反が検出される
        assert!(
            result.violations.iter().any(|v| v.rule == RuleId::SecretInConfig),
            "Expected K021 violation, got {:?}",
            result.violations
        );

        // ヒントが含まれている
        let secret_violation = result
            .violations
            .iter()
            .find(|v| v.rule == RuleId::SecretInConfig)
            .unwrap();
        assert!(secret_violation.hint.is_some());
        assert!(secret_violation.hint.as_ref().unwrap().contains("_file"));
    }

    #[test]
    fn test_lint_secret_in_config_file_suffix_allowed() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // 完全な構造を作成
        create_test_manifest(path, "backend-rust");
        create_backend_rust_structure(path);

        // *_file サフィックスを使った正しい config を作成
        let good_config = r#"
db:
  host: localhost
  port: 5432
  user: myuser
  password_file: /var/run/secrets/k1s0/db_password
auth:
  jwt_private_key_file: /var/run/secrets/k1s0/jwt_private_key.pem
  jwt_public_key_file: /var/run/secrets/k1s0/jwt_public_key.pem
"#;
        fs::write(path.join("config/default.yaml"), good_config).unwrap();

        let linter = Linter::default_linter();
        let result = linter.lint(path);

        // K021 の違反が検出されない
        assert!(
            !result.violations.iter().any(|v| v.rule == RuleId::SecretInConfig),
            "Unexpected K021 violation: {:?}",
            result.violations
        );
    }

    #[test]
    fn test_lint_secret_in_config_token_detected() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // 完全な構造を作成
        create_test_manifest(path, "backend-rust");
        create_backend_rust_structure(path);

        // token を直接書いた config を作成
        let bad_config = r#"
integrations:
  github:
    api_token: ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
"#;
        fs::write(path.join("config/dev.yaml"), bad_config).unwrap();

        let linter = Linter::default_linter();
        let result = linter.lint(path);

        // K021 の違反が検出される
        assert!(
            result.violations.iter().any(|v| v.rule == RuleId::SecretInConfig),
            "Expected K021 violation for token, got {:?}",
            result.violations
        );
    }

    #[test]
    fn test_lint_secret_in_config_empty_value_allowed() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // 完全な構造を作成
        create_test_manifest(path, "backend-rust");
        create_backend_rust_structure(path);

        // 空値やnullは許可
        let config = r#"
db:
  password: null
  secret: ~
  token:
"#;
        fs::write(path.join("config/default.yaml"), config).unwrap();

        let linter = Linter::default_linter();
        let result = linter.lint(path);

        // K021 の違反が検出されない
        assert!(
            !result.violations.iter().any(|v| v.rule == RuleId::SecretInConfig),
            "Unexpected K021 violation for empty/null values: {:?}",
            result.violations
        );
    }

    #[test]
    fn test_lint_secret_in_config_exclude_rule() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // 完全な構造を作成
        create_test_manifest(path, "backend-rust");
        create_backend_rust_structure(path);

        // 機密情報を直接書いた config を作成
        let bad_config = r#"
db:
  password: mysecretpassword123
"#;
        fs::write(path.join("config/default.yaml"), bad_config).unwrap();

        // K021 を除外
        let config = LintConfig {
            rules: None,
            exclude_rules: vec!["K021".to_string()],
            strict: false,
            env_var_allowlist: vec![],
        };
        let linter = Linter::new(config);
        let result = linter.lint(path);

        // K021 の違反が検出されない
        assert!(
            !result.violations.iter().any(|v| v.rule == RuleId::SecretInConfig),
            "Unexpected K021 violation (rule should be excluded): {:?}",
            result.violations
        );
    }

    #[test]
    fn test_secret_key_patterns_matches() {
        let patterns = SecretKeyPatterns::default();

        // マッチするケース
        assert!(patterns.matches_secret_key("password").is_some());
        assert!(patterns.matches_secret_key("db_password").is_some());
        assert!(patterns.matches_secret_key("api_token").is_some());
        assert!(patterns.matches_secret_key("secret_key").is_some());
        assert!(patterns.matches_secret_key("jwt_private_key").is_some());
        assert!(patterns.matches_secret_key("client_secret").is_some());

        // マッチしないケース
        assert!(patterns.matches_secret_key("host").is_none());
        assert!(patterns.matches_secret_key("port").is_none());
        assert!(patterns.matches_secret_key("database_name").is_none());
        assert!(patterns.matches_secret_key("timeout").is_none());
    }

    #[test]
    fn test_parse_yaml_line() {
        // 正常なキー: 値
        assert_eq!(
            parse_yaml_line("key: value"),
            Some(("key".to_string(), "value".to_string()))
        );
        assert_eq!(
            parse_yaml_line("  password: secret123"),
            Some(("password".to_string(), "secret123".to_string()))
        );

        // 値が空
        assert_eq!(
            parse_yaml_line("token:"),
            Some(("token".to_string(), "".to_string()))
        );

        // コメント行
        assert_eq!(parse_yaml_line("# comment"), None);
        assert_eq!(parse_yaml_line("  # indented comment"), None);

        // 空行
        assert_eq!(parse_yaml_line(""), None);
        assert_eq!(parse_yaml_line("   "), None);

        // リスト項目
        assert_eq!(parse_yaml_line("- item"), None);
    }

    #[test]
    fn test_lint_secret_in_config_line_number() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // 完全な構造を作成
        create_test_manifest(path, "backend-rust");
        create_backend_rust_structure(path);

        // 機密情報を直接書いた config を作成（5行目）
        let bad_config = "# config\ndb:\n  host: localhost\n  port: 5432\n  password: secret123\n";
        fs::write(path.join("config/default.yaml"), bad_config).unwrap();

        let linter = Linter::default_linter();
        let result = linter.lint(path);

        // K021 の違反が検出され、行番号が正しい
        let secret_violation = result
            .violations
            .iter()
            .find(|v| v.rule == RuleId::SecretInConfig)
            .unwrap();
        assert_eq!(secret_violation.line, Some(5));
    }

    // K022: Clean Architecture 依存方向違反のテスト

    #[test]
    fn test_rule_id_k022() {
        assert_eq!(RuleId::DependencyDirection.as_str(), "K022");
        assert_eq!(
            RuleId::DependencyDirection.description(),
            "Clean Architecture の依存方向に違反しています"
        );
    }

    #[test]
    fn test_lint_dependency_direction_domain_to_infrastructure() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // 完全な構造を作成
        create_test_manifest(path, "backend-rust");
        create_backend_rust_structure(path);

        // domain から infrastructure への違反コードを追加
        let bad_code = r#"
// domain layer should not depend on infrastructure
use crate::infrastructure::db::UserRepository;

pub struct User {
    pub id: String,
    pub name: String,
}
"#;
        fs::write(path.join("src/domain/mod.rs"), bad_code).unwrap();

        let linter = Linter::default_linter();
        let result = linter.lint(path);

        // K022 の違反が検出される
        assert!(
            result.violations.iter().any(|v| v.rule == RuleId::DependencyDirection),
            "Expected K022 violation, got {:?}",
            result.violations
        );

        // ヒントが含まれている
        let dep_violation = result
            .violations
            .iter()
            .find(|v| v.rule == RuleId::DependencyDirection)
            .unwrap();
        assert!(dep_violation.hint.is_some());
        assert!(dep_violation.hint.as_ref().unwrap().contains("依存"));
    }

    #[test]
    fn test_lint_dependency_direction_domain_to_application() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // 完全な構造を作成
        create_test_manifest(path, "backend-rust");
        create_backend_rust_structure(path);

        // domain から application への違反コードを追加
        let bad_code = r#"
// domain layer should not depend on application
use crate::application::services::UserService;

pub struct User {
    pub id: String,
}
"#;
        fs::write(path.join("src/domain/mod.rs"), bad_code).unwrap();

        let linter = Linter::default_linter();
        let result = linter.lint(path);

        // K022 の違反が検出される
        assert!(
            result.violations.iter().any(|v| v.rule == RuleId::DependencyDirection),
            "Expected K022 violation for domain->application, got {:?}",
            result.violations
        );
    }

    #[test]
    fn test_lint_dependency_direction_application_to_infrastructure() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // 完全な構造を作成
        create_test_manifest(path, "backend-rust");
        create_backend_rust_structure(path);

        // application から infrastructure への違反コードを追加
        let bad_code = r#"
// application layer should not depend on infrastructure directly
use crate::infrastructure::db::UserRepositoryImpl;

pub struct UserService {}
"#;
        fs::write(path.join("src/application/mod.rs"), bad_code).unwrap();

        let linter = Linter::default_linter();
        let result = linter.lint(path);

        // K022 の違反が検出される
        assert!(
            result.violations.iter().any(|v| v.rule == RuleId::DependencyDirection),
            "Expected K022 violation for application->infrastructure, got {:?}",
            result.violations
        );
    }

    #[test]
    fn test_lint_dependency_direction_no_violation() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // 完全な構造を作成
        create_test_manifest(path, "backend-rust");
        create_backend_rust_structure(path);

        // 正しい依存関係のコード
        let domain_code = r#"
// domain layer - no external dependencies
pub struct User {
    pub id: String,
    pub name: String,
}

pub trait UserRepository {
    fn find_by_id(&self, id: &str) -> Option<User>;
}
"#;
        fs::write(path.join("src/domain/mod.rs"), domain_code).unwrap();

        let application_code = r#"
// application layer - depends only on domain
use crate::domain::User;
use crate::domain::UserRepository;

pub struct UserService<R: UserRepository> {
    repository: R,
}
"#;
        fs::write(path.join("src/application/mod.rs"), application_code).unwrap();

        let linter = Linter::default_linter();
        let result = linter.lint(path);

        // K022 の違反が検出されない
        assert!(
            !result.violations.iter().any(|v| v.rule == RuleId::DependencyDirection),
            "Unexpected K022 violation: {:?}",
            result.violations
        );
    }

    #[test]
    fn test_lint_dependency_direction_exclude_rule() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // 完全な構造を作成
        create_test_manifest(path, "backend-rust");
        create_backend_rust_structure(path);

        // domain から infrastructure への違反コードを追加
        let bad_code = r#"
use crate::infrastructure::db::UserRepository;
pub struct User {}
"#;
        fs::write(path.join("src/domain/mod.rs"), bad_code).unwrap();

        // K022 を除外
        let config = LintConfig {
            rules: None,
            exclude_rules: vec!["K022".to_string()],
            strict: false,
            env_var_allowlist: vec![],
        };
        let linter = Linter::new(config);
        let result = linter.lint(path);

        // K022 の違反が検出されない
        assert!(
            !result.violations.iter().any(|v| v.rule == RuleId::DependencyDirection),
            "Unexpected K022 violation (rule should be excluded): {:?}",
            result.violations
        );
    }

    #[test]
    fn test_dependency_rules_rust() {
        let rules = DependencyRules::rust();
        assert!(rules.file_extensions.contains(&"rs"));
        assert!(rules.import_patterns.iter().any(|p| p.contains("use crate::")));
    }

    #[test]
    fn test_dependency_rules_go() {
        let rules = DependencyRules::go();
        assert!(rules.file_extensions.contains(&"go"));
        assert!(rules.import_patterns.iter().any(|p| p.contains("internal/")));
    }

    #[test]
    fn test_dependency_rules_typescript() {
        let rules = DependencyRules::typescript();
        assert!(rules.file_extensions.contains(&"ts"));
        assert!(rules.file_extensions.contains(&"tsx"));
        assert!(rules.import_patterns.iter().any(|p| p.contains("from '../")));
    }

    #[test]
    fn test_dependency_rules_dart() {
        let rules = DependencyRules::dart();
        assert!(rules.file_extensions.contains(&"dart"));
    }

    #[test]
    fn test_lint_dependency_direction_line_number() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // 完全な構造を作成
        create_test_manifest(path, "backend-rust");
        create_backend_rust_structure(path);

        // domain から infrastructure への違反コード（3行目）
        let bad_code = "pub mod user;\n\nuse crate::infrastructure::db::Repo;\n\npub struct X {}\n";
        fs::write(path.join("src/domain/mod.rs"), bad_code).unwrap();

        let linter = Linter::default_linter();
        let result = linter.lint(path);

        // K022 の違反が検出され、行番号が正しい
        let dep_violation = result
            .violations
            .iter()
            .find(|v| v.rule == RuleId::DependencyDirection)
            .unwrap();
        assert_eq!(dep_violation.line, Some(3));
    }
}

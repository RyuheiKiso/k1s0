use std::path::Path;

use crate::manifest::Manifest;

use super::{LintConfig, LintResult, RequiredFiles, RuleId, Severity, Violation};

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

        // K030/K031/K032: gRPC リトライ設定検査
        let check_k030 = !self.is_rule_skipped(RuleId::RetryUsageDetected);
        let check_k031 = !self.is_rule_skipped(RuleId::RetryWithoutAdr);
        let check_k032 = !self.is_rule_skipped(RuleId::RetryConfigIncomplete);
        if check_k030 || check_k031 || check_k032 {
            self.check_retry_usage(&path, &mut result, check_k030, check_k031, check_k032);
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
}

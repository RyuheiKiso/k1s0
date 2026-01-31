use std::path::PathBuf;

use serde::{Deserialize, Serialize};

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
    /// 設定ファイル命名規約違反
    ConfigFileNaming,
    /// Domain 層でのプロトコル依存
    ProtocolDependencyInDomain,
    /// gRPC リトライ設定の検出（可視化目的）
    RetryUsageDetected,
    /// gRPC リトライ設定に ADR 参照がない
    RetryWithoutAdr,
    /// gRPC リトライ設定が不完全
    RetryConfigIncomplete,

    // === 層間依存関係ルール（K040-K047） ===

    /// 層間依存の基本違反（feature -> domain -> framework の順守）
    LayerDependencyViolation,
    /// feature が存在しない domain に依存
    DomainNotFound,
    /// domain バージョン制約の不整合
    DomainVersionMismatch,
    /// 循環依存の検出
    CircularDependency,
    /// 非推奨 domain の使用
    DeprecatedDomainUsage,
    /// min_framework_version 違反
    MinFrameworkVersionViolation,
    /// breaking_changes による影響警告
    BreakingChangeImpact,
    /// domain 層に version が未設定
    DomainVersionMissing,

    // === セキュリティルール（K050-K059） ===

    /// SQL インジェクションリスク
    SqlInjectionRisk,
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
            RuleId::ConfigFileNaming => "K025",
            RuleId::ProtocolDependencyInDomain => "K026",
            RuleId::RetryUsageDetected => "K030",
            RuleId::RetryWithoutAdr => "K031",
            RuleId::RetryConfigIncomplete => "K032",
            // 層間依存関係ルール
            RuleId::LayerDependencyViolation => "K040",
            RuleId::DomainNotFound => "K041",
            RuleId::DomainVersionMismatch => "K042",
            RuleId::CircularDependency => "K043",
            RuleId::DeprecatedDomainUsage => "K044",
            RuleId::MinFrameworkVersionViolation => "K045",
            RuleId::BreakingChangeImpact => "K046",
            RuleId::DomainVersionMissing => "K047",
            // セキュリティルール
            RuleId::SqlInjectionRisk => "K050",
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
            RuleId::ConfigFileNaming => "設定ファイルの命名規約に違反しています",
            RuleId::ProtocolDependencyInDomain => "Domain 層でプロトコル固有の型が使用されています",
            RuleId::RetryUsageDetected => "gRPC リトライ設定が検出されました",
            RuleId::RetryWithoutAdr => "gRPC リトライ設定に ADR 参照がありません",
            RuleId::RetryConfigIncomplete => "gRPC リトライ設定が不完全です",
            // 層間依存関係ルール
            RuleId::LayerDependencyViolation => "層間依存の方向に違反しています",
            RuleId::DomainNotFound => "指定された domain が見つかりません",
            RuleId::DomainVersionMismatch => "domain バージョン制約を満たしていません",
            RuleId::CircularDependency => "循環依存が検出されました",
            RuleId::DeprecatedDomainUsage => "非推奨の domain を使用しています",
            RuleId::MinFrameworkVersionViolation => "min_framework_version 要件を満たしていません",
            RuleId::BreakingChangeImpact => "破壊的変更の影響を受ける可能性があります",
            RuleId::DomainVersionMissing => "domain 層には version が必須です",
            // セキュリティルール
            RuleId::SqlInjectionRisk => "SQL インジェクションのリスクがあります",
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct LintConfig {
    /// 実行するルール（None の場合は全て）
    pub rules: Option<Vec<String>>,
    /// 除外するルール
    pub exclude_rules: Vec<String>,
    /// 警告をエラーとして扱う
    pub strict: bool,
    /// 環境変数参照を許可するファイルパス（glob パターン）
    pub env_var_allowlist: Vec<String>,
    /// 自動修正を試みる
    pub fix: bool,
}


/// 修正結果
#[derive(Debug, Clone)]
pub struct FixResult {
    /// 修正したファイルパス
    pub path: PathBuf,
    /// 修正の説明
    pub description: String,
    /// 成功したかどうか
    pub success: bool,
    /// エラーメッセージ（失敗時）
    pub error: Option<String>,
}

impl FixResult {
    /// 成功した修正結果を作成
    pub fn success(path: impl Into<PathBuf>, description: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            description: description.into(),
            success: true,
            error: None,
        }
    }

    /// 失敗した修正結果を作成
    pub fn failure(path: impl Into<PathBuf>, description: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            description: description.into(),
            success: false,
            error: Some(error.into()),
        }
    }
}

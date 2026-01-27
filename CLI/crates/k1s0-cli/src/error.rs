//! CLI エラー型と終了コード
//!
//! 失敗時に「原因/対象/次のアクション」を必ず出力するためのエラー型を定義する。

use std::fmt;
use std::path::{Path, PathBuf};

/// CLI の終了コード
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ExitCode {
    /// 成功
    Success = 0,
    /// 一般的なエラー
    GeneralError = 1,
    /// 使用方法のエラー（引数不正等）
    UsageError = 2,
    /// 設定エラー（manifest 不正、設定ファイル不正等）
    ConfigError = 3,
    /// IO エラー（ファイル読み書き失敗等）
    IoError = 4,
    /// バリデーションエラー（lint 失敗等）
    ValidationError = 5,
    /// 衝突エラー（ファイル衝突、既存ディレクトリ等）
    ConflictError = 6,
    /// ネットワークエラー
    NetworkError = 7,
    /// 内部エラー（バグ）
    InternalError = 99,
}

impl ExitCode {
    /// 終了コードを i32 に変換
    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

impl From<ExitCode> for std::process::ExitCode {
    fn from(code: ExitCode) -> Self {
        std::process::ExitCode::from(code.as_i32() as u8)
    }
}

/// CLI エラーの種類
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    /// 使用方法のエラー
    Usage,
    /// 設定エラー
    Config,
    /// IO エラー
    Io,
    /// バリデーションエラー
    Validation,
    /// 衝突エラー
    Conflict,
    /// ネットワークエラー
    Network,
    /// 内部エラー
    Internal,
}

impl ErrorKind {
    /// 対応する終了コードを取得
    pub fn exit_code(&self) -> ExitCode {
        match self {
            ErrorKind::Usage => ExitCode::UsageError,
            ErrorKind::Config => ExitCode::ConfigError,
            ErrorKind::Io => ExitCode::IoError,
            ErrorKind::Validation => ExitCode::ValidationError,
            ErrorKind::Conflict => ExitCode::ConflictError,
            ErrorKind::Network => ExitCode::NetworkError,
            ErrorKind::Internal => ExitCode::InternalError,
        }
    }

    /// エラー種別の表示名
    pub fn label(&self) -> &'static str {
        match self {
            ErrorKind::Usage => "使用方法エラー",
            ErrorKind::Config => "設定エラー",
            ErrorKind::Io => "IO エラー",
            ErrorKind::Validation => "バリデーションエラー",
            ErrorKind::Conflict => "衝突エラー",
            ErrorKind::Network => "ネットワークエラー",
            ErrorKind::Internal => "内部エラー",
        }
    }
}

/// CLI エラー
///
/// 失敗時に「原因/対象/次のアクション」を出力するための構造化エラー。
#[derive(Debug)]
pub struct CliError {
    /// エラーの種類
    pub kind: ErrorKind,
    /// エラーメッセージ（原因）
    pub message: String,
    /// 対象（ファイルパス、サービス名等）
    pub target: Option<String>,
    /// 次のアクション（ユーザーへの提案）
    pub hint: Option<String>,
    /// 元のエラー
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl CliError {
    /// 新しいエラーを作成
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            target: None,
            hint: None,
            source: None,
        }
    }

    /// 対象を設定
    pub fn with_target(mut self, target: impl Into<String>) -> Self {
        self.target = Some(target.into());
        self
    }

    /// 対象（パス）を設定
    pub fn with_path(self, path: &Path) -> Self {
        self.with_target(path.display().to_string())
    }

    /// 次のアクションを設定
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    /// 元のエラーを設定
    pub fn with_source(mut self, source: impl std::error::Error + Send + Sync + 'static) -> Self {
        self.source = Some(Box::new(source));
        self
    }

    /// 終了コードを取得
    pub fn exit_code(&self) -> ExitCode {
        self.kind.exit_code()
    }

    // --- ファクトリメソッド ---

    /// 使用方法エラー
    pub fn usage(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Usage, message)
    }

    /// 設定エラー
    pub fn config(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Config, message)
    }

    /// IO エラー
    pub fn io(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Io, message)
    }

    /// バリデーションエラー
    pub fn validation(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Validation, message)
    }

    /// 衝突エラー
    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Conflict, message)
    }

    /// ネットワークエラー
    pub fn network(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Network, message)
    }

    /// 内部エラー
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Internal, message)
    }

    /// manifest が見つからない
    pub fn manifest_not_found(path: &Path) -> Self {
        Self::config("manifest.json が見つかりません")
            .with_path(path)
            .with_hint("k1s0 init を実行してプロジェクトを初期化してください")
    }

    /// サービス名が不正
    pub fn invalid_service_name(name: &str) -> Self {
        Self::usage(format!("サービス名が不正です: {}", name))
            .with_hint("サービス名は kebab-case で指定してください（例: user-management）")
    }

    /// ディレクトリが既に存在
    pub fn directory_exists(path: &Path) -> Self {
        Self::conflict("ディレクトリが既に存在します")
            .with_path(path)
            .with_hint("--force オプションで上書きするか、別の名前を指定してください")
    }

    /// ファイルが見つからない
    pub fn file_not_found(path: &Path) -> Self {
        Self::io("ファイルが見つかりません").with_path(path)
    }

    /// テンプレートが見つからない
    pub fn template_not_found(name: &str) -> Self {
        Self::config(format!("テンプレートが見つかりません: {}", name))
            .with_hint("利用可能なテンプレート: backend-rust, backend-go, frontend-react, frontend-flutter")
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for CliError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|e| e.as_ref() as &(dyn std::error::Error + 'static))
    }
}

impl From<std::io::Error> for CliError {
    fn from(e: std::io::Error) -> Self {
        Self::io(e.to_string()).with_source(e)
    }
}

impl From<serde_json::Error> for CliError {
    fn from(e: serde_json::Error) -> Self {
        Self::config(format!("JSON パースエラー: {}", e)).with_source(e)
    }
}

impl From<serde_yaml::Error> for CliError {
    fn from(e: serde_yaml::Error) -> Self {
        Self::config(format!("YAML パースエラー: {}", e)).with_source(e)
    }
}

impl From<k1s0_generator::Error> for CliError {
    fn from(e: k1s0_generator::Error) -> Self {
        match &e {
            k1s0_generator::Error::ManifestNotFound(path) => {
                Self::manifest_not_found(&PathBuf::from(path))
            }
            k1s0_generator::Error::ManifestValidation(msg) => {
                Self::config(format!("manifest バリデーションエラー: {}", msg))
            }
            k1s0_generator::Error::TemplateNotFound(name) => Self::template_not_found(name),
            k1s0_generator::Error::FileConflict(path) => {
                Self::conflict("ファイルの衝突が検出されました")
                    .with_target(path.clone())
                    .with_hint("--force オプションで上書きするか、手動で解決してください")
            }
            _ => Self::internal(e.to_string()),
        }
    }
}

/// CLI の Result 型
pub type Result<T> = std::result::Result<T, CliError>;

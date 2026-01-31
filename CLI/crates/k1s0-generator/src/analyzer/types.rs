//! アナライザーで使用する共通型定義

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// 検出されたプロジェクトタイプ
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DetectedProjectType {
    /// Rust バックエンド
    BackendRust,
    /// Go バックエンド
    BackendGo,
    /// C# バックエンド
    BackendCsharp,
    /// Python バックエンド
    BackendPython,
    /// React フロントエンド
    FrontendReact,
    /// Flutter フロントエンド
    FrontendFlutter,
    /// 不明
    Unknown,
}

impl std::fmt::Display for DetectedProjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BackendRust => write!(f, "backend-rust"),
            Self::BackendGo => write!(f, "backend-go"),
            Self::BackendCsharp => write!(f, "backend-csharp"),
            Self::BackendPython => write!(f, "backend-python"),
            Self::FrontendReact => write!(f, "frontend-react"),
            Self::FrontendFlutter => write!(f, "frontend-flutter"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

/// アナライザー設定
#[derive(Debug, Clone)]
pub struct AnalyzerConfig {
    /// 分析対象パス
    pub path: PathBuf,
    /// プロジェクトタイプ（明示指定）
    pub project_type: Option<DetectedProjectType>,
    /// 詳細モード
    pub verbose: bool,
}

/// 構造分析結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructureAnalysis {
    /// 存在するディレクトリ
    pub existing_dirs: Vec<String>,
    /// 不足しているディレクトリ
    pub missing_dirs: Vec<String>,
    /// 存在するファイル
    pub existing_files: Vec<String>,
    /// 不足しているファイル
    pub missing_files: Vec<String>,
    /// Clean Architecture 層の検出状況
    pub detected_layers: Vec<String>,
    /// 不足している層
    pub missing_layers: Vec<String>,
}

/// 規約違反
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    /// ルール ID（例: K020）
    pub rule_id: String,
    /// 重要度
    pub severity: ViolationSeverity,
    /// 説明
    pub message: String,
    /// 対象ファイル
    pub file: Option<String>,
    /// 行番号
    pub line: Option<usize>,
    /// 自動修正可能か
    pub auto_fixable: bool,
}

/// 違反の重要度
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ViolationSeverity {
    /// エラー
    Error,
    /// 警告
    Warning,
}

impl std::fmt::Display for ViolationSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Error => write!(f, "error"),
            Self::Warning => write!(f, "warning"),
        }
    }
}

/// 依存関係分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyAnalysis {
    /// 環境変数の使用箇所
    pub env_var_usages: Vec<EnvVarUsage>,
    /// ハードコードされたシークレット
    pub hardcoded_secrets: Vec<SecretUsage>,
    /// 外部依存関係
    pub external_dependencies: Vec<String>,
    /// .env ファイルの検出
    pub env_files: Vec<String>,
}

/// 環境変数使用箇所
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVarUsage {
    /// ファイルパス
    pub file: String,
    /// 行番号
    pub line: usize,
    /// マッチしたパターン
    pub pattern: String,
    /// 変数名（判別できた場合）
    pub var_name: Option<String>,
}

/// シークレット使用箇所
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretUsage {
    /// ファイルパス
    pub file: String,
    /// 行番号
    pub line: usize,
    /// シークレットの種類
    pub kind: String,
}

/// コンプライアンススコア
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceScores {
    /// 構造スコア（0-100）
    pub structure: u32,
    /// 規約スコア（0-100）
    pub convention: u32,
    /// 依存関係スコア（0-100）
    pub dependency: u32,
    /// 総合スコア（0-100）
    pub overall: u32,
}

/// 分析結果の全体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    /// 検出されたプロジェクトタイプ
    pub project_type: DetectedProjectType,
    /// 構造分析
    pub structure: StructureAnalysis,
    /// 規約違反一覧
    pub violations: Vec<Violation>,
    /// 依存関係分析
    pub dependencies: DependencyAnalysis,
    /// スコア
    pub scores: ComplianceScores,
}

/// 移行プラン
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationPlan {
    /// プラン名
    pub name: String,
    /// 対象プロジェクトタイプ
    pub project_type: DetectedProjectType,
    /// 元パス
    pub source_path: String,
    /// フェーズ一覧
    pub phases: Vec<MigrationPhase>,
    /// 分析時のスコア
    pub scores_before: ComplianceScores,
    /// 適用後の予想スコア
    pub scores_after: Option<ComplianceScores>,
    /// 作成日時
    pub created_at: String,
}

impl MigrationPlan {
    /// 全ステップ数を返す
    pub fn total_steps(&self) -> usize {
        self.phases.iter().map(|p| p.steps.len()).sum()
    }

    /// 完了ステップ数を返す
    pub fn completed_steps(&self) -> usize {
        self.phases
            .iter()
            .flat_map(|p| &p.steps)
            .filter(|s| s.status == StepStatus::Completed)
            .count()
    }

    /// スキップされたステップ数を返す
    pub fn skipped_steps(&self) -> usize {
        self.phases
            .iter()
            .flat_map(|p| &p.steps)
            .filter(|s| s.status == StepStatus::Skipped)
            .count()
    }
}

/// 移行フェーズ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationPhase {
    /// フェーズ番号（1始まり）
    pub number: usize,
    /// フェーズ名
    pub name: String,
    /// 説明
    pub description: String,
    /// ステップ一覧
    pub steps: Vec<MigrationStep>,
}

/// 移行ステップ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationStep {
    /// ステップ ID
    pub id: String,
    /// 説明
    pub description: String,
    /// アクション
    pub action: MigrationAction,
    /// リスク
    pub risk: StepRisk,
    /// 状態
    pub status: StepStatus,
    /// エラーメッセージ（失敗時）
    pub error: Option<String>,
}

/// 移行アクション
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MigrationAction {
    /// バックアップ作成
    Backup {
        /// コピー元
        source: String,
        /// コピー先
        destination: String,
    },
    /// ディレクトリ作成
    CreateDirectory {
        /// 作成するパス
        path: String,
    },
    /// ディレクトリ移動
    MoveDirectory {
        /// 移動元
        source: String,
        /// 移動先
        destination: String,
    },
    /// ファイル移動
    MoveFile {
        /// 移動元
        source: String,
        /// 移動先
        destination: String,
    },
    /// ファイル生成
    GenerateFile {
        /// 生成先パス
        path: String,
        /// ファイル内容
        content: String,
    },
    /// ファイル内の文字列置換
    ReplaceInFile {
        /// 対象ファイル
        path: String,
        /// 検索文字列
        search: String,
        /// 置換文字列
        replace: String,
    },
    /// ファイル削除
    DeleteFile {
        /// 削除するパス
        path: String,
    },
    /// コマンド実行
    RunCommand {
        /// コマンド
        command: String,
        /// 引数
        args: Vec<String>,
        /// 作業ディレクトリ
        working_dir: Option<String>,
    },
    /// 手動アクション（自動実行不可）
    ManualAction {
        /// 手順の説明
        instruction: String,
    },
}

/// ステップのリスク
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StepRisk {
    /// 低リスク（新規作成のみ）
    Low,
    /// 中リスク（ファイル移動等）
    Medium,
    /// 高リスク（ファイル削除、内容変更等）
    High,
}

impl std::fmt::Display for StepRisk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
        }
    }
}

/// ステップの状態
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StepStatus {
    /// 未実行
    Pending,
    /// 完了
    Completed,
    /// スキップ
    Skipped,
    /// 失敗
    Failed,
}

impl std::fmt::Display for StepStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Completed => write!(f, "completed"),
            Self::Skipped => write!(f, "skipped"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

/// .env ファイルのエントリ
#[derive(Debug, Clone)]
pub struct EnvEntry {
    /// 変数名
    pub key: String,
    /// 値
    pub value: String,
    /// コメント
    pub comment: Option<String>,
}

/// YAML 設定変換結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigConversion {
    /// 生成する YAML 内容
    pub yaml_content: String,
    /// シークレット参照に変換された項目
    pub secret_refs: Vec<String>,
    /// 変換された変数の数
    pub converted_count: usize,
}

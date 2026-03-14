// エージェント実行エンティティ
// エージェントの実行状態とステップ情報を管理する

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// ExecutionStatus はエージェント実行の状態を表す列挙型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    /// 実行待ち
    Pending,
    /// 実行中
    Running,
    /// 完了
    Completed,
    /// 失敗
    Failed,
    /// キャンセル済み
    Cancelled,
}

impl std::fmt::Display for ExecutionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Running => write!(f, "running"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
            Self::Cancelled => write!(f, "cancelled"),
        }
    }
}

/// Execution はエージェント実行の情報を保持する
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Execution {
    /// 実行の一意識別子
    pub id: String,
    /// 実行するエージェントのID
    pub agent_id: String,
    /// セッション識別子
    pub session_id: String,
    /// テナント識別子
    pub tenant_id: String,
    /// エージェントへの入力テキスト
    pub input: String,
    /// エージェントの出力テキスト
    pub output: Option<String>,
    /// 実行状態
    pub status: ExecutionStatus,
    /// 実行ステップのリスト
    pub steps: Vec<ExecutionStep>,
    /// 作成日時
    pub created_at: DateTime<Utc>,
    /// 更新日時
    pub updated_at: DateTime<Utc>,
}

/// ExecutionStep はエージェントが実行した個別ステップを表す
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStep {
    /// ステップのインデックス番号
    pub index: i32,
    /// ステップ種別（thinking, tool_call, output）
    pub step_type: String,
    /// ステップへの入力
    pub input: String,
    /// ステップの出力
    pub output: String,
    /// 使用されたツール名（tool_callの場合）
    pub tool_name: Option<String>,
    /// ステップの状態
    pub status: String,
}

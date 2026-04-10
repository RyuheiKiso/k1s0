// エージェント定義エンティティ
// AIエージェントの設定情報（モデル、プロンプト、ツールなど）を保持する

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// `AgentDefinition` はAIエージェントの定義を表す
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDefinition {
    /// エージェントの一意識別子
    pub id: String,
    /// エージェントの表示名
    pub name: String,
    /// エージェントの説明
    pub description: String,
    /// 使用するAIモデルのID
    pub model_id: String,
    /// システムプロンプト
    pub system_prompt: String,
    /// 利用可能なツール名のリスト
    pub tools: Vec<String>,
    /// `ReActループの最大ステップ数`
    pub max_steps: i32,
    /// エージェントが有効かどうか
    pub enabled: bool,
    /// 作成日時
    pub created_at: DateTime<Utc>,
    /// 更新日時
    pub updated_at: DateTime<Utc>,
}

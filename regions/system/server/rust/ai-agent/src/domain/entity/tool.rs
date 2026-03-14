// ツールエンティティ
// エージェントが使用可能なツールの定義を保持する

use serde::{Deserialize, Serialize};

/// Tool はエージェントが使用できるツールを表す
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// ツールの名前
    pub name: String,
    /// ツールの説明
    pub description: String,
    /// パラメータのOpenAPI JSONスキーマ
    pub parameters_schema: String,
}

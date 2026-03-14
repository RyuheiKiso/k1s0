// ツールレジストリ
// エージェントが使用可能なツールを管理し、LLM用のスキーマを生成する

use crate::domain::entity::Tool;

/// ToolRegistry はツールの登録・管理を行う
pub struct ToolRegistry {
    /// 登録されたツールのリスト
    tools: Vec<Tool>,
}

impl ToolRegistry {
    /// 新しいToolRegistryを生成する
    pub fn new() -> Self {
        Self { tools: Vec::new() }
    }

    /// ツールを登録する
    pub fn register(&mut self, tool: Tool) {
        self.tools.push(tool);
    }

    /// LLM向けのツールスキーマをJSON形式で取得する
    /// OpenAPI JSONスキーマをserde_json::Valueに変換して返す
    pub fn get_schema_for_llm(&self) -> Vec<serde_json::Value> {
        self.tools
            .iter()
            .filter_map(|tool| {
                // パラメータスキーマをJSONとしてパースする
                let params: serde_json::Value =
                    serde_json::from_str(&tool.parameters_schema).unwrap_or_default();
                Some(serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": tool.name,
                        "description": tool.description,
                        "parameters": params,
                    }
                }))
            })
            .collect()
    }

    /// 登録されたツール一覧を取得する
    pub fn tools(&self) -> &[Tool] {
        &self.tools
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_get_schema() {
        let mut registry = ToolRegistry::new();
        registry.register(Tool {
            name: "search".to_string(),
            description: "Search the web".to_string(),
            parameters_schema: r#"{"type":"object","properties":{"query":{"type":"string"}}}"#
                .to_string(),
        });

        let schemas = registry.get_schema_for_llm();
        assert_eq!(schemas.len(), 1);
        assert_eq!(schemas[0]["function"]["name"], "search");
    }
}

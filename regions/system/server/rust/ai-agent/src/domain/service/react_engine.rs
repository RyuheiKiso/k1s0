// ReActエンジン
// Thought→Action→Observationループでエージェントを実行する
// LLMに推論させ、ツール呼び出しを解析し、結果を観察するサイクルを繰り返す

use tracing::{info, warn};

use crate::domain::entity::{AgentDefinition, ExecutionStep};
use crate::domain::service::tool_registry::ToolRegistry;
use k1s0_bb_ai_client::traits::AiClient;
use k1s0_bb_ai_client::types::{ChatMessage, CompleteRequest};

/// `ReActEngine` はThought→Action→Observationループを実装するエンジン
pub struct ReActEngine {
    /// ツールレジストリ（登録済みツール管理）
    tool_registry: ToolRegistry,
}

impl ReActEngine {
    /// `新しいReActEngineを生成する`
    #[must_use] 
    pub fn new(tool_registry: ToolRegistry) -> Self {
        Self { tool_registry }
    }

    /// `エージェント定義と入力に基づいてReActループを実行する`
    /// `max_stepsに達するか、最終出力が得られるまでループする`
    // HIGH-001 監査対応: ReActエンジンのロジックは構造上行数が多くなるため許容する
    #[allow(clippy::too_many_lines)]
    pub async fn execute(
        &self,
        agent: &AgentDefinition,
        input: &str,
        ai_client: &dyn AiClient,
    ) -> anyhow::Result<Vec<ExecutionStep>> {
        let mut steps = Vec::new();
        let mut messages = vec![
            // システムプロンプトを設定する
            ChatMessage {
                role: "system".to_string(),
                content: agent.system_prompt.clone(),
            },
            // ユーザー入力を追加する
            ChatMessage {
                role: "user".to_string(),
                content: input.to_string(),
            },
        ];

        // ツールスキーマ情報をシステムメッセージに追加する
        let tool_schemas = self.tool_registry.get_schema_for_llm();
        if !tool_schemas.is_empty() {
            let tools_info = serde_json::to_string_pretty(&tool_schemas).unwrap_or_default();
            messages.push(ChatMessage {
                role: "system".to_string(),
                content: format!(
                    "You have access to the following tools:\n{tools_info}\n\
                     To use a tool, respond with JSON: {{\"action\": \"tool_name\", \"input\": {{...}}}}\n\
                     To give a final answer, respond with JSON: {{\"action\": \"final_answer\", \"output\": \"...\"}}"
                ),
            });
        }

        // ReActループ: max_stepsまで繰り返す
        for step_index in 0..agent.max_steps {
            info!(step = step_index, "executing ReAct step");

            // Thought: LLMに推論させる
            let request = CompleteRequest {
                model: agent.model_id.clone(),
                messages: messages.clone(),
                max_tokens: Some(2048),
                temperature: Some(0.7),
                stream: Some(false),
            };

            let response = match ai_client.complete(&request).await {
                Ok(resp) => resp,
                Err(e) => {
                    warn!(error = %e, "AI client error during ReAct step");
                    steps.push(ExecutionStep {
                        index: step_index,
                        step_type: "thinking".to_string(),
                        input: String::new(),
                        output: format!("Error: {e}"),
                        tool_name: None,
                        status: "failed".to_string(),
                    });
                    break;
                }
            };

            let content = response.content.clone();

            // レスポンスをJSON形式でパースしてアクションを判定する
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(action) = parsed.get("action").and_then(|a| a.as_str()) {
                    if action == "final_answer" {
                        // 最終回答の場合はループを終了する
                        let output = parsed
                            .get("output")
                            .and_then(|o| o.as_str())
                            .unwrap_or("")
                            .to_string();
                        steps.push(ExecutionStep {
                            index: step_index,
                            step_type: "output".to_string(),
                            input: String::new(),
                            output,
                            tool_name: None,
                            status: "completed".to_string(),
                        });
                        break;
                    }

                    // Action: ツール呼び出し
                    let tool_input = parsed
                        .get("input")
                        .map(std::string::ToString::to_string)
                        .unwrap_or_default();

                    // Observation: ツール実行結果（実際のツール実行はスタブ）
                    let observation = format!(
                        "Tool '{action}' called with input: {tool_input}. (Tool execution not yet implemented)"
                    );

                    steps.push(ExecutionStep {
                        index: step_index,
                        step_type: "tool_call".to_string(),
                        input: tool_input,
                        output: observation.clone(),
                        tool_name: Some(action.to_string()),
                        status: "completed".to_string(),
                    });

                    // ツール実行結果をメッセージに追加して次のステップに進む
                    messages.push(ChatMessage {
                        role: "assistant".to_string(),
                        content: content.clone(),
                    });
                    messages.push(ChatMessage {
                        role: "user".to_string(),
                        content: format!("Observation: {observation}"),
                    });
                    continue;
                }
            }

            // JSONパースできない場合は直接出力として扱う
            steps.push(ExecutionStep {
                index: step_index,
                step_type: "output".to_string(),
                input: String::new(),
                output: content.clone(),
                tool_name: None,
                status: "completed".to_string(),
            });

            // アシスタントの応答をメッセージに追加する
            messages.push(ChatMessage {
                role: "assistant".to_string(),
                content,
            });
            break;
        }

        Ok(steps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::Tool;

    #[test]
    fn test_react_engine_creation() {
        let registry = ToolRegistry::new();
        let _engine = ReActEngine::new(registry);
    }

    #[test]
    fn test_tool_registry_integration() {
        let mut registry = ToolRegistry::new();
        registry.register(Tool {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
            parameters_schema: r#"{"type":"object"}"#.to_string(),
        });
        let engine = ReActEngine::new(registry);
        assert_eq!(engine.tool_registry.tools().len(), 1);
    }
}

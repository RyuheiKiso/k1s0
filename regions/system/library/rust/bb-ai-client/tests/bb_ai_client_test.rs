#![allow(clippy::unwrap_used)]
// bb-ai-client の統合テスト
// InMemoryAiClient を使用して外部依存なしでテストする

use k1s0_bb_ai_client::{
    AiClient, ChatMessage, CompleteRequest, CompleteResponse, EmbedRequest, EmbedResponse,
    InMemoryAiClient, ModelInfo, Usage,
};

// InMemoryAiClient の complete が正しくレスポンスを返すこと
#[tokio::test]
async fn test_in_memory_complete() {
    let expected = CompleteResponse {
        id: "test-id".to_string(),
        model: "claude-3".to_string(),
        content: "Hello, world!".to_string(),
        usage: Usage {
            input_tokens: 5,
            output_tokens: 10,
        },
    };
    let client = InMemoryAiClient::new(vec![expected.clone()], vec![]);

    let req = CompleteRequest {
        model: "claude-3".to_string(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: "hi".to_string(),
        }],
        max_tokens: None,
        temperature: None,
        stream: None,
    };

    let res = client.complete(&req).await.unwrap();
    assert_eq!(res.id, "test-id");
    assert_eq!(res.content, "Hello, world!");
    assert_eq!(res.usage.input_tokens, 5);
    assert_eq!(res.usage.output_tokens, 10);
}

// InMemoryAiClient の embed が正しくレスポンスを返すこと
#[tokio::test]
async fn test_in_memory_embed() {
    let expected = EmbedResponse {
        model: "embed-v1".to_string(),
        embeddings: vec![vec![0.1, 0.2], vec![0.3, 0.4]],
    };
    let client = InMemoryAiClient::new(vec![], vec![expected]);

    let req = EmbedRequest {
        model: "embed-v1".to_string(),
        texts: vec!["hello".to_string(), "world".to_string()],
    };

    let res = client.embed(&req).await.unwrap();
    assert_eq!(res.model, "embed-v1");
    assert_eq!(res.embeddings.len(), 2);
}

// InMemoryAiClient の list_models がモデル一覧を返すこと
#[tokio::test]
async fn test_in_memory_list_models() {
    let models = vec![ModelInfo {
        id: "model-1".to_string(),
        name: "Model One".to_string(),
        description: "A test model".to_string(),
    }];
    let client = InMemoryAiClient::with_models(vec![], vec![], models);
    let result = client.list_models().await.unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].id, "model-1");
}

// レスポンスキューが空の場合に Unavailable エラーを返すこと
#[tokio::test]
async fn test_in_memory_empty_queue_returns_error() {
    let client = InMemoryAiClient::new(vec![], vec![]);
    let req = CompleteRequest {
        model: "test".to_string(),
        messages: vec![],
        max_tokens: None,
        temperature: None,
        stream: None,
    };
    let result = client.complete(&req).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(
        err,
        k1s0_bb_ai_client::AiClientError::Unavailable(_)
    ));
}

// AiClientError の Display が正しく動作すること
#[tokio::test]
async fn test_error_display() {
    let err = k1s0_bb_ai_client::AiClientError::HttpError("connection refused".to_string());
    assert!(err.to_string().contains("connection refused"));

    let err = k1s0_bb_ai_client::AiClientError::Unavailable("no responses".to_string());
    assert!(err.to_string().contains("no responses"));
}

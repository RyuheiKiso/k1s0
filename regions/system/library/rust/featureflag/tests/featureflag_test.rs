use k1s0_featureflag::{
    EvaluationContext, FeatureFlag, FeatureFlagClient, FeatureFlagError, FlagVariant,
    InMemoryFeatureFlagClient,
};

fn make_flag(key: &str, enabled: bool, variants: Vec<FlagVariant>) -> FeatureFlag {
    FeatureFlag {
        id: format!("id-{key}"),
        flag_key: key.to_string(),
        description: format!("Description for {key}"),
        enabled,
        variants,
    }
}

fn make_variant(name: &str, value: &str, weight: i32) -> FlagVariant {
    FlagVariant {
        name: name.to_string(),
        value: value.to_string(),
        weight,
    }
}

#[tokio::test]
async fn test_evaluate_enabled_flag() {
    let client = InMemoryFeatureFlagClient::new();
    client.set_flag(make_flag("feature-a", true, vec![])).await;

    let ctx = EvaluationContext::new();
    let result = client.evaluate("feature-a", &ctx).await.unwrap();

    assert_eq!(result.flag_key, "feature-a");
    assert!(result.enabled);
    assert_eq!(result.reason, "FLAG_ENABLED");
}

#[tokio::test]
async fn test_evaluate_disabled_flag() {
    let client = InMemoryFeatureFlagClient::new();
    client
        .set_flag(make_flag("feature-b", false, vec![]))
        .await;

    let ctx = EvaluationContext::new();
    let result = client.evaluate("feature-b", &ctx).await.unwrap();

    assert_eq!(result.flag_key, "feature-b");
    assert!(!result.enabled);
    assert_eq!(result.reason, "FLAG_DISABLED");
}

#[tokio::test]
async fn test_evaluate_nonexistent_flag_returns_error() {
    let client = InMemoryFeatureFlagClient::new();
    let ctx = EvaluationContext::new();

    let err = client.evaluate("no-such-flag", &ctx).await.unwrap_err();
    match err {
        FeatureFlagError::FlagNotFound { key } => assert_eq!(key, "no-such-flag"),
        _ => panic!("expected FlagNotFound error"),
    }
}

#[tokio::test]
async fn test_get_flag_by_key() {
    let client = InMemoryFeatureFlagClient::new();
    let variants = vec![make_variant("control", "off", 50)];
    client
        .set_flag(make_flag("feature-c", true, variants))
        .await;

    let flag = client.get_flag("feature-c").await.unwrap();
    assert_eq!(flag.flag_key, "feature-c");
    assert!(flag.enabled);
    assert_eq!(flag.variants.len(), 1);
    assert_eq!(flag.variants[0].name, "control");
}

#[tokio::test]
async fn test_is_enabled_true() {
    let client = InMemoryFeatureFlagClient::new();
    client.set_flag(make_flag("on-flag", true, vec![])).await;

    let ctx = EvaluationContext::new();
    assert!(client.is_enabled("on-flag", &ctx).await.unwrap());
}

#[tokio::test]
async fn test_is_enabled_false() {
    let client = InMemoryFeatureFlagClient::new();
    client.set_flag(make_flag("off-flag", false, vec![])).await;

    let ctx = EvaluationContext::new();
    assert!(!client.is_enabled("off-flag", &ctx).await.unwrap());
}

#[cfg(feature = "mock")]
#[tokio::test]
async fn test_mock_feature_flag_client() {
    use k1s0_featureflag::{EvaluationResult, MockFeatureFlagClient};

    let mut mock = MockFeatureFlagClient::new();
    mock.expect_evaluate()
        .withf(|key, _ctx| key == "mock-flag")
        .returning(|key, _ctx| {
            let flag_key = key.to_string();
            Box::pin(async move {
                Ok(EvaluationResult {
                    flag_key,
                    enabled: true,
                    variant: Some("treatment".to_string()),
                    reason: "MOCK".to_string(),
                })
            })
        });

    let ctx = EvaluationContext::new();
    let result = mock.evaluate("mock-flag", &ctx).await.unwrap();
    assert!(result.enabled);
    assert_eq!(result.variant, Some("treatment".to_string()));
}

#[tokio::test]
async fn test_evaluation_context_builder() {
    let ctx = EvaluationContext::new()
        .with_user_id("user-123")
        .with_tenant_id("tenant-abc")
        .with_attribute("role", "admin");

    assert_eq!(ctx.user_id, Some("user-123".to_string()));
    assert_eq!(ctx.tenant_id, Some("tenant-abc".to_string()));
    assert_eq!(ctx.attributes.get("role"), Some(&"admin".to_string()));
}

#[tokio::test]
async fn test_set_flag_and_retrieve() {
    let client = InMemoryFeatureFlagClient::new();

    // Initially the flag does not exist
    assert!(client.get_flag("dynamic").await.is_err());

    // Set and retrieve
    client.set_flag(make_flag("dynamic", true, vec![])).await;
    let flag = client.get_flag("dynamic").await.unwrap();
    assert_eq!(flag.flag_key, "dynamic");
    assert!(flag.enabled);
}

#[tokio::test]
async fn test_variant_in_evaluation_result() {
    let client = InMemoryFeatureFlagClient::new();
    let variants = vec![
        make_variant("control", "off", 50),
        make_variant("treatment", "on", 50),
    ];
    client
        .set_flag(make_flag("ab-test", true, variants))
        .await;

    let ctx = EvaluationContext::new();
    let result = client.evaluate("ab-test", &ctx).await.unwrap();

    assert!(result.enabled);
    // InMemoryFeatureFlagClient returns the first variant
    assert_eq!(result.variant, Some("control".to_string()));
}

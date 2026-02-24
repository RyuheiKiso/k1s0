use async_graphql::SimpleObject;

#[derive(Debug, Clone, SimpleObject)]
pub struct FeatureFlag {
    pub key: String,
    pub name: String,
    pub enabled: bool,
    pub rollout_percentage: i32,
    pub target_environments: Vec<String>,
}

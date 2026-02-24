use async_graphql::SimpleObject;

#[derive(Debug, Clone, SimpleObject)]
pub struct ConfigEntry {
    pub key: String,
    pub value: String,
    pub updated_at: String,
}

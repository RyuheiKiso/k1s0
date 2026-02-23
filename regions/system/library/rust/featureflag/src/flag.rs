use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagVariant {
    pub name: String,
    pub value: String,
    pub weight: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlag {
    pub id: String,
    pub flag_key: String,
    pub description: String,
    pub enabled: bool,
    pub variants: Vec<FlagVariant>,
}

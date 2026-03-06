use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DomainFilter {
    All,
    System,
    Domain(String),
}

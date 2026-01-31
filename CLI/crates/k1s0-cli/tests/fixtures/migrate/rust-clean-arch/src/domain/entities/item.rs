use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: Option<i32>,
    pub name: String,
    pub description: Option<String>,
}

impl Item {
    pub fn new(name: String, description: Option<String>) -> Self {
        Self {
            id: None,
            name,
            description,
        }
    }
}

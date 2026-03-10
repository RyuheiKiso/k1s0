pub mod config_schema;
pub mod navigation;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValidationDiagnostic {
    pub rule: String,
    pub path: String,
    pub message: String,
    pub line: Option<usize>,
}

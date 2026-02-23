use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct EvaluationContext {
    pub user_id: Option<String>,
    pub tenant_id: Option<String>,
    pub attributes: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct EvaluationResult {
    pub flag_key: String,
    pub enabled: bool,
    pub variant: Option<String>,
    pub reason: String,
}

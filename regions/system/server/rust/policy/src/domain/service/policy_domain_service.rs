pub struct PolicyDomainService;

impl PolicyDomainService {
    pub fn validate_policy_name(name: &str) -> Result<(), String> {
        if name.trim().is_empty() {
            return Err("policy name is required".to_string());
        }
        if name.len() > 128 {
            return Err("policy name must be 128 characters or fewer".to_string());
        }
        Ok(())
    }

    pub fn validate_rego_content(rego_content: &str) -> Result<(), String> {
        let content = rego_content.trim();
        if content.is_empty() {
            return Err("rego_content is required".to_string());
        }
        if !content.contains("package ") {
            return Err("rego_content must contain a package declaration".to_string());
        }
        Ok(())
    }

    /// パッケージパスを正規化する（前後の空白・スラッシュを除去）
    /// H-001 監査対応: 正規化後のパスは OpaClient::validate_package_path で検証される
    pub fn normalize_package_path(package_path: &str) -> String {
        package_path.trim().trim_matches('/').to_string()
    }

    pub fn can_evaluate_policy(enabled: bool) -> bool {
        enabled
    }
}

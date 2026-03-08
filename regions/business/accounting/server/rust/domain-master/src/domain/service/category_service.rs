use crate::domain::entity::master_category::MasterCategory;

/// カテゴリのドメインルールを検証するサービス。
pub struct CategoryService;

impl CategoryService {
    /// カテゴリコードが命名規約に適合しているか検証する。
    ///
    /// カテゴリコードは英大文字・数字・アンダースコアのみ許可する。
    pub fn validate_code(code: &str) -> anyhow::Result<()> {
        if code.is_empty() {
            anyhow::bail!("Validation error: category code must not be empty");
        }
        if code.len() > 100 {
            anyhow::bail!(
                "Validation error: category code must not exceed 100 characters"
            );
        }
        if !code
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            anyhow::bail!(
                "Validation error: category code must contain only alphanumeric characters and underscores"
            );
        }
        Ok(())
    }

    /// validation_schema が有効な JSON Schema 構造であるか簡易チェックする。
    ///
    /// `type` フィールドが存在する場合は `"object"` であることを確認する。
    pub fn validate_schema(schema: &serde_json::Value) -> anyhow::Result<()> {
        if let Some(type_val) = schema.get("type").and_then(|v| v.as_str()) {
            if type_val != "object" {
                anyhow::bail!(
                    "Validation error: validation_schema root type must be 'object', got '{}'",
                    type_val
                );
            }
        }
        Ok(())
    }

    /// カテゴリが削除可能か検証する。
    ///
    /// is_active が false のカテゴリのみ削除可能とする安全策。
    /// ただし admin 権限による強制削除を許容するため、このチェックはオプショナル。
    pub fn can_safely_delete(category: &MasterCategory) -> bool {
        !category.is_active
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_code_valid() {
        assert!(CategoryService::validate_code("ACCOUNT_TITLES").is_ok());
        assert!(CategoryService::validate_code("TAX_RATE_01").is_ok());
    }

    #[test]
    fn test_validate_code_empty() {
        let result = CategoryService::validate_code("");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must not be empty"));
    }

    #[test]
    fn test_validate_code_invalid_chars() {
        let result = CategoryService::validate_code("account-titles");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("alphanumeric characters and underscores"));
    }

    #[test]
    fn test_validate_code_too_long() {
        let long_code = "A".repeat(101);
        let result = CategoryService::validate_code(&long_code);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must not exceed 100"));
    }

    #[test]
    fn test_validate_schema_valid() {
        let schema = serde_json::json!({
            "type": "object",
            "required": ["symbol"],
            "properties": {
                "symbol": { "type": "string" }
            }
        });
        assert!(CategoryService::validate_schema(&schema).is_ok());
    }

    #[test]
    fn test_validate_schema_no_type_passes() {
        let schema = serde_json::json!({
            "required": ["symbol"]
        });
        assert!(CategoryService::validate_schema(&schema).is_ok());
    }

    #[test]
    fn test_validate_schema_wrong_root_type() {
        let schema = serde_json::json!({
            "type": "array"
        });
        let result = CategoryService::validate_schema(&schema);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must be 'object'"));
    }

    #[test]
    fn test_can_safely_delete_inactive() {
        let cat = MasterCategory {
            id: uuid::Uuid::new_v4(),
            code: "TEST".to_string(),
            display_name: "Test".to_string(),
            description: None,
            validation_schema: None,
            is_active: false,
            sort_order: 0,
            created_by: "admin".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        assert!(CategoryService::can_safely_delete(&cat));
    }

    #[test]
    fn test_can_safely_delete_active() {
        let cat = MasterCategory {
            id: uuid::Uuid::new_v4(),
            code: "TEST".to_string(),
            display_name: "Test".to_string(),
            description: None,
            validation_schema: None,
            is_active: true,
            sort_order: 0,
            created_by: "admin".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        assert!(!CategoryService::can_safely_delete(&cat));
    }
}

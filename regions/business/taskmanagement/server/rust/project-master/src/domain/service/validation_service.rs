// バリデーションサービス。入力値のドメインバリデーションを行う。
use crate::domain::entity::project_type::CreateProjectType;
use crate::domain::entity::status_definition::CreateStatusDefinition;
use crate::domain::error::ProjectMasterError;

pub struct ValidationService;

impl ValidationService {
    /// プロジェクトタイプ作成バリデーション
    pub fn validate_create_project_type(
        input: &CreateProjectType,
    ) -> Result<(), ProjectMasterError> {
        if input.code.trim().is_empty() {
            return Err(ProjectMasterError::ValidationFailed(
                "code must not be empty".to_string(),
            ));
        }
        if input.display_name.trim().is_empty() {
            return Err(ProjectMasterError::ValidationFailed(
                "display_name must not be empty".to_string(),
            ));
        }
        Ok(())
    }

    /// ステータス定義作成バリデーション
    pub fn validate_create_status_definition(
        input: &CreateStatusDefinition,
    ) -> Result<(), ProjectMasterError> {
        if input.code.trim().is_empty() {
            return Err(ProjectMasterError::ValidationFailed(
                "code must not be empty".to_string(),
            ));
        }
        if input.display_name.trim().is_empty() {
            return Err(ProjectMasterError::ValidationFailed(
                "display_name must not be empty".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use uuid::Uuid;

    // プロジェクトタイプ作成バリデーション：有効な入力が成功することを確認する
    // 前提: code と display_name が空でない文字列
    // 期待: Ok(()) が返される
    #[test]
    fn test_validate_create_project_type_success() {
        let input = CreateProjectType {
            code: "SOFTWARE".to_string(),
            display_name: "ソフトウェア開発".to_string(),
            description: None,
            default_workflow: None,
            is_active: None,
            sort_order: None,
        };
        let result = ValidationService::validate_create_project_type(&input);
        assert!(result.is_ok());
    }

    // プロジェクトタイプ作成バリデーション：codeが空の場合エラーになることを確認する
    // 前提: code が空文字列
    // 期待: ValidationFailed エラーが返される
    #[test]
    fn test_validate_create_project_type_empty_code() {
        let input = CreateProjectType {
            code: "".to_string(),
            display_name: "ソフトウェア開発".to_string(),
            description: None,
            default_workflow: None,
            is_active: None,
            sort_order: None,
        };
        let result = ValidationService::validate_create_project_type(&input);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ProjectMasterError::ValidationFailed(_)));
    }

    // プロジェクトタイプ作成バリデーション：codeがスペースのみの場合エラーになることを確認する
    // 前提: code がスペースのみ
    // 期待: ValidationFailed エラーが返される
    #[test]
    fn test_validate_create_project_type_whitespace_code() {
        let input = CreateProjectType {
            code: "   ".to_string(),
            display_name: "有効な表示名".to_string(),
            description: None,
            default_workflow: None,
            is_active: None,
            sort_order: None,
        };
        let result = ValidationService::validate_create_project_type(&input);
        assert!(result.is_err());
    }

    // プロジェクトタイプ作成バリデーション：display_nameが空の場合エラーになることを確認する
    // 前提: display_name が空文字列
    // 期待: ValidationFailed エラーが返される
    #[test]
    fn test_validate_create_project_type_empty_display_name() {
        let input = CreateProjectType {
            code: "SOFTWARE".to_string(),
            display_name: "".to_string(),
            description: None,
            default_workflow: None,
            is_active: None,
            sort_order: None,
        };
        let result = ValidationService::validate_create_project_type(&input);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ProjectMasterError::ValidationFailed(_)));
    }

    // ステータス定義作成バリデーション：有効な入力が成功することを確認する
    // 前提: code と display_name が空でない文字列、project_type_id が有効なUUID
    // 期待: Ok(()) が返される
    #[test]
    fn test_validate_create_status_definition_success() {
        let input = CreateStatusDefinition {
            project_type_id: Uuid::new_v4(),
            code: "OPEN".to_string(),
            display_name: "オープン".to_string(),
            description: None,
            color: None,
            allowed_transitions: None,
            is_initial: None,
            is_terminal: None,
            sort_order: None,
        };
        let result = ValidationService::validate_create_status_definition(&input);
        assert!(result.is_ok());
    }

    // ステータス定義作成バリデーション：codeが空の場合エラーになることを確認する
    // 前提: code が空文字列
    // 期待: ValidationFailed エラーが返される
    #[test]
    fn test_validate_create_status_definition_empty_code() {
        let input = CreateStatusDefinition {
            project_type_id: Uuid::new_v4(),
            code: "".to_string(),
            display_name: "有効な名前".to_string(),
            description: None,
            color: None,
            allowed_transitions: None,
            is_initial: None,
            is_terminal: None,
            sort_order: None,
        };
        let result = ValidationService::validate_create_status_definition(&input);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ProjectMasterError::ValidationFailed(_)));
    }

    // ステータス定義作成バリデーション：display_nameが空の場合エラーになることを確認する
    // 前提: display_name が空文字列
    // 期待: ValidationFailed エラーが返される
    #[test]
    fn test_validate_create_status_definition_empty_display_name() {
        let input = CreateStatusDefinition {
            project_type_id: Uuid::new_v4(),
            code: "OPEN".to_string(),
            display_name: "".to_string(),
            description: None,
            color: None,
            allowed_transitions: None,
            is_initial: None,
            is_terminal: None,
            sort_order: None,
        };
        let result = ValidationService::validate_create_status_definition(&input);
        assert!(result.is_err());
    }

    // ステータス定義作成バリデーション：display_nameがスペースのみの場合エラーになることを確認する
    // 前提: display_name がスペースのみ
    // 期待: ValidationFailed エラーが返される
    #[test]
    fn test_validate_create_status_definition_whitespace_display_name() {
        let input = CreateStatusDefinition {
            project_type_id: Uuid::new_v4(),
            code: "DONE".to_string(),
            display_name: "   ".to_string(),
            description: None,
            color: None,
            allowed_transitions: None,
            is_initial: None,
            is_terminal: None,
            sort_order: None,
        };
        let result = ValidationService::validate_create_status_definition(&input);
        assert!(result.is_err());
    }
}

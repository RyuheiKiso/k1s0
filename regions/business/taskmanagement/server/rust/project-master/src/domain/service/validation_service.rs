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

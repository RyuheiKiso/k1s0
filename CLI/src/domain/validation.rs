use super::model::{ProjectType, Template};

#[derive(Debug, PartialEq)]
pub enum ValidationError {
    EmptyName,
    TooLong(usize),
    InvalidCharacters(String),
    IncompatibleTemplate { template: Template, project_type: ProjectType },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::EmptyName => write!(f, "Project name cannot be empty"),
            ValidationError::TooLong(max) => write!(f, "Project name must be {max} characters or less"),
            ValidationError::InvalidCharacters(name) => {
                write!(f, "Project name '{name}' contains invalid characters. Use only alphanumeric, hyphens, and underscores")
            }
            ValidationError::IncompatibleTemplate { template, project_type } => {
                write!(f, "Template '{template}' is not compatible with project type '{project_type}'")
            }
        }
    }
}

const MAX_PROJECT_NAME_LENGTH: usize = 64;

pub fn validate_project_name(name: &str) -> Result<(), ValidationError> {
    if name.is_empty() {
        return Err(ValidationError::EmptyName);
    }

    if name.len() > MAX_PROJECT_NAME_LENGTH {
        return Err(ValidationError::TooLong(MAX_PROJECT_NAME_LENGTH));
    }

    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        return Err(ValidationError::InvalidCharacters(name.to_string()));
    }

    Ok(())
}

pub fn validate_template_compatibility(
    template: &Template,
    project_type: &ProjectType,
) -> Result<(), ValidationError> {
    if template.is_compatible_with(project_type) {
        Ok(())
    } else {
        Err(ValidationError::IncompatibleTemplate {
            template: template.clone(),
            project_type: project_type.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_project_name() {
        assert!(validate_project_name("my-app").is_ok());
        assert!(validate_project_name("my_app").is_ok());
        assert!(validate_project_name("myapp123").is_ok());
        assert!(validate_project_name("a").is_ok());
        assert!(validate_project_name("my-cool-app_v2").is_ok());
    }

    #[test]
    fn test_empty_project_name() {
        assert_eq!(validate_project_name(""), Err(ValidationError::EmptyName));
    }

    #[test]
    fn test_project_name_with_special_characters() {
        assert!(matches!(
            validate_project_name("my app"),
            Err(ValidationError::InvalidCharacters(_))
        ));
        assert!(matches!(
            validate_project_name("my.app"),
            Err(ValidationError::InvalidCharacters(_))
        ));
        assert!(matches!(
            validate_project_name("my@app"),
            Err(ValidationError::InvalidCharacters(_))
        ));
        assert!(matches!(
            validate_project_name("my/app"),
            Err(ValidationError::InvalidCharacters(_))
        ));
    }

    #[test]
    fn test_project_name_too_long() {
        let long_name = "a".repeat(65);
        assert_eq!(
            validate_project_name(&long_name),
            Err(ValidationError::TooLong(64))
        );
    }

    #[test]
    fn test_project_name_max_length_ok() {
        let name = "a".repeat(64);
        assert!(validate_project_name(&name).is_ok());
    }

    #[test]
    fn test_template_compatibility_valid() {
        assert!(validate_template_compatibility(&Template::React, &ProjectType::Frontend).is_ok());
        assert!(validate_template_compatibility(&Template::Flutter, &ProjectType::Frontend).is_ok());
        assert!(validate_template_compatibility(&Template::RustAxum, &ProjectType::Backend).is_ok());
        assert!(validate_template_compatibility(&Template::GoGin, &ProjectType::Backend).is_ok());
    }

    #[test]
    fn test_template_compatibility_invalid() {
        assert_eq!(
            validate_template_compatibility(&Template::RustAxum, &ProjectType::Frontend),
            Err(ValidationError::IncompatibleTemplate {
                template: Template::RustAxum,
                project_type: ProjectType::Frontend,
            })
        );
        assert_eq!(
            validate_template_compatibility(&Template::React, &ProjectType::Backend),
            Err(ValidationError::IncompatibleTemplate {
                template: Template::React,
                project_type: ProjectType::Backend,
            })
        );
    }

    #[test]
    fn test_validation_error_display() {
        assert_eq!(
            ValidationError::EmptyName.to_string(),
            "Project name cannot be empty"
        );
        assert_eq!(
            ValidationError::TooLong(64).to_string(),
            "Project name must be 64 characters or less"
        );
    }
}

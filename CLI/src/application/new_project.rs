use std::path::PathBuf;

use crate::domain::model::{Database, ProjectConfig, ProjectType, Template};
use crate::domain::validation::{validate_project_name, validate_template_compatibility};
use super::interactive::{FileGenerator, UserPrompt};

pub struct NewProjectArgs {
    pub project_type: Option<ProjectType>,
    pub template: Option<String>,
    pub name: Option<String>,
    pub db: Option<String>,
    pub path: Option<PathBuf>,
    pub yes: bool,
}

pub struct NewProjectUseCase<P: UserPrompt, G: FileGenerator> {
    prompt: P,
    generator: G,
}

impl<P: UserPrompt, G: FileGenerator> NewProjectUseCase<P, G> {
    pub fn new(prompt: P, generator: G) -> Self {
        Self { prompt, generator }
    }

    pub fn execute(&self, args: NewProjectArgs) -> Result<ProjectConfig, String> {
        let is_interactive = !args.yes
            && (args.project_type.is_none() || args.template.is_none() || args.name.is_none());

        let project_type = if let Some(pt) = args.project_type {
            pt
        } else if is_interactive {
            self.prompt.select_project_type()?
        } else {
            return Err("Project type is required in non-interactive mode".to_string());
        };

        let template = if let Some(t) = args.template {
            t.parse::<Template>().map_err(|e| e.to_string())?
        } else if is_interactive {
            self.prompt.select_template(&project_type)?
        } else {
            return Err("Template is required in non-interactive mode (use --template)".to_string());
        };

        validate_template_compatibility(&template, &project_type)
            .map_err(|e| e.to_string())?;

        let database = if let Some(db) = args.db {
            db.parse::<Database>().map_err(|e| e.to_string())?
        } else if matches!(project_type, ProjectType::Backend) && is_interactive {
            self.prompt.select_database()?
        } else {
            Database::None
        };

        let name = if let Some(n) = args.name {
            n
        } else if is_interactive {
            self.prompt.input_project_name(None)?
        } else {
            return Err("Project name is required in non-interactive mode (use --name)".to_string());
        };

        validate_project_name(&name).map_err(|e| e.to_string())?;

        let path = if let Some(p) = args.path {
            p.join(&name)
        } else if is_interactive {
            self.prompt.input_path(&std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))?
                .join(&name)
        } else {
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(&name)
        };

        let config = ProjectConfig {
            name,
            project_type,
            template,
            database,
            path,
        };

        if is_interactive && !args.yes {
            if !self.prompt.confirm(&config)? {
                return Err("Project creation cancelled by user".to_string());
            }
        }

        self.generator.generate(&config)?;

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct MockPrompt {
        project_type: ProjectType,
        name: String,
        template: Template,
        database: Database,
        path: PathBuf,
        confirm_result: bool,
        calls: RefCell<Vec<String>>,
    }

    impl MockPrompt {
        fn new() -> Self {
            Self {
                project_type: ProjectType::Frontend,
                name: "test-app".to_string(),
                template: Template::React,
                database: Database::None,
                path: PathBuf::from("/tmp"),
                confirm_result: true,
                calls: RefCell::new(Vec::new()),
            }
        }
    }

    impl UserPrompt for MockPrompt {
        fn select_project_type(&self) -> Result<ProjectType, String> {
            self.calls.borrow_mut().push("select_project_type".to_string());
            Ok(self.project_type.clone())
        }

        fn input_project_name(&self, _default: Option<&str>) -> Result<String, String> {
            self.calls.borrow_mut().push("input_project_name".to_string());
            Ok(self.name.clone())
        }

        fn select_template(&self, _project_type: &ProjectType) -> Result<Template, String> {
            self.calls.borrow_mut().push("select_template".to_string());
            Ok(self.template.clone())
        }

        fn select_database(&self) -> Result<Database, String> {
            self.calls.borrow_mut().push("select_database".to_string());
            Ok(self.database.clone())
        }

        fn input_path(&self, _default: &std::path::Path) -> Result<PathBuf, String> {
            self.calls.borrow_mut().push("input_path".to_string());
            Ok(self.path.clone())
        }

        fn confirm(&self, _config: &ProjectConfig) -> Result<bool, String> {
            self.calls.borrow_mut().push("confirm".to_string());
            Ok(self.confirm_result)
        }
    }

    struct MockGenerator {
        calls: RefCell<Vec<ProjectConfig>>,
    }

    impl MockGenerator {
        fn new() -> Self {
            Self {
                calls: RefCell::new(Vec::new()),
            }
        }
    }

    impl FileGenerator for MockGenerator {
        fn generate(&self, config: &ProjectConfig) -> Result<(), String> {
            self.calls.borrow_mut().push(config.clone());
            Ok(())
        }
    }

    #[test]
    fn test_non_interactive_all_args_frontend() {
        let prompt = MockPrompt::new();
        let generator = MockGenerator::new();
        let use_case = NewProjectUseCase::new(prompt, generator);

        let args = NewProjectArgs {
            project_type: Some(ProjectType::Frontend),
            template: Some("react".to_string()),
            name: Some("my-app".to_string()),
            db: None,
            path: Some(PathBuf::from("/tmp")),
            yes: true,
        };

        let config = use_case.execute(args).unwrap();
        assert_eq!(config.name, "my-app");
        assert_eq!(config.template, Template::React);
        assert_eq!(config.project_type, ProjectType::Frontend);
        assert_eq!(config.database, Database::None);
        assert_eq!(config.path, PathBuf::from("/tmp/my-app"));
    }

    #[test]
    fn test_non_interactive_all_args_backend_with_db() {
        let prompt = MockPrompt::new();
        let generator = MockGenerator::new();
        let use_case = NewProjectUseCase::new(prompt, generator);

        let args = NewProjectArgs {
            project_type: Some(ProjectType::Backend),
            template: Some("rust".to_string()),
            name: Some("my-service".to_string()),
            db: Some("postgresql".to_string()),
            path: Some(PathBuf::from("/tmp")),
            yes: true,
        };

        let config = use_case.execute(args).unwrap();
        assert_eq!(config.name, "my-service");
        assert_eq!(config.template, Template::RustAxum);
        assert_eq!(config.database, Database::PostgreSql);
    }

    #[test]
    fn test_non_interactive_missing_name_error() {
        let prompt = MockPrompt::new();
        let generator = MockGenerator::new();
        let use_case = NewProjectUseCase::new(prompt, generator);

        let args = NewProjectArgs {
            project_type: Some(ProjectType::Frontend),
            template: Some("react".to_string()),
            name: None,
            db: None,
            path: None,
            yes: true,
        };

        let result = use_case.execute(args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("name is required"));
    }

    #[test]
    fn test_non_interactive_missing_template_error() {
        let prompt = MockPrompt::new();
        let generator = MockGenerator::new();
        let use_case = NewProjectUseCase::new(prompt, generator);

        let args = NewProjectArgs {
            project_type: Some(ProjectType::Frontend),
            template: None,
            name: Some("my-app".to_string()),
            db: None,
            path: None,
            yes: true,
        };

        let result = use_case.execute(args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Template is required"));
    }

    #[test]
    fn test_incompatible_template_error() {
        let prompt = MockPrompt::new();
        let generator = MockGenerator::new();
        let use_case = NewProjectUseCase::new(prompt, generator);

        let args = NewProjectArgs {
            project_type: Some(ProjectType::Frontend),
            template: Some("rust".to_string()),
            name: Some("my-app".to_string()),
            db: None,
            path: None,
            yes: true,
        };

        let result = use_case.execute(args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not compatible"));
    }

    #[test]
    fn test_invalid_name_error() {
        let prompt = MockPrompt::new();
        let generator = MockGenerator::new();
        let use_case = NewProjectUseCase::new(prompt, generator);

        let args = NewProjectArgs {
            project_type: Some(ProjectType::Frontend),
            template: Some("react".to_string()),
            name: Some("my app!".to_string()),
            db: None,
            path: None,
            yes: true,
        };

        let result = use_case.execute(args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("invalid characters"));
    }

    #[test]
    fn test_interactive_mode_prompts_for_missing_args() {
        let prompt = MockPrompt {
            project_type: ProjectType::Frontend,
            name: "my-app".to_string(),
            template: Template::React,
            database: Database::None,
            path: PathBuf::from("/tmp"),
            confirm_result: true,
            calls: RefCell::new(Vec::new()),
        };
        let generator = MockGenerator::new();
        let use_case = NewProjectUseCase::new(prompt, generator);

        let args = NewProjectArgs {
            project_type: Some(ProjectType::Frontend),
            template: None,
            name: None,
            db: None,
            path: None,
            yes: false,
        };

        let config = use_case.execute(args).unwrap();
        assert_eq!(config.name, "my-app");
        assert_eq!(config.template, Template::React);

        let calls = use_case.prompt.calls.borrow();
        assert!(calls.contains(&"input_project_name".to_string()));
        assert!(calls.contains(&"select_template".to_string()));
        assert!(calls.contains(&"confirm".to_string()));
    }

    #[test]
    fn test_interactive_mode_skips_prompts_for_provided_args() {
        let prompt = MockPrompt {
            project_type: ProjectType::Frontend,
            name: "fallback".to_string(),
            template: Template::Flutter,
            database: Database::None,
            path: PathBuf::from("/tmp"),
            confirm_result: true,
            calls: RefCell::new(Vec::new()),
        };
        let generator = MockGenerator::new();
        let use_case = NewProjectUseCase::new(prompt, generator);

        let args = NewProjectArgs {
            project_type: Some(ProjectType::Frontend),
            template: Some("react".to_string()),
            name: Some("my-app".to_string()),
            db: None,
            path: None,
            yes: false,
        };

        let config = use_case.execute(args).unwrap();
        assert_eq!(config.name, "my-app");
        assert_eq!(config.template, Template::React);

        let calls = use_case.prompt.calls.borrow();
        assert!(!calls.contains(&"input_project_name".to_string()));
        assert!(!calls.contains(&"select_template".to_string()));
    }

    #[test]
    fn test_interactive_backend_prompts_for_database() {
        let prompt = MockPrompt {
            project_type: ProjectType::Backend,
            name: "my-svc".to_string(),
            template: Template::RustAxum,
            database: Database::PostgreSql,
            path: PathBuf::from("/tmp"),
            confirm_result: true,
            calls: RefCell::new(Vec::new()),
        };
        let generator = MockGenerator::new();
        let use_case = NewProjectUseCase::new(prompt, generator);

        let args = NewProjectArgs {
            project_type: Some(ProjectType::Backend),
            template: None,
            name: None,
            db: None,
            path: None,
            yes: false,
        };

        let config = use_case.execute(args).unwrap();
        assert_eq!(config.database, Database::PostgreSql);

        let calls = use_case.prompt.calls.borrow();
        assert!(calls.contains(&"select_database".to_string()));
    }

    #[test]
    fn test_user_cancels_confirmation() {
        let prompt = MockPrompt {
            project_type: ProjectType::Frontend,
            name: "my-app".to_string(),
            template: Template::React,
            database: Database::None,
            path: PathBuf::from("/tmp"),
            confirm_result: false,
            calls: RefCell::new(Vec::new()),
        };
        let generator = MockGenerator::new();
        let use_case = NewProjectUseCase::new(prompt, generator);

        let args = NewProjectArgs {
            project_type: Some(ProjectType::Frontend),
            template: None,
            name: None,
            db: None,
            path: None,
            yes: false,
        };

        let result = use_case.execute(args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("cancelled"));
    }

    #[test]
    fn test_generator_called_with_correct_config() {
        let prompt = MockPrompt::new();
        let generator = MockGenerator::new();
        let use_case = NewProjectUseCase::new(prompt, generator);

        let args = NewProjectArgs {
            project_type: Some(ProjectType::Frontend),
            template: Some("react".to_string()),
            name: Some("my-app".to_string()),
            db: None,
            path: Some(PathBuf::from("/out")),
            yes: true,
        };

        use_case.execute(args).unwrap();
        let calls = use_case.generator.calls.borrow();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "my-app");
        assert_eq!(calls[0].template, Template::React);
    }
}

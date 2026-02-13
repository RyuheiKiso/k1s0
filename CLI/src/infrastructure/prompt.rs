use std::path::{Path, PathBuf};

use dialoguer::{Confirm, Input, Select};

use crate::application::interactive::UserPrompt;
use crate::domain::model::{Database, ProjectConfig, ProjectType, Template};

pub struct DialoguerPrompt;

impl DialoguerPrompt {
    pub fn new() -> Self {
        Self
    }
}

impl UserPrompt for DialoguerPrompt {
    fn select_project_type(&self) -> Result<ProjectType, String> {
        let items = vec!["frontend", "backend"];
        let selection = Select::new()
            .with_prompt("Select project type")
            .items(&items)
            .default(0)
            .interact()
            .map_err(|e| format!("Prompt error: {e}"))?;

        match selection {
            0 => Ok(ProjectType::Frontend),
            1 => Ok(ProjectType::Backend),
            _ => Err("Invalid selection".to_string()),
        }
    }

    fn input_project_name(&self, default: Option<&str>) -> Result<String, String> {
        let mut input = Input::<String>::new().with_prompt("Project name");
        if let Some(d) = default {
            input = input.default(d.to_string());
        }
        input.interact_text().map_err(|e| format!("Prompt error: {e}"))
    }

    fn select_template(&self, project_type: &ProjectType) -> Result<Template, String> {
        let templates = Template::templates_for(project_type);
        let items: Vec<String> = templates.iter().map(|t| t.to_string()).collect();

        let selection = Select::new()
            .with_prompt("Select template")
            .items(&items)
            .default(0)
            .interact()
            .map_err(|e| format!("Prompt error: {e}"))?;

        templates.into_iter().nth(selection).ok_or("Invalid selection".to_string())
    }

    fn select_database(&self) -> Result<Database, String> {
        let items = vec!["postgresql", "none"];
        let selection = Select::new()
            .with_prompt("Select database")
            .items(&items)
            .default(1)
            .interact()
            .map_err(|e| format!("Prompt error: {e}"))?;

        match selection {
            0 => Ok(Database::PostgreSql),
            1 => Ok(Database::None),
            _ => Err("Invalid selection".to_string()),
        }
    }

    fn input_path(&self, default: &Path) -> Result<PathBuf, String> {
        let input: String = Input::new()
            .with_prompt("Output path")
            .default(default.to_string_lossy().to_string())
            .interact_text()
            .map_err(|e| format!("Prompt error: {e}"))?;

        Ok(PathBuf::from(input))
    }

    fn confirm(&self, config: &ProjectConfig) -> Result<bool, String> {
        println!("\nProject configuration:");
        println!("  Name:     {}", config.name);
        println!("  Type:     {}", config.project_type);
        println!("  Template: {}", config.template);
        println!("  Database: {}", config.database);
        println!("  Path:     {}", config.path.display());
        println!();

        Confirm::new()
            .with_prompt("Create project?")
            .default(true)
            .interact()
            .map_err(|e| format!("Prompt error: {e}"))
    }
}

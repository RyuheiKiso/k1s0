use std::path::{Path, PathBuf};

use crate::domain::model::{Database, ProjectConfig, ProjectType, Template};

pub trait UserPrompt {
    fn select_project_type(&self) -> Result<ProjectType, String>;
    fn input_project_name(&self, default: Option<&str>) -> Result<String, String>;
    fn select_template(&self, project_type: &ProjectType) -> Result<Template, String>;
    fn select_database(&self) -> Result<Database, String>;
    fn input_path(&self, default: &Path) -> Result<PathBuf, String>;
    fn confirm(&self, config: &ProjectConfig) -> Result<bool, String>;
}

pub trait FileGenerator {
    fn generate(&self, config: &ProjectConfig) -> Result<(), String>;
}

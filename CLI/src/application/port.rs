use crate::domain::region::{ProjectType, Region};
use crate::domain::workspace::WorkspacePath;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainMenuChoice {
    CreateProject,
    Settings,
    Exit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsMenuChoice {
    ShowWorkspacePath,
    SetWorkspacePath,
    Back,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionChoice {
    System,
    Business,
    Service,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectTypeChoice {
    Library,
    Service,
}

pub trait UserPrompt {
    fn show_main_menu(&self) -> MainMenuChoice;
    fn show_settings_menu(&self) -> SettingsMenuChoice;
    fn show_region_menu(&self) -> RegionChoice;
    fn show_project_type_menu(&self) -> ProjectTypeChoice;
    fn input_path(&self, prompt: &str) -> String;
    fn show_message(&self, message: &str);
    fn show_banner(&self) {}
}

pub trait ConfigStore {
    fn load_workspace_path(&self) -> Option<WorkspacePath>;
    fn save_workspace_path(&self, path: &WorkspacePath) -> Result<(), Box<dyn std::error::Error>>;
}

pub trait RegionCheckout {
    fn setup(
        &self,
        workspace: &WorkspacePath,
        region: &Region,
        project_type: Option<&ProjectType>,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

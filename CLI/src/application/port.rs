use crate::domain::workspace::WorkspacePath;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainMenuChoice {
    CreateProject,
    Settings,
    Exit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsMenuChoice {
    SetWorkspacePath,
    Back,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionChoice {
    System,
    Business,
    Service,
}

pub trait UserPrompt {
    fn show_main_menu(&self) -> MainMenuChoice;
    fn show_settings_menu(&self) -> SettingsMenuChoice;
    fn show_region_menu(&self) -> RegionChoice;
    fn input_path(&self, prompt: &str) -> String;
    fn show_message(&self, message: &str);
}

pub trait ConfigStore {
    fn load_workspace_path(&self) -> Option<WorkspacePath>;
    fn save_workspace_path(&self, path: &WorkspacePath) -> Result<(), Box<dyn std::error::Error>>;
}

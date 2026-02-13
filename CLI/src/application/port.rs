use crate::domain::region::{
    BusinessRegionName, ClientFramework, Language, ProjectType, Region, ServiceType,
};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LanguageChoice {
    Rust,
    Go,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceTypeChoice {
    Client,
    Server,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientFrameworkChoice {
    React,
    Flutter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BusinessRegionAction {
    SelectExisting,
    CreateNew,
}

pub trait UserPrompt {
    fn show_main_menu(&self) -> MainMenuChoice;
    fn show_settings_menu(&self) -> SettingsMenuChoice;
    fn show_region_menu(&self) -> RegionChoice;
    fn show_project_type_menu(&self) -> ProjectTypeChoice;
    fn show_language_menu(&self) -> LanguageChoice;
    fn show_service_type_menu(&self) -> ServiceTypeChoice;
    fn show_client_framework_menu(&self) -> ClientFrameworkChoice;
    fn show_business_region_action_menu(&self) -> BusinessRegionAction;
    fn show_business_region_list(&self, regions: &[String]) -> String;
    fn input_business_region_name(&self) -> String;
    fn input_path(&self, prompt: &str) -> String;
    fn show_message(&self, message: &str);
    fn show_banner(&self) {}
}

pub trait ConfigStore {
    fn load_workspace_path(&self) -> Option<WorkspacePath>;
    fn save_workspace_path(&self, path: &WorkspacePath) -> Result<(), Box<dyn std::error::Error>>;
}

pub trait RegionCheckout {
    #[allow(clippy::too_many_arguments)]
    fn setup(
        &self,
        workspace: &WorkspacePath,
        region: &Region,
        project_type: Option<&ProjectType>,
        language: Option<&Language>,
        business_region_name: Option<&BusinessRegionName>,
        service_type: Option<&ServiceType>,
        client_framework: Option<&ClientFramework>,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

pub trait BusinessRegionRepository {
    fn list(&self, workspace: &WorkspacePath) -> Result<Vec<String>, Box<dyn std::error::Error>>;
}

use std::cell::RefCell;

use crate::application::port::{
    BusinessRegionAction, ClientFrameworkChoice, LanguageChoice, MainMenuChoice, ProjectTypeChoice,
    RegionChoice, ServiceTypeChoice, SettingsMenuChoice, UserPrompt,
};

pub struct ScriptedPrompt {
    messages: RefCell<Vec<String>>,
    region_choice: RegionChoice,
    project_type_choice: ProjectTypeChoice,
    language_choice: LanguageChoice,
    service_type_choice: ServiceTypeChoice,
    client_framework_choice: ClientFrameworkChoice,
    business_region_action: BusinessRegionAction,
    business_region_list_selection: String,
    business_region_name_input: String,
    path_input: String,
}

impl ScriptedPrompt {
    pub fn new(region_choice: RegionChoice) -> Self {
        Self {
            messages: RefCell::new(Vec::new()),
            region_choice,
            project_type_choice: ProjectTypeChoice::Library,
            language_choice: LanguageChoice::Rust,
            service_type_choice: ServiceTypeChoice::Client,
            client_framework_choice: ClientFrameworkChoice::React,
            business_region_action: BusinessRegionAction::CreateNew,
            business_region_list_selection: String::new(),
            business_region_name_input: String::new(),
            path_input: String::new(),
        }
    }

    pub fn with_project_type(mut self, pt: ProjectTypeChoice) -> Self {
        self.project_type_choice = pt;
        self
    }

    pub fn with_language(mut self, lang: LanguageChoice) -> Self {
        self.language_choice = lang;
        self
    }

    pub fn with_service_type(mut self, st: ServiceTypeChoice) -> Self {
        self.service_type_choice = st;
        self
    }

    pub fn with_client_framework(mut self, cf: ClientFrameworkChoice) -> Self {
        self.client_framework_choice = cf;
        self
    }

    pub fn with_business_region_action(mut self, action: BusinessRegionAction) -> Self {
        self.business_region_action = action;
        self
    }

    pub fn with_business_region_list_selection(mut self, selection: &str) -> Self {
        self.business_region_list_selection = selection.to_string();
        self
    }

    pub fn with_business_region_name_input(mut self, name: &str) -> Self {
        self.business_region_name_input = name.to_string();
        self
    }

    pub fn with_path_input(mut self, path: &str) -> Self {
        self.path_input = path.to_string();
        self
    }

    pub fn messages(&self) -> Vec<String> {
        self.messages.borrow().clone()
    }
}

impl UserPrompt for ScriptedPrompt {
    fn show_main_menu(&self) -> MainMenuChoice {
        MainMenuChoice::Exit
    }

    fn show_settings_menu(&self) -> SettingsMenuChoice {
        SettingsMenuChoice::Back
    }

    fn show_region_menu(&self) -> RegionChoice {
        self.region_choice
    }

    fn show_project_type_menu(&self) -> ProjectTypeChoice {
        self.project_type_choice
    }

    fn show_business_project_type_menu(&self) -> ProjectTypeChoice {
        self.project_type_choice
    }

    fn show_language_menu(&self) -> LanguageChoice {
        self.language_choice
    }

    fn show_service_type_menu(&self) -> ServiceTypeChoice {
        self.service_type_choice
    }

    fn show_client_framework_menu(&self) -> ClientFrameworkChoice {
        self.client_framework_choice
    }

    fn show_business_region_action_menu(&self) -> BusinessRegionAction {
        self.business_region_action
    }

    fn show_business_region_list(&self, _regions: &[String]) -> String {
        self.business_region_list_selection.clone()
    }

    fn input_business_region_name(&self) -> String {
        self.business_region_name_input.clone()
    }

    fn input_path(&self, _prompt: &str) -> String {
        self.path_input.clone()
    }

    fn show_message(&self, message: &str) {
        self.messages.borrow_mut().push(message.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_configured_region_choice() {
        let prompt = ScriptedPrompt::new(RegionChoice::Business);
        assert_eq!(prompt.show_region_menu(), RegionChoice::Business);
    }

    #[test]
    fn returns_configured_project_type() {
        let prompt =
            ScriptedPrompt::new(RegionChoice::System).with_project_type(ProjectTypeChoice::Service);
        assert_eq!(prompt.show_project_type_menu(), ProjectTypeChoice::Service);
        assert_eq!(
            prompt.show_business_project_type_menu(),
            ProjectTypeChoice::Service
        );
    }

    #[test]
    fn returns_configured_language() {
        let prompt = ScriptedPrompt::new(RegionChoice::System).with_language(LanguageChoice::Go);
        assert_eq!(prompt.show_language_menu(), LanguageChoice::Go);
    }

    #[test]
    fn returns_configured_service_type() {
        let prompt =
            ScriptedPrompt::new(RegionChoice::Service).with_service_type(ServiceTypeChoice::Server);
        assert_eq!(prompt.show_service_type_menu(), ServiceTypeChoice::Server);
    }

    #[test]
    fn returns_configured_client_framework() {
        let prompt = ScriptedPrompt::new(RegionChoice::Business)
            .with_client_framework(ClientFrameworkChoice::Flutter);
        assert_eq!(
            prompt.show_client_framework_menu(),
            ClientFrameworkChoice::Flutter
        );
    }

    #[test]
    fn returns_configured_business_region_action() {
        let prompt = ScriptedPrompt::new(RegionChoice::Business)
            .with_business_region_action(BusinessRegionAction::SelectExisting);
        assert_eq!(
            prompt.show_business_region_action_menu(),
            BusinessRegionAction::SelectExisting
        );
    }

    #[test]
    fn returns_configured_business_region_list_selection() {
        let prompt = ScriptedPrompt::new(RegionChoice::Business)
            .with_business_region_list_selection("sales");
        let regions = vec!["sales".to_string(), "hr".to_string()];
        assert_eq!(prompt.show_business_region_list(&regions), "sales");
    }

    #[test]
    fn returns_configured_business_region_name_input() {
        let prompt = ScriptedPrompt::new(RegionChoice::Business)
            .with_business_region_name_input("marketing");
        assert_eq!(prompt.input_business_region_name(), "marketing");
    }

    #[test]
    fn returns_configured_path_input() {
        let prompt = ScriptedPrompt::new(RegionChoice::System).with_path_input(r"C:\workspace");
        assert_eq!(prompt.input_path("任意のプロンプト"), r"C:\workspace");
    }

    #[test]
    fn records_messages() {
        let prompt = ScriptedPrompt::new(RegionChoice::System);
        prompt.show_message("メッセージ1");
        prompt.show_message("メッセージ2");
        assert_eq!(prompt.messages(), vec!["メッセージ1", "メッセージ2"]);
    }

    #[test]
    fn defaults_are_sensible() {
        let prompt = ScriptedPrompt::new(RegionChoice::System);
        assert_eq!(prompt.show_project_type_menu(), ProjectTypeChoice::Library);
        assert_eq!(prompt.show_language_menu(), LanguageChoice::Rust);
        assert_eq!(prompt.show_service_type_menu(), ServiceTypeChoice::Client);
        assert_eq!(
            prompt.show_client_framework_menu(),
            ClientFrameworkChoice::React
        );
        assert_eq!(
            prompt.show_business_region_action_menu(),
            BusinessRegionAction::CreateNew
        );
        assert!(prompt.messages().is_empty());
    }
}

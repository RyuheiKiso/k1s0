use super::port::{ConfigStore, UserPrompt};

pub struct ShowWorkspaceUseCase<'a, P: UserPrompt, C: ConfigStore> {
    prompt: &'a P,
    config: &'a C,
}

impl<'a, P: UserPrompt, C: ConfigStore> ShowWorkspaceUseCase<'a, P, C> {
    pub fn new(prompt: &'a P, config: &'a C) -> Self {
        Self { prompt, config }
    }

    pub fn execute(&self) {
        match self.config.load_workspace_path() {
            Some(ws) => {
                self.prompt
                    .show_message(&format!("ワークスペースパス: {}", ws.to_string_lossy()));
            }
            None => {
                self.prompt.show_message("ワークスペースパスが未設定です。");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;
    use crate::application::port::{MainMenuChoice, RegionChoice, SettingsMenuChoice};
    use crate::domain::workspace::WorkspacePath;

    struct MockPrompt {
        messages: RefCell<Vec<String>>,
    }

    impl MockPrompt {
        fn new() -> Self {
            Self {
                messages: RefCell::new(Vec::new()),
            }
        }
    }

    impl UserPrompt for MockPrompt {
        fn show_main_menu(&self) -> MainMenuChoice {
            MainMenuChoice::Exit
        }
        fn show_settings_menu(&self) -> SettingsMenuChoice {
            SettingsMenuChoice::Back
        }
        fn show_region_menu(&self) -> RegionChoice {
            RegionChoice::System
        }
        fn show_project_type_menu(&self) -> crate::application::port::ProjectTypeChoice {
            crate::application::port::ProjectTypeChoice::Library
        }
        fn show_language_menu(&self) -> crate::application::port::LanguageChoice {
            crate::application::port::LanguageChoice::Rust
        }
        fn show_service_type_menu(&self) -> crate::application::port::ServiceTypeChoice {
            crate::application::port::ServiceTypeChoice::Client
        }
        fn show_business_region_action_menu(
            &self,
        ) -> crate::application::port::BusinessRegionAction {
            crate::application::port::BusinessRegionAction::CreateNew
        }
        fn show_business_region_list(&self, _regions: &[String]) -> String {
            String::new()
        }
        fn input_business_region_name(&self) -> String {
            String::new()
        }
        fn input_path(&self, _prompt: &str) -> String {
            String::new()
        }
        fn show_message(&self, message: &str) {
            self.messages.borrow_mut().push(message.to_string());
        }
    }

    struct MockConfig {
        workspace: Option<WorkspacePath>,
    }

    impl ConfigStore for MockConfig {
        fn load_workspace_path(&self) -> Option<WorkspacePath> {
            self.workspace.clone()
        }
        fn save_workspace_path(
            &self,
            _path: &WorkspacePath,
        ) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }
    }

    #[test]
    fn shows_configured_workspace_path() {
        let prompt = MockPrompt::new();
        let config = MockConfig {
            workspace: Some(WorkspacePath::new(r"C:\my\workspace").unwrap()),
        };
        let uc = ShowWorkspaceUseCase::new(&prompt, &config);

        uc.execute();

        let msgs = prompt.messages.borrow();
        assert_eq!(msgs.len(), 1);
        assert!(msgs[0].contains(r"C:\my\workspace"));
    }

    #[test]
    fn shows_not_configured_message() {
        let prompt = MockPrompt::new();
        let config = MockConfig { workspace: None };
        let uc = ShowWorkspaceUseCase::new(&prompt, &config);

        uc.execute();

        let msgs = prompt.messages.borrow();
        assert_eq!(msgs.len(), 1);
        assert!(msgs[0].contains("未設定"));
    }
}

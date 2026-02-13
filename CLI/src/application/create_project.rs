use super::port::{ConfigStore, UserPrompt};

pub struct CreateProjectUseCase<'a, P: UserPrompt, C: ConfigStore> {
    prompt: &'a P,
    config: &'a C,
}

impl<'a, P: UserPrompt, C: ConfigStore> CreateProjectUseCase<'a, P, C> {
    pub fn new(prompt: &'a P, config: &'a C) -> Self {
        Self { prompt, config }
    }

    pub fn execute(&self) {
        match self.config.load_workspace_path() {
            Some(ws) => {
                self.prompt
                    .show_message(&format!("ワークスペース: {}", ws.to_string_lossy()));
            }
            None => {
                self.prompt
                    .show_message("ワークスペースが未設定です。「設定」から設定してください。");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;
    use crate::application::port::{MainMenuChoice, SettingsMenuChoice};
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
    fn shows_workspace_when_configured() {
        let prompt = MockPrompt::new();
        let config = MockConfig {
            workspace: Some(WorkspacePath::new(r"C:\projects").unwrap()),
        };
        let uc = CreateProjectUseCase::new(&prompt, &config);

        uc.execute();

        let msgs = prompt.messages.borrow();
        assert_eq!(msgs.len(), 1);
        assert!(msgs[0].contains(r"C:\projects"));
    }

    #[test]
    fn prompts_settings_when_no_workspace() {
        let prompt = MockPrompt::new();
        let config = MockConfig { workspace: None };
        let uc = CreateProjectUseCase::new(&prompt, &config);

        uc.execute();

        let msgs = prompt.messages.borrow();
        assert_eq!(msgs.len(), 1);
        assert!(msgs[0].contains("未設定"));
    }
}

use crate::domain::workspace::WorkspacePath;

use super::port::{ConfigStore, UserPrompt};

pub struct ConfigureWorkspaceUseCase<'a, P: UserPrompt, C: ConfigStore> {
    prompt: &'a P,
    config: &'a C,
}

impl<'a, P: UserPrompt, C: ConfigStore> ConfigureWorkspaceUseCase<'a, P, C> {
    pub fn new(prompt: &'a P, config: &'a C) -> Self {
        Self { prompt, config }
    }

    pub fn execute(&self) {
        let raw = self
            .prompt
            .input_path("ワークスペースパスを入力してください");
        match WorkspacePath::new(&raw) {
            Ok(ws) => match self.config.save_workspace_path(&ws) {
                Ok(()) => {
                    self.prompt
                        .show_message(&format!("保存しました: {}", ws.to_string_lossy()));
                }
                Err(e) => {
                    self.prompt
                        .show_message(&format!("保存に失敗しました: {e}"));
                }
            },
            Err(e) => {
                self.prompt.show_message(&format!("無効なパスです: {e}"));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;
    use crate::application::port::{MainMenuChoice, RegionChoice, SettingsMenuChoice};

    struct MockPrompt {
        path_input: String,
        messages: RefCell<Vec<String>>,
    }

    impl MockPrompt {
        fn new(path_input: &str) -> Self {
            Self {
                path_input: path_input.to_string(),
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
        fn input_path(&self, _prompt: &str) -> String {
            self.path_input.clone()
        }
        fn show_message(&self, message: &str) {
            self.messages.borrow_mut().push(message.to_string());
        }
    }

    struct MockConfig {
        saved: RefCell<Option<WorkspacePath>>,
        fail_save: bool,
    }

    impl MockConfig {
        fn new(fail_save: bool) -> Self {
            Self {
                saved: RefCell::new(None),
                fail_save,
            }
        }
    }

    impl super::super::port::ConfigStore for MockConfig {
        fn load_workspace_path(&self) -> Option<WorkspacePath> {
            self.saved.borrow().clone()
        }
        fn save_workspace_path(
            &self,
            path: &WorkspacePath,
        ) -> Result<(), Box<dyn std::error::Error>> {
            if self.fail_save {
                return Err("disk error".into());
            }
            *self.saved.borrow_mut() = Some(path.clone());
            Ok(())
        }
    }

    #[test]
    fn saves_valid_absolute_path() {
        let prompt = MockPrompt::new(r"C:\workspace");
        let config = MockConfig::new(false);
        let uc = ConfigureWorkspaceUseCase::new(&prompt, &config);

        uc.execute();

        let saved = config.saved.borrow();
        assert!(saved.is_some());
        assert_eq!(saved.as_ref().unwrap().to_string_lossy(), r"C:\workspace");

        let msgs = prompt.messages.borrow();
        assert!(msgs[0].contains("保存しました"));
    }

    #[test]
    fn rejects_empty_path() {
        let prompt = MockPrompt::new("");
        let config = MockConfig::new(false);
        let uc = ConfigureWorkspaceUseCase::new(&prompt, &config);

        uc.execute();

        assert!(config.saved.borrow().is_none());
        let msgs = prompt.messages.borrow();
        assert!(msgs[0].contains("無効なパス"));
    }

    #[test]
    fn rejects_relative_path() {
        let prompt = MockPrompt::new("relative/path");
        let config = MockConfig::new(false);
        let uc = ConfigureWorkspaceUseCase::new(&prompt, &config);

        uc.execute();

        assert!(config.saved.borrow().is_none());
        let msgs = prompt.messages.borrow();
        assert!(msgs[0].contains("無効なパス"));
    }

    #[test]
    fn handles_save_failure() {
        let prompt = MockPrompt::new(r"C:\workspace");
        let config = MockConfig::new(true);
        let uc = ConfigureWorkspaceUseCase::new(&prompt, &config);

        uc.execute();

        let msgs = prompt.messages.borrow();
        assert!(msgs[0].contains("保存に失敗しました"));
    }
}

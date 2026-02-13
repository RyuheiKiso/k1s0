use crate::domain::region::{ProjectType, Region};

use super::port::{ConfigStore, ProjectTypeChoice, RegionCheckout, RegionChoice, UserPrompt};

pub struct CreateProjectUseCase<'a, P: UserPrompt, C: ConfigStore, R: RegionCheckout> {
    prompt: &'a P,
    config: &'a C,
    checkout: &'a R,
}

impl<'a, P: UserPrompt, C: ConfigStore, R: RegionCheckout> CreateProjectUseCase<'a, P, C, R> {
    pub fn new(prompt: &'a P, config: &'a C, checkout: &'a R) -> Self {
        Self {
            prompt,
            config,
            checkout,
        }
    }

    pub fn execute(&self) {
        match self.config.load_workspace_path() {
            Some(ws) => {
                self.prompt
                    .show_message(&format!("ワークスペース: {}", ws.to_string_lossy()));
                let choice = self.prompt.show_region_menu();
                let region = match choice {
                    RegionChoice::System => Region::System,
                    RegionChoice::Business => Region::Business,
                    RegionChoice::Service => Region::Service,
                };
                let project_type = match region {
                    Region::System => {
                        let pt_choice = self.prompt.show_project_type_menu();
                        Some(match pt_choice {
                            ProjectTypeChoice::Library => ProjectType::Library,
                            ProjectTypeChoice::Service => ProjectType::Service,
                        })
                    }
                    _ => None,
                };
                match self.checkout.setup(&ws, &region, project_type.as_ref()) {
                    Ok(()) => {
                        self.prompt
                            .show_message(&format!("{}のチェックアウトが完了しました", region));
                    }
                    Err(e) => {
                        self.prompt
                            .show_message(&format!("チェックアウトに失敗しました: {e}"));
                    }
                }
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
    use crate::application::port::{
        MainMenuChoice, ProjectTypeChoice, RegionChoice, SettingsMenuChoice,
    };
    use crate::domain::workspace::WorkspacePath;

    struct MockPrompt {
        messages: RefCell<Vec<String>>,
        region_choice: RefCell<RegionChoice>,
        project_type_choice: RefCell<ProjectTypeChoice>,
    }

    impl MockPrompt {
        fn new(region_choice: RegionChoice) -> Self {
            Self {
                messages: RefCell::new(Vec::new()),
                region_choice: RefCell::new(region_choice),
                project_type_choice: RefCell::new(ProjectTypeChoice::Library),
            }
        }

        fn with_project_type(mut self, pt: ProjectTypeChoice) -> Self {
            self.project_type_choice = RefCell::new(pt);
            self
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
            *self.region_choice.borrow()
        }
        fn show_project_type_menu(&self) -> ProjectTypeChoice {
            *self.project_type_choice.borrow()
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

    struct MockCheckout {
        called_with: RefCell<Option<(String, Vec<String>)>>,
        should_fail: bool,
    }

    impl MockCheckout {
        fn success() -> Self {
            Self {
                called_with: RefCell::new(None),
                should_fail: false,
            }
        }

        fn failure() -> Self {
            Self {
                called_with: RefCell::new(None),
                should_fail: true,
            }
        }
    }

    impl RegionCheckout for MockCheckout {
        fn setup(
            &self,
            workspace: &WorkspacePath,
            region: &Region,
            project_type: Option<&ProjectType>,
        ) -> Result<(), Box<dyn std::error::Error>> {
            let targets: Vec<String> = region
                .checkout_targets(project_type)
                .iter()
                .map(|s| s.to_string())
                .collect();
            *self.called_with.borrow_mut() = Some((workspace.to_string_lossy(), targets));
            if self.should_fail {
                Err("git error".into())
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn executes_sparse_checkout_for_system_library() {
        let prompt = MockPrompt::new(RegionChoice::System).with_project_type(ProjectTypeChoice::Library);
        let config = MockConfig {
            workspace: Some(WorkspacePath::new(r"C:\projects").unwrap()),
        };
        let checkout = MockCheckout::success();
        let uc = CreateProjectUseCase::new(&prompt, &config, &checkout);

        uc.execute();

        let called = checkout.called_with.borrow();
        let (ws, targets) = called.as_ref().unwrap();
        assert_eq!(ws, r"C:\projects");
        assert_eq!(targets, &["system-region/library"]);

        let msgs = prompt.messages.borrow();
        assert!(msgs[1].contains("チェックアウトが完了しました"));
    }

    #[test]
    fn executes_sparse_checkout_for_system_service() {
        let prompt = MockPrompt::new(RegionChoice::System).with_project_type(ProjectTypeChoice::Service);
        let config = MockConfig {
            workspace: Some(WorkspacePath::new(r"C:\projects").unwrap()),
        };
        let checkout = MockCheckout::success();
        let uc = CreateProjectUseCase::new(&prompt, &config, &checkout);

        uc.execute();

        let called = checkout.called_with.borrow();
        let (_, targets) = called.as_ref().unwrap();
        assert_eq!(targets, &["system-region/service"]);
    }

    #[test]
    fn executes_sparse_checkout_for_business_region() {
        let prompt = MockPrompt::new(RegionChoice::Business);
        let config = MockConfig {
            workspace: Some(WorkspacePath::new(r"C:\projects").unwrap()),
        };
        let checkout = MockCheckout::success();
        let uc = CreateProjectUseCase::new(&prompt, &config, &checkout);

        uc.execute();

        let called = checkout.called_with.borrow();
        let (_, targets) = called.as_ref().unwrap();
        assert_eq!(targets, &["system-region", "business-region"]);

        let msgs = prompt.messages.borrow();
        assert!(msgs[1].contains("部門固有領域"));
    }

    #[test]
    fn executes_sparse_checkout_for_service_region() {
        let prompt = MockPrompt::new(RegionChoice::Service);
        let config = MockConfig {
            workspace: Some(WorkspacePath::new(r"C:\projects").unwrap()),
        };
        let checkout = MockCheckout::success();
        let uc = CreateProjectUseCase::new(&prompt, &config, &checkout);

        uc.execute();

        let called = checkout.called_with.borrow();
        let (_, targets) = called.as_ref().unwrap();
        assert_eq!(
            targets,
            &["system-region", "business-region", "service-region"]
        );
    }

    #[test]
    fn shows_error_when_checkout_fails() {
        let prompt = MockPrompt::new(RegionChoice::System);
        let config = MockConfig {
            workspace: Some(WorkspacePath::new(r"C:\projects").unwrap()),
        };
        let checkout = MockCheckout::failure();
        let uc = CreateProjectUseCase::new(&prompt, &config, &checkout);

        uc.execute();

        let msgs = prompt.messages.borrow();
        assert!(msgs[1].contains("チェックアウトに失敗しました"));
    }

    #[test]
    fn prompts_settings_when_no_workspace() {
        let prompt = MockPrompt::new(RegionChoice::System);
        let config = MockConfig { workspace: None };
        let checkout = MockCheckout::success();
        let uc = CreateProjectUseCase::new(&prompt, &config, &checkout);

        uc.execute();

        let msgs = prompt.messages.borrow();
        assert_eq!(msgs.len(), 1);
        assert!(msgs[0].contains("未設定"));
    }
}

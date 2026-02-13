use crate::domain::region::{BusinessRegionName, Language, ProjectType, Region, ServiceType};

use super::port::{
    BusinessRegionAction, BusinessRegionRepository, ConfigStore, LanguageChoice, ProjectTypeChoice,
    RegionCheckout, RegionChoice, ServiceTypeChoice, UserPrompt,
};

pub struct CreateProjectUseCase<
    'a,
    P: UserPrompt,
    C: ConfigStore,
    R: RegionCheckout,
    B: BusinessRegionRepository,
> {
    prompt: &'a P,
    config: &'a C,
    checkout: &'a R,
    business_region_repo: &'a B,
}

impl<'a, P: UserPrompt, C: ConfigStore, R: RegionCheckout, B: BusinessRegionRepository>
    CreateProjectUseCase<'a, P, C, R, B>
{
    pub fn new(prompt: &'a P, config: &'a C, checkout: &'a R, business_region_repo: &'a B) -> Self {
        Self {
            prompt,
            config,
            checkout,
            business_region_repo,
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
                let mut project_type = None;
                let mut language = None;
                let mut business_region_name = None;
                let mut service_type = None;

                match region {
                    Region::System => {
                        let pt_choice = self.prompt.show_project_type_menu();
                        project_type = Some(match pt_choice {
                            ProjectTypeChoice::Library => ProjectType::Library,
                            ProjectTypeChoice::Service => ProjectType::Service,
                        });
                        let lang_choice = self.prompt.show_language_menu();
                        language = Some(match lang_choice {
                            LanguageChoice::Rust => Language::Rust,
                            LanguageChoice::Go => Language::Go,
                        });
                    }
                    Region::Business => match self.resolve_business_region(&ws) {
                        Some((name, _)) => {
                            let pt_choice = self.prompt.show_project_type_menu();
                            project_type = Some(match pt_choice {
                                ProjectTypeChoice::Library => ProjectType::Library,
                                ProjectTypeChoice::Service => ProjectType::Service,
                            });
                            let lang_choice = self.prompt.show_language_menu();
                            language = Some(match lang_choice {
                                LanguageChoice::Rust => Language::Rust,
                                LanguageChoice::Go => Language::Go,
                            });
                            business_region_name = Some(name);
                        }
                        None => return,
                    },
                    Region::Service => {
                        let regions = self.business_region_repo.list(&ws).unwrap_or_default();
                        if regions.is_empty() {
                            self.prompt.show_message(
                                "部門固有領域が存在しません。先に部門固有領域を作成してください。",
                            );
                            return;
                        }
                        let selected = self.prompt.show_business_region_list(&regions);
                        match BusinessRegionName::new(&selected) {
                            Ok(name) => {
                                business_region_name = Some(name);
                            }
                            Err(e) => {
                                self.prompt.show_message(&format!("領域名が不正です: {e}"));
                                return;
                            }
                        }
                        let st_choice = self.prompt.show_service_type_menu();
                        service_type = Some(match st_choice {
                            ServiceTypeChoice::Client => ServiceType::Client,
                            ServiceTypeChoice::Server => ServiceType::Server,
                        });
                    }
                }
                match self.checkout.setup(
                    &ws,
                    &region,
                    project_type.as_ref(),
                    language.as_ref(),
                    business_region_name.as_ref(),
                    service_type.as_ref(),
                ) {
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

    fn resolve_business_region(
        &self,
        workspace: &crate::domain::workspace::WorkspacePath,
    ) -> Option<(BusinessRegionName, bool)> {
        let regions = self
            .business_region_repo
            .list(workspace)
            .unwrap_or_default();

        let (raw_name, is_existing) = if regions.is_empty() {
            (self.prompt.input_business_region_name(), false)
        } else {
            let action = self.prompt.show_business_region_action_menu();
            match action {
                BusinessRegionAction::SelectExisting => {
                    (self.prompt.show_business_region_list(&regions), true)
                }
                BusinessRegionAction::CreateNew => {
                    (self.prompt.input_business_region_name(), false)
                }
            }
        };

        match BusinessRegionName::new(&raw_name) {
            Ok(name) => Some((name, is_existing)),
            Err(e) => {
                self.prompt.show_message(&format!("領域名が不正です: {e}"));
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;
    use crate::application::port::{
        BusinessRegionAction, LanguageChoice, MainMenuChoice, ProjectTypeChoice, RegionChoice,
        ServiceTypeChoice, SettingsMenuChoice,
    };
    use crate::domain::workspace::WorkspacePath;

    struct MockPrompt {
        messages: RefCell<Vec<String>>,
        region_choice: RefCell<RegionChoice>,
        project_type_choice: RefCell<ProjectTypeChoice>,
        language_choice: RefCell<LanguageChoice>,
        business_region_action: RefCell<BusinessRegionAction>,
        business_region_list_selection: RefCell<String>,
        business_region_name_input: RefCell<String>,
        service_type_choice: RefCell<ServiceTypeChoice>,
    }

    impl MockPrompt {
        fn new(region_choice: RegionChoice) -> Self {
            Self {
                messages: RefCell::new(Vec::new()),
                region_choice: RefCell::new(region_choice),
                project_type_choice: RefCell::new(ProjectTypeChoice::Library),
                language_choice: RefCell::new(LanguageChoice::Rust),
                business_region_action: RefCell::new(BusinessRegionAction::CreateNew),
                business_region_list_selection: RefCell::new(String::new()),
                business_region_name_input: RefCell::new(String::new()),
                service_type_choice: RefCell::new(ServiceTypeChoice::Client),
            }
        }

        fn with_project_type(self, pt: ProjectTypeChoice) -> Self {
            *self.project_type_choice.borrow_mut() = pt;
            self
        }

        fn with_language(self, lang: LanguageChoice) -> Self {
            *self.language_choice.borrow_mut() = lang;
            self
        }

        fn with_business_region_action(self, action: BusinessRegionAction) -> Self {
            *self.business_region_action.borrow_mut() = action;
            self
        }

        fn with_business_region_list_selection(self, selection: &str) -> Self {
            *self.business_region_list_selection.borrow_mut() = selection.to_string();
            self
        }

        fn with_business_region_name_input(self, name: &str) -> Self {
            *self.business_region_name_input.borrow_mut() = name.to_string();
            self
        }

        fn with_service_type(self, st: ServiceTypeChoice) -> Self {
            *self.service_type_choice.borrow_mut() = st;
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
        fn show_language_menu(&self) -> LanguageChoice {
            *self.language_choice.borrow()
        }
        fn show_service_type_menu(&self) -> ServiceTypeChoice {
            *self.service_type_choice.borrow()
        }
        fn show_business_region_action_menu(&self) -> BusinessRegionAction {
            *self.business_region_action.borrow()
        }
        fn show_business_region_list(&self, _regions: &[String]) -> String {
            self.business_region_list_selection.borrow().clone()
        }
        fn input_business_region_name(&self) -> String {
            self.business_region_name_input.borrow().clone()
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
            language: Option<&Language>,
            business_region_name: Option<&BusinessRegionName>,
            service_type: Option<&ServiceType>,
        ) -> Result<(), Box<dyn std::error::Error>> {
            let targets =
                region.checkout_targets(project_type, language, business_region_name, service_type);
            *self.called_with.borrow_mut() = Some((workspace.to_string_lossy(), targets));
            if self.should_fail {
                Err("git error".into())
            } else {
                Ok(())
            }
        }
    }

    struct MockBusinessRegionRepo {
        regions: Vec<String>,
    }

    impl MockBusinessRegionRepo {
        fn empty() -> Self {
            Self { regions: vec![] }
        }

        fn with_regions(regions: &[&str]) -> Self {
            Self {
                regions: regions.iter().map(|s| s.to_string()).collect(),
            }
        }
    }

    impl BusinessRegionRepository for MockBusinessRegionRepo {
        fn list(
            &self,
            _workspace: &WorkspacePath,
        ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
            Ok(self.regions.clone())
        }
    }

    #[test]
    fn executes_sparse_checkout_for_system_library_rust() {
        let prompt = MockPrompt::new(RegionChoice::System)
            .with_project_type(ProjectTypeChoice::Library)
            .with_language(LanguageChoice::Rust);
        let config = MockConfig {
            workspace: Some(WorkspacePath::new(r"C:\projects").unwrap()),
        };
        let checkout = MockCheckout::success();
        let repo = MockBusinessRegionRepo::empty();
        let uc = CreateProjectUseCase::new(&prompt, &config, &checkout, &repo);

        uc.execute();

        let called = checkout.called_with.borrow();
        let (ws, targets) = called.as_ref().unwrap();
        assert_eq!(ws, r"C:\projects");
        assert_eq!(targets, &["system-region/library/rust"]);

        let msgs = prompt.messages.borrow();
        assert!(msgs[1].contains("チェックアウトが完了しました"));
    }

    #[test]
    fn executes_sparse_checkout_for_system_library_go() {
        let prompt = MockPrompt::new(RegionChoice::System)
            .with_project_type(ProjectTypeChoice::Library)
            .with_language(LanguageChoice::Go);
        let config = MockConfig {
            workspace: Some(WorkspacePath::new(r"C:\projects").unwrap()),
        };
        let checkout = MockCheckout::success();
        let repo = MockBusinessRegionRepo::empty();
        let uc = CreateProjectUseCase::new(&prompt, &config, &checkout, &repo);

        uc.execute();

        let called = checkout.called_with.borrow();
        let (_, targets) = called.as_ref().unwrap();
        assert_eq!(targets, &["system-region/library/go"]);
    }

    #[test]
    fn executes_sparse_checkout_for_system_service_rust() {
        let prompt = MockPrompt::new(RegionChoice::System)
            .with_project_type(ProjectTypeChoice::Service)
            .with_language(LanguageChoice::Rust);
        let config = MockConfig {
            workspace: Some(WorkspacePath::new(r"C:\projects").unwrap()),
        };
        let checkout = MockCheckout::success();
        let repo = MockBusinessRegionRepo::empty();
        let uc = CreateProjectUseCase::new(&prompt, &config, &checkout, &repo);

        uc.execute();

        let called = checkout.called_with.borrow();
        let (_, targets) = called.as_ref().unwrap();
        assert_eq!(targets, &["system-region/service/rust"]);
    }

    #[test]
    fn executes_sparse_checkout_for_system_service_go() {
        let prompt = MockPrompt::new(RegionChoice::System)
            .with_project_type(ProjectTypeChoice::Service)
            .with_language(LanguageChoice::Go);
        let config = MockConfig {
            workspace: Some(WorkspacePath::new(r"C:\projects").unwrap()),
        };
        let checkout = MockCheckout::success();
        let repo = MockBusinessRegionRepo::empty();
        let uc = CreateProjectUseCase::new(&prompt, &config, &checkout, &repo);

        uc.execute();

        let called = checkout.called_with.borrow();
        let (_, targets) = called.as_ref().unwrap();
        assert_eq!(targets, &["system-region/service/go"]);
    }

    #[test]
    fn business_region_select_existing_library_rust() {
        let prompt = MockPrompt::new(RegionChoice::Business)
            .with_business_region_action(BusinessRegionAction::SelectExisting)
            .with_business_region_list_selection("sales")
            .with_project_type(ProjectTypeChoice::Library)
            .with_language(LanguageChoice::Rust);
        let config = MockConfig {
            workspace: Some(WorkspacePath::new(r"C:\projects").unwrap()),
        };
        let checkout = MockCheckout::success();
        let repo = MockBusinessRegionRepo::with_regions(&["sales", "hr"]);
        let uc = CreateProjectUseCase::new(&prompt, &config, &checkout, &repo);

        uc.execute();

        let called = checkout.called_with.borrow();
        let (_, targets) = called.as_ref().unwrap();
        assert_eq!(
            targets,
            &["system-region", "business-region/sales/library/rust"]
        );

        let msgs = prompt.messages.borrow();
        assert!(msgs[1].contains("部門固有領域"));
    }

    #[test]
    fn business_region_select_existing_library_go() {
        let prompt = MockPrompt::new(RegionChoice::Business)
            .with_business_region_action(BusinessRegionAction::SelectExisting)
            .with_business_region_list_selection("sales")
            .with_project_type(ProjectTypeChoice::Library)
            .with_language(LanguageChoice::Go);
        let config = MockConfig {
            workspace: Some(WorkspacePath::new(r"C:\projects").unwrap()),
        };
        let checkout = MockCheckout::success();
        let repo = MockBusinessRegionRepo::with_regions(&["sales", "hr"]);
        let uc = CreateProjectUseCase::new(&prompt, &config, &checkout, &repo);

        uc.execute();

        let called = checkout.called_with.borrow();
        let (_, targets) = called.as_ref().unwrap();
        assert_eq!(
            targets,
            &["system-region", "business-region/sales/library/go"]
        );
    }

    #[test]
    fn business_region_select_existing_service_rust() {
        let prompt = MockPrompt::new(RegionChoice::Business)
            .with_business_region_action(BusinessRegionAction::SelectExisting)
            .with_business_region_list_selection("sales")
            .with_project_type(ProjectTypeChoice::Service)
            .with_language(LanguageChoice::Rust);
        let config = MockConfig {
            workspace: Some(WorkspacePath::new(r"C:\projects").unwrap()),
        };
        let checkout = MockCheckout::success();
        let repo = MockBusinessRegionRepo::with_regions(&["sales", "hr"]);
        let uc = CreateProjectUseCase::new(&prompt, &config, &checkout, &repo);

        uc.execute();

        let called = checkout.called_with.borrow();
        let (_, targets) = called.as_ref().unwrap();
        assert_eq!(
            targets,
            &["system-region", "business-region/sales/service/rust"]
        );
    }

    #[test]
    fn business_region_select_existing_service_go() {
        let prompt = MockPrompt::new(RegionChoice::Business)
            .with_business_region_action(BusinessRegionAction::SelectExisting)
            .with_business_region_list_selection("sales")
            .with_project_type(ProjectTypeChoice::Service)
            .with_language(LanguageChoice::Go);
        let config = MockConfig {
            workspace: Some(WorkspacePath::new(r"C:\projects").unwrap()),
        };
        let checkout = MockCheckout::success();
        let repo = MockBusinessRegionRepo::with_regions(&["sales", "hr"]);
        let uc = CreateProjectUseCase::new(&prompt, &config, &checkout, &repo);

        uc.execute();

        let called = checkout.called_with.borrow();
        let (_, targets) = called.as_ref().unwrap();
        assert_eq!(
            targets,
            &["system-region", "business-region/sales/service/go"]
        );
    }

    #[test]
    fn business_region_create_new_library_rust() {
        let prompt = MockPrompt::new(RegionChoice::Business)
            .with_business_region_action(BusinessRegionAction::CreateNew)
            .with_business_region_name_input("marketing")
            .with_project_type(ProjectTypeChoice::Library)
            .with_language(LanguageChoice::Rust);
        let config = MockConfig {
            workspace: Some(WorkspacePath::new(r"C:\projects").unwrap()),
        };
        let checkout = MockCheckout::success();
        let repo = MockBusinessRegionRepo::with_regions(&["sales"]);
        let uc = CreateProjectUseCase::new(&prompt, &config, &checkout, &repo);

        uc.execute();

        let called = checkout.called_with.borrow();
        let (_, targets) = called.as_ref().unwrap();
        assert_eq!(
            targets,
            &["system-region", "business-region/marketing/library/rust"]
        );
    }

    #[test]
    fn business_region_create_new_service_go() {
        let prompt = MockPrompt::new(RegionChoice::Business)
            .with_business_region_action(BusinessRegionAction::CreateNew)
            .with_business_region_name_input("finance")
            .with_project_type(ProjectTypeChoice::Service)
            .with_language(LanguageChoice::Go);
        let config = MockConfig {
            workspace: Some(WorkspacePath::new(r"C:\projects").unwrap()),
        };
        let checkout = MockCheckout::success();
        let repo = MockBusinessRegionRepo::with_regions(&["sales"]);
        let uc = CreateProjectUseCase::new(&prompt, &config, &checkout, &repo);

        uc.execute();

        let called = checkout.called_with.borrow();
        let (_, targets) = called.as_ref().unwrap();
        assert_eq!(
            targets,
            &["system-region", "business-region/finance/service/go"]
        );
    }

    #[test]
    fn business_region_empty_list_goes_to_new_with_library() {
        let prompt = MockPrompt::new(RegionChoice::Business)
            .with_business_region_name_input("new-dept")
            .with_project_type(ProjectTypeChoice::Library)
            .with_language(LanguageChoice::Go);
        let config = MockConfig {
            workspace: Some(WorkspacePath::new(r"C:\projects").unwrap()),
        };
        let checkout = MockCheckout::success();
        let repo = MockBusinessRegionRepo::empty();
        let uc = CreateProjectUseCase::new(&prompt, &config, &checkout, &repo);

        uc.execute();

        let called = checkout.called_with.borrow();
        let (_, targets) = called.as_ref().unwrap();
        assert_eq!(
            targets,
            &["system-region", "business-region/new-dept/library/go"]
        );
    }

    #[test]
    fn business_region_invalid_name_aborts_checkout() {
        let prompt = MockPrompt::new(RegionChoice::Business).with_business_region_name_input("");
        let config = MockConfig {
            workspace: Some(WorkspacePath::new(r"C:\projects").unwrap()),
        };
        let checkout = MockCheckout::success();
        let repo = MockBusinessRegionRepo::empty();
        let uc = CreateProjectUseCase::new(&prompt, &config, &checkout, &repo);

        uc.execute();

        assert!(
            checkout.called_with.borrow().is_none(),
            "checkout should not be called when region name is invalid"
        );

        let msgs = prompt.messages.borrow();
        assert!(msgs.iter().any(|m| m.contains("領域名が不正です")));
        assert!(
            !msgs
                .iter()
                .any(|m| m.contains("チェックアウトが完了しました"))
        );
    }

    #[test]
    fn service_region_selects_existing_business_region() {
        let prompt =
            MockPrompt::new(RegionChoice::Service).with_business_region_list_selection("sales");
        let config = MockConfig {
            workspace: Some(WorkspacePath::new(r"C:\projects").unwrap()),
        };
        let checkout = MockCheckout::success();
        let repo = MockBusinessRegionRepo::with_regions(&["sales", "hr"]);
        let uc = CreateProjectUseCase::new(&prompt, &config, &checkout, &repo);

        uc.execute();

        let called = checkout.called_with.borrow();
        let (_, targets) = called.as_ref().unwrap();
        assert_eq!(
            targets,
            &[
                "system-region",
                "business-region/sales",
                "service-region/client"
            ]
        );

        let msgs = prompt.messages.borrow();
        assert!(
            msgs.iter()
                .any(|m| m.contains("チェックアウトが完了しました"))
        );
    }

    #[test]
    fn service_region_client_checkout() {
        let prompt = MockPrompt::new(RegionChoice::Service)
            .with_business_region_list_selection("sales")
            .with_service_type(ServiceTypeChoice::Client);
        let config = MockConfig {
            workspace: Some(WorkspacePath::new(r"C:\projects").unwrap()),
        };
        let checkout = MockCheckout::success();
        let repo = MockBusinessRegionRepo::with_regions(&["sales", "hr"]);
        let uc = CreateProjectUseCase::new(&prompt, &config, &checkout, &repo);

        uc.execute();

        let called = checkout.called_with.borrow();
        let (_, targets) = called.as_ref().unwrap();
        assert_eq!(
            targets,
            &[
                "system-region",
                "business-region/sales",
                "service-region/client"
            ]
        );
    }

    #[test]
    fn service_region_server_checkout() {
        let prompt = MockPrompt::new(RegionChoice::Service)
            .with_business_region_list_selection("sales")
            .with_service_type(ServiceTypeChoice::Server);
        let config = MockConfig {
            workspace: Some(WorkspacePath::new(r"C:\projects").unwrap()),
        };
        let checkout = MockCheckout::success();
        let repo = MockBusinessRegionRepo::with_regions(&["sales", "hr"]);
        let uc = CreateProjectUseCase::new(&prompt, &config, &checkout, &repo);

        uc.execute();

        let called = checkout.called_with.borrow();
        let (_, targets) = called.as_ref().unwrap();
        assert_eq!(
            targets,
            &[
                "system-region",
                "business-region/sales",
                "service-region/server"
            ]
        );
    }

    #[test]
    fn service_region_no_business_regions_shows_error() {
        let prompt = MockPrompt::new(RegionChoice::Service);
        let config = MockConfig {
            workspace: Some(WorkspacePath::new(r"C:\projects").unwrap()),
        };
        let checkout = MockCheckout::success();
        let repo = MockBusinessRegionRepo::empty();
        let uc = CreateProjectUseCase::new(&prompt, &config, &checkout, &repo);

        uc.execute();

        assert!(
            checkout.called_with.borrow().is_none(),
            "checkout should not be called when no business regions exist"
        );

        let msgs = prompt.messages.borrow();
        assert!(
            msgs.iter()
                .any(|m| m.contains("部門固有領域が存在しません"))
        );
    }

    #[test]
    fn shows_error_when_checkout_fails() {
        let prompt = MockPrompt::new(RegionChoice::System);
        let config = MockConfig {
            workspace: Some(WorkspacePath::new(r"C:\projects").unwrap()),
        };
        let checkout = MockCheckout::failure();
        let repo = MockBusinessRegionRepo::empty();
        let uc = CreateProjectUseCase::new(&prompt, &config, &checkout, &repo);

        uc.execute();

        let msgs = prompt.messages.borrow();
        assert!(msgs[1].contains("チェックアウトに失敗しました"));
    }

    #[test]
    fn prompts_settings_when_no_workspace() {
        let prompt = MockPrompt::new(RegionChoice::System);
        let config = MockConfig { workspace: None };
        let checkout = MockCheckout::success();
        let repo = MockBusinessRegionRepo::empty();
        let uc = CreateProjectUseCase::new(&prompt, &config, &checkout, &repo);

        uc.execute();

        let msgs = prompt.messages.borrow();
        assert_eq!(msgs.len(), 1);
        assert!(msgs[0].contains("未設定"));
    }
}

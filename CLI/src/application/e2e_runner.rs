#![allow(dead_code)]

use std::cell::RefCell;

use crate::application::configure_workspace::ConfigureWorkspaceUseCase;
use crate::application::create_project::CreateProjectUseCase;
use crate::application::port::{
    BusinessRegionAction, BusinessRegionRepository, ClientFrameworkChoice, ConfigStore,
    LanguageChoice, ProjectTypeChoice, RegionCheckout, RegionChoice, ServiceTypeChoice,
};
use crate::application::show_workspace::ShowWorkspaceUseCase;
use crate::domain::region::{
    BusinessRegionName, ClientFramework, Language, ProjectType, Region, ServiceType,
};
use crate::domain::workspace::WorkspacePath;
use crate::infrastructure::scripted_prompt::ScriptedPrompt;
use crate::infrastructure::ui;

type CategoryFilter = Vec<(&'static str, Box<dyn Fn(&str) -> bool>)>;
type BusinessScenario<'a> = (
    &'a str,
    ProjectTypeChoice,
    Option<LanguageChoice>,
    Option<ClientFrameworkChoice>,
    &'a [&'a str],
);
type NewBusinessScenario<'a> = (
    &'a str,
    &'a str,
    ProjectTypeChoice,
    Option<LanguageChoice>,
    Option<ClientFrameworkChoice>,
    &'a [&'a str],
);
type ServiceScenario<'a> = (
    &'a str,
    ServiceTypeChoice,
    Option<LanguageChoice>,
    Option<ClientFrameworkChoice>,
    &'a [&'a str],
);

pub struct E2eResult {
    pub name: String,
    pub passed: bool,
    pub detail: Option<String>,
}

struct VerifyingCheckout {
    called_with: RefCell<Option<Vec<String>>>,
    should_fail: bool,
}

impl VerifyingCheckout {
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

    fn targets(&self) -> Option<Vec<String>> {
        self.called_with.borrow().clone()
    }
}

impl RegionCheckout for VerifyingCheckout {
    fn setup(
        &self,
        _workspace: &WorkspacePath,
        region: &Region,
        project_type: Option<&ProjectType>,
        language: Option<&Language>,
        business_region_name: Option<&BusinessRegionName>,
        service_type: Option<&ServiceType>,
        client_framework: Option<&ClientFramework>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let targets = region.checkout_targets(
            project_type,
            language,
            business_region_name,
            service_type,
            client_framework,
        );
        *self.called_with.borrow_mut() = Some(targets);
        if self.should_fail {
            Err("simulated checkout failure".into())
        } else {
            Ok(())
        }
    }
}

struct StubBusinessRegionRepo {
    regions: Vec<String>,
}

impl StubBusinessRegionRepo {
    fn with_regions(regions: &[&str]) -> Self {
        Self {
            regions: regions.iter().map(|s| s.to_string()).collect(),
        }
    }

    fn empty() -> Self {
        Self { regions: vec![] }
    }
}

impl BusinessRegionRepository for StubBusinessRegionRepo {
    fn list(&self, _workspace: &WorkspacePath) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        Ok(self.regions.clone())
    }
}

struct MemoryConfigStore {
    workspace: RefCell<Option<WorkspacePath>>,
    fail_save: bool,
}

impl MemoryConfigStore {
    fn with_workspace(path: &str) -> Self {
        Self {
            workspace: RefCell::new(Some(WorkspacePath::new(path).unwrap())),
            fail_save: false,
        }
    }

    fn empty() -> Self {
        Self {
            workspace: RefCell::new(None),
            fail_save: false,
        }
    }

    fn with_failing_save() -> Self {
        Self {
            workspace: RefCell::new(None),
            fail_save: true,
        }
    }
}

impl ConfigStore for MemoryConfigStore {
    fn load_workspace_path(&self) -> Option<WorkspacePath> {
        self.workspace.borrow().clone()
    }

    fn save_workspace_path(&self, path: &WorkspacePath) -> Result<(), Box<dyn std::error::Error>> {
        if self.fail_save {
            return Err("disk error".into());
        }
        *self.workspace.borrow_mut() = Some(path.clone());
        Ok(())
    }
}

fn run_create_scenario(
    name: &str,
    prompt: ScriptedPrompt,
    config: &MemoryConfigStore,
    checkout: &VerifyingCheckout,
    repo: &StubBusinessRegionRepo,
    expected_targets: &[&str],
) -> E2eResult {
    CreateProjectUseCase::new(&prompt, config, checkout, repo).execute();
    let actual = checkout.targets();
    let expected: Vec<String> = expected_targets.iter().map(|s| s.to_string()).collect();
    match actual {
        Some(targets) if targets == expected => {
            let msgs = prompt.messages();
            if msgs
                .iter()
                .any(|m| m.contains("チェックアウトが完了しました"))
            {
                E2eResult {
                    name: name.to_string(),
                    passed: true,
                    detail: None,
                }
            } else {
                E2eResult {
                    name: name.to_string(),
                    passed: false,
                    detail: Some(format!(
                        "checkout targets matched but success message not found, got {:?}",
                        msgs
                    )),
                }
            }
        }
        Some(targets) => E2eResult {
            name: name.to_string(),
            passed: false,
            detail: Some(format!("expected {:?}, got {:?}", expected, targets)),
        },
        None => E2eResult {
            name: name.to_string(),
            passed: false,
            detail: Some("checkout was not called".to_string()),
        },
    }
}

fn run_message_scenario(
    name: &str,
    prompt: &ScriptedPrompt,
    expected_substring: &str,
) -> E2eResult {
    let msgs = prompt.messages();
    if msgs.iter().any(|m| m.contains(expected_substring)) {
        E2eResult {
            name: name.to_string(),
            passed: true,
            detail: None,
        }
    } else {
        E2eResult {
            name: name.to_string(),
            passed: false,
            detail: Some(format!(
                "expected message containing '{}', got {:?}",
                expected_substring, msgs
            )),
        }
    }
}

fn system_region_scenarios() -> Vec<E2eResult> {
    let ws = r"C:\e2e-test";
    let scenarios: Vec<(&str, ProjectTypeChoice, LanguageChoice, &[&str])> = vec![
        (
            "System / Library / Rust",
            ProjectTypeChoice::Library,
            LanguageChoice::Rust,
            &["system-region/library/rust"],
        ),
        (
            "System / Library / Go",
            ProjectTypeChoice::Library,
            LanguageChoice::Go,
            &["system-region/library/go"],
        ),
        (
            "System / Service / Rust",
            ProjectTypeChoice::Service,
            LanguageChoice::Rust,
            &["system-region/service/rust"],
        ),
        (
            "System / Service / Go",
            ProjectTypeChoice::Service,
            LanguageChoice::Go,
            &["system-region/service/go"],
        ),
    ];

    scenarios
        .into_iter()
        .map(|(name, pt, lang, expected)| {
            let prompt = ScriptedPrompt::new(RegionChoice::System)
                .with_project_type(pt)
                .with_language(lang);
            let config = MemoryConfigStore::with_workspace(ws);
            let checkout = VerifyingCheckout::success();
            let repo = StubBusinessRegionRepo::empty();
            run_create_scenario(name, prompt, &config, &checkout, &repo, expected)
        })
        .collect()
}

fn business_region_scenarios() -> Vec<E2eResult> {
    let ws = r"C:\e2e-test";
    let mut results = Vec::new();

    // 既存領域選択パターン
    let existing_scenarios: Vec<BusinessScenario> = vec![
        (
            "Business / sales(既存) / Library / Rust",
            ProjectTypeChoice::Library,
            Some(LanguageChoice::Rust),
            None,
            &["system-region", "business-region/sales/library/rust"],
        ),
        (
            "Business / sales(既存) / Library / Go",
            ProjectTypeChoice::Library,
            Some(LanguageChoice::Go),
            None,
            &["system-region", "business-region/sales/library/go"],
        ),
        (
            "Business / sales(既存) / Service / Rust",
            ProjectTypeChoice::Service,
            Some(LanguageChoice::Rust),
            None,
            &["system-region", "business-region/sales/service/rust"],
        ),
        (
            "Business / sales(既存) / Service / Go",
            ProjectTypeChoice::Service,
            Some(LanguageChoice::Go),
            None,
            &["system-region", "business-region/sales/service/go"],
        ),
        (
            "Business / sales(既存) / Client / React",
            ProjectTypeChoice::Client,
            None,
            Some(ClientFrameworkChoice::React),
            &["system-region", "business-region/sales/client/react"],
        ),
        (
            "Business / sales(既存) / Client / Flutter",
            ProjectTypeChoice::Client,
            None,
            Some(ClientFrameworkChoice::Flutter),
            &["system-region", "business-region/sales/client/flutter"],
        ),
    ];

    for (name, pt, lang, cf, expected) in existing_scenarios {
        let mut prompt = ScriptedPrompt::new(RegionChoice::Business)
            .with_business_region_action(BusinessRegionAction::SelectExisting)
            .with_business_region_list_selection("sales")
            .with_project_type(pt);
        if let Some(l) = lang {
            prompt = prompt.with_language(l);
        }
        if let Some(c) = cf {
            prompt = prompt.with_client_framework(c);
        }
        let config = MemoryConfigStore::with_workspace(ws);
        let checkout = VerifyingCheckout::success();
        let repo = StubBusinessRegionRepo::with_regions(&["sales", "hr"]);
        results.push(run_create_scenario(
            name, prompt, &config, &checkout, &repo, expected,
        ));
    }

    // 新規作成パターン
    let new_scenarios: Vec<NewBusinessScenario> = vec![
        (
            "Business / marketing(新規) / Library / Rust",
            "marketing",
            ProjectTypeChoice::Library,
            Some(LanguageChoice::Rust),
            None,
            &["system-region", "business-region/marketing/library/rust"],
        ),
        (
            "Business / marketing(新規) / Library / Go",
            "marketing",
            ProjectTypeChoice::Library,
            Some(LanguageChoice::Go),
            None,
            &["system-region", "business-region/marketing/library/go"],
        ),
        (
            "Business / marketing(新規) / Service / Rust",
            "marketing",
            ProjectTypeChoice::Service,
            Some(LanguageChoice::Rust),
            None,
            &["system-region", "business-region/marketing/service/rust"],
        ),
        (
            "Business / marketing(新規) / Service / Go",
            "marketing",
            ProjectTypeChoice::Service,
            Some(LanguageChoice::Go),
            None,
            &["system-region", "business-region/marketing/service/go"],
        ),
        (
            "Business / marketing(新規) / Client / React",
            "marketing",
            ProjectTypeChoice::Client,
            None,
            Some(ClientFrameworkChoice::React),
            &["system-region", "business-region/marketing/client/react"],
        ),
        (
            "Business / marketing(新規) / Client / Flutter",
            "marketing",
            ProjectTypeChoice::Client,
            None,
            Some(ClientFrameworkChoice::Flutter),
            &["system-region", "business-region/marketing/client/flutter"],
        ),
    ];

    for (name, region_name, pt, lang, cf, expected) in new_scenarios {
        let mut prompt = ScriptedPrompt::new(RegionChoice::Business)
            .with_business_region_action(BusinessRegionAction::CreateNew)
            .with_business_region_name_input(region_name)
            .with_project_type(pt);
        if let Some(l) = lang {
            prompt = prompt.with_language(l);
        }
        if let Some(c) = cf {
            prompt = prompt.with_client_framework(c);
        }
        let config = MemoryConfigStore::with_workspace(ws);
        let checkout = VerifyingCheckout::success();
        let repo = StubBusinessRegionRepo::with_regions(&["sales"]);
        results.push(run_create_scenario(
            name, prompt, &config, &checkout, &repo, expected,
        ));
    }

    // 既存リストが空 → アクションメニューをスキップして自動新規作成
    {
        let prompt = ScriptedPrompt::new(RegionChoice::Business)
            .with_business_region_name_input("new-dept")
            .with_project_type(ProjectTypeChoice::Library)
            .with_language(LanguageChoice::Go);
        let config = MemoryConfigStore::with_workspace(ws);
        let checkout = VerifyingCheckout::success();
        let repo = StubBusinessRegionRepo::empty();
        results.push(run_create_scenario(
            "Business / 空リスト自動新規 / Library / Go",
            prompt,
            &config,
            &checkout,
            &repo,
            &["system-region", "business-region/new-dept/library/go"],
        ));
    }

    results
}

fn service_region_scenarios() -> Vec<E2eResult> {
    let ws = r"C:\e2e-test";
    let scenarios: Vec<ServiceScenario> = vec![
        (
            "Service / sales / Server / Rust",
            ServiceTypeChoice::Server,
            Some(LanguageChoice::Rust),
            None,
            &[
                "system-region",
                "business-region/sales",
                "service-region/server/rust",
            ],
        ),
        (
            "Service / sales / Server / Go",
            ServiceTypeChoice::Server,
            Some(LanguageChoice::Go),
            None,
            &[
                "system-region",
                "business-region/sales",
                "service-region/server/go",
            ],
        ),
        (
            "Service / sales / Client / React",
            ServiceTypeChoice::Client,
            None,
            Some(ClientFrameworkChoice::React),
            &[
                "system-region",
                "business-region/sales",
                "service-region/client/react",
            ],
        ),
        (
            "Service / sales / Client / Flutter",
            ServiceTypeChoice::Client,
            None,
            Some(ClientFrameworkChoice::Flutter),
            &[
                "system-region",
                "business-region/sales",
                "service-region/client/flutter",
            ],
        ),
    ];

    scenarios
        .into_iter()
        .map(|(name, st, lang, cf, expected)| {
            let mut prompt = ScriptedPrompt::new(RegionChoice::Service)
                .with_business_region_list_selection("sales")
                .with_service_type(st);
            if let Some(l) = lang {
                prompt = prompt.with_language(l);
            }
            if let Some(c) = cf {
                prompt = prompt.with_client_framework(c);
            }
            let config = MemoryConfigStore::with_workspace(ws);
            let checkout = VerifyingCheckout::success();
            let repo = StubBusinessRegionRepo::with_regions(&["sales", "hr"]);
            run_create_scenario(name, prompt, &config, &checkout, &repo, expected)
        })
        .collect()
}

fn workspace_scenarios() -> Vec<E2eResult> {
    let mut results = Vec::new();

    // 設定→確認のラウンドトリップ
    {
        let name = "設定→確認のラウンドトリップ";
        let config = MemoryConfigStore::empty();

        let set_prompt =
            ScriptedPrompt::new(RegionChoice::System).with_path_input(r"C:\e2e-workspace");
        ConfigureWorkspaceUseCase::new(&set_prompt, &config).execute();

        let show_prompt = ScriptedPrompt::new(RegionChoice::System);
        ShowWorkspaceUseCase::new(&show_prompt, &config).execute();

        results.push(run_message_scenario(
            name,
            &show_prompt,
            r"C:\e2e-workspace",
        ));
    }

    // 未設定時のメッセージ表示
    {
        let name = "未設定時のメッセージ表示";
        let config = MemoryConfigStore::empty();
        let prompt = ScriptedPrompt::new(RegionChoice::System);
        ShowWorkspaceUseCase::new(&prompt, &config).execute();
        results.push(run_message_scenario(name, &prompt, "未設定"));
    }

    // 無効なパス入力
    {
        let name = "ワークスペース設定 - 無効なパス";
        let config = MemoryConfigStore::empty();
        let prompt = ScriptedPrompt::new(RegionChoice::System).with_path_input("");
        ConfigureWorkspaceUseCase::new(&prompt, &config).execute();
        results.push(run_message_scenario(name, &prompt, "無効なパス"));
    }

    // 保存失敗
    {
        let name = "ワークスペース設定 - 保存エラー";
        let config = MemoryConfigStore::with_failing_save();
        let prompt = ScriptedPrompt::new(RegionChoice::System).with_path_input(r"C:\valid-path");
        ConfigureWorkspaceUseCase::new(&prompt, &config).execute();
        results.push(run_message_scenario(name, &prompt, "保存に失敗しました"));
    }

    results
}

fn error_scenarios() -> Vec<E2eResult> {
    let mut results = Vec::new();

    // ワークスペース未設定でプロジェクト作成
    {
        let name = "ワークスペース未設定でプロジェクト作成";
        let prompt = ScriptedPrompt::new(RegionChoice::System);
        let config = MemoryConfigStore::empty();
        let checkout = VerifyingCheckout::success();
        let repo = StubBusinessRegionRepo::empty();
        CreateProjectUseCase::new(&prompt, &config, &checkout, &repo).execute();
        results.push(run_message_scenario(name, &prompt, "未設定"));
    }

    // 不正なビジネス領域名
    {
        let name = "不正なビジネス領域名";
        let prompt =
            ScriptedPrompt::new(RegionChoice::Business).with_business_region_name_input("");
        let config = MemoryConfigStore::with_workspace(r"C:\e2e-test");
        let checkout = VerifyingCheckout::success();
        let repo = StubBusinessRegionRepo::empty();
        CreateProjectUseCase::new(&prompt, &config, &checkout, &repo).execute();
        results.push(run_message_scenario(name, &prompt, "領域名が不正です"));
    }

    // チェックアウト失敗
    {
        let name = "チェックアウト失敗時のメッセージ";
        let prompt = ScriptedPrompt::new(RegionChoice::System);
        let config = MemoryConfigStore::with_workspace(r"C:\e2e-test");
        let checkout = VerifyingCheckout::failure();
        let repo = StubBusinessRegionRepo::empty();
        CreateProjectUseCase::new(&prompt, &config, &checkout, &repo).execute();
        results.push(run_message_scenario(name, &prompt, "失敗しました"));
    }

    // 部門固有領域が空でService Region選択
    {
        let name = "部門固有領域が空でService Region選択";
        let prompt = ScriptedPrompt::new(RegionChoice::Service);
        let config = MemoryConfigStore::with_workspace(r"C:\e2e-test");
        let checkout = VerifyingCheckout::success();
        let repo = StubBusinessRegionRepo::empty();
        CreateProjectUseCase::new(&prompt, &config, &checkout, &repo).execute();
        results.push(run_message_scenario(
            name,
            &prompt,
            "部門固有領域が存在しません",
        ));
    }

    // Service Regionで不正なビジネス領域名
    {
        let name = "不正なビジネス領域名(Service Region)";
        let prompt =
            ScriptedPrompt::new(RegionChoice::Service).with_business_region_list_selection("");
        let config = MemoryConfigStore::with_workspace(r"C:\e2e-test");
        let checkout = VerifyingCheckout::success();
        let repo = StubBusinessRegionRepo::with_regions(&["sales"]);
        CreateProjectUseCase::new(&prompt, &config, &checkout, &repo).execute();
        results.push(run_message_scenario(name, &prompt, "領域名が不正です"));
    }

    results
}

pub fn run_all() -> Vec<E2eResult> {
    let mut results = Vec::new();
    results.extend(system_region_scenarios());
    results.extend(business_region_scenarios());
    results.extend(service_region_scenarios());
    results.extend(workspace_scenarios());
    results.extend(error_scenarios());
    results
}

pub fn print_results(results: &[E2eResult]) {
    let categories: CategoryFilter = vec![
        (
            "プロジェクト作成 - System Region",
            Box::new(|n: &str| n.starts_with("System")),
        ),
        (
            "プロジェクト作成 - Business Region",
            Box::new(|n: &str| n.starts_with("Business")),
        ),
        (
            "プロジェクト作成 - Service Region",
            Box::new(|n: &str| n.starts_with("Service")),
        ),
        (
            "ワークスペース設定",
            Box::new(|n: &str| n.contains("設定") || n.contains("未設定時")),
        ),
        (
            "エラーハンドリング",
            Box::new(|n: &str| {
                n.contains("未設定でプロジェクト")
                    || n.contains("不正")
                    || n.contains("失敗")
                    || n.contains("空で")
            }),
        ),
    ];

    println!("\nE2Eテスト実行中...\n");

    for (category, filter) in &categories {
        let items: Vec<_> = results.iter().filter(|r| filter(&r.name)).collect();
        if items.is_empty() {
            continue;
        }
        println!("[{}]", category);
        for item in &items {
            if item.passed {
                println!("  {} {}", ui::success_mark(), item.name);
            } else {
                let detail = item.detail.as_deref().unwrap_or("");
                println!("  {} {} - {}", ui::failure_mark(), item.name, detail);
            }
        }
        println!();
    }

    let passed = results.iter().filter(|r| r.passed).count();
    let total = results.len();
    println!("結果: {}/{} 成功", passed, total);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_all_returns_30_scenarios() {
        let results = run_all();
        assert_eq!(results.len(), 30);
    }

    #[test]
    fn all_scenarios_pass() {
        let results = run_all();
        for result in &results {
            assert!(
                result.passed,
                "FAILED: {} - {:?}",
                result.name, result.detail
            );
        }
    }

    #[test]
    fn system_scenarios_produce_4_results() {
        let results = system_region_scenarios();
        assert_eq!(results.len(), 4);
        assert!(results.iter().all(|r| r.passed));
    }

    #[test]
    fn business_scenarios_produce_13_results() {
        let results = business_region_scenarios();
        assert_eq!(results.len(), 13);
        assert!(results.iter().all(|r| r.passed));
    }

    #[test]
    fn service_scenarios_produce_4_results() {
        let results = service_region_scenarios();
        assert_eq!(results.len(), 4);
        assert!(results.iter().all(|r| r.passed));
    }

    #[test]
    fn workspace_scenarios_produce_4_results() {
        let results = workspace_scenarios();
        assert_eq!(results.len(), 4);
        assert!(results.iter().all(|r| r.passed));
    }

    #[test]
    fn error_scenarios_produce_5_results() {
        let results = error_scenarios();
        assert_eq!(results.len(), 5);
        assert!(results.iter().all(|r| r.passed));
    }
}

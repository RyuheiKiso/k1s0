mod application;
mod domain;
mod infrastructure;

use std::path::PathBuf;

use application::configure_workspace::ConfigureWorkspaceUseCase;
use application::create_project::CreateProjectUseCase;
use application::port::{
    BusinessRegionRepository, MainMenuChoice, RegionCheckout, SettingsMenuChoice, UserPrompt,
};
use application::show_workspace::ShowWorkspaceUseCase;
use infrastructure::business_region_repository::GitBusinessRegionRepository;
use infrastructure::config_file::TomlConfigStore;
use infrastructure::prompt::DialoguerPrompt;
use infrastructure::sparse_checkout::GitSparseCheckout;
use infrastructure::stdin_prompt::StdinPrompt;

fn run(
    prompt: &impl UserPrompt,
    config: &impl application::port::ConfigStore,
    checkout: &impl RegionCheckout,
    business_region_repo: &impl BusinessRegionRepository,
) {
    loop {
        match prompt.show_main_menu() {
            MainMenuChoice::CreateProject => {
                CreateProjectUseCase::new(prompt, config, checkout, business_region_repo).execute();
            }
            MainMenuChoice::Settings => {
                settings_loop(prompt, config);
            }
            MainMenuChoice::Exit => {
                prompt.show_message("終了します。");
                break;
            }
        }
    }
}

fn settings_loop(prompt: &impl UserPrompt, config: &impl application::port::ConfigStore) {
    loop {
        match prompt.show_settings_menu() {
            SettingsMenuChoice::ShowWorkspacePath => {
                ShowWorkspaceUseCase::new(prompt, config).execute();
            }
            SettingsMenuChoice::SetWorkspacePath => {
                ConfigureWorkspaceUseCase::new(prompt, config).execute();
            }
            SettingsMenuChoice::Back => break,
        }
    }
}

fn main() {
    let config_path = match std::env::var("K1S0_CONFIG_DIR") {
        Ok(dir) => PathBuf::from(dir).join("config.toml"),
        Err(_) => TomlConfigStore::default_path(),
    };
    let config = TomlConfigStore::new(config_path);
    let checkout = GitSparseCheckout;
    let business_region_repo = GitBusinessRegionRepository;

    if std::env::var("K1S0_STDIN_MODE").is_ok() {
        let prompt = StdinPrompt::new();
        prompt.show_banner();
        run(&prompt, &config, &checkout, &business_region_repo);
    } else {
        let prompt = DialoguerPrompt;
        prompt.show_banner();
        run(&prompt, &config, &checkout, &business_region_repo);
    }
}

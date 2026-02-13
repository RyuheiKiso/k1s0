mod application;
mod domain;
mod infrastructure;

use application::configure_workspace::ConfigureWorkspaceUseCase;
use application::create_project::CreateProjectUseCase;
use application::port::{MainMenuChoice, SettingsMenuChoice, UserPrompt};
use application::show_workspace::ShowWorkspaceUseCase;
use infrastructure::config_file::TomlConfigStore;
use infrastructure::prompt::DialoguerPrompt;

fn run(prompt: &impl UserPrompt, config: &impl application::port::ConfigStore) {
    loop {
        match prompt.show_main_menu() {
            MainMenuChoice::CreateProject => {
                CreateProjectUseCase::new(prompt, config).execute();
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
    let prompt = DialoguerPrompt;
    let config = TomlConfigStore::new(TomlConfigStore::default_path());
    prompt.show_banner();
    run(&prompt, &config);
}

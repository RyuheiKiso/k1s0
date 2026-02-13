use dialoguer::{Input, Select};

use crate::application::port::{MainMenuChoice, SettingsMenuChoice, UserPrompt};

pub struct DialoguerPrompt;

impl UserPrompt for DialoguerPrompt {
    fn show_main_menu(&self) -> MainMenuChoice {
        let items = &["プロジェクト作成", "設定", "終了"];
        let selection = Select::new()
            .with_prompt("k1s0 メインメニュー")
            .items(items)
            .default(0)
            .interact()
            .unwrap_or(2);

        match selection {
            0 => MainMenuChoice::CreateProject,
            1 => MainMenuChoice::Settings,
            _ => MainMenuChoice::Exit,
        }
    }

    fn show_settings_menu(&self) -> SettingsMenuChoice {
        let items = &["ワークスペースパス設定", "戻る"];
        let selection = Select::new()
            .with_prompt("設定メニュー")
            .items(items)
            .default(0)
            .interact()
            .unwrap_or(1);

        match selection {
            0 => SettingsMenuChoice::SetWorkspacePath,
            _ => SettingsMenuChoice::Back,
        }
    }

    fn input_path(&self, prompt: &str) -> String {
        Input::new()
            .with_prompt(prompt)
            .interact_text()
            .unwrap_or_default()
    }

    fn show_message(&self, message: &str) {
        println!("{message}");
    }
}

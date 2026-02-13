use dialoguer::theme::ColorfulTheme;
use dialoguer::{Input, Select};

use crate::application::port::{
    BusinessRegionAction, ClientFrameworkChoice, LanguageChoice, MainMenuChoice, ProjectTypeChoice,
    RegionChoice, ServiceTypeChoice, SettingsMenuChoice, UserPrompt,
};
use crate::infrastructure::ui;

pub struct DialoguerPrompt;

impl UserPrompt for DialoguerPrompt {
    fn show_main_menu(&self) -> MainMenuChoice {
        let items = &["プロジェクト作成", "設定", "終了"];
        let selection = Select::with_theme(&ColorfulTheme::default())
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
        let items = &["ワークスペースパス確認", "ワークスペースパス設定", "戻る"];
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("設定メニュー")
            .items(items)
            .default(0)
            .interact()
            .unwrap_or(2);

        match selection {
            0 => SettingsMenuChoice::ShowWorkspacePath,
            1 => SettingsMenuChoice::SetWorkspacePath,
            _ => SettingsMenuChoice::Back,
        }
    }

    fn show_region_menu(&self) -> RegionChoice {
        let items = &[
            "system-region  : システム共通領域",
            "business-region : 部門固有領域",
            "service-region  : 業務固有領域",
        ];
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("どの領域の開発を実施しますか？")
            .items(items)
            .default(0)
            .interact()
            .unwrap_or(0);

        match selection {
            0 => RegionChoice::System,
            1 => RegionChoice::Business,
            _ => RegionChoice::Service,
        }
    }

    fn show_project_type_menu(&self) -> ProjectTypeChoice {
        let items = &["Library : ライブラリ", "Service : サービス"];
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("プロジェクト種別を選択してください")
            .items(items)
            .default(0)
            .interact()
            .unwrap_or(0);

        match selection {
            0 => ProjectTypeChoice::Library,
            _ => ProjectTypeChoice::Service,
        }
    }

    fn show_language_menu(&self) -> LanguageChoice {
        let items = &["Rust", "Go"];
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("言語を選択してください")
            .items(items)
            .default(0)
            .interact()
            .unwrap_or(0);

        match selection {
            0 => LanguageChoice::Rust,
            _ => LanguageChoice::Go,
        }
    }

    fn show_service_type_menu(&self) -> ServiceTypeChoice {
        let items = &["Client : クライアント", "Server : サーバー"];
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("サービス種別を選択してください")
            .items(items)
            .default(0)
            .interact()
            .unwrap_or(0);

        match selection {
            0 => ServiceTypeChoice::Client,
            _ => ServiceTypeChoice::Server,
        }
    }

    fn show_client_framework_menu(&self) -> ClientFrameworkChoice {
        let items = &[
            "React : Webアプリケーション",
            "Flutter : クロスプラットフォーム",
        ];
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("クライアントフレームワークを選択してください")
            .items(items)
            .default(0)
            .interact()
            .unwrap_or(0);

        match selection {
            0 => ClientFrameworkChoice::React,
            _ => ClientFrameworkChoice::Flutter,
        }
    }

    fn show_business_region_action_menu(&self) -> BusinessRegionAction {
        let items = &["既存の部門固有領域を選択", "新規追加"];
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("部門固有領域の操作を選択してください")
            .items(items)
            .default(0)
            .interact()
            .unwrap_or(0);

        match selection {
            0 => BusinessRegionAction::SelectExisting,
            _ => BusinessRegionAction::CreateNew,
        }
    }

    fn show_business_region_list(&self, regions: &[String]) -> String {
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("部門固有領域を選択してください")
            .items(regions)
            .default(0)
            .interact()
            .unwrap_or(0);

        regions[selection].clone()
    }

    fn input_business_region_name(&self) -> String {
        Input::with_theme(&ColorfulTheme::default())
            .with_prompt("部門固有領域名を入力してください")
            .interact_text()
            .unwrap_or_default()
    }

    fn input_path(&self, prompt: &str) -> String {
        Input::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .interact_text()
            .unwrap_or_default()
    }

    fn show_message(&self, message: &str) {
        println!("{}", ui::format_message(message));
    }

    fn show_banner(&self) {
        ui::render_banner();
    }
}

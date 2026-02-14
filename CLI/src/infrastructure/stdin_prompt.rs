use std::io::{self, BufRead};

use crate::application::port::{
    BusinessRegionAction, ClientFrameworkChoice, LanguageChoice, MainMenuChoice, ProjectTypeChoice,
    RegionChoice, ServiceTypeChoice, SettingsMenuChoice, UserPrompt,
};
use crate::infrastructure::ui;

pub struct StdinPrompt {
    reader: io::Stdin,
}

impl StdinPrompt {
    pub fn new() -> Self {
        Self {
            reader: io::stdin(),
        }
    }

    fn read_line(&self) -> String {
        let mut line = String::new();
        self.reader.lock().read_line(&mut line).unwrap_or(0);
        line.trim().to_string()
    }

    fn read_selection(&self) -> usize {
        self.read_line().parse::<usize>().unwrap_or(0)
    }
}

impl UserPrompt for StdinPrompt {
    fn show_main_menu(&self) -> MainMenuChoice {
        println!("k1s0 メインメニュー");
        println!("0: プロジェクト作成");
        println!("1: 設定");
        println!("2: 終了");
        match self.read_selection() {
            0 => MainMenuChoice::CreateProject,
            1 => MainMenuChoice::Settings,
            _ => MainMenuChoice::Exit,
        }
    }

    fn show_settings_menu(&self) -> SettingsMenuChoice {
        println!("設定メニュー");
        println!("0: ワークスペースパス確認");
        println!("1: ワークスペースパス設定");
        println!("2: 戻る");
        match self.read_selection() {
            0 => SettingsMenuChoice::ShowWorkspacePath,
            1 => SettingsMenuChoice::SetWorkspacePath,
            _ => SettingsMenuChoice::Back,
        }
    }

    fn show_region_menu(&self) -> RegionChoice {
        println!("どの領域の開発を実施しますか？");
        println!("0: system-region");
        println!("1: business-region");
        println!("2: service-region");
        match self.read_selection() {
            0 => RegionChoice::System,
            1 => RegionChoice::Business,
            _ => RegionChoice::Service,
        }
    }

    fn show_project_type_menu(&self) -> ProjectTypeChoice {
        println!("プロジェクト種別を選択してください");
        println!("0: Library");
        println!("1: Service");
        match self.read_selection() {
            0 => ProjectTypeChoice::Library,
            _ => ProjectTypeChoice::Service,
        }
    }

    fn show_business_project_type_menu(&self) -> ProjectTypeChoice {
        println!("プロジェクト種別を選択してください");
        println!("0: Library");
        println!("1: Service");
        println!("2: Client");
        match self.read_selection() {
            0 => ProjectTypeChoice::Library,
            1 => ProjectTypeChoice::Service,
            _ => ProjectTypeChoice::Client,
        }
    }

    fn show_language_menu(&self) -> LanguageChoice {
        println!("言語を選択してください");
        println!("0: Rust");
        println!("1: Go");
        match self.read_selection() {
            0 => LanguageChoice::Rust,
            _ => LanguageChoice::Go,
        }
    }

    fn show_service_type_menu(&self) -> ServiceTypeChoice {
        println!("サービス種別を選択してください");
        println!("0: Client");
        println!("1: Server");
        match self.read_selection() {
            0 => ServiceTypeChoice::Client,
            _ => ServiceTypeChoice::Server,
        }
    }

    fn show_client_framework_menu(&self) -> ClientFrameworkChoice {
        println!("クライアントフレームワークを選択してください");
        println!("0: React");
        println!("1: Flutter");
        match self.read_selection() {
            0 => ClientFrameworkChoice::React,
            _ => ClientFrameworkChoice::Flutter,
        }
    }

    fn show_business_region_action_menu(&self) -> BusinessRegionAction {
        println!("部門固有領域の操作を選択してください");
        println!("0: 既存の部門固有領域を選択");
        println!("1: 新規追加");
        match self.read_selection() {
            0 => BusinessRegionAction::SelectExisting,
            _ => BusinessRegionAction::CreateNew,
        }
    }

    fn show_business_region_list(&self, regions: &[String]) -> String {
        println!("部門固有領域を選択してください");
        for (i, region) in regions.iter().enumerate() {
            println!("{i}: {region}");
        }
        let selection = self.read_selection();
        if selection < regions.len() {
            regions[selection].clone()
        } else {
            regions[0].clone()
        }
    }

    fn input_business_region_name(&self) -> String {
        println!("部門固有領域名を入力してください");
        self.read_line()
    }

    fn input_path(&self, prompt: &str) -> String {
        println!("{prompt}");
        self.read_line()
    }

    fn show_message(&self, message: &str) {
        println!("{}", ui::format_message(message));
    }

    fn show_banner(&self) {
        ui::render_banner();
    }
}

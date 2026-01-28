//! `k1s0 new-screen` コマンド
//!
//! React/Flutter 画面の雛形を生成する。
//! 画面追加の手順を「雛形生成 → 画面実装 → config 追記」に統一する。

use std::path::{Path, PathBuf};

use clap::{Args, ValueEnum};
use k1s0_generator::{Context, Tera};

use crate::error::{CliError, Result};
use crate::output::output;

/// フロントエンドタイプ
#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum FrontendType {
    /// React
    #[value(name = "react")]
    React,
    /// Flutter
    #[value(name = "flutter")]
    Flutter,
}

impl std::fmt::Display for FrontendType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FrontendType::React => write!(f, "react"),
            FrontendType::Flutter => write!(f, "flutter"),
        }
    }
}

impl FrontendType {
    /// スクリーンテンプレートディレクトリの相対パスを取得
    fn screen_template_path(&self) -> &'static str {
        match self {
            FrontendType::React => "CLI/templates/frontend-react/screen",
            FrontendType::Flutter => "CLI/templates/frontend-flutter/screen",
        }
    }

    /// ページファイルの相対パスを取得
    fn page_relative_path(&self, screen_id: &str) -> String {
        match self {
            FrontendType::React => {
                // React: PascalCase (例: UsersListPage.tsx)
                let file_name = generate_component_name(screen_id);
                format!("src/pages/{}.tsx", file_name)
            }
            FrontendType::Flutter => {
                // Flutter: snake_case (例: users_list_page.dart)
                let file_name = screen_id.replace('.', "_");
                format!("lib/src/presentation/pages/{}_page.dart", file_name)
            }
        }
    }

    /// ページテンプレートファイル名を取得
    fn page_template_name(&self) -> &'static str {
        match self {
            FrontendType::React => "Page.tsx.tera",
            FrontendType::Flutter => "page.dart.tera",
        }
    }
}

/// `k1s0 new-screen` の引数
#[derive(Args, Debug)]
pub struct NewScreenArgs {
    /// フロントエンドタイプ
    #[arg(short = 't', long = "type", value_enum, default_value = "react")]
    pub frontend_type: FrontendType,

    /// 画面ID（ドット区切り、例: users.list, users.detail）
    #[arg(short, long)]
    pub screen_id: String,

    /// 画面タイトル
    #[arg(short = 'T', long)]
    pub title: String,

    /// 対象の feature ディレクトリ
    #[arg(short, long)]
    pub feature_dir: String,

    /// メニューに追加する（メニュー設定を出力）
    #[arg(long)]
    pub with_menu: bool,

    /// URL パス（指定しない場合は screen_id から自動生成）
    #[arg(short, long)]
    pub path: Option<String>,

    /// 必要な権限（カンマ区切り）
    #[arg(long)]
    pub permissions: Option<String>,

    /// 必要な feature flag（カンマ区切り）
    #[arg(long)]
    pub flags: Option<String>,

    /// 既存のファイルを上書きする
    #[arg(short = 'F', long)]
    pub force: bool,
}

/// `k1s0 new-screen` を実行する
pub fn execute(args: NewScreenArgs) -> Result<()> {
    let out = output();

    // screen_id のバリデーション
    if !is_valid_screen_id(&args.screen_id) {
        let msg = format!(
            "画面ID '{}' は無効です。小文字英数字とドット（.）のみ使用できます。例: users.list",
            args.screen_id
        );
        return Err(CliError::validation(msg).with_target("screen_id"));
    }

    // feature_dir の存在確認
    let feature_dir = PathBuf::from(&args.feature_dir);
    if !feature_dir.exists() {
        return Err(CliError::validation(format!(
            "feature ディレクトリが存在しません: {}",
            args.feature_dir
        ))
        .with_target("feature_dir")
        .with_hint("k1s0 new-feature で feature を作成してください"));
    }

    // パスの生成
    let url_path = args.path.clone().unwrap_or_else(|| {
        // screen_id からパスを生成（例: users.list -> /users/list）
        let parts: Vec<&str> = args.screen_id.split('.').collect();
        format!("/{}", parts.join("/"))
    });

    // コンポーネント名の生成（例: users.list -> UsersListPage）
    let component_name = generate_component_name(&args.screen_id);

    // ファイルパスの生成
    let file_name = generate_file_name(&args.screen_id);

    out.header("k1s0 new-screen");
    out.newline();

    out.list_item("frontend", &args.frontend_type.to_string());
    out.list_item("screen_id", &args.screen_id);
    out.list_item("title", &args.title);
    out.list_item("path", &url_path);
    out.list_item("component", &component_name);
    out.list_item("file", &file_name);
    out.list_item("feature_dir", &args.feature_dir);
    out.newline();

    // テンプレートディレクトリを検索
    let template_dir = find_screen_template_dir(args.frontend_type)?;
    out.info(&format!("テンプレート: {}", template_dir.display()));
    out.newline();

    // テンプレートコンテキストを作成
    let context = create_screen_context(&args, &component_name, &file_name, &url_path);

    // ページファイルを生成
    let page_relative_path = args.frontend_type.page_relative_path(&args.screen_id);
    let page_output_path = feature_dir.join(&page_relative_path);

    // 既存ファイルの確認
    if page_output_path.exists() && !args.force {
        return Err(CliError::conflict(format!(
            "ファイルが既に存在します: {}",
            page_output_path.display()
        ))
        .with_target(page_output_path.display().to_string())
        .with_hint("--force オプションで上書きするか、別の screen_id を指定してください"));
    }

    // テンプレートをレンダリング
    let tera = create_tera(&template_dir)?;
    let page_template_name = args.frontend_type.page_template_name();
    let page_content = tera.render(page_template_name, &context).map_err(|e| {
        CliError::internal(format!("テンプレートのレンダリングに失敗: {}", e))
    })?;

    // ディレクトリを作成
    if let Some(parent) = page_output_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            CliError::io(format!(
                "ディレクトリの作成に失敗: {}: {}",
                parent.display(),
                e
            ))
        })?;
    }

    // ファイルを書き込み
    std::fs::write(&page_output_path, &page_content).map_err(|e| {
        CliError::io(format!(
            "ファイルの書き込みに失敗: {}: {}",
            page_output_path.display(),
            e
        ))
    })?;

    out.file_added(&page_relative_path);
    out.newline();

    // 設定ファイル追記用のスニペットを出力
    match args.frontend_type {
        FrontendType::React => {
            output_react_config_snippets(&tera, &context, &args)?;
        }
        FrontendType::Flutter => {
            output_flutter_config_snippets(&tera, &context, &args)?;
        }
    }

    out.newline();
    out.success(&format!(
        "画面 '{}' を生成しました: {}",
        args.screen_id,
        page_output_path.display()
    ));

    out.newline();
    out.header("次のステップ:");
    out.hint("1. 上記の設定スニペットを config/default.yaml に追加");
    if matches!(args.frontend_type, FrontendType::React) {
        out.hint("2. src/config/screens.ts に画面を登録");
    }
    out.hint("3. 画面の実装を行う");

    Ok(())
}

/// スクリーンテンプレートディレクトリを検索する
fn find_screen_template_dir(frontend_type: FrontendType) -> Result<PathBuf> {
    let relative_path = frontend_type.screen_template_path();

    // カレントディレクトリから探す
    let current_dir = std::env::current_dir().map_err(|e| {
        CliError::io(format!("カレントディレクトリの取得に失敗: {}", e))
    })?;

    let template_dir = current_dir.join(relative_path);
    if template_dir.exists() {
        return Ok(template_dir);
    }

    // 親ディレクトリを辿って探す（モノレポ内のどこからでも実行できるように）
    let mut search_dir = current_dir.clone();
    for _ in 0..5 {
        if let Some(parent) = search_dir.parent() {
            let candidate: PathBuf = parent.join(relative_path);
            if candidate.exists() {
                return Ok(candidate);
            }
            search_dir = parent.to_path_buf();
        } else {
            break;
        }
    }

    Err(CliError::template_not_found(relative_path)
        .with_hint("k1s0 リポジトリのルートディレクトリから実行してください"))
}

/// Tera テンプレートエンジンを作成する
fn create_tera(template_dir: &Path) -> Result<Tera> {
    let pattern = format!("{}/**/*.tera", template_dir.display());
    Tera::new(&pattern).map_err(|e| {
        CliError::internal(format!("テンプレートエンジンの初期化に失敗: {}", e))
    })
}

/// スクリーン用のテンプレートコンテキストを作成する
fn create_screen_context(
    args: &NewScreenArgs,
    component_name: &str,
    file_name: &str,
    url_path: &str,
) -> Context {
    let mut context = Context::new();

    context.insert("screen_id", &args.screen_id);
    context.insert("title", &args.title);
    context.insert("path", url_path);
    context.insert("component_name", component_name);
    context.insert("file_name", file_name);

    // 権限とフラグ
    let permissions: Vec<String> = args
        .permissions
        .as_ref()
        .map(|p| p.split(',').map(|s| format!("\"{}\"", s.trim())).collect())
        .unwrap_or_default();
    let flags: Vec<String> = args
        .flags
        .as_ref()
        .map(|f| f.split(',').map(|s| format!("\"{}\"", s.trim())).collect())
        .unwrap_or_default();

    context.insert("permissions", &permissions);
    context.insert("flags", &flags);
    context.insert("with_menu", &args.with_menu);

    context
}

/// React 用の設定スニペットを出力
fn output_react_config_snippets(tera: &Tera, context: &Context, args: &NewScreenArgs) -> Result<()> {
    let out = output();

    // screens.ts スニペット
    out.header("src/config/screens.ts に追加:");
    out.newline();
    let screens_snippet = tera.render("screens.ts.tera", context).map_err(|e| {
        CliError::internal(format!("screens.ts スニペットの生成に失敗: {}", e))
    })?;
    out.info(&screens_snippet);
    out.newline();

    // route.yaml スニペット
    out.header("config/default.yaml の ui.navigation.routes に追加:");
    out.newline();
    let route_snippet = tera.render("route.yaml.tera", context).map_err(|e| {
        CliError::internal(format!("route.yaml スニペットの生成に失敗: {}", e))
    })?;
    out.info(&route_snippet);

    // menu.yaml スニペット（オプション）
    if args.with_menu {
        out.newline();
        out.header("config/default.yaml の ui.navigation.menu.items に追加:");
        out.newline();
        let menu_snippet = tera.render("menu.yaml.tera", context).map_err(|e| {
            CliError::internal(format!("menu.yaml スニペットの生成に失敗: {}", e))
        })?;
        out.info(&menu_snippet);
    }

    Ok(())
}

/// Flutter 用の設定スニペットを出力
fn output_flutter_config_snippets(tera: &Tera, context: &Context, args: &NewScreenArgs) -> Result<()> {
    let out = output();

    // route.yaml スニペット
    out.header("config/default.yaml の ui.navigation.routes に追加:");
    out.newline();
    let route_snippet = tera.render("route.yaml.tera", context).map_err(|e| {
        CliError::internal(format!("route.yaml スニペットの生成に失敗: {}", e))
    })?;
    out.info(&route_snippet);

    // menu.yaml スニペット（オプション）
    if args.with_menu {
        out.newline();
        out.header("config/default.yaml の ui.navigation.menu.items に追加:");
        out.newline();
        let menu_snippet = tera.render("menu.yaml.tera", context).map_err(|e| {
            CliError::internal(format!("menu.yaml スニペットの生成に失敗: {}", e))
        })?;
        out.info(&menu_snippet);
    }

    Ok(())
}

/// 画面ID からコンポーネント名を生成
fn generate_component_name(screen_id: &str) -> String {
    screen_id
        .split('.')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<String>()
        + "Page"
}

/// 画面ID からファイル名を生成
fn generate_file_name(screen_id: &str) -> String {
    screen_id
        .split('.')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<String>()
        + "Page"
}

/// 画面ID が有効かどうかを検証する
fn is_valid_screen_id(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    // 先頭・末尾がドットでない
    if s.starts_with('.') || s.ends_with('.') {
        return false;
    }

    // 連続するドットがない
    if s.contains("..") {
        return false;
    }

    // 許可される文字のみ
    s.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_screen_id() {
        assert!(is_valid_screen_id("home"));
        assert!(is_valid_screen_id("users.list"));
        assert!(is_valid_screen_id("users.detail"));
        assert!(is_valid_screen_id("settings.profile.edit"));

        assert!(!is_valid_screen_id("")); // 空
        assert!(!is_valid_screen_id(".users")); // 先頭ドット
        assert!(!is_valid_screen_id("users.")); // 末尾ドット
        assert!(!is_valid_screen_id("users..list")); // 連続ドット
        assert!(!is_valid_screen_id("Users")); // 大文字
    }

    #[test]
    fn test_generate_component_name() {
        assert_eq!(generate_component_name("home"), "HomePage");
        assert_eq!(generate_component_name("users.list"), "UsersListPage");
        assert_eq!(generate_component_name("users.detail"), "UsersDetailPage");
        assert_eq!(
            generate_component_name("settings.profile.edit"),
            "SettingsProfileEditPage"
        );
    }

    #[test]
    fn test_generate_file_name() {
        assert_eq!(generate_file_name("home"), "HomePage");
        assert_eq!(generate_file_name("users.list"), "UsersListPage");
    }
}

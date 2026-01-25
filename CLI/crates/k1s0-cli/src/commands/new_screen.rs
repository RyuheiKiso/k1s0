//! `k1s0 new-screen` コマンド
//!
//! React/Flutter 画面の雛形を生成する。
//! 画面追加の手順を「雛形生成 → 画面実装 → config 追記」に統一する。

use clap::{Args, ValueEnum};

use crate::error::{CliError, Result};
use crate::output::output;

/// フロントエンドタイプ
#[derive(ValueEnum, Clone, Debug)]
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
    #[arg(short, long)]
    pub force: bool,
}

/// `k1s0 new-screen` を実行する
pub fn execute(args: NewScreenArgs) -> Result<()> {
    let out = output();

    // screen_id のバリデーション
    if !is_valid_screen_id(&args.screen_id) {
        return Err(CliError::validation(format!(
            "画面ID '{}' は無効です。小文字英数字とドット（.）のみ使用できます。例: users.list",
            args.screen_id
        ))
        .with_target("screen_id"));
    }

    // パスの生成
    let url_path = args.path.clone().unwrap_or_else(|| {
        // screen_id からパスを生成（例: users.list -> /users）
        let parts: Vec<&str> = args.screen_id.split('.').collect();
        if parts.len() == 1 {
            format!("/{}", parts[0])
        } else {
            format!("/{}", parts[0])
        }
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

    match args.frontend_type {
        FrontendType::React => generate_react_screen(&args, &component_name, &file_name, &url_path),
        FrontendType::Flutter => {
            generate_flutter_screen(&args, &component_name, &file_name, &url_path)
        }
    }
}

/// React 画面の雛形を生成
fn generate_react_screen(
    args: &NewScreenArgs,
    component_name: &str,
    file_name: &str,
    url_path: &str,
) -> Result<()> {
    let out = output();

    out.header("生成されるファイル:");
    out.newline();

    let page_path = format!("{}/src/pages/{}.tsx", args.feature_dir, file_name);
    out.hint(&format!("  {}", page_path));
    out.newline();

    out.header("ファイル内容:");
    out.newline();

    // ページコンポーネントのテンプレート
    let page_content = generate_react_page_template(component_name, &args.title);
    out.info(&format!("// {}.tsx", file_name));
    out.info(&page_content);
    out.newline();

    out.header("画面レジストリへの登録:");
    out.newline();
    out.hint("src/config/screens.ts に以下を追加:");
    out.newline();

    let screens_entry = format!(
        r#"import {{ {} }} from '../pages/{}';

// screens 配列に追加:
{{
  id: '{}',
  component: {},
  meta: {{ title: '{}' }},
}},"#,
        component_name, file_name, args.screen_id, component_name, args.title
    );
    out.info(&screens_entry);
    out.newline();

    out.header("config/default.yaml への追記:");
    out.newline();

    let route_config = generate_route_config(args, url_path);
    out.hint("ui.navigation.routes に追加:");
    out.info(&route_config);
    out.newline();

    if args.with_menu {
        let menu_config = generate_menu_config(args, url_path);
        out.hint("ui.navigation.menu.items に追加:");
        out.info(&menu_config);
        out.newline();
    }

    out.header("次のステップ:");
    out.hint("1. 上記のファイルを作成");
    out.hint("2. src/config/screens.ts に画面を登録");
    out.hint("3. config/default.yaml に route を追加");
    if args.with_menu {
        out.hint("4. config/default.yaml に menu 項目を追加");
    }
    out.hint("5. 画面の実装を行う");
    out.newline();

    out.success("雛形の生成が完了しました");

    Ok(())
}

/// Flutter 画面の雛形を生成
fn generate_flutter_screen(
    args: &NewScreenArgs,
    component_name: &str,
    file_name: &str,
    url_path: &str,
) -> Result<()> {
    let out = output();

    out.header("生成されるファイル:");
    out.newline();

    let page_path = format!(
        "{}/lib/src/presentation/pages/{}_page.dart",
        args.feature_dir, file_name
    );
    out.hint(&format!("  {}", page_path));
    out.newline();

    out.header("ファイル内容:");
    out.newline();

    // ページコンポーネントのテンプレート
    let page_content = generate_flutter_page_template(component_name, &args.title);
    out.info(&format!("// {}_page.dart", file_name));
    out.info(&page_content);
    out.newline();

    out.header("config/default.yaml への追記:");
    out.newline();

    let route_config = generate_route_config(args, url_path);
    out.hint("ui.navigation.routes に追加:");
    out.info(&route_config);
    out.newline();

    if args.with_menu {
        let menu_config = generate_menu_config(args, url_path);
        out.hint("ui.navigation.menu.items に追加:");
        out.info(&menu_config);
        out.newline();
    }

    out.header("次のステップ:");
    out.hint("1. 上記のファイルを作成");
    out.hint("2. 画面レジストリに登録");
    out.hint("3. config/default.yaml に route を追加");
    if args.with_menu {
        out.hint("4. config/default.yaml に menu 項目を追加");
    }
    out.hint("5. 画面の実装を行う");
    out.newline();

    out.success("雛形の生成が完了しました");

    Ok(())
}

/// React ページテンプレートを生成
fn generate_react_page_template(component_name: &str, title: &str) -> String {
    format!(
        r#"/**
 * {} - {}
 */

import {{ Box, Typography, Toolbar }} from '@mui/material';

/**
 * {} コンポーネント
 */
export function {}() {{
  return (
    <Box>
      {{/* AppBar の高さ分のスペーサー */}}
      <Toolbar />

      <Typography variant="h4" component="h1" gutterBottom>
        {}
      </Typography>

      <Typography variant="body1">
        TODO: 画面を実装
      </Typography>
    </Box>
  );
}}"#,
        component_name, title, component_name, component_name, title
    )
}

/// Flutter ページテンプレートを生成
fn generate_flutter_page_template(component_name: &str, title: &str) -> String {
    format!(
        r#"import 'package:flutter/material.dart';

/// {} - {}
class {}Page extends StatelessWidget {{
  const {}Page({{super.key}});

  @override
  Widget build(BuildContext context) {{
    return Scaffold(
      appBar: AppBar(
        title: const Text('{}'),
      ),
      body: const Center(
        child: Text('TODO: 画面を実装'),
      ),
    );
  }}
}}"#,
        component_name, title, component_name, component_name, title
    )
}

/// ルート設定を生成
fn generate_route_config(args: &NewScreenArgs, url_path: &str) -> String {
    let mut config = format!(
        r#"- path: {}
  screen_id: {}
  title: {}"#,
        url_path, args.screen_id, args.title
    );

    // 権限・フラグの追加
    let permissions = args
        .permissions
        .as_ref()
        .map(|p| p.split(',').map(|s| s.trim()).collect::<Vec<_>>())
        .unwrap_or_default();

    let flags = args
        .flags
        .as_ref()
        .map(|f| f.split(',').map(|s| s.trim()).collect::<Vec<_>>())
        .unwrap_or_default();

    if !permissions.is_empty() || !flags.is_empty() {
        config.push_str("\n  requires:");
        if !permissions.is_empty() {
            config.push_str(&format!(
                "\n    permissions: [{}]",
                permissions
                    .iter()
                    .map(|p| format!("\"{}\"", p))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        } else {
            config.push_str("\n    permissions: []");
        }
        if !flags.is_empty() {
            config.push_str(&format!(
                "\n    flags: [{}]",
                flags
                    .iter()
                    .map(|f| format!("\"{}\"", f))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        } else {
            config.push_str("\n    flags: []");
        }
    }

    config
}

/// メニュー設定を生成
fn generate_menu_config(args: &NewScreenArgs, url_path: &str) -> String {
    let mut config = format!(
        r#"- label: {}
  to: {}
  icon: default"#,
        args.title, url_path
    );

    // 権限・フラグの追加
    let permissions = args
        .permissions
        .as_ref()
        .map(|p| p.split(',').map(|s| s.trim()).collect::<Vec<_>>())
        .unwrap_or_default();

    let flags = args
        .flags
        .as_ref()
        .map(|f| f.split(',').map(|s| s.trim()).collect::<Vec<_>>())
        .unwrap_or_default();

    if !permissions.is_empty() || !flags.is_empty() {
        config.push_str("\n  requires:");
        if !permissions.is_empty() {
            config.push_str(&format!(
                "\n    permissions: [{}]",
                permissions
                    .iter()
                    .map(|p| format!("\"{}\"", p))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if !flags.is_empty() {
            config.push_str(&format!(
                "\n    flags: [{}]",
                flags
                    .iter()
                    .map(|f| format!("\"{}\"", f))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }

    config
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

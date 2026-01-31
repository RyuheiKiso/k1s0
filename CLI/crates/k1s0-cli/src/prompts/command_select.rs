//! コマンド選択プロンプト
//!
//! 引数なしで k1s0 を実行した際のサブコマンド選択を提供します。

use inquire::Select;

use crate::error::Result;
use crate::prompts::{cancelled_error, get_render_config};

/// 選択可能なコマンド
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectedCommand {
    /// 新しいフィーチャーサービスを作成
    NewFeature,
    /// 新しいドメインライブラリを作成
    NewDomain,
    /// 新しい画面を作成
    NewScreen,
    /// リポジトリを初期化
    Init,
    /// 規約チェックを実行
    Lint,
    /// テンプレートをアップグレード
    Upgrade,
    /// ドメイン管理
    Domain,
    /// シェル補完スクリプトを生成
    Completions,
    /// playground 環境の起動・停止
    Playground,
    /// 既存プロジェクトの移行
    Migrate,
}

impl SelectedCommand {
    /// コマンドが対話モードで継続できるかどうか
    pub fn supports_interactive(&self) -> bool {
        matches!(
            self,
            Self::NewFeature | Self::NewDomain | Self::NewScreen | Self::Init | Self::Playground | Self::Migrate
        )
    }

    /// コマンドのサブコマンド名を取得
    pub fn subcommand_name(&self) -> &'static str {
        match self {
            Self::NewFeature => "new-feature",
            Self::NewDomain => "new-domain",
            Self::NewScreen => "new-screen",
            Self::Init => "init",
            Self::Lint => "lint",
            Self::Upgrade => "upgrade",
            Self::Domain => "domain",
            Self::Completions => "completions",
            Self::Playground => "playground",
            Self::Migrate => "migrate",
        }
    }
}

/// コマンドの選択肢
struct CommandOption {
    command: SelectedCommand,
    label: &'static str,
    description: &'static str,
}

impl std::fmt::Display for CommandOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:<16} {}", self.label, self.description)
    }
}

/// コマンドを選択するプロンプト
///
/// 利用可能なサブコマンドから 1 つを選択できます。
///
/// # Returns
///
/// 選択された `SelectedCommand`
///
/// # Errors
///
/// ユーザーがキャンセルした場合
pub fn select_command() -> Result<SelectedCommand> {
    let options = vec![
        CommandOption {
            command: SelectedCommand::NewFeature,
            label: "new-feature",
            description: "新しいフィーチャーサービスを作成",
        },
        CommandOption {
            command: SelectedCommand::NewDomain,
            label: "new-domain",
            description: "新しいドメインライブラリを作成",
        },
        CommandOption {
            command: SelectedCommand::NewScreen,
            label: "new-screen",
            description: "新しい画面を作成",
        },
        CommandOption {
            command: SelectedCommand::Init,
            label: "init",
            description: "リポジトリを初期化",
        },
        CommandOption {
            command: SelectedCommand::Lint,
            label: "lint",
            description: "規約チェックを実行",
        },
        CommandOption {
            command: SelectedCommand::Upgrade,
            label: "upgrade",
            description: "テンプレートをアップグレード",
        },
        CommandOption {
            command: SelectedCommand::Domain,
            label: "domain",
            description: "ドメイン管理（list, version, dependents, impact）",
        },
        CommandOption {
            command: SelectedCommand::Completions,
            label: "completions",
            description: "シェル補完スクリプトを生成",
        },
        CommandOption {
            command: SelectedCommand::Playground,
            label: "playground",
            description: "playground 環境の起動・停止",
        },
        CommandOption {
            command: SelectedCommand::Migrate,
            label: "migrate",
            description: "既存プロジェクトを k1s0 構造に移行",
        },
    ];

    let answer = Select::new("実行するコマンドを選択してください:", options)
        .with_render_config(get_render_config())
        .with_help_message("矢印キーで選択、Enter で確定")
        .prompt()
        .map_err(|_| cancelled_error())?;

    Ok(answer.command)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selected_command_supports_interactive() {
        assert!(SelectedCommand::NewFeature.supports_interactive());
        assert!(SelectedCommand::NewDomain.supports_interactive());
        assert!(SelectedCommand::NewScreen.supports_interactive());
        assert!(SelectedCommand::Init.supports_interactive());
        assert!(!SelectedCommand::Lint.supports_interactive());
        assert!(!SelectedCommand::Upgrade.supports_interactive());
        assert!(!SelectedCommand::Domain.supports_interactive());
        assert!(!SelectedCommand::Completions.supports_interactive());
        assert!(SelectedCommand::Playground.supports_interactive());
    }

    #[test]
    fn test_selected_command_subcommand_name() {
        assert_eq!(SelectedCommand::NewFeature.subcommand_name(), "new-feature");
        assert_eq!(SelectedCommand::NewDomain.subcommand_name(), "new-domain");
        assert_eq!(SelectedCommand::NewScreen.subcommand_name(), "new-screen");
        assert_eq!(SelectedCommand::Init.subcommand_name(), "init");
        assert_eq!(SelectedCommand::Lint.subcommand_name(), "lint");
        assert_eq!(SelectedCommand::Upgrade.subcommand_name(), "upgrade");
        assert_eq!(SelectedCommand::Domain.subcommand_name(), "domain");
        assert_eq!(
            SelectedCommand::Completions.subcommand_name(),
            "completions"
        );
        assert_eq!(
            SelectedCommand::Playground.subcommand_name(),
            "playground"
        );
    }
}

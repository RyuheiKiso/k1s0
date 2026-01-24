//! `k1s0 completions` コマンド
//!
//! シェル補完スクリプトを生成する。

use anyhow::Result;
use clap::{Args, CommandFactory, ValueEnum};
use clap_complete::{generate, Shell};

/// シェルの種類
#[derive(ValueEnum, Clone, Debug)]
pub enum ShellType {
    Bash,
    Zsh,
    Fish,
    PowerShell,
}

impl From<ShellType> for Shell {
    fn from(shell: ShellType) -> Self {
        match shell {
            ShellType::Bash => Shell::Bash,
            ShellType::Zsh => Shell::Zsh,
            ShellType::Fish => Shell::Fish,
            ShellType::PowerShell => Shell::PowerShell,
        }
    }
}

/// `k1s0 completions` の引数
#[derive(Args, Debug)]
pub struct CompletionsArgs {
    /// 補完スクリプトを生成するシェル
    #[arg(value_enum)]
    pub shell: ShellType,
}

/// `k1s0 completions` を実行する
pub fn execute(args: CompletionsArgs) -> Result<()> {
    let mut cmd = crate::Cli::command();
    let shell: Shell = args.shell.into();
    generate(shell, &mut cmd, "k1s0", &mut std::io::stdout());

    Ok(())
}

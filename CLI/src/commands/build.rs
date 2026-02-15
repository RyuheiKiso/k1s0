use anyhow::Result;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::prompt::{self, ConfirmResult};

/// ビルドモード。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildMode {
    Development,
    Production,
}

impl BuildMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            BuildMode::Development => "development",
            BuildMode::Production => "production",
        }
    }
}

const BUILD_MODE_LABELS: &[&str] = &["development", "production"];

/// ビルド設定。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildConfig {
    /// ビルド対象のパス一覧
    pub targets: Vec<String>,
    /// ビルドモード
    pub mode: BuildMode,
}

/// ビルドフローのステップ。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    Targets,
    Mode,
    Confirm,
}

/// ビルドコマンドを実行する。
///
/// CLIフロー.md の「ビルド」セクションに準拠:
/// [1] ビルド対象の選択（複数選択可、「すべて」あり）
/// [2] ビルドモードの選択 (development / production)
/// [確認] 確認画面
///
/// 各ステップで Esc を押すと前のステップに戻る。
/// 最初のステップで Esc → メインメニューに戻る。
pub fn run() -> Result<()> {
    println!("\n--- ビルド ---\n");

    let mut step = Step::Targets;
    let mut targets: Vec<String> = Vec::new();
    let mut mode = BuildMode::Development;

    loop {
        match step {
            Step::Targets => {
                match step_select_targets()? {
                    Some(t) => {
                        if t.is_empty() {
                            println!("ビルド対象が見つかりません。先にひな形を生成してください。");
                            return Ok(());
                        }
                        targets = t;
                        step = Step::Mode;
                    }
                    None => return Ok(()), // 最初のステップで Esc → メインメニューに戻る
                }
            }
            Step::Mode => {
                match step_build_mode()? {
                    Some(m) => {
                        mode = m;
                        step = Step::Confirm;
                    }
                    None => step = Step::Targets, // Esc → Targets に戻る
                }
            }
            Step::Confirm => {
                let config = BuildConfig {
                    targets: targets.clone(),
                    mode,
                };
                print_confirmation(&config);
                match prompt::confirm_prompt()? {
                    ConfirmResult::Yes => {
                        execute_build(&config)?;
                        println!("\nビルドが完了しました。");
                        return Ok(());
                    }
                    ConfirmResult::GoBack => step = Step::Mode, // GoBack → Mode に戻る
                    ConfirmResult::Cancel => {
                        println!("キャンセルしました。");
                        return Ok(());
                    }
                }
            }
        }
    }
}

/// ステップ1: ビルド対象の選択
fn step_select_targets() -> Result<Option<Vec<String>>> {
    let all_targets = scan_buildable_targets();
    if all_targets.is_empty() {
        return Ok(Some(Vec::new()));
    }

    let mut items: Vec<&str> = vec!["すべて"];
    for t in &all_targets {
        items.push(t.as_str());
    }

    let selected = prompt::multi_select_prompt(
        "ビルド対象を選択してください（複数選択可）",
        &items,
    )?;

    match selected {
        None => Ok(None),
        Some(indices) => {
            if indices.is_empty() {
                println!("少なくとも1つの対象を選択してください。");
                step_select_targets()
            } else if indices.contains(&0) {
                // 「すべて」が選択されている
                Ok(Some(all_targets))
            } else {
                let targets: Vec<String> = indices
                    .iter()
                    .map(|&i| all_targets[i - 1].clone())
                    .collect();
                Ok(Some(targets))
            }
        }
    }
}

/// ステップ2: ビルドモード選択
fn step_build_mode() -> Result<Option<BuildMode>> {
    let idx = prompt::select_prompt("ビルドモードを選択してください", BUILD_MODE_LABELS)?;
    Ok(idx.map(|i| match i {
        0 => BuildMode::Development,
        1 => BuildMode::Production,
        _ => unreachable!(),
    }))
}

/// 確認内容を表示する。
fn print_confirmation(config: &BuildConfig) {
    println!("\n[確認] 以下の対象をビルドします。よろしいですか？");
    for target in &config.targets {
        println!("    対象:   {}", target);
    }
    println!("    モード: {}", config.mode.as_str());
}

/// ビルド実行。
pub fn execute_build(config: &BuildConfig) -> Result<()> {
    for target in &config.targets {
        println!("\nビルド中: {} ({})", target, config.mode.as_str());
        let target_path = Path::new(target);

        if !target_path.is_dir() {
            println!("  警告: ディレクトリが見つかりません: {}", target);
            continue;
        }

        // 言語/フレームワーク種別を検出してビルドコマンドを決定
        if target_path.join("go.mod").exists() {
            run_command("go", &["build", "./..."], target_path)?;
        } else if target_path.join("Cargo.toml").exists() {
            let args = if config.mode == BuildMode::Production {
                vec!["build", "--release"]
            } else {
                vec!["build"]
            };
            run_command("cargo", &args, target_path)?;
        } else if target_path.join("package.json").exists() {
            let script = if config.mode == BuildMode::Production {
                "build"
            } else {
                "dev"
            };
            run_command("npm", &["run", script], target_path)?;
        } else if target_path.join("pubspec.yaml").exists() {
            run_command("flutter", &["build"], target_path)?;
        } else {
            println!("  警告: ビルド方法が不明なディレクトリです: {}", target);
        }
    }
    Ok(())
}

/// 外部コマンドを実行する。
fn run_command(cmd: &str, args: &[&str], cwd: &Path) -> Result<()> {
    println!("  実行: {} {}", cmd, args.join(" "));
    let status = Command::new(cmd).args(args).current_dir(cwd).status();
    match status {
        Ok(s) if s.success() => {
            println!("  完了");
            Ok(())
        }
        Ok(s) => {
            let code = s.code().unwrap_or(-1);
            anyhow::bail!("コマンドがエラーで終了しました (exit code: {})", code);
        }
        Err(e) => {
            println!("  警告: コマンド '{}' の実行に失敗しました: {}", cmd, e);
            Ok(())
        }
    }
}

/// ビルド可能な対象を走査する。
/// regions/ 配下の server/ client/ library/ を探す。
pub fn scan_buildable_targets() -> Vec<String> {
    scan_buildable_targets_at(Path::new("."))
}

/// 指定ディレクトリを起点にビルド可能な対象を走査する。
pub fn scan_buildable_targets_at(base_dir: &Path) -> Vec<String> {
    let mut targets = Vec::new();
    let regions = base_dir.join("regions");
    if !regions.is_dir() {
        return targets;
    }
    scan_targets_recursive(&regions, &mut targets);
    targets.sort();
    targets
}

fn scan_targets_recursive(path: &Path, targets: &mut Vec<String>) {
    if !path.is_dir() {
        return;
    }

    // ビルド可能なプロジェクトを検出
    let is_buildable = path.join("go.mod").exists()
        || path.join("Cargo.toml").exists()
        || path.join("package.json").exists()
        || path.join("pubspec.yaml").exists();

    if is_buildable {
        if let Some(p) = path.to_str() {
            targets.push(p.to_string());
        }
        return; // これ以上深く探索しない
    }

    // サブディレクトリを探索
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                scan_targets_recursive(&entry.path(), targets);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // --- BuildMode ---

    #[test]
    fn test_build_mode_as_str() {
        assert_eq!(BuildMode::Development.as_str(), "development");
        assert_eq!(BuildMode::Production.as_str(), "production");
    }

    // --- BuildConfig ---

    #[test]
    fn test_build_config_creation() {
        let config = BuildConfig {
            targets: vec!["regions/system/server/go/auth".to_string()],
            mode: BuildMode::Development,
        };
        assert_eq!(config.targets.len(), 1);
        assert_eq!(config.mode, BuildMode::Development);
    }

    // --- scan_buildable_targets ---

    #[test]
    fn test_scan_buildable_targets_empty() {
        let tmp = TempDir::new().unwrap();

        let targets = scan_buildable_targets_at(tmp.path());

        assert!(targets.is_empty());
    }

    #[test]
    fn test_scan_buildable_targets_with_projects() {
        let tmp = TempDir::new().unwrap();

        // Go プロジェクト
        let go_path = tmp.path().join("regions/system/server/go/auth");
        fs::create_dir_all(&go_path).unwrap();
        fs::write(go_path.join("go.mod"), "module auth\n").unwrap();

        // Rust プロジェクト
        let rust_path = tmp.path().join("regions/business/accounting/server/rust/ledger");
        fs::create_dir_all(&rust_path).unwrap();
        fs::write(rust_path.join("Cargo.toml"), "[package]\n").unwrap();

        // React プロジェクト
        let react_path = tmp.path().join("regions/service/order/client/react");
        fs::create_dir_all(&react_path).unwrap();
        fs::write(react_path.join("package.json"), "{}").unwrap();

        let targets = scan_buildable_targets_at(tmp.path());

        assert_eq!(targets.len(), 3);
    }

    // --- execute_build ---

    #[test]
    fn test_execute_build_nonexistent_target() {
        let config = BuildConfig {
            targets: vec!["/nonexistent/path".to_string()],
            mode: BuildMode::Development,
        };
        // 存在しないパスは警告を出すが、エラーにはならない
        let result = execute_build(&config);
        assert!(result.is_ok());
    }
}

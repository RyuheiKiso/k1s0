use anyhow::Result;
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::progress::ProgressEvent;

/// ビルドモード。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

/// ビルド設定。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildConfig {
    /// ビルド対象のパス一覧
    pub targets: Vec<String>,
    /// ビルドモード
    pub mode: BuildMode,
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

/// プログレスコールバック付きビルド実行。
pub fn execute_build_with_progress(
    config: &BuildConfig,
    on_progress: impl Fn(ProgressEvent),
) -> Result<()> {
    let total = config.targets.len();
    for (i, target) in config.targets.iter().enumerate() {
        let step = i + 1;
        on_progress(ProgressEvent::StepStarted {
            step,
            total,
            message: format!("ビルド中: {} ({})", target, config.mode.as_str()),
        });

        let target_path = Path::new(target);
        if !target_path.is_dir() {
            on_progress(ProgressEvent::Warning {
                message: format!("ディレクトリが見つかりません: {}", target),
            });
            on_progress(ProgressEvent::StepCompleted {
                step,
                total,
                message: format!("スキップ: {}", target),
            });
            continue;
        }

        let result = run_command_with_progress(target_path, config.mode, &on_progress);
        match result {
            Ok(()) => {
                on_progress(ProgressEvent::StepCompleted {
                    step,
                    total,
                    message: format!("ビルド完了: {}", target),
                });
            }
            Err(e) => {
                on_progress(ProgressEvent::Error {
                    message: format!("ビルド失敗: {} - {}", target, e),
                });
            }
        }
    }
    on_progress(ProgressEvent::Finished {
        success: true,
        message: "すべてのビルドが完了しました".to_string(),
    });
    Ok(())
}

fn run_command_with_progress(
    target_path: &Path,
    mode: BuildMode,
    on_progress: &impl Fn(ProgressEvent),
) -> Result<()> {
    if target_path.join("go.mod").exists() {
        on_progress(ProgressEvent::Log {
            message: "実行: go build ./...".to_string(),
        });
        run_command("go", &["build", "./..."], target_path)?;
    } else if target_path.join("Cargo.toml").exists() {
        let args = if mode == BuildMode::Production {
            vec!["build", "--release"]
        } else {
            vec!["build"]
        };
        on_progress(ProgressEvent::Log {
            message: format!("実行: cargo {}", args.join(" ")),
        });
        run_command("cargo", &args, target_path)?;
    } else if target_path.join("package.json").exists() {
        let script = if mode == BuildMode::Production {
            "build"
        } else {
            "dev"
        };
        on_progress(ProgressEvent::Log {
            message: format!("実行: npm run {}", script),
        });
        run_command("npm", &["run", script], target_path)?;
    } else if target_path.join("pubspec.yaml").exists() {
        on_progress(ProgressEvent::Log {
            message: "実行: flutter build".to_string(),
        });
        run_command("flutter", &["build"], target_path)?;
    } else {
        on_progress(ProgressEvent::Warning {
            message: format!("ビルド方法が不明なディレクトリです: {}", target_path.display()),
        });
    }
    Ok(())
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
            targets: vec!["regions/system/server/rust/auth".to_string()],
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

        // Rust サーバープロジェクト (system tier)
        let rust_system_path = tmp.path().join("regions/system/server/rust/auth");
        fs::create_dir_all(&rust_system_path).unwrap();
        fs::write(rust_system_path.join("Cargo.toml"), "[package]\n").unwrap();

        // Rust プロジェクト (business tier)
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

    // --- execute_build_with_progress ---

    #[test]
    fn test_execute_build_with_progress_nonexistent_target() {
        let config = BuildConfig {
            targets: vec!["/nonexistent/path".to_string()],
            mode: BuildMode::Development,
        };
        let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let events_clone = events.clone();
        let result = execute_build_with_progress(&config, move |event| {
            events_clone.lock().unwrap().push(event);
        });
        assert!(result.is_ok());

        let collected = events.lock().unwrap();
        // StepStarted + Warning + StepCompleted + Finished = 4 events
        assert!(collected.len() >= 3);
        assert!(matches!(&collected[0], ProgressEvent::StepStarted { step: 1, total: 1, .. }));
        assert!(matches!(&collected[1], ProgressEvent::Warning { .. }));
        assert!(matches!(collected.last().unwrap(), ProgressEvent::Finished { success: true, .. }));
    }

    #[test]
    fn test_execute_build_with_progress_empty_targets() {
        let config = BuildConfig {
            targets: vec![],
            mode: BuildMode::Development,
        };
        let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let events_clone = events.clone();
        let result = execute_build_with_progress(&config, move |event| {
            events_clone.lock().unwrap().push(event);
        });
        assert!(result.is_ok());

        let collected = events.lock().unwrap();
        // Only Finished event
        assert_eq!(collected.len(), 1);
        assert!(matches!(&collected[0], ProgressEvent::Finished { success: true, .. }));
    }

    #[test]
    fn test_execute_build_with_progress_unknown_project_type() {
        let tmp = TempDir::new().unwrap();
        // ビルドファイルなしのディレクトリ
        let config = BuildConfig {
            targets: vec![tmp.path().to_string_lossy().to_string()],
            mode: BuildMode::Development,
        };
        let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let events_clone = events.clone();
        let result = execute_build_with_progress(&config, move |event| {
            events_clone.lock().unwrap().push(event);
        });
        assert!(result.is_ok());

        let collected = events.lock().unwrap();
        // Should have StepStarted, Warning (unknown), StepCompleted, Finished
        assert!(collected.iter().any(|e| matches!(e, ProgressEvent::Warning { .. })));
    }
}

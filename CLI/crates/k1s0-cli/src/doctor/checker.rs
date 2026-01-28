//! ツールチェッカー
//!
//! 各ツールのインストール状況とバージョンをチェックする。

use std::process::Command;

use crate::doctor::requirements::{ToolCategory, ToolRequirement};

/// チェック結果
#[derive(Debug, Clone)]
pub enum CheckStatus {
    /// チェック成功
    Ok {
        /// 検出されたバージョン
        version: String,
    },
    /// バージョン不一致
    VersionMismatch {
        /// 検出されたバージョン
        actual: String,
        /// 必要なバージョン
        required: String,
    },
    /// ツールが見つからない
    NotFound,
    /// チェック中にエラー発生
    Error(String),
}

impl CheckStatus {
    /// 成功かどうか
    pub fn is_ok(&self) -> bool {
        matches!(self, CheckStatus::Ok { .. })
    }

    /// 見つからなかったかどうか
    pub fn is_not_found(&self) -> bool {
        matches!(self, CheckStatus::NotFound)
    }

    /// バージョン文字列を取得（あれば）
    pub fn version(&self) -> Option<&str> {
        match self {
            CheckStatus::Ok { version } => Some(version),
            CheckStatus::VersionMismatch { actual, .. } => Some(actual),
            _ => None,
        }
    }
}

/// ツールチェック結果
#[derive(Debug, Clone)]
pub struct ToolCheck {
    /// ツール要件
    pub requirement: &'static ToolRequirement,
    /// チェック結果
    pub status: CheckStatus,
    /// ツールのパス（見つかった場合）
    pub path: Option<String>,
}

impl ToolCheck {
    /// 成功かどうか
    pub fn is_ok(&self) -> bool {
        self.status.is_ok()
    }

    /// 問題があるかどうか（必須ツールで失敗、またはバージョン不一致）
    pub fn has_problem(&self) -> bool {
        match &self.status {
            CheckStatus::Ok { .. } => false,
            CheckStatus::VersionMismatch { .. } => true,
            CheckStatus::NotFound => self.requirement.required,
            CheckStatus::Error(_) => self.requirement.required,
        }
    }
}

/// コマンドを実行して出力を取得
fn run_command(cmd: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                "not found".to_string()
            } else {
                e.to_string()
            }
        })?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        // 一部のツールは stderr に出力する
        Ok(if stdout.trim().is_empty() {
            stderr
        } else {
            stdout
        })
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

/// which/where コマンドでパスを取得
fn find_tool_path(tool: &str) -> Option<String> {
    #[cfg(target_os = "windows")]
    let (cmd, args) = ("where", vec![tool]);
    #[cfg(not(target_os = "windows"))]
    let (cmd, args) = ("which", vec![tool]);

    run_command(cmd, &args)
        .ok()
        .map(|s| s.lines().next().unwrap_or("").trim().to_string())
        .filter(|s| !s.is_empty())
}

/// バージョン文字列を解析して比較可能な形式に変換
fn parse_version(version_str: &str) -> Option<(u32, u32, u32)> {
    // バージョン番号のパターンを探す（X.Y.Z）
    let re_pattern = regex::Regex::new(r"(\d+)\.(\d+)\.(\d+)").ok()?;
    let caps = re_pattern.captures(version_str)?;

    let major = caps.get(1)?.as_str().parse().ok()?;
    let minor = caps.get(2)?.as_str().parse().ok()?;
    let patch = caps.get(3)?.as_str().parse().ok()?;

    Some((major, minor, patch))
}

/// バージョンを比較（actual >= required ならtrue）
fn compare_versions(actual: &str, required: &str) -> bool {
    let actual_ver = parse_version(actual);
    let required_ver = parse_version(required);

    match (actual_ver, required_ver) {
        (Some((a_major, a_minor, a_patch)), Some((r_major, r_minor, r_patch))) => {
            (a_major, a_minor, a_patch) >= (r_major, r_minor, r_patch)
        }
        _ => true, // パースできない場合は OK とみなす
    }
}

/// Rust のバージョンをチェック
fn check_rust_version() -> (CheckStatus, Option<String>) {
    let path = find_tool_path("rustc");

    match run_command("rustc", &["--version"]) {
        Ok(output) => {
            // "rustc X.Y.Z (hash date)" 形式
            if let Some(version) = parse_version(&output) {
                let version_str = format!("{}.{}.{}", version.0, version.1, version.2);
                if compare_versions(&version_str, super::requirements::RUST_MIN_VERSION) {
                    (CheckStatus::Ok { version: version_str }, path)
                } else {
                    (
                        CheckStatus::VersionMismatch {
                            actual: version_str,
                            required: super::requirements::RUST_MIN_VERSION.to_string(),
                        },
                        path,
                    )
                }
            } else {
                (
                    CheckStatus::Error("バージョンを解析できません".to_string()),
                    path,
                )
            }
        }
        Err(e) if e == "not found" => (CheckStatus::NotFound, None),
        Err(e) => (CheckStatus::Error(e), path),
    }
}

/// Cargo のバージョンをチェック
fn check_cargo_version() -> (CheckStatus, Option<String>) {
    let path = find_tool_path("cargo");

    match run_command("cargo", &["--version"]) {
        Ok(output) => {
            if let Some(version) = parse_version(&output) {
                let version_str = format!("{}.{}.{}", version.0, version.1, version.2);
                (CheckStatus::Ok { version: version_str }, path)
            } else {
                (
                    CheckStatus::Error("バージョンを解析できません".to_string()),
                    path,
                )
            }
        }
        Err(e) if e == "not found" => (CheckStatus::NotFound, None),
        Err(e) => (CheckStatus::Error(e), path),
    }
}

/// Node.js のバージョンをチェック
fn check_node_version() -> (CheckStatus, Option<String>) {
    let path = find_tool_path("node");

    match run_command("node", &["--version"]) {
        Ok(output) => {
            // "vX.Y.Z" 形式
            if let Some(version) = parse_version(&output) {
                let version_str = format!("{}.{}.{}", version.0, version.1, version.2);
                if compare_versions(&version_str, super::requirements::NODE_MIN_VERSION) {
                    (CheckStatus::Ok { version: version_str }, path)
                } else {
                    (
                        CheckStatus::VersionMismatch {
                            actual: version_str,
                            required: super::requirements::NODE_MIN_VERSION.to_string(),
                        },
                        path,
                    )
                }
            } else {
                (
                    CheckStatus::Error("バージョンを解析できません".to_string()),
                    path,
                )
            }
        }
        Err(e) if e == "not found" => (CheckStatus::NotFound, None),
        Err(e) => (CheckStatus::Error(e), path),
    }
}

/// pnpm のバージョンをチェック
fn check_pnpm_version() -> (CheckStatus, Option<String>) {
    let path = find_tool_path("pnpm");

    match run_command("pnpm", &["--version"]) {
        Ok(output) => {
            if let Some(version) = parse_version(&output) {
                let version_str = format!("{}.{}.{}", version.0, version.1, version.2);
                if compare_versions(&version_str, super::requirements::PNPM_MIN_VERSION) {
                    (CheckStatus::Ok { version: version_str }, path)
                } else {
                    (
                        CheckStatus::VersionMismatch {
                            actual: version_str,
                            required: super::requirements::PNPM_MIN_VERSION.to_string(),
                        },
                        path,
                    )
                }
            } else {
                (
                    CheckStatus::Error("バージョンを解析できません".to_string()),
                    path,
                )
            }
        }
        Err(e) if e == "not found" => (CheckStatus::NotFound, None),
        Err(e) => (CheckStatus::Error(e), path),
    }
}

/// Go のバージョンをチェック
fn check_go_version() -> (CheckStatus, Option<String>) {
    let path = find_tool_path("go");

    match run_command("go", &["version"]) {
        Ok(output) => {
            // "go version goX.Y.Z platform/arch" 形式
            if let Some(version) = parse_version(&output) {
                let version_str = format!("{}.{}.{}", version.0, version.1, version.2);
                if compare_versions(&version_str, super::requirements::GO_MIN_VERSION) {
                    (CheckStatus::Ok { version: version_str }, path)
                } else {
                    (
                        CheckStatus::VersionMismatch {
                            actual: version_str,
                            required: super::requirements::GO_MIN_VERSION.to_string(),
                        },
                        path,
                    )
                }
            } else {
                (
                    CheckStatus::Error("バージョンを解析できません".to_string()),
                    path,
                )
            }
        }
        Err(e) if e == "not found" => (CheckStatus::NotFound, None),
        Err(e) => (CheckStatus::Error(e), path),
    }
}

/// golangci-lint のバージョンをチェック
fn check_golangci_lint_version() -> (CheckStatus, Option<String>) {
    let path = find_tool_path("golangci-lint");

    match run_command("golangci-lint", &["--version"]) {
        Ok(output) => {
            if let Some(version) = parse_version(&output) {
                let version_str = format!("{}.{}.{}", version.0, version.1, version.2);
                if compare_versions(&version_str, super::requirements::GOLANGCI_LINT_MIN_VERSION) {
                    (CheckStatus::Ok { version: version_str }, path)
                } else {
                    (
                        CheckStatus::VersionMismatch {
                            actual: version_str,
                            required: super::requirements::GOLANGCI_LINT_MIN_VERSION.to_string(),
                        },
                        path,
                    )
                }
            } else {
                (
                    CheckStatus::Error("バージョンを解析できません".to_string()),
                    path,
                )
            }
        }
        Err(e) if e == "not found" => (CheckStatus::NotFound, None),
        Err(e) => (CheckStatus::Error(e), path),
    }
}

/// buf のバージョンをチェック
fn check_buf_version() -> (CheckStatus, Option<String>) {
    let path = find_tool_path("buf");

    match run_command("buf", &["--version"]) {
        Ok(output) => {
            if let Some(version) = parse_version(&output) {
                let version_str = format!("{}.{}.{}", version.0, version.1, version.2);
                if compare_versions(&version_str, super::requirements::BUF_MIN_VERSION) {
                    (CheckStatus::Ok { version: version_str }, path)
                } else {
                    (
                        CheckStatus::VersionMismatch {
                            actual: version_str,
                            required: super::requirements::BUF_MIN_VERSION.to_string(),
                        },
                        path,
                    )
                }
            } else {
                (
                    CheckStatus::Error("バージョンを解析できません".to_string()),
                    path,
                )
            }
        }
        Err(e) if e == "not found" => (CheckStatus::NotFound, None),
        Err(e) => (CheckStatus::Error(e), path),
    }
}

/// Flutter のバージョンをチェック
fn check_flutter_version() -> (CheckStatus, Option<String>) {
    let path = find_tool_path("flutter");

    match run_command("flutter", &["--version"]) {
        Ok(output) => {
            // "Flutter X.Y.Z ..." 形式
            if let Some(version) = parse_version(&output) {
                let version_str = format!("{}.{}.{}", version.0, version.1, version.2);
                if compare_versions(&version_str, super::requirements::FLUTTER_MIN_VERSION) {
                    (CheckStatus::Ok { version: version_str }, path)
                } else {
                    (
                        CheckStatus::VersionMismatch {
                            actual: version_str,
                            required: super::requirements::FLUTTER_MIN_VERSION.to_string(),
                        },
                        path,
                    )
                }
            } else {
                (
                    CheckStatus::Error("バージョンを解析できません".to_string()),
                    path,
                )
            }
        }
        Err(e) if e == "not found" => (CheckStatus::NotFound, None),
        Err(e) => (CheckStatus::Error(e), path),
    }
}

/// Dart のバージョンをチェック
fn check_dart_version() -> (CheckStatus, Option<String>) {
    let path = find_tool_path("dart");

    match run_command("dart", &["--version"]) {
        Ok(output) => {
            // "Dart SDK version: X.Y.Z ..." 形式
            if let Some(version) = parse_version(&output) {
                let version_str = format!("{}.{}.{}", version.0, version.1, version.2);
                if compare_versions(&version_str, super::requirements::DART_MIN_VERSION) {
                    (CheckStatus::Ok { version: version_str }, path)
                } else {
                    (
                        CheckStatus::VersionMismatch {
                            actual: version_str,
                            required: super::requirements::DART_MIN_VERSION.to_string(),
                        },
                        path,
                    )
                }
            } else {
                (
                    CheckStatus::Error("バージョンを解析できません".to_string()),
                    path,
                )
            }
        }
        Err(e) if e == "not found" => (CheckStatus::NotFound, None),
        Err(e) => (CheckStatus::Error(e), path),
    }
}

/// 指定されたツールをチェック
pub fn check_tool(requirement: &'static ToolRequirement) -> ToolCheck {
    let (status, path) = match requirement.name {
        "rustc" => check_rust_version(),
        "cargo" => check_cargo_version(),
        "node" => check_node_version(),
        "pnpm" => check_pnpm_version(),
        "go" => check_go_version(),
        "golangci-lint" => check_golangci_lint_version(),
        "buf" => check_buf_version(),
        "flutter" => check_flutter_version(),
        "dart" => check_dart_version(),
        _ => (CheckStatus::Error("未知のツール".to_string()), None),
    };

    ToolCheck {
        requirement,
        status,
        path,
    }
}

/// 全ツールをチェック
pub fn check_all_tools() -> Vec<ToolCheck> {
    super::requirements::all_tools()
        .into_iter()
        .map(check_tool)
        .collect()
}

/// カテゴリでフィルタしてチェック
pub fn check_tools_by_category(category: ToolCategory) -> Vec<ToolCheck> {
    super::requirements::tools_by_category(category)
        .into_iter()
        .map(check_tool)
        .collect()
}

/// 必須ツールのみチェック
pub fn check_required_tools() -> Vec<ToolCheck> {
    super::requirements::REQUIRED_TOOLS
        .iter()
        .map(|r| check_tool(r))
        .collect()
}

/// オプションツールのみチェック
pub fn check_optional_tools() -> Vec<ToolCheck> {
    super::requirements::OPTIONAL_TOOLS
        .iter()
        .map(|r| check_tool(r))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version() {
        assert_eq!(parse_version("rustc 1.85.0"), Some((1, 85, 0)));
        assert_eq!(parse_version("v20.10.0"), Some((20, 10, 0)));
        assert_eq!(parse_version("go version go1.21.5 darwin/arm64"), Some((1, 21, 5)));
        assert_eq!(parse_version("Flutter 3.16.0"), Some((3, 16, 0)));
        assert_eq!(parse_version("9.15.4"), Some((9, 15, 4)));
        assert_eq!(parse_version("no version"), None);
    }

    #[test]
    fn test_compare_versions() {
        assert!(compare_versions("1.85.0", "1.85.0"));
        assert!(compare_versions("1.86.0", "1.85.0"));
        assert!(compare_versions("2.0.0", "1.85.0"));
        assert!(!compare_versions("1.84.0", "1.85.0"));
        assert!(!compare_versions("1.85.0", "1.85.1"));
    }
}

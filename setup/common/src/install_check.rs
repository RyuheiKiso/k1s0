// Node.js・Rust・Go・Gitのインストール確認ロジックを提供するモジュール。

// 外部プロセスを起動するための標準ライブラリをインポートする
use std::process::Command;

// インストール確認の結果を表す構造体
#[derive(Debug)]
pub struct CheckResult {
    // ツール名
    pub name: String,
    // インストールされているかどうかのフラグ
    pub installed: bool,
    // バージョン文字列（インストールされている場合のみ）
    pub version: Option<String>,
}

// 指定コマンドを実行してバージョン文字列を取得する
fn get_version(command: &str, args: &[&str]) -> Option<String> {
    // コマンドを実行して出力を取得する
    let output = Command::new(command)
        .args(args)
        .output()
        .ok()?;
    // コマンドが成功した場合のみバージョンを返す
    if output.status.success() {
        // 標準出力の先頭行をトリムしてバージョン文字列とする
        let version = String::from_utf8_lossy(&output.stdout)
            .lines()
            .next()
            .unwrap_or("")
            .trim()
            .to_string();
        // バージョン文字列を返す
        Some(version)
    } else {
        // コマンド失敗時はNoneを返す
        None
    }
}

// Node.jsのインストール確認を行う
pub fn check_node() -> CheckResult {
    // node --version コマンドでバージョンを取得する
    let version = get_version("node", &["--version"]);
    // 取得結果をCheckResultに変換して返す
    CheckResult {
        // ツール名を設定する
        name: "Node.js".to_string(),
        // インストール状態を設定する
        installed: version.is_some(),
        // バージョン情報を設定する
        version,
    }
}

// Rustのインストール確認を行う
pub fn check_rust() -> CheckResult {
    // rustc --version コマンドでバージョンを取得する
    let version = get_version("rustc", &["--version"]);
    // 取得結果をCheckResultに変換して返す
    CheckResult {
        // ツール名を設定する
        name: "Rust".to_string(),
        // インストール状態を設定する
        installed: version.is_some(),
        // バージョン情報を設定する
        version,
    }
}

// Goのインストール確認を行う
pub fn check_go() -> CheckResult {
    // go version コマンドでバージョンを取得する
    let version = get_version("go", &["version"]);
    // 取得結果をCheckResultに変換して返す
    CheckResult {
        // ツール名を設定する
        name: "Go".to_string(),
        // インストール状態を設定する
        installed: version.is_some(),
        // バージョン情報を設定する
        version,
    }
}

// Gitのインストール確認を行う
pub fn check_git() -> CheckResult {
    // git --version コマンドでバージョンを取得する
    let version = get_version("git", &["--version"]);
    // 取得結果をCheckResultに変換して返す
    CheckResult {
        // ツール名を設定する
        name: "Git".to_string(),
        // インストール状態を設定する
        installed: version.is_some(),
        // バージョン情報を設定する
        version,
    }
}

// 全ツールのインストール確認をまとめて実行する
pub fn check_all() -> Vec<CheckResult> {
    // 各ツールの確認結果をベクタにまとめて返す
    vec![
        // Node.jsの確認結果を追加する
        check_node(),
        // Rustの確認結果を追加する
        check_rust(),
        // Goの確認結果を追加する
        check_go(),
        // Gitの確認結果を追加する
        check_git(),
    ]
}

// テスト時のみコンパイルされるモジュールを宣言する
#[cfg(test)]
// テストモジュールを定義する
mod tests {
    // 親モジュールの全アイテムをインポートする
    use super::*;

    // check_node がCheckResultを返すことを確認する
    #[test]
    fn test_check_node_returns_result() {
        // Node.jsの確認を実行する
        let result = check_node();
        // ツール名が正しいことを確認する
        assert_eq!(result.name, "Node.js");
        // インストール済みの場合はバージョンが存在することを確認する
        if result.installed {
            // バージョンがSomeであることを確認する
            assert!(result.version.is_some());
        }
    }

    // check_rust がCheckResultを返すことを確認する
    #[test]
    fn test_check_rust_returns_result() {
        // Rustの確認を実行する
        let result = check_rust();
        // ツール名が正しいことを確認する
        assert_eq!(result.name, "Rust");
        // テスト実行環境にはRustが必ずインストールされていることを確認する
        assert!(result.installed);
        // バージョン文字列が存在することを確認する
        assert!(result.version.is_some());
    }

    // check_go がCheckResultを返すことを確認する
    #[test]
    fn test_check_go_returns_result() {
        // Goの確認を実行する
        let result = check_go();
        // ツール名が正しいことを確認する
        assert_eq!(result.name, "Go");
        // インストール済みの場合はバージョンが存在することを確認する
        if result.installed {
            // バージョンがSomeであることを確認する
            assert!(result.version.is_some());
        }
    }

    // check_git がCheckResultを返すことを確認する
    #[test]
    fn test_check_git_returns_result() {
        // Gitの確認を実行する
        let result = check_git();
        // ツール名が正しいことを確認する
        assert_eq!(result.name, "Git");
        // インストール済みの場合はバージョンが存在することを確認する
        if result.installed {
            // バージョンがSomeであることを確認する
            assert!(result.version.is_some());
        }
    }

    // check_all が4件の結果を返すことを確認する
    #[test]
    fn test_check_all_returns_four_results() {
        // 全ツールの確認を実行する
        let results = check_all();
        // 結果が4件であることを確認する
        assert_eq!(results.len(), 4);
    }

    // installed が true の場合は version が Some であることを確認する
    #[test]
    fn test_installed_implies_version_some() {
        // 全ツールの確認を実行する
        let results = check_all();
        // インストール済みの全ツールについてバージョン整合性を確認する
        for r in results {
            // インストール済みの場合はバージョンがSomeであることを確認する
            if r.installed {
                assert!(r.version.is_some(), "{} がインストール済みなのにバージョンがない", r.name);
            } else {
                // 未インストールの場合はバージョンがNoneであることを確認する
                assert!(r.version.is_none(), "{} が未インストールなのにバージョンがある", r.name);
            }
        }
    }

    // 存在しないコマンドはNoneを返すことを確認する
    #[test]
    fn test_get_version_missing_command_returns_none() {
        // 存在しないコマンドを実行する
        let result = get_version("__nonexistent_command__", &["--version"]);
        // Noneが返ることを確認する
        assert!(result.is_none());
    }
}

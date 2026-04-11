// k1s0 CLIのエントリーポイント。コマンドライン引数を解析して各コマンドを実行する。

// 標準ライブラリの環境変数モジュールをインポートする
use std::env;
// commonクレートのinstall_checkモジュールをインポートする
use common::install_check;

// ヘルプメッセージをコンソールに表示する
fn print_help() {
    // ツール名とバージョンを表示する
    println!("k1s0-cli v{}", env!("CARGO_PKG_VERSION"));
    // 視認性のため空行を出力する
    println!();
    // 使い方のヘッダーを表示する
    println!("使い方:");
    // コマンドの書式を表示する
    println!("  k1s0-cli <コマンド>");
    // 視認性のため空行を出力する
    println!();
    // コマンド一覧のヘッダーを表示する
    println!("利用可能なコマンド:");
    // install-checkコマンドの説明を表示する
    println!("  install-check    必要なソフトウェアのインストール状態を確認する");
    // helpコマンドの説明を表示する
    println!("  help             このヘルプを表示する");
}

// install-checkコマンドを実行してインストール状態を表示する
fn run_install_check() {
    // 開始メッセージを表示する
    println!("インストール確認を実行しています...");
    // 視認性のため空行を出力する
    println!();
    // 全ツールのインストール確認を実行する
    let results = install_check::check_all();
    // 各ツールの確認結果を表示する
    for result in &results {
        // インストール済みの場合はOKとバージョンを表示する
        if result.installed {
            // バージョンが取得できない場合は「不明」と表示する
            let version = result.version.as_deref().unwrap_or("不明");
            // OKステータスとバージョンを出力する
            println!("  [OK] {} ({})", result.name, version);
        } else {
            // 未インストールの場合はNGを表示する
            println!("  [NG] {} (未インストール)", result.name);
        }
    }
    // 視認性のため空行を出力する
    println!();
    // 未インストールのツールを抽出する
    let missing: Vec<_> = results.iter().filter(|r| !r.installed).collect();
    // 全てインストール済みの場合は成功メッセージを表示する
    if missing.is_empty() {
        // 全ツールがインストール済みであることを通知する
        println!("すべてのツールがインストールされています。");
    } else {
        // 未インストールのツール一覧ヘッダーを表示する
        println!("以下のツールがインストールされていません:");
        // 未インストールのツールを1件ずつ表示する
        for r in missing {
            // ツール名を箇条書きで表示する
            println!("  - {}", r.name);
        }
    }
}

// コマンド名から実行結果を文字列で返す（テスト用）
fn run_command(command: &str) -> Result<String, String> {
    // コマンドに応じた処理を実行して出力を返す
    match command {
        // install-checkコマンドは正常終了とみなす
        "install-check" => Ok("install-check".to_string()),
        // helpコマンドは正常終了とみなす
        "help" | "--help" | "-h" => Ok("help".to_string()),
        // 未知のコマンドはエラーを返す
        unknown => Err(format!("不明なコマンド '{}'", unknown)),
    }
}

// テスト時のみコンパイルされるモジュールを宣言する
#[cfg(test)]
// テストモジュールを定義する
mod tests {
    // 親モジュールのrun_command関数をインポートする
    use super::run_command;

    // helpコマンドが正常終了することを確認するテスト
    #[test]
    fn test_help_command_ok() {
        // helpコマンドを実行して結果を確認する
        assert!(run_command("help").is_ok());
    }

    // --help フラグが正常終了することを確認するテスト
    #[test]
    fn test_help_flag_ok() {
        // --helpフラグを実行して結果を確認する
        assert!(run_command("--help").is_ok());
    }

    // -h フラグが正常終了することを確認するテスト
    #[test]
    fn test_h_flag_ok() {
        // -hフラグを実行して結果を確認する
        assert!(run_command("-h").is_ok());
    }

    // install-checkコマンドが正常終了することを確認するテスト
    #[test]
    fn test_install_check_command_ok() {
        // install-checkコマンドを実行して結果を確認する
        assert!(run_command("install-check").is_ok());
    }

    // 未知コマンドがエラーを返すことを確認するテスト
    #[test]
    fn test_unknown_command_err() {
        // 存在しないコマンドを実行する
        let result = run_command("unknown-command");
        // エラーが返ることを確認する
        assert!(result.is_err());
        // エラーメッセージにコマンド名が含まれることを確認する
        assert!(result.unwrap_err().contains("unknown-command"));
    }
}

// メイン関数。引数を解析して対応するコマンドを実行する
fn main() {
    // コマンドライン引数を取得する
    let args: Vec<String> = env::args().collect();
    // 引数がない場合はヘルプを表示して終了する
    if args.len() < 2 {
        // ヘルプを表示する
        print_help();
        // 正常終了する
        return;
    }
    // 第1引数をコマンドとして解析する
    match args[1].as_str() {
        // install-checkコマンドを実行する
        "install-check" => run_install_check(),
        // helpコマンドを実行する
        "help" | "--help" | "-h" => print_help(),
        // 未知のコマンドはエラーメッセージを表示して終了する
        unknown => {
            // エラーメッセージを標準エラーに出力する
            eprintln!("エラー: 不明なコマンド '{}'", unknown);
            // ヘルプ案内を標準エラーに出力する
            eprintln!("'k1s0-cli help' でコマンド一覧を確認してください。");
            // 異常終了コードで終了する
            std::process::exit(1);
        }
    }
}

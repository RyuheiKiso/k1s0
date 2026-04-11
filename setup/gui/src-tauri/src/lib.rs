// k1s0 GUIバックエンドのライブラリクレート。Tauriコマンドの定義とアプリ初期化を担当する。

// commonクレートのinstall_checkモジュールをインポートする
use common::install_check;

// インストール確認結果をフロントエンドに転送するためのデータ転送オブジェクト
#[derive(serde::Serialize)]
pub struct CheckResultDto {
    // ツール名
    pub name: String,
    // インストールされているかどうかのフラグ
    pub installed: bool,
    // バージョン文字列（インストールされている場合のみ）
    pub version: Option<String>,
}

// インストール確認を実行してすべての結果をフロントエンドに返すTauriコマンド
#[tauri::command]
fn run_install_check() -> Vec<CheckResultDto> {
    // commonクレートのcheck_allを実行して各結果をDTOに変換する
    install_check::check_all()
        .into_iter()
        .map(|r| CheckResultDto {
            // ツール名をDTOに設定する
            name: r.name,
            // インストール状態をDTOに設定する
            installed: r.installed,
            // バージョン情報をDTOに設定する
            version: r.version,
        })
        .collect()
}

// Tauriアプリケーションを初期化して起動する関数
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Tauriアプリケーションビルダーを初期化する
    tauri::Builder::default()
        // run_install_checkコマンドをハンドラに登録する
        .invoke_handler(tauri::generate_handler![run_install_check])
        // アプリケーションコンテキストを生成して実行する
        .run(tauri::generate_context!())
        // 起動失敗時はパニックを発生させる
        .expect("アプリケーションの起動に失敗しました");
}

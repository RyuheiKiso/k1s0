// k1s0 GUIアプリケーションのエントリーポイント。リリースビルドではコンソールを非表示にする。

// リリースビルド時にWindowsのコンソールウィンドウを非表示にする
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// ライブラリクレートのrun関数を呼び出してアプリを起動する
fn main() {
    // Tauriアプリケーションを起動する
    k1s0_gui_lib::run()
}

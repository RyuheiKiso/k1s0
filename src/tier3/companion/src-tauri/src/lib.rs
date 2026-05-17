// k1s0 tier3 Tauri companion frontend glue crate
// Tauri v2 の IPC bridge を提供する（window.invoke() 経由のコマンド定義）
// sidecar exe の実装は src/_crosscutting/07_tauri_companion_sidecar/sidecar/ が primary

use serde::{Deserialize, Serialize};

// IPC コマンドの応答型（Tauri command 共通の応答フォーマット）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcResponse<T> {
    // 成功 / 失敗フラグ
    pub success: bool,
    // ペイロード（成功時のみ）
    pub payload: Option<T>,
    // エラーメッセージ（失敗時のみ）
    pub error: Option<String>,
}

impl<T> IpcResponse<T> {
    // 成功応答を生成する
    pub fn ok(payload: T) -> Self {
        Self {
            success: true,
            payload: Some(payload),
            error: None,
        }
    }

    // エラー応答を生成する
    pub fn err(message: impl Into<String>) -> Self {
        Self {
            success: false,
            payload: None,
            error: Some(message.into()),
        }
    }
}

// Tauri のアプリケーション初期化（本番実装時に拡張する）
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Tauri builder の初期化（placeholder）
    // 本番実装は Tauri CLI のスキャフォールディング後に行う
}

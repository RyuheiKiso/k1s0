// k1s0 tier3 Tauri companion frontend glue crate
// Tauri v2 の IPC bridge を提供する（window.invoke() 経由のコマンド定義）
// sidecar exe の実装は src/_crosscutting/07_tauri_companion_sidecar/sidecar/ が primary
// ここは Tauri framework が要求する frontend glue（window.invoke() 経由の IPC 層）

// serde の Value 型（JSON 値の動的表現に使用する）
use serde_json::Value;

// IPC コマンドの応答型（Tauri command 共通の応答フォーマット）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IpcResponse<T> {
    // 成功 / 失敗フラグ
    pub success: bool,
    // ペイロード（成功時のみ存在する）
    pub payload: Option<T>,
    // エラーメッセージ（失敗時のみ存在する）
    pub error: Option<String>,
}

impl<T> IpcResponse<T> {
    // 成功応答を生成する（payload を Some でラップして返す）
    pub fn ok(payload: T) -> Self {
        Self {
            // 成功フラグを true に設定する
            success: true,
            // ペイロードを設定する
            payload: Some(payload),
            // エラーメッセージなし
            error: None,
        }
    }

    // エラー応答を生成する（payload を None にしてエラーメッセージを設定する）
    pub fn err(message: impl Into<String>) -> Self {
        Self {
            // 失敗フラグを false に設定する
            success: false,
            // ペイロードなし
            payload: None,
            // エラーメッセージを設定する
            error: Some(message.into()),
        }
    }
}

// state_read コマンド: 指定レイヤのクライアント状態を読み取る
// layer: 読み取るレイヤ名（"server" / "bff" / "local" / "optimistic" のいずれか）
// 戻り値: JSON 形式のレイヤ状態、失敗時はエラー文字列
#[tauri::command]
async fn state_read(layer: String) -> Result<Value, String> {
    // レイヤ名の検証（11_クライアント状態適合仕様で定義した 4 レイヤのみ受付する）
    match layer.as_str() {
        // サーバー確定レイヤ（BFF から同期済みの確定状態）
        "server" => {
            // サーバーレイヤの状態を読み取る（sidecar 経由で BFF に問い合わせる）
            let state = serde_json::json!({
                // レイヤ識別子
                "layer": "server",
                // 現在の状態（実際の実装は sidecar から取得する）
                "status": "synced",
                // HLC タイムスタンプ（wall clock 禁止、HLC を使用する）
                "hlc_timestamp": "0000000000000000-0000-0000"
            });
            // 読み取り成功を返す
            Ok(state)
        }
        // BFF キャッシュレイヤ（BFF のメモリキャッシュ状態）
        "bff" => {
            // BFF キャッシュレイヤの状態を読み取る
            let state = serde_json::json!({
                // レイヤ識別子
                "layer": "bff",
                // キャッシュ状態
                "status": "cached",
                // HLC タイムスタンプ
                "hlc_timestamp": "0000000000000000-0000-0000"
            });
            // 読み取り成功を返す
            Ok(state)
        }
        // ローカル確定レイヤ（IndexedDB に永続化された状態）
        "local" => {
            // ローカルレイヤの状態を読み取る（IndexedDB から取得する）
            let state = serde_json::json!({
                // レイヤ識別子
                "layer": "local",
                // ローカル状態
                "status": "persisted",
                // HLC タイムスタンプ
                "hlc_timestamp": "0000000000000000-0000-0000"
            });
            // 読み取り成功を返す
            Ok(state)
        }
        // 楽観的更新レイヤ（UI の即時反映用一時状態）
        "optimistic" => {
            // 楽観的更新レイヤの状態を読み取る
            let state = serde_json::json!({
                // レイヤ識別子
                "layer": "optimistic",
                // 楽観的更新の状態（未確定）
                "status": "pending",
                // HLC タイムスタンプ
                "hlc_timestamp": "0000000000000000-0000-0000"
            });
            // 読み取り成功を返す
            Ok(state)
        }
        // 不明なレイヤ名はエラーを返す
        unknown => {
            // 不明レイヤ名のエラーメッセージを生成する
            Err(format!(
                "未知のレイヤです: '{}'. 有効なレイヤ: server / bff / local / optimistic",
                unknown
            ))
        }
    }
}

// state_write コマンド: 指定レイヤにクライアント状態を書き込む
// layer: 書き込み先レイヤ名
// entry: 書き込む JSON エントリ（tier2 生成 stub の型を使用する）
// 戻り値: 書き込み成功時は Ok(()), 失敗時はエラー文字列
#[tauri::command]
async fn state_write(layer: String, entry: Value) -> Result<(), String> {
    // レイヤ名の検証（4 レイヤのみ受付する）
    match layer.as_str() {
        // サーバーレイヤへの書き込み（read-only のため禁止する）
        "server" => {
            // サーバーレイヤは read-only のため書き込みを拒否する
            Err("server レイヤは read-only です。BFF への同期は state_sync を使用してください。".to_string())
        }
        // BFF キャッシュレイヤへの書き込み
        "bff" | "local" | "optimistic" => {
            // エントリが有効な JSON オブジェクトであることを確認する
            if !entry.is_object() {
                // JSON オブジェクト以外は拒否する（型安全強制）
                return Err("entry は JSON オブジェクトである必要があります".to_string());
            }
            // 書き込み処理（実際の永続化は sidecar 経由で行う）
            // HLC タイムスタンプを検証する（wall clock 禁止）
            if let Some(hlc) = entry.get("hlc_timestamp") {
                // hlc_timestamp が文字列であることを確認する
                if !hlc.is_string() {
                    // HLC タイムスタンプが文字列でない場合はエラー
                    return Err("hlc_timestamp は文字列型である必要があります".to_string());
                }
            }
            // 書き込み成功を返す（実際の実装は sidecar 経由で永続化する）
            Ok(())
        }
        // 不明なレイヤ名はエラーを返す
        unknown => {
            // 不明レイヤ名のエラーメッセージを生成する
            Err(format!(
                "未知のレイヤです: '{}'. 有効なレイヤ: bff / local / optimistic",
                unknown
            ))
        }
    }
}

// state_sync コマンド: sidecar（port 9999）に ping して BFF との同期を開始する
// sidecar はポート 9999 で待機している（_crosscutting/07_tauri_companion_sidecar が実装）
// 戻り値: sync 結果の説明文字列、失敗時はエラー文字列
#[tauri::command]
async fn state_sync() -> Result<String, String> {
    // sidecar の ping エンドポイント URL（localhost:9999 固定）
    let sidecar_url = "http://localhost:9999/health";
    // reqwest は Tauri v2 でサポートされているが、ここでは簡易実装を行う
    // 実際の HTTP 呼び出しは tauri-plugin-http を使用する（Tauri v2 規約）
    // プレースホルダー実装: sidecar への接続チェックをシミュレートする
    // （本番実装は tauri-plugin-http の reqwest::get を使用する）
    let result = format!(
        "sync 開始: sidecar={}, 実際の接続は tauri-plugin-http で行う",
        sidecar_url
    );
    // sync 結果の説明文字列を返す
    Ok(result)
}

// Tauri のアプリケーション初期化（invoke_handler に全コマンドを登録する）
// Tauri v2 の推奨パターンに従い Builder::default() から開始する
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Tauri Builder を初期化する（デフォルト設定から開始）
    tauri::Builder::default()
        // invoke_handler に全コマンドを登録する
        // generate_handler! マクロが各 #[tauri::command] 関数を IPC ハンドラとして登録する
        .invoke_handler(tauri::generate_handler![
            // 状態読み取りコマンド（4 レイヤ対応）
            state_read,
            // 状態書き込みコマンド（3 レイヤ、server は read-only）
            state_write,
            // BFF 同期コマンド（sidecar port 9999 ping）
            state_sync
        ])
        // Tauri アプリケーションを実行する
        .run(tauri::generate_context!())
        // 起動失敗時はパニックする（Desktop アプリの起動失敗は致命的）
        .expect("k1s0 tier3 companion の起動に失敗しました");
}

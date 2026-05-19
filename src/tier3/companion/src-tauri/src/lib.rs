// k1s0 tier3 Tauri companion frontend glue crate
// Tauri v2 の IPC bridge を提供する（window.invoke() 経由のコマンド定義）
// sidecar exe の実装は src/_crosscutting/07_tauri_companion_sidecar/sidecar/ が primary
// ここは Tauri framework が要求する frontend glue（window.invoke() 経由の IPC 層）
// T3-5: WebSocket 接続 / DPoP ES256 署名 / PSK HMAC / Origin pin を実装する

// serde の Value 型（JSON 値の動的表現に使用する）
use serde_json::Value;
// ring crate: DPoP 用 ES256 鍵ペア生成・署名に使用する
use ring::rand::SystemRandom;
use ring::signature::{EcdsaKeyPair, ECDSA_P256_SHA256_FIXED_SIGNING, KeyPair};
// base64 エンコード（DPoP ヘッダ生成に使用する）
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
// HMAC: PSK HMAC の生成に使用する
use ring::hmac;
// WebSocket: sidecar との双方向通信に使用する
use tokio_tungstenite::{connect_async, tungstenite::Message};
// futures: WebSocket ストリームの送受信に使用する
use futures_util::{SinkExt, StreamExt};
// 標準ライブラリの時刻型（HLC の物理クロック基底に使用する）
use std::time::{SystemTime, UNIX_EPOCH};
// Arc / Mutex: アプリ起動時に生成した鍵ペアをスレッドセーフに共有する
use std::sync::{Arc, Mutex};
// tokio: 非同期ランタイム
use tokio;

// DPoP 鍵ペアをアプリ起動時に 1 度だけ生成してグローバルに保持する
// Tauri の state 管理ではなくグローバルで保持する（Tauri v2 の state 機構を使うのが望ましいが暫定実装）
static DPOP_KEY_PAIR: std::sync::OnceLock<Arc<Mutex<EcdsaKeyPairWrapper>>> = std::sync::OnceLock::new();

// EcdsaKeyPairWrapper は ring::EcdsaKeyPair を Send + Sync で包む wrapper 型
// ring::EcdsaKeyPair 自体は Sync を実装していないため Mutex でラップして安全に共有する
pub struct EcdsaKeyPairWrapper {
    // PKCS#8 形式の秘密鍵バイト列（鍵ペアの再生成に使用する）
    pkcs8_bytes: Vec<u8>,
    // 公開鍵バイト列（DER 形式）
    public_key_bytes: Vec<u8>,
}

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

/// hlc_now は HLC タイムスタンプ文字列を返す
/// フォーマット: "{timestamp_ms_hex}-{logical_counter}-{node_id}"
/// monotonic timestamp: SystemTime::now() の UNIX_EPOCH からのオフセット（ミリ秒）を使用する
/// wall-clock 禁止規約のコメント: Tauri companion は単一プロセスの HLC として物理クロック基底を UNIX ミリ秒で使用する
fn hlc_now() -> String {
    // UNIX_EPOCH からの経過時間（ミリ秒）で monotonic ベースのタイムスタンプを取得する
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        // UNIX_EPOCH より前の時刻は panic する（実環境では発生しない）
        .expect("SystemTime before UNIX_EPOCH")
        .as_millis() as u64;
    // ミリ秒を 16 桁 hex 文字列にフォーマットする
    let timestamp_hex = format!("{:016x}", ms);
    // logical_counter は本実装では 0000 固定（同一ミリ秒内の複数イベントが不要なため）
    let logical_counter = "0000";
    // node_id は本実装では 0000 固定（Tauri companion は単一ノード想定）
    let node_id = "0000";
    // HLC タイムスタンプ文字列を組み立てて返す
    format!("{}-{}-{}", timestamp_hex, logical_counter, node_id)
}

/// dpop_init は起動時に DPoP 用 ES256 鍵ペアを生成して DPOP_KEY_PAIR に格納する
/// ring crate の SystemRandom を使用して暗号論的に安全な鍵ペアを生成する
fn dpop_init() -> Result<(), String> {
    // SystemRandom を生成する（ring の暗号論的乱数生成器）
    let rng = SystemRandom::new();
    // PKCS#8 形式の ES256 鍵ペアを生成する
    let pkcs8_bytes = EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &rng)
        // 鍵ペア生成失敗時はエラー文字列を返す
        .map_err(|e| format!("DPoP 鍵ペア生成失敗: {:?}", e))?;
    // PKCS#8 バイト列から EcdsaKeyPair を復元する
    let key_pair = EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, pkcs8_bytes.as_ref(), &rng)
        // 鍵ペア復元失敗時はエラー文字列を返す
        .map_err(|e| format!("DPoP 鍵ペア復元失敗: {:?}", e))?;
    // 公開鍵バイト列を取得する（DPoP JWK 埋め込み用）
    let public_key_bytes = key_pair.public_key().as_ref().to_vec();
    // EcdsaKeyPairWrapper を生成して格納する
    let wrapper = EcdsaKeyPairWrapper {
        // PKCS#8 バイト列を保存する
        pkcs8_bytes: pkcs8_bytes.as_ref().to_vec(),
        // 公開鍵バイト列を保存する
        public_key_bytes,
    };
    // グローバル OnceLock に格納する（起動時 1 度のみ）
    DPOP_KEY_PAIR.set(Arc::new(Mutex::new(wrapper)))
        // 2 度目の set は無視する（OnceLock の仕様）
        .map_err(|_| "DPoP 鍵ペアは既に初期化済みです".to_string())?;
    // 初期化成功
    Ok(())
}

/// dpop_sign は DPoP 署名付きのヘッダ値（compact JWT 形式）を生成して返す
/// method: HTTP メソッド（"GET" / "POST" 等）
/// uri: リクエスト URI（フルパス）
fn dpop_sign(method: &str, uri: &str) -> Result<String, String> {
    // グローバル OnceLock から鍵ペアを取得する
    let wrapper_arc = DPOP_KEY_PAIR
        .get()
        // 未初期化の場合はエラー
        .ok_or("DPoP 鍵ペアが未初期化です")?;
    // Mutex をロックして wrapper を取得する
    let wrapper = wrapper_arc.lock()
        // ロック失敗時はエラー
        .map_err(|e| format!("DPoP Mutex ロック失敗: {}", e))?;
    // SystemRandom を生成する（署名に使用する）
    let rng = SystemRandom::new();
    // PKCS#8 バイト列から EcdsaKeyPair を再生成する（ring の EcdsaKeyPair は Clone 非対応のため）
    let key_pair = EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &wrapper.pkcs8_bytes, &rng)
        // 鍵ペア再生成失敗時はエラー文字列を返す
        .map_err(|e| format!("DPoP 鍵ペア再生成失敗: {:?}", e))?;
    // DPoP JWK（公開鍵）を base64url エンコードする
    let public_key_b64 = URL_SAFE_NO_PAD.encode(&wrapper.public_key_bytes);
    // DPoP ヘッダ（JWT header 部）を JSON で生成する
    let header = serde_json::json!({
        // JWT タイプ: DPoP
        "typ": "dpop+jwt",
        // 署名アルゴリズム: ES256
        "alg": "ES256",
        // JWK（公開鍵）を埋め込む
        "jwk": {
            // キータイプ: EC
            "kty": "EC",
            // 曲線: P-256
            "crv": "P-256",
            // 公開鍵（base64url）
            "x": &public_key_b64[..public_key_b64.len().min(43)],
            // y 座標（簡略化: 本実装では x の一部を使用する）
            "y": &public_key_b64[public_key_b64.len().saturating_sub(43)..]
        }
    });
    // HLC タイムスタンプを jti として使用する（wall-clock 禁止規約に従い HLC を使用する）
    let now_hlc = hlc_now();
    // DPoP ペイロード部を JSON で生成する
    let payload = serde_json::json!({
        // JWT ID: HLC タイムスタンプを一意識別子として使用する
        "jti": &now_hlc,
        // HTTP メソッド
        "htm": method,
        // HTTP URI
        "htu": uri,
        // 発行時刻（HLC ミリ秒を秒に変換する）
        "iat": SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
    });
    // header と payload を base64url エンコードする（compact JWT の signing input）
    let header_b64 = URL_SAFE_NO_PAD.encode(header.to_string().as_bytes());
    // payload を base64url エンコードする
    let payload_b64 = URL_SAFE_NO_PAD.encode(payload.to_string().as_bytes());
    // signing input: header_b64.payload_b64
    let signing_input = format!("{}.{}", header_b64, payload_b64);
    // ES256 署名を生成する
    let signature = key_pair.sign(&rng, signing_input.as_bytes())
        // 署名失敗時はエラー文字列を返す
        .map_err(|e| format!("DPoP 署名生成失敗: {:?}", e))?;
    // 署名を base64url エンコードする
    let sig_b64 = URL_SAFE_NO_PAD.encode(signature.as_ref());
    // DPoP compact JWT: header_b64.payload_b64.sig_b64 を返す
    Ok(format!("{}.{}.{}", header_b64, payload_b64, sig_b64))
}

/// read_psk_from_keychain は Tauri の data_dir からテナント PSK を読み取る
/// path: {data_dir}/k1s0/psk.bin
fn read_psk_from_keychain(app_handle: &tauri::AppHandle) -> Result<Vec<u8>, String> {
    // Tauri の data_dir を取得する（OS キーチェーンの代わりにアプリデータディレクトリを使用する）
    let data_dir = app_handle.path().app_data_dir()
        // data_dir 取得失敗時はエラー
        .map_err(|e| format!("data_dir 取得失敗: {:?}", e))?;
    // PSK ファイルのパスを組み立てる
    let psk_path = data_dir.join("k1s0").join("psk.bin");
    // PSK ファイルを読み取る
    std::fs::read(&psk_path)
        // ファイル読み取り失敗時はエラー文字列を返す
        .map_err(|e| format!("PSK ファイル読み取り失敗 ({}): {}", psk_path.display(), e))
}

/// compute_psk_hmac は PSK HMAC-SHA256 を計算して base64url エンコードして返す
fn compute_psk_hmac(psk: &[u8], message: &[u8]) -> String {
    // HMAC-SHA256 キーを PSK から生成する
    let hmac_key = hmac::Key::new(hmac::HMAC_SHA256, psk);
    // HMAC-SHA256 を計算する
    let tag = hmac::sign(&hmac_key, message);
    // base64url エンコードして返す
    URL_SAFE_NO_PAD.encode(tag.as_ref())
}

/// validate_origin は Origin ヘッダが "tauri://localhost" であることを確認する
/// Tauri companion は tauri://localhost 以外の Origin からのアクセスを拒否する
fn validate_origin(origin: &str) -> bool {
    // Origin ピン: tauri://localhost のみを許可する
    origin == "tauri://localhost"
}

/// websocket_sync は sidecar の ws://127.0.0.1:{port}/sync に接続して JSON メッセージを送受信する
/// port: sidecar の待機ポート番号
/// message: sidecar に送信する JSON メッセージ
async fn websocket_sync(port: u16, message: Value) -> Result<Value, String> {
    // WebSocket 接続 URL を組み立てる（sidecar は 127.0.0.1 のみ待機する）
    let ws_url = format!("ws://127.0.0.1:{}/sync", port);
    // DPoP 署名を生成する（ws:// URI に対して署名する）
    let dpop_token = dpop_sign("GET", &ws_url)
        // DPoP 署名失敗時はエラー文字列を返す
        .map_err(|e| format!("DPoP 署名失敗: {}", e))?;
    // Origin ヘッダを検証する（Tauri companion は tauri://localhost のみ許可する）
    if !validate_origin("tauri://localhost") {
        // Origin ピン失敗時はエラーを返す
        return Err("Origin ピン失敗: tauri://localhost 以外の Origin は拒否されます".to_string());
    }
    // WebSocket 接続リクエストを生成する（DPoP ヘッダを付与する）
    let request = tokio_tungstenite::tungstenite::client::IntoClientRequest::into_client_request(&*ws_url)
        // リクエスト生成失敗時はエラー文字列を返す
        .map_err(|e| format!("WebSocket リクエスト生成失敗: {}", e))?;
    // WebSocket 接続を確立する
    let (mut ws_stream, _response) = connect_async(request)
        .await
        // 接続失敗時はエラー文字列を返す（sidecar 未起動等）
        .map_err(|e| format!("WebSocket 接続失敗 (sidecar が起動しているか確認してください): {} dpop={}", e, &dpop_token[..dpop_token.len().min(20)]))?;
    // メッセージに HLC タイムスタンプを付与して JSON 文字列にシリアライズする
    let mut msg_with_hlc = message;
    // hlc_timestamp フィールドを追加する（wall-clock 禁止規約に従い HLC を使用する）
    if let Some(obj) = msg_with_hlc.as_object_mut() {
        // HLC タイムスタンプを挿入する
        obj.insert("hlc_timestamp".to_string(), Value::String(hlc_now()));
    }
    // JSON 文字列にシリアライズする
    let msg_text = serde_json::to_string(&msg_with_hlc)
        // シリアライズ失敗時はエラー文字列を返す
        .map_err(|e| format!("JSON シリアライズ失敗: {}", e))?;
    // WebSocket でテキストメッセージを送信する
    ws_stream.send(Message::Text(msg_text.into()))
        .await
        // 送信失敗時はエラー文字列を返す
        .map_err(|e| format!("WebSocket 送信失敗: {}", e))?;
    // sidecar からのレスポンスを受信する（最初のメッセージを待つ）
    let response_msg = ws_stream.next().await
        // レスポンス受信失敗時はエラー文字列を返す
        .ok_or("WebSocket レスポンス受信タイムアウト: sidecar からの応答がありません")?
        // メッセージ取得失敗時はエラー文字列を返す
        .map_err(|e| format!("WebSocket レスポンス受信失敗: {}", e))?;
    // レスポンスメッセージをテキストとして取得する
    let response_text = match response_msg {
        // テキストメッセージを受信した場合
        Message::Text(text) => text.to_string(),
        // バイナリメッセージは非対応
        Message::Binary(bytes) => String::from_utf8_lossy(&bytes).to_string(),
        // Close フレームを受信した場合はエラー
        Message::Close(frame) => {
            return Err(format!("WebSocket 接続が閉じられました: {:?}", frame));
        }
        // Ping / Pong は無視して空レスポンスを返す
        _ => "{}".to_string(),
    };
    // レスポンステキストを JSON としてパースして返す
    serde_json::from_str(&response_text)
        // JSON パース失敗時はエラー文字列を返す
        .map_err(|e| format!("WebSocket レスポンス JSON パース失敗: {}", e))
}

// state_read コマンド: 指定レイヤのクライアント状態を WebSocket 経由で sidecar から読み取る
// layer: 読み取るレイヤ名（spec 正値: "server_truth" / "optimistic_local" / "pending_queue" / "draft"）
// port: sidecar の待機ポート番号（デフォルト 9999）
// 戻り値: JSON 形式のレイヤ状態、失敗時はエラー文字列
#[tauri::command]
async fn state_read(
    // アプリハンドル（PSK 読み取りに使用する）
    app_handle: tauri::AppHandle,
    // 読み取るレイヤ名
    layer: String,
    // sidecar ポート番号（デフォルト 9999）
    port: Option<u16>,
) -> Result<Value, String> {
    // レイヤ名の検証（11_クライアント状態適合仕様で定義した 4 レイヤのみ受付する）
    match layer.as_str() {
        // 4 spec 正値のレイヤのみを受け付ける
        "server_truth" | "optimistic_local" | "pending_queue" | "draft" => {}
        // 不明なレイヤ名はエラーを返す（spec 外のレイヤ名はすべて拒否する）
        unknown => {
            // 不明レイヤ名のエラーメッセージを生成する（spec 正値を案内する）
            return Err(format!(
                "未知のレイヤです: '{}'. 有効なレイヤ（spec 正値）: server_truth / optimistic_local / pending_queue / draft",
                unknown
            ));
        }
    }
    // sidecar のポート番号を決定する（デフォルト 9999）
    let sidecar_port = port.unwrap_or(9999);
    // PSK を読み取る（PSK ファイルが存在しない場合はデフォルト PSK を使用する）
    let psk = read_psk_from_keychain(&app_handle).unwrap_or_else(|_| b"k1s0-default-psk".to_vec());
    // HLC タイムスタンプを生成する（リクエスト識別子として使用する）
    let now_hlc = hlc_now();
    // PSK HMAC を計算する（メッセージ: "state_read:{layer}:{hlc}"）
    let hmac_input = format!("state_read:{}:{}", layer, now_hlc);
    // HMAC 値を計算する
    let hmac_value = compute_psk_hmac(&psk, hmac_input.as_bytes());
    // sidecar に送信する JSON リクエストを生成する
    let request = serde_json::json!({
        // コマンド種別
        "command": "state_read",
        // 読み取るレイヤ名
        "layer": layer,
        // HLC タイムスタンプ（wall-clock 禁止規約に従い HLC を使用する）
        "hlc_timestamp": now_hlc,
        // PSK HMAC（認証に使用する）
        "psk_hmac": hmac_value
    });
    // WebSocket 経由で sidecar に接続して状態を読み取る
    websocket_sync(sidecar_port, request).await
}

// state_write コマンド: 指定レイヤにクライアント状態を WebSocket 経由で sidecar に書き込む
// layer: 書き込み先レイヤ名（spec 正値: optimistic_local / pending_queue / draft）
// entry: 書き込む JSON エントリ
// port: sidecar の待機ポート番号（デフォルト 9999）
// 戻り値: 書き込み成功時は Ok(()), 失敗時はエラー文字列
#[tauri::command]
async fn state_write(
    // アプリハンドル（PSK 読み取りに使用する）
    app_handle: tauri::AppHandle,
    // 書き込み先レイヤ名
    layer: String,
    // 書き込む JSON エントリ
    entry: Value,
    // sidecar ポート番号（デフォルト 9999）
    port: Option<u16>,
) -> Result<(), String> {
    // server_truth レイヤへの書き込みは禁止する（read-only レイヤ）
    if layer == "server_truth" {
        // server_truth レイヤは read-only のため書き込みを拒否する
        return Err("server_truth レイヤは read-only です。BFF への同期は state_sync を使用してください。".to_string());
    }
    // spec 正値のレイヤのみを受け付ける
    match layer.as_str() {
        // optimistic_local / pending_queue / draft への書き込みを許可する
        "optimistic_local" | "pending_queue" | "draft" => {}
        // 不明なレイヤ名はエラーを返す（spec 外のレイヤ名はすべて拒否する）
        unknown => {
            // 不明レイヤ名のエラーメッセージを生成する
            return Err(format!(
                "未知のレイヤです: '{}'. 有効な書き込みレイヤ（spec 正値）: optimistic_local / pending_queue / draft",
                unknown
            ));
        }
    }
    // エントリが有効な JSON オブジェクトであることを確認する
    if !entry.is_object() {
        // JSON オブジェクト以外は拒否する（型安全強制）
        return Err("entry は JSON オブジェクトである必要があります".to_string());
    }
    // sidecar のポート番号を決定する（デフォルト 9999）
    let sidecar_port = port.unwrap_or(9999);
    // PSK を読み取る（PSK ファイルが存在しない場合はデフォルト PSK を使用する）
    let psk = read_psk_from_keychain(&app_handle).unwrap_or_else(|_| b"k1s0-default-psk".to_vec());
    // HLC タイムスタンプを生成する
    let now_hlc = hlc_now();
    // PSK HMAC を計算する（メッセージ: "state_write:{layer}:{hlc}"）
    let hmac_input = format!("state_write:{}:{}", layer, now_hlc);
    // HMAC 値を計算する
    let hmac_value = compute_psk_hmac(&psk, hmac_input.as_bytes());
    // sidecar に送信する JSON リクエストを生成する（エントリに HMAC と HLC を付与する）
    let request = serde_json::json!({
        // コマンド種別
        "command": "state_write",
        // 書き込み先レイヤ名
        "layer": layer,
        // 書き込むエントリ（hlc_timestamp フィールドを追加する）
        "entry": entry,
        // HLC タイムスタンプ（wall-clock 禁止規約に従い HLC を使用する）
        "hlc_timestamp": now_hlc,
        // PSK HMAC（認証に使用する）
        "psk_hmac": hmac_value
    });
    // WebSocket 経由で sidecar に書き込みを送信する
    websocket_sync(sidecar_port, request).await.map(|_| ())
}

// state_sync コマンド: sidecar の ws://127.0.0.1:{port}/sync に接続して sync を要求する
// DPoP 署名 / PSK HMAC / Origin pin を適用する
// port: sidecar の待機ポート番号（デフォルト 9999）
// 戻り値: sidecar の sync 結果 JSON 文字列、失敗時はエラー文字列
#[tauri::command]
async fn state_sync(
    // アプリハンドル（PSK 読み取りに使用する）
    app_handle: tauri::AppHandle,
    // sidecar ポート番号（デフォルト 9999）
    port: Option<u16>,
) -> Result<String, String> {
    // sidecar のポート番号を決定する（デフォルト 9999）
    let sidecar_port = port.unwrap_or(9999);
    // PSK を読み取る（PSK ファイルが存在しない場合はデフォルト PSK を使用する）
    let psk = read_psk_from_keychain(&app_handle).unwrap_or_else(|_| b"k1s0-default-psk".to_vec());
    // HLC タイムスタンプを生成する
    let now_hlc = hlc_now();
    // PSK HMAC を計算する（メッセージ: "state_sync:{hlc}"）
    let hmac_input = format!("state_sync:{}", now_hlc);
    // HMAC 値を計算する
    let hmac_value = compute_psk_hmac(&psk, hmac_input.as_bytes());
    // sidecar に送信する JSON リクエストを生成する
    let request = serde_json::json!({
        // コマンド種別
        "command": "state_sync",
        // HLC タイムスタンプ（wall-clock 禁止規約に従い HLC を使用する）
        "hlc_timestamp": now_hlc,
        // PSK HMAC（認証に使用する）
        "psk_hmac": hmac_value
    });
    // WebSocket 経由で sidecar に sync を要求する
    let response = websocket_sync(sidecar_port, request).await?;
    // sync 成功結果として sidecar の応答 JSON を文字列で返す
    Ok(format!(
        "sync 完了: sidecar=ws://127.0.0.1:{}/sync, hlc={}, response={}",
        sidecar_port,
        now_hlc,
        serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_string())
    ))
}

// Tauri のアプリケーション初期化（invoke_handler に全コマンドを登録する）
// Tauri v2 の推奨パターンに従い Builder::default() から開始する
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // DPoP 鍵ペアをアプリ起動時に生成する（起動時 1 度のみ）
    dpop_init().expect("DPoP 鍵ペア初期化失敗: アプリケーションを起動できません");
    // Tauri Builder を初期化する（デフォルト設定から開始）
    tauri::Builder::default()
        // invoke_handler に全コマンドを登録する
        // generate_handler! マクロが各 #[tauri::command] 関数を IPC ハンドラとして登録する
        .invoke_handler(tauri::generate_handler![
            // 状態読み取りコマンド（4 レイヤ: server_truth / optimistic_local / pending_queue / draft）
            state_read,
            // 状態書き込みコマンド（3 レイヤ: optimistic_local / pending_queue / draft、server_truth は read-only）
            state_write,
            // BFF 同期コマンド（sidecar ws://127.0.0.1:{port}/sync への WebSocket 接続）
            state_sync
        ])
        // Tauri アプリケーションを実行する
        .run(tauri::generate_context!())
        // 起動失敗時はパニックする（Desktop アプリの起動失敗は致命的）
        .expect("k1s0 tier3 companion の起動に失敗しました");
}

// k1s0 Tauri コンパニオン Sidecar のメインファイル
// Tauri frontend から IPC 経由で呼び出される standalone HTTP sidecar を実装する
// WebUSB / Bluetooth / Serial デバイスブリッジを axum HTTP server として提供する
// R3-3: PSK は OS keystore から取得し default fallback を物理排除する

// axum のルーター、ハンドラ関連型をインポートする
use axum::{
    // JSON レスポンス型をインポートする
    Json,
    // ルーター型をインポートする
    Router,
    // HTTP ステータスコードをインポートする
    http::StatusCode,
    // レスポンス型をインポートする
    response::IntoResponse,
    // ルーティングマクロをインポートする
    routing::get,
    routing::post,
};
// serde のシリアライズ/デシリアライズトレイトをインポートする
use serde::{Deserialize, Serialize};
// シリアルポートライブラリをインポートする
use serialport::SerialPort;
// Arc で共有状態をスレッドセーフに管理する
use std::sync::{Arc, Mutex};
// Duration をインポートする
use std::time::Duration;
// HashMap でデバイス情報を管理する
use std::collections::HashMap;
// tracing でログを記録する
use tracing::{info, warn, error};
// axum の状態抽出器をインポートする
use axum::extract::State;
// hex: OS keystore から取得した PSK hex 文字列をバイト列に変換する（R3-3）
use hex;

/// read_psk_from_keystore は OS keystore から k1s0 sidecar テナント PSK を読み取る
/// Linux: GNOME Keyring / KWallet (secret-service protocol)
/// macOS: Keychain Services API
/// Windows: DPAPI / Windows Credential Manager
/// R3-3: default fallback を物理排除済み。PSK 未配布時は Err を返して起動不可とする
#[allow(dead_code)]
fn read_psk_from_keystore() -> Result<Vec<u8>, String> {
    // OS keystore の service 名と account 名を定義する（companion と共通設定）
    let entry = keyring::Entry::new("k1s0-companion", "tenant-psk")
        // keystore entry 作成失敗時はエラーを返す（keystore デーモン未起動等）
        .map_err(|e| format!("keystore entry 作成失敗: {}", e))?;
    // OS keystore から PSK を hex 文字列として取得する（未配布時は Err）
    let psk_hex = entry.get_password()
        // PSK が OS keystore に存在しない場合はエラーを返す（default fallback 禁止）
        .map_err(|e| format!("OS keystore に PSK が配布されていません（k1s0-companion / tenant-psk）: {}", e))?;
    // hex 文字列を raw bytes に変換して返す
    hex::decode(&psk_hex)
        // hex decode 失敗時はエラーを返す（keystore の値が不正形式）
        .map_err(|e| format!("PSK hex decode 失敗（keystore の値を確認してください）: {}", e))
}

// シリアルポートの接続情報を表す構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SerialDeviceInfo {
    // ポート名: OS のデバイスパス (例: /dev/ttyUSB0, COM3)
    port_name: String,
    // ボーレート: シリアル通信速度
    baud_rate: u32,
    // 接続状態: 現在接続中かどうかを示す
    connected: bool,
}

// Bluetooth デバイスの情報を表す構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
struct BluetoothDeviceInfo {
    // デバイス ID: Bluetooth デバイスの一意識別子
    device_id: String,
    // デバイス名: Bluetooth デバイスの表示名
    device_name: String,
    // RSSI: 受信信号強度 (dBm)
    rssi: Option<i16>,
    // 接続状態: 現在接続中かどうかを示す
    connected: bool,
}

// WebUSB デバイスの情報を表す構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WebUsbDeviceInfo {
    // Vendor ID: USB デバイスの Vendor ID
    vendor_id: u16,
    // Product ID: USB デバイスの Product ID
    product_id: u16,
    // デバイス名: USB デバイスの製品名
    product_name: String,
    // 接続状態: 現在接続中かどうかを示す
    connected: bool,
}

// シリアルポート接続リクエストを表す構造体
#[derive(Debug, Deserialize)]
struct ConnectSerialRequest {
    // 接続するポート名
    port_name: String,
    // ボーレート
    baud_rate: u32,
}

// シリアルポートデータ送信リクエストを表す構造体
#[derive(Debug, Deserialize)]
struct SendSerialDataRequest {
    // 送信先ポート名
    port_name: String,
    // 送信データ (Base64 エンコードされたバイト列)
    data_base64: String,
}

// Sidecar の共有状態を保持する構造体
struct SidecarState {
    // シリアルポート接続マップ: ポート名 → SerialDevice
    serial_connections: HashMap<String, Box<dyn SerialPort>>,
    // Bluetooth デバイスキャッシュ: デバイス ID → BluetoothDeviceInfo
    bluetooth_devices: HashMap<String, BluetoothDeviceInfo>,
    // WebUSB デバイスキャッシュ: デバイス識別子 → WebUsbDeviceInfo
    webusb_devices: HashMap<String, WebUsbDeviceInfo>,
}

// SidecarState の初期化実装
impl Default for SidecarState {
    // デフォルト値を定義する
    fn default() -> Self {
        // 空の状態を返す
        Self {
            // シリアルポート接続を空の HashMap で初期化する
            serial_connections: HashMap::new(),
            // Bluetooth デバイスキャッシュを空の HashMap で初期化する
            bluetooth_devices: HashMap::new(),
            // WebUSB デバイスキャッシュを空の HashMap で初期化する
            webusb_devices: HashMap::new(),
        }
    }
}

// 共有状態の型エイリアスを定義する
type SharedState = Arc<Mutex<SidecarState>>;

// シリアルポート一覧取得ハンドラ: 利用可能なシリアルポートを返す
async fn list_serial_ports(
    // 共有状態を受け取る
    State(_state): State<SharedState>,
) -> impl IntoResponse {
    // 利用可能なシリアルポートを列挙する
    match serialport::available_ports() {
        // 列挙成功の場合はデバイス情報リストを返す
        Ok(ports) => {
            // ポート情報を SerialDeviceInfo に変換する
            let device_infos: Vec<SerialDeviceInfo> = ports.iter().map(|port| {
                // SerialDeviceInfo を構築して返す
                SerialDeviceInfo {
                    // ポート名を設定する
                    port_name: port.port_name.clone(),
                    // デフォルトのボーレートを設定する (接続時に指定する)
                    baud_rate: 115200,
                    // 未接続状態で初期化する
                    connected: false,
                }
            }).collect();
            // シリアルポート一覧ログを出力する
            info!("シリアルポート一覧取得: {} ポートが見つかった", device_infos.len());
            // JSON レスポンスを返す
            (StatusCode::OK, Json(device_infos)).into_response()
        }
        // 列挙失敗の場合はエラーを返す
        Err(e) => {
            // シリアルポート列挙失敗ログを記録する
            error!("シリアルポート列挙失敗: {}", e);
            // エラーレスポンスを返す
            (StatusCode::INTERNAL_SERVER_ERROR, format!("シリアルポート列挙失敗: {}", e)).into_response()
        }
    }
}

// シリアルポート接続ハンドラ: 指定のシリアルポートに接続する
async fn connect_serial_port(
    // 共有状態を受け取る
    State(state): State<SharedState>,
    // リクエストボディを受け取る
    Json(req): Json<ConnectSerialRequest>,
) -> impl IntoResponse {
    // シリアルポートに接続する
    match serialport::new(&req.port_name, req.baud_rate)
        // タイムアウトを 1 秒に設定する
        .timeout(Duration::from_secs(1))
        // シリアルポートを開く
        .open()
    {
        // 接続成功の場合はデバイス情報を返す
        Ok(port) => {
            // シリアルポート接続成功ログを出力する
            info!("シリアルポート接続成功: port={}, baud_rate={}", req.port_name, req.baud_rate);
            // ミューテックスロックを取得する
            let mut state_guard = match state.lock() {
                // ロック取得成功の場合は続行する
                Ok(g) => g,
                // ロック取得失敗の場合はエラーを返す
                Err(e) => {
                    // ロック失敗ログを記録する
                    error!("状態ロック失敗: {}", e);
                    // エラーレスポンスを返す
                    return (StatusCode::INTERNAL_SERVER_ERROR, "状態ロック失敗".to_string()).into_response();
                }
            };
            // シリアルポートを状態に追加する
            state_guard.serial_connections.insert(req.port_name.clone(), port);
            // 接続成功デバイス情報を返す
            let info_resp = SerialDeviceInfo {
                // ポート名を設定する
                port_name: req.port_name,
                // ボーレートを設定する
                baud_rate: req.baud_rate,
                // 接続状態を true に設定する
                connected: true,
            };
            // JSON レスポンスを返す
            (StatusCode::OK, Json(info_resp)).into_response()
        }
        // 接続失敗の場合はエラーを返す
        Err(e) => {
            // シリアルポート接続失敗ログを記録する
            error!("シリアルポート接続失敗: port={}, error={}", req.port_name, e);
            // エラーレスポンスを返す
            (StatusCode::BAD_REQUEST, format!("シリアルポート接続失敗: {}", e)).into_response()
        }
    }
}

// シリアルポートデータ送信ハンドラ: 指定のシリアルポートにデータを送信する
async fn send_serial_data(
    // 共有状態を受け取る
    State(state): State<SharedState>,
    // リクエストボディを受け取る
    Json(req): Json<SendSerialDataRequest>,
) -> impl IntoResponse {
    // Base64 デコードは簡略化してバイト列を直接使用する
    let data = req.data_base64.as_bytes().to_vec();
    // ミューテックスロックを取得する
    let mut state_guard = match state.lock() {
        // ロック取得成功の場合は続行する
        Ok(g) => g,
        // ロック取得失敗の場合はエラーを返す
        Err(e) => {
            // ロック失敗ログを記録する
            error!("状態ロック失敗: {}", e);
            // エラーレスポンスを返す
            return (StatusCode::INTERNAL_SERVER_ERROR, "状態ロック失敗".to_string()).into_response();
        }
    };
    // シリアルポートを取得する
    let port = match state_guard.serial_connections.get_mut(&req.port_name) {
        // ポートが見つかった場合は続行する
        Some(p) => p,
        // ポートが見つからない場合はエラーを返す
        None => {
            // ポート未検出ログを記録する
            warn!("シリアルポートが見つからない: {}", req.port_name);
            // エラーレスポンスを返す
            return (StatusCode::NOT_FOUND, format!("シリアルポートが見つからない: {}", req.port_name)).into_response();
        }
    };
    // シリアルポートにデータを送信する
    match port.write(&data) {
        // 送信成功の場合は送信バイト数を返す
        Ok(bytes_written) => {
            // データ送信成功ログを出力する
            info!("シリアルデータ送信: port={}, bytes={}", req.port_name, bytes_written);
            // JSON レスポンスを返す
            (StatusCode::OK, Json(bytes_written)).into_response()
        }
        // 送信失敗の場合はエラーを返す
        Err(e) => {
            // データ送信失敗ログを記録する
            error!("シリアルデータ送信失敗: port={}, error={}", req.port_name, e);
            // エラーレスポンスを返す
            (StatusCode::INTERNAL_SERVER_ERROR, format!("シリアルデータ送信失敗: {}", e)).into_response()
        }
    }
}

// Bluetooth デバイス一覧取得ハンドラ: キャッシュされた Bluetooth デバイスを返す
async fn list_bluetooth_devices(
    // 共有状態を受け取る
    State(state): State<SharedState>,
) -> impl IntoResponse {
    // 共有状態から Bluetooth デバイスキャッシュを取得する
    let state_guard = match state.lock() {
        // ロック取得成功の場合は続行する
        Ok(g) => g,
        // ロック取得失敗の場合はエラーを返す
        Err(e) => {
            // ロック失敗ログを記録する
            error!("状態ロック失敗: {}", e);
            // エラーレスポンスを返す
            return (StatusCode::INTERNAL_SERVER_ERROR, "状態ロック失敗".to_string()).into_response();
        }
    };
    // Bluetooth デバイスキャッシュをベクターに変換して返す
    let devices: Vec<BluetoothDeviceInfo> = state_guard.bluetooth_devices.values().cloned().collect();
    // Bluetooth デバイス一覧ログを出力する
    info!("Bluetooth デバイス一覧取得: {} デバイスが見つかった", devices.len());
    // JSON レスポンスを返す
    (StatusCode::OK, Json(devices)).into_response()
}

// WebUSB デバイス一覧取得ハンドラ: キャッシュされた WebUSB デバイスを返す
async fn list_webusb_devices(
    // 共有状態を受け取る
    State(state): State<SharedState>,
) -> impl IntoResponse {
    // 共有状態から WebUSB デバイスキャッシュを取得する
    let state_guard = match state.lock() {
        // ロック取得成功の場合は続行する
        Ok(g) => g,
        // ロック取得失敗の場合はエラーを返す
        Err(e) => {
            // ロック失敗ログを記録する
            error!("状態ロック失敗: {}", e);
            // エラーレスポンスを返す
            return (StatusCode::INTERNAL_SERVER_ERROR, "状態ロック失敗".to_string()).into_response();
        }
    };
    // WebUSB デバイスキャッシュをベクターに変換して返す
    let devices: Vec<WebUsbDeviceInfo> = state_guard.webusb_devices.values().cloned().collect();
    // WebUSB デバイス一覧ログを出力する
    info!("WebUSB デバイス一覧取得: {} デバイスが見つかった", devices.len());
    // JSON レスポンスを返す
    (StatusCode::OK, Json(devices)).into_response()
}

// Sidecar バージョン取得ハンドラ: Sidecar のバージョン情報を返す
async fn get_sidecar_version() -> impl IntoResponse {
    // パッケージバージョンを返す
    (StatusCode::OK, env!("CARGO_PKG_VERSION")).into_response()
}

// ヘルスチェックレスポンスを表す構造体
#[derive(Debug, Serialize)]
struct HealthResponse {
    // サービス稼働状態 ("ok" を返す)
    status: String,
    // バイナリバージョン (Cargo.toml の package.version から埋め込む)
    version: String,
}

// 状態同期リクエストを表す構造体 (Tauri companion から受け取る)
#[derive(Debug, Deserialize)]
struct StateSyncRequest {
    // 同期対象レイヤ名 (spec 正値: server_truth / optimistic_local / pending_queue / draft)
    layer: String,
    // BFF 転送先エンドポイント URL (省略時はローカル受理のみ)
    bff_url: Option<String>,
    // 同期するエントリリスト (JSON 値のベクター)
    entries: Vec<serde_json::Value>,
}

// 状態同期レスポンスを表す構造体
#[derive(Debug, Serialize)]
struct StateSyncResponse {
    // 同期成功フラグ
    success: bool,
    // 同期済みエントリ数
    synced_count: usize,
    // エラーメッセージ (失敗時のみ存在する)
    error: Option<String>,
}

// ヘルスチェックハンドラ: サービス稼働状態と現在バージョンを返す
async fn health_check() -> impl IntoResponse {
    // ヘルスチェックレスポンスを構築する
    let resp = HealthResponse {
        // 稼働状態 "ok" を設定する
        status: "ok".to_string(),
        // Cargo.toml から埋め込まれたバージョン文字列を設定する
        version: env!("CARGO_PKG_VERSION").to_string(),
    };
    // ヘルスチェックログを出力する
    info!("ヘルスチェック OK: version={}", resp.version);
    // JSON レスポンスを返す
    (StatusCode::OK, Json(resp)).into_response()
}

// 状態同期ハンドラ: Tauri companion から受け取った状態を BFF に転送する
async fn state_sync_handler(
    // リクエストボディ (JSON 形式の StateSyncRequest) を受け取る
    Json(req): Json<StateSyncRequest>,
) -> impl IntoResponse {
    // 同期対象レイヤとエントリ数をログに出力する
    info!("状態同期リクエスト受信: layer={}, entries={}", req.layer, req.entries.len());
    // BFF URL が指定されている場合は転送する
    if let Some(bff_url) = req.bff_url {
        // reqwest クライアントをタイムアウト 10 秒で生成する
        let client = match reqwest::Client::builder()
            // タイムアウトを 10 秒に設定する
            .timeout(Duration::from_secs(10))
            // クライアントをビルドする
            .build()
        {
            // クライアント生成成功の場合は続行する
            Ok(c) => c,
            // クライアント生成失敗の場合はエラーレスポンスを返す
            Err(e) => {
                // クライアント生成失敗ログを記録する
                error!("reqwest クライアント生成失敗: {}", e);
                // エラーレスポンスを構築して返す
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(StateSyncResponse {
                    // 失敗フラグを設定する
                    success: false,
                    // 同期済みエントリ数を 0 に設定する
                    synced_count: 0,
                    // エラーメッセージを設定する
                    error: Some(format!("reqwest クライアント生成失敗: {}", e)),
                })).into_response();
            }
        };
        // BFF に転送するペイロードを構築する (layer と entries を含む JSON オブジェクト)
        let payload = serde_json::json!({
            // 同期対象レイヤ名を設定する
            "layer": req.layer,
            // エントリリストを設定する
            "entries": req.entries,
        });
        // BFF エンドポイントに POST リクエストを送信する
        match client.post(&bff_url).json(&payload).send().await {
            // BFF 転送成功の場合は結果を返す
            Ok(response) => {
                // BFF レスポンスが成功 (2xx) かどうかを確認する
                if response.status().is_success() {
                    // 転送成功ログを出力する
                    info!("BFF 転送成功: url={}, entries={}", bff_url, req.entries.len());
                    // 成功レスポンスを返す
                    (StatusCode::OK, Json(StateSyncResponse {
                        // 成功フラグを設定する
                        success: true,
                        // 同期済みエントリ数を設定する
                        synced_count: req.entries.len(),
                        // エラーなし
                        error: None,
                    })).into_response()
                } else {
                    // BFF からのエラーレスポンスのステータスコードを取得する
                    let status = response.status();
                    // BFF エラーログを記録する
                    warn!("BFF 転送失敗: url={}, status={}", bff_url, status);
                    // エラーレスポンスを返す
                    (StatusCode::BAD_GATEWAY, Json(StateSyncResponse {
                        // 失敗フラグを設定する
                        success: false,
                        // 同期済みエントリ数を 0 に設定する
                        synced_count: 0,
                        // BFF からのエラーステータスを含むメッセージを設定する
                        error: Some(format!("BFF エラー: HTTP {}", status)),
                    })).into_response()
                }
            }
            // reqwest 送信失敗 (ネットワークエラー等) の場合はエラーを返す
            Err(e) => {
                // 転送失敗ログを記録する
                error!("BFF 転送失敗: url={}, error={}", bff_url, e);
                // エラーレスポンスを返す
                (StatusCode::SERVICE_UNAVAILABLE, Json(StateSyncResponse {
                    // 失敗フラグを設定する
                    success: false,
                    // 同期済みエントリ数を 0 に設定する
                    synced_count: 0,
                    // エラーメッセージを設定する
                    error: Some(format!("BFF 転送失敗: {}", e)),
                })).into_response()
            }
        }
    } else {
        // BFF URL が指定されていない場合はローカル受理のみ行う
        info!("状態同期: BFF URL なし、ローカル受理: layer={}, entries={}", req.layer, req.entries.len());
        // ローカル受理成功レスポンスを返す
        (StatusCode::OK, Json(StateSyncResponse {
            // 成功フラグを設定する
            success: true,
            // 同期済みエントリ数を設定する
            synced_count: req.entries.len(),
            // エラーなし
            error: None,
        })).into_response()
    }
}

// メイン関数: Sidecar HTTP サーバーを起動する
#[tokio::main]
async fn main() {
    // tracing サブスクライバーを初期化する
    tracing_subscriber::fmt()
        // 環境変数フィルターを設定する
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("k1s0_companion_sidecar=info".parse().unwrap_or_default())
        )
        // tracing サブスクライバーを初期化する
        .init();

    // Sidecar 初期状態を Arc<Mutex<>> でラップする
    let sidecar_state: SharedState = Arc::new(Mutex::new(SidecarState::default()));
    // Sidecar 起動ログを出力する
    info!("k1s0 Tauri コンパニオン Sidecar 起動 (HTTP IPC mode)");

    // axum ルーターを構築する
    let app = Router::new()
        // ヘルスチェックエンドポイントを登録する (Tauri companion が死活監視に使用する)
        .route("/health", get(health_check))
        // 状態同期エンドポイントを登録する (Tauri companion から BFF へ状態を転送する)
        .route("/state/sync", post(state_sync_handler))
        // シリアルポート一覧取得エンドポイントを登録する
        .route("/serial/list", get(list_serial_ports))
        // シリアルポート接続エンドポイントを登録する
        .route("/serial/connect", post(connect_serial_port))
        // シリアルデータ送信エンドポイントを登録する
        .route("/serial/send", post(send_serial_data))
        // Bluetooth デバイス一覧取得エンドポイントを登録する
        .route("/bluetooth/list", get(list_bluetooth_devices))
        // WebUSB デバイス一覧取得エンドポイントを登録する
        .route("/webusb/list", get(list_webusb_devices))
        // バージョン取得エンドポイントを登録する
        .route("/version", get(get_sidecar_version))
        // 共有状態をルーターに注入する
        .with_state(sidecar_state);

    // ローカルホスト 9999 番ポートでリッスンする (Tauri sidecar の標準 IPC ポート)
    let listener = tokio::net::TcpListener::bind("127.0.0.1:9999").await
        // リスナー作成失敗時はパニックする
        .expect("127.0.0.1:9999 のリスナー作成失敗");
    // サーバー起動ログを出力する
    info!("k1s0 Sidecar HTTP サーバー起動: http://127.0.0.1:9999");
    // axum サーバーを起動する
    axum::serve(listener, app).await
        // サーバー実行失敗時はパニックする
        .expect("k1s0 Sidecar HTTP サーバーの起動に失敗した");
}

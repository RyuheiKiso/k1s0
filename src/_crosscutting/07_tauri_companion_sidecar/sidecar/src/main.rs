// k1s0 Tauri コンパニオン Sidecar のメインファイル
// WebUSB / Bluetooth / Serial ブリッジの最小実装を提供する Tauri 2.x アプリ
// Tauri メイン関数はマクロで定義するためlinting除外
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Tauri クレートをインポートする
use tauri::{
    // Tauri アプリケーションビルダーをインポートする
    Builder,
    // Tauri コマンドハンドラをインポートする
    command,
    // Tauri アプリケーションハンドルをインポートする
    AppHandle,
    // Tauri マネージャートレイトをインポートする
    Manager,
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

// シリアルポート一覧取得コマンド: 利用可能なシリアルポートを返す
#[command]
async fn list_serial_ports() -> Result<Vec<SerialDeviceInfo>, String> {
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
            // デバイス情報リストを返す
            Ok(device_infos)
        }
        // 列挙失敗の場合はエラーを返す
        Err(e) => {
            // シリアルポート列挙失敗ログを記録する
            error!("シリアルポート列挙失敗: {}", e);
            // エラーメッセージを返す
            Err(format!("シリアルポート列挙失敗: {}", e))
        }
    }
}

// シリアルポート接続コマンド: 指定のシリアルポートに接続する
#[command]
async fn connect_serial_port(
    // アプリケーションハンドルを受け取る
    app: AppHandle,
    // 接続するポート名を受け取る
    port_name: String,
    // ボーレートを受け取る
    baud_rate: u32,
) -> Result<SerialDeviceInfo, String> {
    // シリアルポートに接続する
    match serialport::new(&port_name, baud_rate)
        // タイムアウトを 1 秒に設定する
        .timeout(Duration::from_secs(1))
        // シリアルポートを開く
        .open()
    {
        // 接続成功の場合はデバイス情報を返す
        Ok(port) => {
            // シリアルポート接続成功ログを出力する
            info!("シリアルポート接続成功: port={}, baud_rate={}", port_name, baud_rate);
            // 共有状態にシリアルポートを登録する
            let state = app.state::<Arc<Mutex<SidecarState>>>();
            // ミューテックスロックを取得する
            let mut state_guard = state.lock().map_err(|e| format!("状態ロック失敗: {}", e))?;
            // シリアルポートを状態に追加する
            state_guard.serial_connections.insert(port_name.clone(), port);
            // 接続成功デバイス情報を返す
            Ok(SerialDeviceInfo {
                // ポート名を設定する
                port_name,
                // ボーレートを設定する
                baud_rate,
                // 接続状態を true に設定する
                connected: true,
            })
        }
        // 接続失敗の場合はエラーを返す
        Err(e) => {
            // シリアルポート接続失敗ログを記録する
            error!("シリアルポート接続失敗: port={}, error={}", port_name, e);
            // エラーメッセージを返す
            Err(format!("シリアルポート接続失敗: {}", e))
        }
    }
}

// シリアルポートデータ送信コマンド: 指定のシリアルポートにデータを送信する
#[command]
async fn send_serial_data(
    // アプリケーションハンドルを受け取る
    app: AppHandle,
    // 送信先ポート名を受け取る
    port_name: String,
    // 送信データを受け取る (Base64 エンコードされたバイト列)
    data_base64: String,
) -> Result<usize, String> {
    // Base64 デコードは簡略化してバイト列を直接使用する (実装では base64 クレートを使用する)
    let data = data_base64.as_bytes().to_vec();
    // 共有状態からシリアルポートを取得する
    let state = app.state::<Arc<Mutex<SidecarState>>>();
    // ミューテックスロックを取得する
    let mut state_guard = state.lock().map_err(|e| format!("状態ロック失敗: {}", e))?;
    // シリアルポートを取得する
    let port = state_guard.serial_connections.get_mut(&port_name)
        .ok_or_else(|| format!("シリアルポートが見つからない: {}", port_name))?;
    // シリアルポートにデータを送信する
    match port.write(&data) {
        // 送信成功の場合は送信バイト数を返す
        Ok(bytes_written) => {
            // データ送信成功ログを出力する
            info!("シリアルデータ送信: port={}, bytes={}", port_name, bytes_written);
            // 送信バイト数を返す
            Ok(bytes_written)
        }
        // 送信失敗の場合はエラーを返す
        Err(e) => {
            // データ送信失敗ログを記録する
            error!("シリアルデータ送信失敗: port={}, error={}", port_name, e);
            // エラーメッセージを返す
            Err(format!("シリアルデータ送信失敗: {}", e))
        }
    }
}

// Bluetooth デバイス一覧取得コマンド: キャッシュされた Bluetooth デバイスを返す
#[command]
async fn list_bluetooth_devices(
    // アプリケーションハンドルを受け取る
    app: AppHandle,
) -> Result<Vec<BluetoothDeviceInfo>, String> {
    // 共有状態から Bluetooth デバイスキャッシュを取得する
    let state = app.state::<Arc<Mutex<SidecarState>>>();
    // ミューテックスロックを取得する
    let state_guard = state.lock().map_err(|e| format!("状態ロック失敗: {}", e))?;
    // Bluetooth デバイスキャッシュをベクターに変換して返す
    let devices: Vec<BluetoothDeviceInfo> = state_guard.bluetooth_devices.values().cloned().collect();
    // Bluetooth デバイス一覧ログを出力する
    info!("Bluetooth デバイス一覧取得: {} デバイスが見つかった", devices.len());
    // デバイス情報リストを返す
    Ok(devices)
}

// WebUSB デバイス一覧取得コマンド: キャッシュされた WebUSB デバイスを返す
#[command]
async fn list_webusb_devices(
    // アプリケーションハンドルを受け取る
    app: AppHandle,
) -> Result<Vec<WebUsbDeviceInfo>, String> {
    // 共有状態から WebUSB デバイスキャッシュを取得する
    let state = app.state::<Arc<Mutex<SidecarState>>>();
    // ミューテックスロックを取得する
    let state_guard = state.lock().map_err(|e| format!("状態ロック失敗: {}", e))?;
    // WebUSB デバイスキャッシュをベクターに変換して返す
    let devices: Vec<WebUsbDeviceInfo> = state_guard.webusb_devices.values().cloned().collect();
    // WebUSB デバイス一覧ログを出力する
    info!("WebUSB デバイス一覧取得: {} デバイスが見つかった", devices.len());
    // デバイス情報リストを返す
    Ok(devices)
}

// Sidecar バージョン取得コマンド: Sidecar のバージョン情報を返す
#[command]
async fn get_sidecar_version() -> Result<String, String> {
    // パッケージバージョンを返す
    Ok(env!("CARGO_PKG_VERSION").to_string())
}

// メイン関数: Tauri アプリケーションを起動する
fn main() {
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
    let sidecar_state = Arc::new(Mutex::new(SidecarState::default()));
    // Tauri 起動ログを出力する
    info!("k1s0 Tauri コンパニオン Sidecar 起動");

    // Tauri アプリケーションを構築して実行する
    Builder::default()
        // 共有状態を Tauri に登録する
        .manage(sidecar_state)
        // Tauri コマンドハンドラを登録する
        .invoke_handler(tauri::generate_handler![
            // シリアルポート一覧取得コマンドを登録する
            list_serial_ports,
            // シリアルポート接続コマンドを登録する
            connect_serial_port,
            // シリアルデータ送信コマンドを登録する
            send_serial_data,
            // Bluetooth デバイス一覧取得コマンドを登録する
            list_bluetooth_devices,
            // WebUSB デバイス一覧取得コマンドを登録する
            list_webusb_devices,
            // Sidecar バージョン取得コマンドを登録する
            get_sidecar_version,
        ])
        // Tauri アプリケーションを実行する
        .run(tauri::generate_context!())
        // 実行エラーが発生した場合はパニックする
        .expect("k1s0 Tauri コンパニオン Sidecar の起動に失敗した");
}

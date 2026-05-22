// k1s0 tier3 BLE デバイスアダプター
// btleplug クレートを使って BLE スキャン/接続/GATT 読み書きを実装する
// T3-Y1 の BLE adapter 実装要件に従い非同期 BLE 操作を提供する
// wall-clock TTL 禁止規律に従い Instant ではなく HLC を使ったタイムアウト管理は上位レイヤに委譲する

// btleplug: BLE アクセスライブラリ（cross-platform BLE）
use btleplug::api::{
    // Central: BLE Central デバイス（スキャン/接続管理）のインターフェース
    Central,
    // Manager: BLE アダプター管理のインターフェース
    Manager as _,
    // Peripheral: BLE Peripheral（デバイス本体）のインターフェース
    Peripheral as _,
    // ScanFilter: スキャンフィルター（UUID / RSSI 等）
    ScanFilter,
    // CharacteristicFlags: GATT characteristic の読み書き/通知フラグ
    CharacteristicFlags,
};
// btleplug::platform: プラットフォーム依存の実装（Manager / Adapter / Peripheral）
use btleplug::platform::{Manager, Peripheral};
// serde: Tauri IPC レスポンスのシリアライズに使用する
use serde::{Deserialize, Serialize};
// uuid: BLE サービス UUID / キャラクタリスティック UUID の解析に使用する
use uuid::Uuid;
// tokio: 非同期操作に使用する
use tokio::time::Duration;
// futures: ストリームの操作に使用する
use futures::stream::StreamExt;

// BLE デバイス情報を表す構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BleDeviceInfo {
    // BLE デバイスの識別子（MAC アドレス / UUID 等のプラットフォーム依存の文字列）
    pub id: String,
    // デバイスのローカル名（アドバタイズパケットから取得する）
    pub local_name: Option<String>,
    // RSSI（受信信号強度）dBm 値
    pub rssi: Option<i16>,
}

// BLE GATT 読み取り結果を表す構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BleReadResult {
    // キャラクタリスティック UUID
    pub characteristic_uuid: String,
    // 読み取ったデータ（hex 文字列）
    pub data_hex: String,
    // 読み取ったバイト数
    pub bytes_read: usize,
}

/// scan_ble_devices は BLE デバイスをスキャンして検出したデバイス一覧を返す
/// scan_duration_ms ミリ秒間スキャンを実行して検出したデバイスを返す
pub async fn scan_ble_devices(
    // スキャン時間（ミリ秒）
    scan_duration_ms: u64,
) -> Result<Vec<BleDeviceInfo>, String> {
    // BLE Manager を生成する（プラットフォーム依存の BLE バックエンドを使用する）
    let manager = Manager::new()
        .await
        // BLE Manager 生成失敗時はエラーを返す（BLE 未対応デバイス等）
        .map_err(|e| format!("BLE Manager 生成失敗: {}", e))?;

    // BLE アダプター一覧を取得する
    let adapters = manager.adapters()
        .await
        // アダプター取得失敗時はエラーを返す
        .map_err(|e| format!("BLE アダプター取得失敗: {}", e))?;

    // 使用可能なアダプターが存在しない場合はエラーを返す
    let adapter = adapters.into_iter().next()
        .ok_or("BLE アダプターが見つかりません")?;

    // スキャンを開始する（ScanFilter::default() で全デバイスをスキャンする）
    adapter.start_scan(ScanFilter::default())
        .await
        // スキャン開始失敗時はエラーを返す
        .map_err(|e| format!("BLE スキャン開始失敗: {}", e))?;

    // 指定時間待機する（スキャン継続）
    tokio::time::sleep(Duration::from_millis(scan_duration_ms)).await;

    // スキャンを停止する
    adapter.stop_scan()
        .await
        // スキャン停止失敗時はエラーを返す（致命的ではないが記録する）
        .map_err(|e| format!("BLE スキャン停止失敗: {}", e))?;

    // 検出したペリフェラル一覧を取得する
    let peripherals = adapter.peripherals()
        .await
        // ペリフェラル取得失敗時はエラーを返す
        .map_err(|e| format!("BLE ペリフェラル取得失敗: {}", e))?;

    // ペリフェラル一覧を BleDeviceInfo に変換する
    let mut devices = Vec::new();

    // 各ペリフェラルの情報を収集する
    for peripheral in peripherals {
        // ペリフェラルのプロパティを取得する
        let properties = peripheral.properties()
            .await
            // プロパティ取得失敗のデバイスはスキップする
            .unwrap_or(None);

        // ペリフェラル ID を文字列で取得する
        let id = peripheral.id().to_string();

        // ローカル名と RSSI を抽出する
        let (local_name, rssi) = if let Some(props) = properties {
            // ローカル名を取得する（アドバタイズパケットに含まれない場合は None）
            (props.local_name, props.rssi.map(|r| r as i16))
        } else {
            // プロパティなしの場合は両方 None
            (None, None)
        };

        // BleDeviceInfo を構築して追加する
        devices.push(BleDeviceInfo {
            // デバイス ID を設定する
            id,
            // ローカル名を設定する
            local_name,
            // RSSI を設定する
            rssi,
        });
    }

    // デバイス一覧を返す
    Ok(devices)
}

/// read_ble_characteristic は BLE ペリフェラルの GATT キャラクタリスティックを読み取る
/// peripheral_id でデバイスを特定し characteristic_uuid の値を読み取る
pub async fn read_ble_characteristic(
    // BLE ペリフェラル ID（scan_ble_devices の id フィールド）
    peripheral_id: &str,
    // 読み取るキャラクタリスティック UUID（RFC 4122 形式）
    characteristic_uuid: &str,
) -> Result<BleReadResult, String> {
    // BLE Manager を生成する
    let manager = Manager::new()
        .await
        // BLE Manager 生成失敗時はエラーを返す
        .map_err(|e| format!("BLE Manager 生成失敗: {}", e))?;

    // BLE アダプターを取得する
    let adapters = manager.adapters()
        .await
        // アダプター取得失敗時はエラーを返す
        .map_err(|e| format!("BLE アダプター取得失敗: {}", e))?;

    // 使用可能なアダプターを取得する
    let adapter = adapters.into_iter().next()
        .ok_or("BLE アダプターが見つかりません")?;

    // キャラクタリスティック UUID を解析する
    let char_uuid = Uuid::parse_str(characteristic_uuid)
        // UUID 解析失敗時はエラーを返す
        .map_err(|e| format!("キャラクタリスティック UUID 解析失敗: {}", e))?;

    // 指定 ID のペリフェラルを取得する
    let peripherals = adapter.peripherals()
        .await
        // ペリフェラル取得失敗時はエラーを返す
        .map_err(|e| format!("BLE ペリフェラル取得失敗: {}", e))?;

    // peripheral_id に一致するペリフェラルを検索する
    let peripheral = peripherals.into_iter()
        .find(|p| p.id().to_string() == peripheral_id)
        .ok_or_else(|| format!("BLE ペリフェラルが見つかりません: {}", peripheral_id))?;

    // ペリフェラルに接続する
    peripheral.connect()
        .await
        // 接続失敗時はエラーを返す（デバイスが範囲外等）
        .map_err(|e| format!("BLE 接続失敗 {}: {}", peripheral_id, e))?;

    // GATT サービスを探索する（キャラクタリスティック一覧を取得するために必要）
    peripheral.discover_services()
        .await
        // サービス探索失敗時はエラーを返す
        .map_err(|e| format!("GATT サービス探索失敗: {}", e))?;

    // 全キャラクタリスティックを取得する
    let characteristics = peripheral.characteristics();

    // 指定 UUID のキャラクタリスティックを検索する
    let characteristic = characteristics.iter()
        .find(|c| c.uuid == char_uuid)
        .ok_or_else(|| format!("キャラクタリスティックが見つかりません: {}", characteristic_uuid))?
        .clone();

    // キャラクタリスティック値を読み取る
    let data = peripheral.read(&characteristic)
        .await
        // 読み取り失敗時はエラーを返す（権限なし等）
        .map_err(|e| format!("GATT 読み取り失敗 {}: {}", characteristic_uuid, e))?;

    // 接続を切断する（リソース解放）
    let _ = peripheral.disconnect().await;

    // 読み取ったデータを hex 文字列に変換する
    let data_hex = data.iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(" ");

    // 読み取り結果を返す
    Ok(BleReadResult {
        // キャラクタリスティック UUID を設定する
        characteristic_uuid: characteristic_uuid.to_string(),
        // hex 文字列を設定する
        data_hex,
        // バイト数を設定する
        bytes_read: data.len(),
    })
}

/// write_ble_characteristic は BLE ペリフェラルの GATT キャラクタリスティックに値を書き込む
pub async fn write_ble_characteristic(
    // BLE ペリフェラル ID
    peripheral_id: &str,
    // 書き込むキャラクタリスティック UUID
    characteristic_uuid: &str,
    // 書き込むデータ
    data: Vec<u8>,
    // 応答要求フラグ（true: Write With Response, false: Write Without Response）
    with_response: bool,
) -> Result<(), String> {
    // BLE Manager を生成する
    let manager = Manager::new()
        .await
        // BLE Manager 生成失敗時はエラーを返す
        .map_err(|e| format!("BLE Manager 生成失敗: {}", e))?;

    // BLE アダプターを取得する
    let adapters = manager.adapters()
        .await
        .map_err(|e| format!("BLE アダプター取得失敗: {}", e))?;

    // 使用可能なアダプターを取得する
    let adapter = adapters.into_iter().next()
        .ok_or("BLE アダプターが見つかりません")?;

    // キャラクタリスティック UUID を解析する
    let char_uuid = Uuid::parse_str(characteristic_uuid)
        .map_err(|e| format!("キャラクタリスティック UUID 解析失敗: {}", e))?;

    // 指定 ID のペリフェラルを検索して接続する
    let peripherals = adapter.peripherals().await
        .map_err(|e| format!("BLE ペリフェラル取得失敗: {}", e))?;
    let peripheral = peripherals.into_iter()
        .find(|p| p.id().to_string() == peripheral_id)
        .ok_or_else(|| format!("BLE ペリフェラルが見つかりません: {}", peripheral_id))?;

    // 接続する
    peripheral.connect().await
        .map_err(|e| format!("BLE 接続失敗: {}", e))?;

    // GATT サービスを探索する
    peripheral.discover_services().await
        .map_err(|e| format!("GATT サービス探索失敗: {}", e))?;

    // キャラクタリスティックを取得する
    let characteristics = peripheral.characteristics();
    let characteristic = characteristics.iter()
        .find(|c| c.uuid == char_uuid)
        .ok_or_else(|| format!("キャラクタリスティックが見つかりません: {}", characteristic_uuid))?
        .clone();

    // WriteType を決定する（with_response で分岐する）
    let write_type = if with_response {
        // Write With Response: 書き込み確認応答を要求する
        btleplug::api::WriteType::WithResponse
    } else {
        // Write Without Response: 確認応答なしで書き込む（高速だが信頼性低い）
        btleplug::api::WriteType::WithoutResponse
    };

    // キャラクタリスティックに書き込む
    peripheral.write(&characteristic, &data, write_type)
        .await
        .map_err(|e| format!("GATT 書き込み失敗 {}: {}", characteristic_uuid, e))?;

    // 接続を切断する
    let _ = peripheral.disconnect().await;

    // 書き込み成功を返す
    Ok(())
}

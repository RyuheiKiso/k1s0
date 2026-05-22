// k1s0 tier3 USB デバイスアダプター
// rusb クレートを使って USB デバイスの検出・読み取り・書き込みを実装する
// T3-Y1 の USB adapter 実装要件に従いデバイス列挙 / バルク転送を提供する

// rusb: USB デバイスアクセスライブラリ（libusb の Rust binding）
use rusb::{Context, Device, DeviceHandle, DeviceList, GlobalContext, UsbContext};
// serde: シリアライズ/デシリアライズ（Tauri IPC レスポンスに使用する）
use serde::{Deserialize, Serialize};
// std::time: USB タイムアウト設定に使用する
use std::time::Duration;

// USB デバイス情報を表す構造体（Tauri IPC レスポンスとして使用する）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsbDeviceInfo {
    // USB ベンダー ID（16 進数文字列）
    pub vendor_id: u16,
    // USB プロダクト ID（16 進数文字列）
    pub product_id: u16,
    // USB バスアドレス（デバイス一意識別に使用する）
    pub bus_number: u8,
    // USB デバイスアドレス（同一バス上のデバイスを区別する）
    pub device_address: u8,
    // デバイス説明文（製品名 / 製造元名）
    pub description: String,
}

// USB 読み取り結果を表す構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsbReadResult {
    // 読み取ったバイト数
    pub bytes_read: usize,
    // 読み取ったデータ（hex 文字列として返す）
    pub data_hex: String,
}

/// list_usb_devices は接続されている USB デバイスの一覧を返す
/// libusb の GlobalContext を使ってデバイスを列挙する
pub fn list_usb_devices() -> Result<Vec<UsbDeviceInfo>, String> {
    // libusb GlobalContext を使って USB デバイスリストを取得する
    let device_list = DeviceList::new()
        // デバイスリスト取得失敗時はエラーを返す（libusb 未インストール等）
        .map_err(|e| format!("USB デバイスリスト取得失敗: {}", e))?;

    // デバイスリストを UsbDeviceInfo に変換する
    let mut devices = Vec::new();

    // 全デバイスを走査して情報を収集する
    for device in device_list.iter() {
        // デバイスディスクリプタを取得する
        let descriptor = match device.device_descriptor() {
            // ディスクリプタ取得成功
            Ok(d) => d,
            // ディスクリプタ取得失敗のデバイスはスキップする
            Err(_) => continue,
        };

        // USB バス番号を取得する
        let bus_number = device.bus_number();
        // USB デバイスアドレスを取得する
        let device_address = device.address();

        // デバイス情報を構築して追加する
        devices.push(UsbDeviceInfo {
            // ベンダー ID を設定する
            vendor_id: descriptor.vendor_id(),
            // プロダクト ID を設定する
            product_id: descriptor.product_id(),
            // バス番号を設定する
            bus_number,
            // デバイスアドレスを設定する
            device_address,
            // 説明文を設定する（VID:PID 形式）
            description: format!(
                "USB Device VID={:04X} PID={:04X}",
                descriptor.vendor_id(),
                descriptor.product_id()
            ),
        });
    }

    // デバイス一覧を返す
    Ok(devices)
}

/// read_usb_bulk は指定した USB デバイスのバルク IN エンドポイントからデータを読み取る
/// vendor_id / product_id でデバイスを特定し、endpoint_address のバルク転送を実行する
pub fn read_usb_bulk(
    // ターゲットデバイスのベンダー ID
    vendor_id: u16,
    // ターゲットデバイスのプロダクト ID
    product_id: u16,
    // バルク IN エンドポイントアドレス（通常 0x81 等）
    endpoint_address: u8,
    // 読み取るバイト数
    read_size: usize,
) -> Result<UsbReadResult, String> {
    // libusb GlobalContext を使ってデバイスリストを取得する
    let device_list = DeviceList::new()
        // デバイスリスト取得失敗時はエラーを返す
        .map_err(|e| format!("USB デバイスリスト取得失敗: {}", e))?;

    // 指定の VID/PID を持つデバイスを検索する
    let device = device_list.iter().find(|d| {
        // デバイスディスクリプタを取得する（失敗時は false を返す）
        d.device_descriptor()
            // VID と PID が一致するデバイスを探す
            .map(|desc| desc.vendor_id() == vendor_id && desc.product_id() == product_id)
            // ディスクリプタ取得失敗のデバイスは対象外
            .unwrap_or(false)
    });

    // デバイスが見つからない場合はエラーを返す
    let device = device
        .ok_or_else(|| format!("USB デバイスが見つかりません: VID={:04X} PID={:04X}", vendor_id, product_id))?;

    // デバイスをオープンしてハンドルを取得する
    let handle = device.open()
        // デバイスオープン失敗時はエラーを返す（権限不足等）
        .map_err(|e| format!("USB デバイスオープン失敗: {}", e))?;

    // 読み取りバッファを確保する
    let mut buf = vec![0u8; read_size];

    // タイムアウトを設定する（1 秒）
    let timeout = Duration::from_secs(1);

    // バルク IN 転送を実行する
    let bytes_read = handle.read_bulk(endpoint_address, &mut buf, timeout)
        // バルク転送失敗時はエラーを返す
        .map_err(|e| format!("USB バルク読み取り失敗 endpoint={:02X}: {}", endpoint_address, e))?;

    // 読み取ったデータを hex 文字列に変換する
    let data_hex = buf[..bytes_read]
        .iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(" ");

    // 読み取り結果を返す
    Ok(UsbReadResult {
        // 読み取ったバイト数を設定する
        bytes_read,
        // hex 文字列を設定する
        data_hex,
    })
}

/// write_usb_bulk は指定した USB デバイスのバルク OUT エンドポイントにデータを書き込む
/// vendor_id / product_id でデバイスを特定し、endpoint_address のバルク転送を実行する
pub fn write_usb_bulk(
    // ターゲットデバイスのベンダー ID
    vendor_id: u16,
    // ターゲットデバイスのプロダクト ID
    product_id: u16,
    // バルク OUT エンドポイントアドレス（通常 0x01 等）
    endpoint_address: u8,
    // 書き込むデータ
    data: &[u8],
) -> Result<usize, String> {
    // libusb GlobalContext を使ってデバイスリストを取得する
    let device_list = DeviceList::new()
        // デバイスリスト取得失敗時はエラーを返す
        .map_err(|e| format!("USB デバイスリスト取得失敗: {}", e))?;

    // 指定の VID/PID を持つデバイスを検索する
    let device = device_list.iter().find(|d| {
        // デバイスディスクリプタを取得する（失敗時は false を返す）
        d.device_descriptor()
            // VID と PID が一致するデバイスを探す
            .map(|desc| desc.vendor_id() == vendor_id && desc.product_id() == product_id)
            // ディスクリプタ取得失敗のデバイスは対象外
            .unwrap_or(false)
    });

    // デバイスが見つからない場合はエラーを返す
    let device = device
        .ok_or_else(|| format!("USB デバイスが見つかりません: VID={:04X} PID={:04X}", vendor_id, product_id))?;

    // デバイスをオープンしてハンドルを取得する
    let handle = device.open()
        // デバイスオープン失敗時はエラーを返す
        .map_err(|e| format!("USB デバイスオープン失敗: {}", e))?;

    // タイムアウトを設定する（1 秒）
    let timeout = Duration::from_secs(1);

    // バルク OUT 転送を実行する
    let bytes_written = handle.write_bulk(endpoint_address, data, timeout)
        // バルク転送失敗時はエラーを返す
        .map_err(|e| format!("USB バルク書き込み失敗 endpoint={:02X}: {}", endpoint_address, e))?;

    // 書き込んだバイト数を返す
    Ok(bytes_written)
}

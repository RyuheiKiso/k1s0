// k1s0 tier3 シリアルポートアダプター
// serialport クレートを使ってシリアル通信（RS-232C / USB-Serial）を実装する
// T3-Y1 の serial adapter 実装要件に従い列挙/読み取り/書き込みを提供する

// serialport: シリアルポートアクセスライブラリ（cross-platform）
use serialport::{SerialPort, SerialPortInfo, SerialPortType};
// serde: Tauri IPC レスポンスのシリアライズに使用する
use serde::{Deserialize, Serialize};
// std::time: 読み取りタイムアウト設定に使用する
use std::time::Duration;
// std::io: Read / Write トレイトに使用する
use std::io::{Read, Write};

// シリアルポート情報を表す構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerialPortDetails {
    // ポート名（Linux: "/dev/ttyUSB0", Windows: "COM3" 等）
    pub port_name: String,
    // ポートタイプ（USB / PCI / Unknown）
    pub port_type: String,
    // USB VID（USB-Serial アダプターの場合のみ）
    pub usb_vid: Option<u16>,
    // USB PID（USB-Serial アダプターの場合のみ）
    pub usb_pid: Option<u16>,
    // 製品名（USB-Serial アダプターの場合のみ）
    pub product: Option<String>,
}

// シリアル読み取り結果を表す構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerialReadResult {
    // 読み取ったバイト数
    pub bytes_read: usize,
    // 読み取ったデータ（hex 文字列）
    pub data_hex: String,
    // 読み取ったデータ（UTF-8 文字列として解釈できる場合）
    pub data_text: Option<String>,
}

/// list_serial_ports は利用可能なシリアルポートの一覧を返す
/// USB-Serial アダプターの VID/PID も含めて返す
pub fn list_serial_ports() -> Result<Vec<SerialPortDetails>, String> {
    // シリアルポート一覧を取得する
    let ports = serialport::available_ports()
        // ポート一覧取得失敗時はエラーを返す
        .map_err(|e| format!("シリアルポート一覧取得失敗: {}", e))?;

    // SerialPortInfo を SerialPortDetails に変換する
    let mut result = Vec::new();

    // 各ポートの情報を収集する
    for port in ports {
        // ポートタイプに応じて USB VID/PID と製品名を抽出する
        let (port_type_str, usb_vid, usb_pid, product) = match &port.port_type {
            // USB-Serial アダプターの場合は VID/PID/製品名を取得する
            SerialPortType::UsbPort(usb_info) => (
                // ポートタイプを "USB" として設定する
                "USB".to_string(),
                // USB VID を設定する
                Some(usb_info.vid),
                // USB PID を設定する
                Some(usb_info.pid),
                // 製品名を設定する（None の場合もある）
                usb_info.product.clone(),
            ),
            // PCI ポート（内蔵シリアル）の場合
            SerialPortType::PciPort => ("PCI".to_string(), None, None, None),
            // Bluetooth 仮想シリアルポートの場合
            SerialPortType::BluetoothPort => ("Bluetooth".to_string(), None, None, None),
            // 不明なポートタイプの場合
            SerialPortType::Unknown => ("Unknown".to_string(), None, None, None),
        };

        // SerialPortDetails を構築して追加する
        result.push(SerialPortDetails {
            // ポート名を設定する
            port_name: port.port_name,
            // ポートタイプを設定する
            port_type: port_type_str,
            // USB VID を設定する
            usb_vid,
            // USB PID を設定する
            usb_pid,
            // 製品名を設定する
            product,
        });
    }

    // シリアルポート一覧を返す
    Ok(result)
}

/// read_serial は指定したシリアルポートからデータを読み取る
/// port_name でポートを開き read_size バイト読み取って返す
pub fn read_serial(
    // ポート名（Linux: "/dev/ttyUSB0", Windows: "COM3" 等）
    port_name: &str,
    // ボーレート（9600 / 115200 等）
    baud_rate: u32,
    // 読み取るバイト数
    read_size: usize,
    // タイムアウト（ミリ秒）
    timeout_ms: u64,
) -> Result<SerialReadResult, String> {
    // シリアルポートを開く（ボーレートとタイムアウトを設定する）
    let mut port = serialport::new(port_name, baud_rate)
        // タイムアウトを設定する
        .timeout(Duration::from_millis(timeout_ms))
        // ポートをオープンする
        .open()
        // ポートオープン失敗時はエラーを返す（ポートが存在しない等）
        .map_err(|e| format!("シリアルポートオープン失敗 {}: {}", port_name, e))?;

    // 読み取りバッファを確保する
    let mut buf = vec![0u8; read_size];

    // シリアルポートからデータを読み取る
    let bytes_read = port.read(&mut buf)
        // 読み取り失敗時はエラーを返す（タイムアウト等）
        .map_err(|e| format!("シリアル読み取り失敗 {}: {}", port_name, e))?;

    // 読み取ったデータを hex 文字列に変換する
    let data_hex = buf[..bytes_read]
        .iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(" ");

    // 読み取ったデータを UTF-8 テキストとして解釈を試みる
    let data_text = std::str::from_utf8(&buf[..bytes_read])
        // UTF-8 として解釈できる場合は Some に包む
        .map(|s| s.to_string())
        // 解釈失敗の場合は None を返す（バイナリデータ）
        .ok();

    // 読み取り結果を返す
    Ok(SerialReadResult {
        // バイト数を設定する
        bytes_read,
        // hex 文字列を設定する
        data_hex,
        // テキスト解釈結果を設定する
        data_text,
    })
}

/// write_serial は指定したシリアルポートにデータを書き込む
pub fn write_serial(
    // ポート名
    port_name: &str,
    // ボーレート
    baud_rate: u32,
    // 書き込むデータ
    data: &[u8],
    // タイムアウト（ミリ秒）
    timeout_ms: u64,
) -> Result<usize, String> {
    // シリアルポートを開く
    let mut port = serialport::new(port_name, baud_rate)
        // タイムアウトを設定する
        .timeout(Duration::from_millis(timeout_ms))
        // ポートをオープンする
        .open()
        // ポートオープン失敗時はエラーを返す
        .map_err(|e| format!("シリアルポートオープン失敗 {}: {}", port_name, e))?;

    // データを書き込む
    let bytes_written = port.write(data)
        // 書き込み失敗時はエラーを返す
        .map_err(|e| format!("シリアル書き込み失敗 {}: {}", port_name, e))?;

    // 書き込んだバイト数を返す
    Ok(bytes_written)
}

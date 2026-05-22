// k1s0 tier3 Modbus TCP クライアントアダプター
// tokio-modbus クレートを使って Modbus TCP クライアント操作を実装する
// T3-Y1 の modbus adapter 実装要件に従いコイル/レジスタの読み書きを提供する

// tokio_modbus: Modbus TCP/RTU クライアントライブラリ
use tokio_modbus::prelude::*;
// serde: Tauri IPC レスポンスのシリアライズに使用する
use serde::{Deserialize, Serialize};
// std::net: SocketAddr（Modbus TCP 接続先の指定に使用する）
use std::net::SocketAddr;

// Modbus コイル読み取り結果を表す構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModbusCoilResult {
    // 読み取り開始アドレス
    pub start_address: u16,
    // 読み取ったコイル値（true=ON / false=OFF のビット列）
    pub coils: Vec<bool>,
}

// Modbus レジスタ読み取り結果を表す構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModbusRegisterResult {
    // 読み取り開始アドレス
    pub start_address: u16,
    // 読み取ったレジスタ値（16bit unsigned integer の配列）
    pub registers: Vec<u16>,
}

/// read_modbus_coils は Modbus TCP サーバーからコイル（デジタル入力）を読み取る
/// Function Code 0x01 (Read Coils) を使用する
pub async fn read_modbus_coils(
    // Modbus TCP サーバーのソケットアドレス（例: "192.168.1.100:502"）
    server_addr: &str,
    // 読み取り開始アドレス（0 ベース）
    start_address: u16,
    // 読み取るコイル数
    count: u16,
) -> Result<ModbusCoilResult, String> {
    // ソケットアドレスを解析する
    let socket_addr: SocketAddr = server_addr.parse()
        // アドレス解析失敗時はエラーを返す
        .map_err(|e| format!("Modbus TCP アドレス解析失敗 {}: {}", server_addr, e))?;

    // Modbus TCP クライアントを生成して接続する
    let mut ctx = tcp::connect(socket_addr)
        .await
        // 接続失敗時はエラーを返す（サーバー未起動等）
        .map_err(|e| format!("Modbus TCP 接続失敗 {}: {}", server_addr, e))?;

    // Function Code 0x01: Read Coils を実行する
    let coils = ctx.read_coils(start_address, count)
        .await
        // 読み取り失敗時はエラーを返す
        .map_err(|e| format!("Modbus Read Coils 失敗 addr={} count={}: {}", start_address, count, e))?
        // 例外レスポンスをエラーとして扱う
        .map_err(|e| format!("Modbus Read Coils 例外 addr={} count={}: {:?}", start_address, count, e))?;

    // 読み取り結果を返す
    Ok(ModbusCoilResult {
        // 開始アドレスを設定する
        start_address,
        // コイル値を設定する
        coils,
    })
}

/// read_modbus_holding_registers は Modbus TCP サーバーから保持レジスタを読み取る
/// Function Code 0x03 (Read Holding Registers) を使用する
pub async fn read_modbus_holding_registers(
    // Modbus TCP サーバーのソケットアドレス
    server_addr: &str,
    // 読み取り開始アドレス（0 ベース）
    start_address: u16,
    // 読み取るレジスタ数
    count: u16,
) -> Result<ModbusRegisterResult, String> {
    // ソケットアドレスを解析する
    let socket_addr: SocketAddr = server_addr.parse()
        // アドレス解析失敗時はエラーを返す
        .map_err(|e| format!("Modbus TCP アドレス解析失敗 {}: {}", server_addr, e))?;

    // Modbus TCP クライアントを生成して接続する
    let mut ctx = tcp::connect(socket_addr)
        .await
        // 接続失敗時はエラーを返す
        .map_err(|e| format!("Modbus TCP 接続失敗 {}: {}", server_addr, e))?;

    // Function Code 0x03: Read Holding Registers を実行する
    let registers = ctx.read_holding_registers(start_address, count)
        .await
        // 読み取り失敗時はエラーを返す
        .map_err(|e| format!("Modbus Read Holding Registers 失敗 addr={} count={}: {}", start_address, count, e))?
        // 例外レスポンスをエラーとして扱う
        .map_err(|e| format!("Modbus Read Holding Registers 例外 addr={} count={}: {:?}", start_address, count, e))?;

    // 読み取り結果を返す
    Ok(ModbusRegisterResult {
        // 開始アドレスを設定する
        start_address,
        // レジスタ値を設定する
        registers,
    })
}

/// read_modbus_input_registers は Modbus TCP サーバーから入力レジスタを読み取る
/// Function Code 0x04 (Read Input Registers) を使用する
pub async fn read_modbus_input_registers(
    // Modbus TCP サーバーのソケットアドレス
    server_addr: &str,
    // 読み取り開始アドレス
    start_address: u16,
    // 読み取るレジスタ数
    count: u16,
) -> Result<ModbusRegisterResult, String> {
    // ソケットアドレスを解析する
    let socket_addr: SocketAddr = server_addr.parse()
        // アドレス解析失敗時はエラーを返す
        .map_err(|e| format!("Modbus TCP アドレス解析失敗 {}: {}", server_addr, e))?;

    // Modbus TCP クライアントを生成して接続する
    let mut ctx = tcp::connect(socket_addr)
        .await
        // 接続失敗時はエラーを返す
        .map_err(|e| format!("Modbus TCP 接続失敗 {}: {}", server_addr, e))?;

    // Function Code 0x04: Read Input Registers を実行する
    let registers = ctx.read_input_registers(start_address, count)
        .await
        // 読み取り失敗時はエラーを返す
        .map_err(|e| format!("Modbus Read Input Registers 失敗 addr={} count={}: {}", start_address, count, e))?
        // 例外レスポンスをエラーとして扱う
        .map_err(|e| format!("Modbus Read Input Registers 例外 addr={} count={}: {:?}", start_address, count, e))?;

    // 読み取り結果を返す
    Ok(ModbusRegisterResult {
        // 開始アドレスを設定する
        start_address,
        // レジスタ値を設定する
        registers,
    })
}

/// write_modbus_single_coil は Modbus TCP サーバーの単一コイルに書き込む
/// Function Code 0x05 (Write Single Coil) を使用する
pub async fn write_modbus_single_coil(
    // Modbus TCP サーバーのソケットアドレス
    server_addr: &str,
    // 書き込み先コイルアドレス（0 ベース）
    coil_address: u16,
    // 書き込む値（true=ON / false=OFF）
    value: bool,
) -> Result<(), String> {
    // ソケットアドレスを解析する
    let socket_addr: SocketAddr = server_addr.parse()
        // アドレス解析失敗時はエラーを返す
        .map_err(|e| format!("Modbus TCP アドレス解析失敗 {}: {}", server_addr, e))?;

    // Modbus TCP クライアントを生成して接続する
    let mut ctx = tcp::connect(socket_addr)
        .await
        // 接続失敗時はエラーを返す
        .map_err(|e| format!("Modbus TCP 接続失敗 {}: {}", server_addr, e))?;

    // Function Code 0x05: Write Single Coil を実行する
    ctx.write_single_coil(coil_address, value)
        .await
        // 書き込み失敗時はエラーを返す
        .map_err(|e| format!("Modbus Write Single Coil 失敗 addr={} value={}: {}", coil_address, value, e))?
        // 例外レスポンスをエラーとして扱う
        .map_err(|e| format!("Modbus Write Single Coil 例外 addr={}: {:?}", coil_address, e))?;

    // 書き込み成功を返す
    Ok(())
}

/// write_modbus_single_register は Modbus TCP サーバーの単一保持レジスタに書き込む
/// Function Code 0x06 (Write Single Register) を使用する
pub async fn write_modbus_single_register(
    // Modbus TCP サーバーのソケットアドレス
    server_addr: &str,
    // 書き込み先レジスタアドレス（0 ベース）
    register_address: u16,
    // 書き込む値（16bit unsigned integer）
    value: u16,
) -> Result<(), String> {
    // ソケットアドレスを解析する
    let socket_addr: SocketAddr = server_addr.parse()
        // アドレス解析失敗時はエラーを返す
        .map_err(|e| format!("Modbus TCP アドレス解析失敗 {}: {}", server_addr, e))?;

    // Modbus TCP クライアントを生成して接続する
    let mut ctx = tcp::connect(socket_addr)
        .await
        // 接続失敗時はエラーを返す
        .map_err(|e| format!("Modbus TCP 接続失敗 {}: {}", server_addr, e))?;

    // Function Code 0x06: Write Single Register を実行する
    ctx.write_single_register(register_address, value)
        .await
        // 書き込み失敗時はエラーを返す
        .map_err(|e| format!("Modbus Write Single Register 失敗 addr={} value={}: {}", register_address, value, e))?
        // 例外レスポンスをエラーとして扱う
        .map_err(|e| format!("Modbus Write Single Register 例外 addr={}: {:?}", register_address, e))?;

    // 書き込み成功を返す
    Ok(())
}

// k1s0 tier3 デバイスアダプターモジュール定義
// USB / BLE / Serial / OPC-UA / Modbus の 5 種類のデバイスアダプターを公開する
// 各アダプターは T3-Y1 の実装要件に従い独立したサブモジュールとして提供する

// usb アダプター: rusb を使った USB デバイス検出/読み取り/書き込みを提供する
pub mod usb;
// ble アダプター: btleplug を使った BLE スキャン/接続/GATT 読み書きを提供する
pub mod ble;
// serial アダプター: serialport を使ったシリアルポート通信を提供する
pub mod serial;
// opcua アダプター: opcua crate を使った OPC-UA クライアントを提供する
pub mod opcua;
// modbus アダプター: tokio-modbus を使った Modbus TCP クライアントを提供する
pub mod modbus;

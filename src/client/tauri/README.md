# Tauri Desktop SDK

tauri sdk_distribution_class の実装。
Rust sidecar + TypeScript WebView bridge による Tauri デスクトップアプリ向け SDK。

## 主要特性
- Rust sidecar が gRPC bidi / HTTP/3 + WebTransport / WebSocket を担当する
- platform_keystore (OS keychain) による device_at_rest_encryption を実施する
- refresh_token は platform_keystore (OS keychain secure storage) に保管する
- Connect-RPC の Tauri WebSocket bridge を含む
- companion および hlc_lib/typescript に依存する

## 対応仕様
- spec 18: client SDK 配布適合仕様 (class: tauri)

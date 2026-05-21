---
id: detail.cross_bff.tauri_companion_sidecar
axis: cross_bff
phase: cross_cutting
kind: cross_cut_spec
status: published
version: 1.0.0
depends_on:
  - arch.tier3.application_form
  - arch.tier3.device_offline
  - detail.cross_bff.bff_auth_edge
  - detail.tier3.tier3_enforcement
  - detail.infra.clock_integrity_conformance
  - detail.meta.axis_registry_conformance
covered_by:
  defense_in_depth_layers: [A, B, C, D, E]
  proof_classes:
    - v1_temporal_safety_proof
    - v1_program_correctness_proof
related_axes:
  - tier3
  - client
  - infra
---

# Tauri Companion sidecar（v1）

## 位置づけ
- 06_UI 技術スタック / 22_端末オフラインデバイス / 30_互換性ポリシー / 38_製造業 pack 適用例 と双方向 lock
- 全 tier3 Web SPA に Tauri sidecar exe を MDM 必須配布することで、Firefox / Safari でも WebUSB / Web Bluetooth / Web Serial 相当の機能を達成する

## 不可避性
- WebUSB / Web Bluetooth / Web Serial は Chromium 系（Chrome / Edge）でのみ実装。WebKit（Safari）と Gecko（Firefox）は公式に「harmful」表明し実装拒否（W3C TAG response, MDN Compat data, Mozilla position, WebKit standards-positions）
- サポートブラウザを Chrome / Edge / Firefox / Safari の最新 +1 全て、業務にバーコード / プリンタ / RFID / ラベル印刷を要求する以上、Firefox / Safari 単独では業務機能を成立させられない
- sidecar exe を端末に常駐させ、SPA から localhost WebSocket bridge 経由で device access を仲介する経路が唯一の現実解

## 配置
- 配布: Tauri 製 native exe（Windows / macOS / Linux 三種）を MDM（Microsoft Intune / Jamf / Workspace ONE 等）で全業務端末に push 配布。Linux の MDM 成熟度が低い現場では cluster-api 製自製 MDM 配布 channel（Backstage Software Template + cosign + Harbor + per-distro package: deb / rpm / AppImage）で代替
- 起動: OS startup で auto-start、tray icon 常駐
- listen: `127.0.0.1:<random_port>`（起動時に決定、Windows registry / macOS launchd plist / Linux systemd user service の per-user state に保存）

## 認証 + transport（全 UA 共通の (b) 経路を default、Chromium kiosk 限定の (a) 経路を optional）

### (b) default 経路
- `ws://127.0.0.1:<port>/k1s0/v1/...`（TLS なし）
- 三段防御:
  - **Origin pin**: SPA Origin allow-list を sidecar 起動時に Backstage 配布の per-tenant config から固定
  - **DPoP 署名済 payload**: 短期 EdDSA pubkey を sidecar 起動時に SPA 経由で BFF へ POST 登録、SPA からの全 message は sidecar が当該 pubkey で署名検証
  - **per-tenant PSK の HMAC**: PSK は sidecar が起動時に **OS keystore（Windows DPAPI / macOS Keychain / Linux Secret Service）から取得**、SPA 側 JS は触れない、HMAC 計算は SPA → BFF → sidecar の 3 hop で BFF が cookie session 単位の challenge を発行し sidecar が HMAC-SHA256 で response 署名、SPA は session cookie のみで bridge 開始
- 本経路は localhost wss を諦める代わりに **Firefox / Safari / Chrome / Edge / Linux 全 UA で trust store の差を踏まずに成立**し、PSK が JS / SPA の memory に乗らないため XSS で漏洩しない

### (a) optional 経路（chrome_edge_kiosk 限定）
- `wss://127.0.0.1:<port>/k1s0/v1/...` + Origin pin + per-tenant PSK の HMAC + 自署 cert（mkcert 相当の per-device root CA を OS trust store と Firefox の NSS trust store の **両方** に MDM script で install）
- **chrome_edge_kiosk ua_subclass 限定** で利用可（Firefox は OS keystore を共有しない / Linux NSS は手作業必要 / ACME public CA は loopback 不発行 / per-device cert rotation コストが高い、これらの honest な制約を 22_端末オフラインデバイス と双方向 lock）
- kiosk 環境のみで運用可能、それ以外は (b) を利用

## 役割

### 役割 TS-A: device adapter（Rust 実装）
- WebUSB 相当: `rusb` crate で USB device 列挙 / 通信。バーコードリーダ / プリンタ / RFID reader 等を支援
  - **Windows**: device class ごとに WinUSB INF / driver bundle（per VID:PID）を MDM 同梱で配布
  - **Linux**: udev rule（per VID:PID の ACL / mode）を MDM 配布
  - device pack（業界 pack ごとに sidecar bundle に同梱する INF / udev / Info.plist の集合）は tier3 業界 pack の責務として別管理
- Web Bluetooth 相当: `btleplug` crate で BLE peripheral scan / GATT operation
  - **macOS**: `Info.plist` の `NSBluetoothAlwaysUsageDescription` + 起動時の user 許諾 prompt が必須
  - **Windows**: Bluetooth radio enable + pairing 状態は user / OS 設定依存
  - **Linux**: BlueZ + ユーザの bluetooth group 所属が必要
- Web Serial 相当: `serialport` crate で RS-232 / USB-to-Serial を露出。OPC UA / Modbus / EtherNet/IP も Tauri sidecar 内 driver で対応（業務要件に応じて `opc-ua` / `tokio-modbus` 等を bundle）

「自動」表現の honest 化: 本企画は「MDM で全自動 device access」を約束しない。device class（VID:PID 単位）ごとに OS 仕様の必然的 step（INF 署名 / udev rule / Info.plist / user 許諾）を tier3 業界 pack に同梱する責務を明示。

### 役割 TS-B: SPA bridge
- SPA → sidecar: 既定経路は `ws://127.0.0.1:<port>/k1s0/v1/{usb, bluetooth, serial, opcua, modbus}`（TLS なし、配置節 (b) 経路）に JSON-RPC 2.0 で device 操作を送信、各 message は DPoP 署名 + BFF 発行 challenge の HMAC を含める。kiosk optional は `wss://127.0.0.1:<port>/...`
- sidecar → SPA: 非同期 device event を WebSocket message で push、DPoP 署名検証で偽装拒否
- 全 message は protobuf binary でも JSON でも可（SDK が wire を抽象化、proto FQN 等価原則と整合）
- PSK は SPA 側 JS から不可視（OS keystore に sidecar が保管、handshake は session cookie + DPoP のみ）。XSS 漏洩経路ゼロを規律化

### 役割 TS-C: SDK API 同型
- SPA SDK は browser native API（Chrome/Edge）と sidecar bridge（Firefox/Safari）を同 SDK 表面 API の adapter として持つ
- browser detect 後、SDK 内部で adapter 自動選択（SPA 業務コードは wire 種別を意識しない、Connect-RPC UA-aware adapter と同じ思想）

## v1_no_sidecar capability_class
- kiosk / BYOD / MDM 不可端末向けに「sidecar 配布不可」を明示する capability_class を新設
- 該当端末では device 機能は degraded mode（fallback ファイル選択 + サーバ側変換）で提供する旨を 22_端末オフラインデバイス が宣言
- `capability_matrix.lock.yaml` に `v1_no_sidecar` entry を追加、該当 tier3 機能は「v1_no_sidecar 環境では non-supports」と機械可読に明示

## MDM 配布パイプライン
- Backstage Software Template に「Tauri sidecar 配布申請」workflow を新設、tier3 ごとの sidecar 設定（device 種別 / pre-shared key / 自署 cert）を生成
- cosign 署名済 sidecar exe を Harbor から MDM service に push、MDM service が端末に install
- sidecar update は MDM の auto-update 機構で配布、cosign verify 後 install
- sidecar 自体の署名鍵は OpenBao Transit + HSM-backed（CI runner には short-lived signing token のみ）

## 副作用
- tier3 数 × OS 種別 = sidecar variant 数。Backstage で自動生成・自動署名・自動配布する規律で hand-roll を禁止
- MDM 不可端末は `v1_no_sidecar` 経路で degrade、UX は劣化するが機能セット自体は保たれる
- sidecar 自体の脆弱性管理は 10_security/08_脆弱性管理方針 のスキャン対象に Tauri / Rust crate を追加
- sidecar の API surface は Origin pin / DPoP 署名 / per-tenant PSK の HMAC（PSK は OS keystore 保管、SPA JS 不可視）の三段で防御、MITM / cross-tier3 attack / XSS 経由 PSK 漏洩を構造的に遮断。kiosk optional の (a) 経路では追加で TLS（自署 cert）

## 整合
- 整合 1: 22_端末オフラインデバイス の「Chrome/Edge のみ」記述を「直 API は Chrome/Edge、Firefox/Safari は Tauri sidecar bridge 経由（本ファイル）」に書き換え
- 整合 2: 06_UI 技術スタック のブラウザサポート節は本 sidecar により Firefox/Safari でも device 機能成立する旨を反映
- 整合 3: 30_互換性ポリシー のブラウザマトリクスは `v1_no_sidecar` capability_class を追加
- 整合 4: 38_製造業 pack 適用例 のバーコード / プリンタ記述は本 sidecar 経由で全ブラウザ成立
- 整合 5: 18_クライアント SDK 配布適合仕様 `capability_matrix.lock.yaml` に `v1_no_sidecar` 軸を追加し、機械可読化
- 整合 6: 13_時刻整合適合仕様（cross-axis double-bound）。Tauri Companion sidecar の transport adapter は `v1_application_hlc_only`（native rust 経路、`k1s0_hlc_lib` Rust 実装を直接 link）に分類し、TTL 判定 / deadline propagation / Idempotency-Key TTL の clock source を `CLOCK_MONOTONIC_RAW` 必須化。`SystemTime::now()` を idempotency / TTL に使う経路は Rust `#[deny]` lint で reject。WebSocket bridge 経由で host webview（Firefox / Safari）と通信する際は `v1_browser_client_skew_tolerant` 経路として server-anchor HLC token を opaque pass-through、host 側 SPA の wall-clock を idempotency 判定に使う path はゼロ。sidecar の wire 上 deadline は sender-issued HLC tuple + receiver-side HLC compare に固定し、wall-clock subtraction を禁止

## 関連参照
- [アプリケーション形態](../../03_概要設計/04_tier3設計方針/02_アプリケーション形態.md)
- [端末オフラインデバイス](../../03_概要設計/04_tier3設計方針/06_端末オフラインデバイス.md)
- [BFF auth-edge](06_BFF_auth_edge.md)
- [クライアント SDK 配布適合仕様](../01_適合仕様/18_クライアントSDK配布適合仕様.md)
- [時刻整合適合仕様](../01_適合仕様/13_時刻整合適合仕様.md)

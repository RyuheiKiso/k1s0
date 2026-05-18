---
id: arch.tier3.manufacturing_pack_application
axis: tier3
phase: architecture
kind: policy
status: published
version: 1.0.0
depends_on:
  - arch.tier3.tier3_index
  - arch.tier2.industry_extension_model
  - plan.target_use_case
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# tier3 製造業 pack 適用例

## 一文方針
- 1.0.0 ship の製造業 pack 適用例として、9 業務シナリオ（FA 生産指示 / 進捗実績 / 検査結果 / SCADA / 図面 review / 在庫 / 受注 / 警報配信 / 計量装置）+ 25h オフライン現場業務 + レガシー .NET Framework 並行運用 を tier3 として実装する。

## 9 業務シナリオ（v1.0.0 stress test 対象）

### FA ドメイン
1. **設備リモート操作**（v1_interactive、bidi 双方向）
2. **ライン稼働監視 live tile**（v1_live_snapshot、latest-wins）
3. **生産指示 / 進捗実績**（v1_event_feed、順序 + replay）
4. **SCADA テレメトリ収集**（v1_bulk_upload、at-least-once）
5. **計量装置連続データ**（v1_bulk_upload、continuous push）

### 調達ドメイン
6. **発注 / 検収**（v1_event_feed、業務 workflow）
7. **受注 sub（基幹 → 製造管理）**（v1_event_feed、順序 + replay）
8. **出荷指示双方向確認**（v1_interactive）

### 検査ドメイン
9. **品質検査結果配信**（v1_event_feed、順序 + replay）
10. **不良票 / 警報配信**（v1_alert、低 lag + replay）
11. **図面 collaborative review**（v1_interactive、双方向 presence）

### 在庫
12. **在庫最新値表示**（v1_live_snapshot）

## オフライン業務シナリオ
- 工場現場の検査担当が React SPA で 5 件オフライン記録 → 復帰
- 25h オフライン → Idempotency-Key TTL 超過 → user 確認 → 新 key で resend
- 4 layer state + IndexedDB encrypted で透過

## レガシー資産統合シナリオ
- レガシー .NET Framework 4.8 ERP が NuGet で `k1s0.Library.NetFx` を組込
- CLR Profiler で観測 attach、Companion 経由で送信
- HTTP/1.1 + SSE（v1_legacy_http11 専用 listener、別ポート 8443-legacy）
- per-tab 6 subscription で縮退

## device 機能
- バーコードリーダ / プリンタ / RFID / ラベル印刷
- Chromium 系: WebUSB 直 API
- Firefox / Safari: Tauri sidecar bridge

## 並行編集シナリオ（BusinessConflict subtype）
- stale_write: actor A 数量 / actor B 納期 → field-level rebase + auto resend
- lost_update: 同 field 並行編集 → 3-way merge UI
- supersede: 同 actor 後続 op → silent toast
- concurrent_edit: presence indicator → user choice

## tier3 実装の構成
- **発注 tier3**: 調達ドメイン UI（Web SPA）
- **検査 tier3**: 検査ドメイン UI（Web SPA + デスクトップ exe）
- **FA tier3**: FA ドメイン UI（Web SPA + デスクトップ exe）
- **legacy 連携 tier3**: ERP 並行運用（レガシー .NET Framework）

## 受入条件
- 9 業務シナリオ全 Playwright E2E green
- 25h オフライン best-effort 動作
- レガシー .NET Framework 4.8 ERP との統合動作
- WebUSB / Web Bluetooth / Web Serial の Chromium 直 + Tauri sidecar bridge 動作
- BusinessConflict subtype 4 種の UI 分岐動作

## 関連参照
- [tier3 設計方針 index](README.md)
- [アプリケーション形態](02_アプリケーション形態.md)
- [リアルタイム更新 UX](05_リアルタイム更新UX.md)
- [端末オフラインデバイス](06_端末オフラインデバイス.md)
- [レガシー資産統合](09_レガシー資産統合.md)
- [業務エラー UX](13_業務エラーUX.md)
- [ターゲットと利用シナリオ](../../01_企画/05_ターゲットと利用シナリオ/README.md)
- [業界 pack 戦略](../../01_企画/07_業界pack戦略/README.md)

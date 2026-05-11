---
id: req.functional.tier3_functional
axis: tier3
phase: requirement
kind: requirement
status: draft
depends_on:
  - req.functional.functional_index
  - arch.tier3.tier3_index
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# tier3 機能カテゴリ要件

## 一文方針
- tier3 は個別業務 UI を 3 アプリケーション形態（Web SPA / デスクトップ exe / レガシー .NET Framework）× 業務シナリオで機能要件として宣言する。4 layer client state + per-tab 16 subscription + offline 業務 + WCAG 2.1 AA + i18n を必須要件とする。

## 主要機能カテゴリ
- **3 アプリケーション形態**: Web SPA（TypeScript + React）/ デスクトップ exe（C# .NET 8 / Rust + Tauri）/ レガシー .NET Framework 4.6.2+ + WinForms / WPF
- **製造業 pack 9 業務シナリオ**: 設備リモート操作 / ライン稼働監視 / 品質検査結果配信 / SCADA テレメトリ / 図面 review / 在庫 / 受注 / 警報配信 / 計量装置
- **4 layer client state**: Server Truth / Optimistic Local / Pending Queue / Draft の決定論的衝突解決 tree
- **per-tab 16 subscription**: HTTP/2 multiplex
- **オフライン業務**: PWA installed + IndexedDB encrypted + 25h オフライン best-effort
- **device 機能**: WebUSB / Web Bluetooth / Web Serial（Chromium 直 + Firefox / Safari は Tauri sidecar bridge）
- **BFF auth-edge**: 1 tier3 ごとに 1 BFF Service、httpOnly cookie + refresh_token rotation
- **WCAG 2.1 AA**: axe-core で merge 阻止
- **Web Vitals**: Lighthouse CI で merge 阻止
- **i18n**: 1.0.0 で日本語 / 英語

## 詳細設計参照
- [tier3 設計方針](../../03_概要設計/04_tier3設計方針/README.md)（17 方針）
- [クライアント状態適合仕様](../../04_詳細設計/01_適合仕様/11_クライアント状態適合仕様.md)
- [tier3 強制機構](../../04_詳細設計/02_強制機構/03_tier3強制機構.md)（13 層）

## 受入条件
- 9 業務シナリオ全 Playwright E2E green
- WCAG 2.1 AA 違反ゼロ（axe-core）
- Web Vitals 閾値内（Lighthouse CI）
- 4 layer client state property test green
- BFF auth-edge / Tauri Companion sidecar 動作

## 関連参照
- [機能要件 index](README.md)
- [tier3 設計方針 index](../../03_概要設計/04_tier3設計方針/README.md)

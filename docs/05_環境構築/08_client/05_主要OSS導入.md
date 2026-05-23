---
id: env.client.client_oss_install
axis: client
phase: env_setup
kind: enforcement
status: draft
depends_on:
  - env.client.client_repository_acquisition
covered_by:
  defense_in_depth_layers: [B]
  proof_classes: []
---

# 主要 OSS 導入

## 一文方針

- client 軸エンジニアは Tauri CLI / Playwright / WiX Toolset を導入し、electron は代替用に留め推奨しない旨を明記した上で、5 distribution_class 全ての手元ビルドが通ることを本ページの検収条件とする。

> **pre-P0 注記**: `src/` は P10 deliverable（pre-P0 時点で実体ゼロ）。以下の手順は P10 完了後に有効。

## Tauri CLI（cargo install tauri-cli）

03_必須ランタイム でインストール済み。確認コマンド:

```bash
cargo tauri --version
```

Tauri v2.0 は WebView2（Windows）/ WebKitGTK（Linux）/ WKWebView（macOS）を使ってネイティブ UI を描画する。Electron との違いは Chromium をバンドルしないため、配布 artifact サイズが大幅に小さい。

## Playwright（pnpm add -D @playwright/test）

```bash
# プロジェクトルートで実行
pnpm add -D @playwright/test
pnpm playwright install --with-deps
pnpm playwright --version
```

browser binary のインストール先: `~/.cache/ms-playwright/`

## WiX Toolset（Windows インストーラ）

WiX Toolset は v1_full_native_with_companion と v1_thick_native_via_tauri の Windows インストーラ（.msi）を生成するために使う。Tauri v2.0 は WiX 4.x を使用する。

```bash
# Windows 側（PowerShell）でインストール
dotnet tool install --global wix
wix --version

# または winget
winget install WiXToolset.WiX
```

WSL2 から WiX は直接呼べない。Windows インストーラの生成は `cargo tauri build` が内部的に WiX を呼ぶため、Tauri のビルドは Windows ホスト側（または GitHub Actions の windows runner）で実行する。

## electron（代替用、非推奨）

**electron は非推奨**。v1_thick_native_via_tauri で Tauri が採用されているため、新規開発に electron を使わない。既存の electron アプリからの移行先として参照する場合のみインストールする。

```bash
# 移行調査目的のみ
pnpm add -D electron
```

electron を本番 distribution に使う場合は client 軸エンジニアとの合意が必要であり、release_gate の client cell が red になる。

## fast-check（Property-based testing 用）

v1_browser_spa_typescript の property-based test で使用する。

```bash
pnpm add -D fast-check
```

## 検収コマンド

```bash
cargo tauri --version
pnpm playwright --version
pnpm playwright show-browsers   # インストール済み browser 一覧
dotnet --version
```

## 関連参照

- [04_リポジトリ取得手順](04_リポジトリ取得手順.md)
- [06_開発エディタIDE設定](06_開発エディタIDE設定.md)
- [07_テスト検証環境](07_テスト検証環境.md)

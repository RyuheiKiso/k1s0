---
id: env.tier3.tier3_oss_install
axis: tier3
phase: env_setup
kind: enforcement
status: published
depends_on:
  - env.tier3.tier3_repository_acquisition
covered_by:
  defense_in_depth_layers: [B]
  proof_classes: []
---

# 主要 OSS 導入

## 一文方針

- Vite / React 19 / Storybook / Playwright / WiX Toolset v4.x / Tauri 2.0+ CLI を導入し、各ツールが起動することを本ページの検収とする。

## Vite + React 19

```bash
cd src/tier3/typescript/spa
pnpm add react@19 react-dom@19
pnpm add -D vite @vitejs/plugin-react typescript
pnpm exec vite --version
```

## Storybook

業務 UI コンポーネントのカタログ管理に使用する。

```bash
cd src/tier3/typescript/spa
pnpm exec storybook init
# または
pnpm add -D @storybook/react @storybook/react-vite
pnpm exec storybook --version
```

## Playwright（E2E テスト）

```bash
pnpm add -D @playwright/test
pnpm exec playwright install --with-deps
pnpm exec playwright --version
```

ブラウザは Chromium / Firefox / WebKit の 3 種をインストールする。

## Tauri CLI

```bash
cargo install tauri-cli
cargo tauri --version
```

または pnpm 経由:

```bash
pnpm add -D @tauri-apps/cli
pnpm exec tauri --version
```

## WiX Toolset（インストーラ生成）

Windows インストーラ（.msi）の生成に使用する。Windows 環境で実行する。

```bash
# Windows 側で実行
# WiX Toolset v4.x
dotnet tool install --global wix
wix --version
```

## Vitest（unit テスト）

```bash
pnpm add -D vitest @vitest/ui
pnpm exec vitest --version
```

## 検収コマンド

```bash
pnpm exec vite --version
pnpm exec playwright --version
pnpm exec vitest --version
cargo tauri --version
```

## 関連参照

- [04_リポジトリ取得手順](04_リポジトリ取得手順.md)
- [06_開発エディタIDE設定](06_開発エディタIDE設定.md)
- [07_テスト検証環境](07_テスト検証環境.md)

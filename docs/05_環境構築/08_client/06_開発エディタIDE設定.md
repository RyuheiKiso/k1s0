---
id: env.client.client_editor_ide
axis: client
phase: env_setup
kind: enforcement
status: draft
depends_on:
  - env.client.client_oss_install
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# 開発エディタ / IDE 設定

## 一文方針

- client 軸エンジニアは VS Code に Tauri extension / .NET extension / Playwright extension を導入し、5 distribution_class 全ての開発を手元で完結できる状態を検収条件とする。

## VS Code の必須 extension

| extension | ID | 用途 |
|---|---|---|
| rust-analyzer | `rust-lang.rust-analyzer` | Rust / Tauri コードの IntelliSense / 補完 |
| Tauri | `tauri-apps.tauri-vscode` | Tauri プロジェクト管理 / コマンドパレット |
| C# | `ms-dotnettools.csharp` | .NET 8 / C# IntelliSense |
| .NET MAUI | `ms-dotnettools.dotnet-maui` | クロスプラットフォーム .NET UI 開発 |
| Playwright | `ms-playwright.playwright` | Playwright test runner / デバッグ |
| ESLint | `dbaeumer.vscode-eslint` | TypeScript / JS lint |
| Prettier | `esbenp.prettier-vscode` | TypeScript / JS format |
| Remote - WSL | `ms-vscode-remote.remote-wsl` | WSL2 からの VS Code 起動 |

```bash
# CLI でインストール
code --install-extension rust-lang.rust-analyzer
code --install-extension tauri-apps.tauri-vscode
code --install-extension ms-dotnettools.csharp
code --install-extension ms-dotnettools.dotnet-maui
code --install-extension ms-playwright.playwright
code --install-extension dbaeumer.vscode-eslint
code --install-extension esbenp.prettier-vscode
code --install-extension ms-vscode-remote.remote-wsl
```

## VS Code settings.json 推奨設定

```json
{
  "editor.formatOnSave": true,
  "editor.defaultFormatter": "esbenp.prettier-vscode",
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  },
  "[csharp]": {
    "editor.defaultFormatter": "ms-dotnettools.csharp"
  },
  "rust-analyzer.cargo.features": "all",
  "playwright.reuseBrowser": false
}
```

## rust-analyzer の設定

Tauri プロジェクトでは `src-tauri/` 配下が Cargo プロジェクトになる。`rust-analyzer` が正しく動作するために workspace の root を認識させる必要がある。

```json
{
  "rust-analyzer.linkedProjects": [
    "./src-tauri/Cargo.toml"
  ]
}
```

## Playwright extension の活用

Playwright VS Code extension では以下が使える:

- Test Explorer でテストを GUI 実行
- 特定ブラウザを選択してデバッグ実行
- Playwright Inspector でセレクタ検証

## 検収コマンド

```bash
code --list-extensions | grep -E "rust-lang|tauri|dotnettools|playwright|eslint|prettier|remote-wsl"
```

6 件以上ヒットすることを確認する。

## 関連参照

- [05_主要OSS導入](05_主要OSS導入.md)
- [07_テスト検証環境](07_テスト検証環境.md)
- [08_lintとformat適用](08_lintとformat適用.md)

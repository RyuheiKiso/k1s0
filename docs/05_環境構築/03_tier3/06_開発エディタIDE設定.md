---
id: env.tier3.tier3_editor_ide
axis: tier3
phase: env_setup
kind: enforcement
status: draft
depends_on:
  - env.tier3.tier3_oss_install
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# 開発エディタ / IDE 設定

## 一文方針

- VS Code を主エディタとし、Tauri VS Code extension / .NET extension / XAML editor を導入して全 3 プラットフォーム（Browser / WPF / Tauri）の開発が可能な状態にすることを本ページの検収とする。

## VS Code 必須拡張

| 拡張 ID | 対象 | 役割 |
|---|---|---|
| `ms-vscode.vscode-typescript-next` | TypeScript / React | language server |
| `dbaeumer.vscode-eslint` | TypeScript | ESLint 統合 |
| `esbenp.prettier-vscode` | TypeScript | Prettier 統合 |
| `ms-dotnettools.csharp` | C# / WPF | .NET 8 LSP |
| `ms-dotnettools.vscode-dotnet-runtime` | .NET | .NET ランタイム管理 |
| `formulahendry.dotnet-test-explorer` | .NET | テスト UI |
| `rust-lang.rust-analyzer` | Rust / Tauri | LSP / cargo check 統合 |
| `tauri-apps.tauri-vscode` | Tauri | Tauri 固有補完 / コマンド統合 |
| `redhat.vscode-xml` | XAML | XAML スキーマ補完 |

### 一括インストールコマンド

```bash
code --install-extension ms-vscode.vscode-typescript-next
code --install-extension dbaeumer.vscode-eslint
code --install-extension esbenp.prettier-vscode
code --install-extension ms-dotnettools.csharp
code --install-extension ms-dotnettools.vscode-dotnet-runtime
code --install-extension formulahendry.dotnet-test-explorer
code --install-extension rust-lang.rust-analyzer
code --install-extension tauri-apps.tauri-vscode
code --install-extension redhat.vscode-xml
```

## workspace settings.json（推奨）

```json
{
  "[typescript]": {
    "editor.formatOnSave": true,
    "editor.defaultFormatter": "esbenp.prettier-vscode"
  },
  "[typescriptreact]": {
    "editor.formatOnSave": true,
    "editor.defaultFormatter": "esbenp.prettier-vscode"
  },
  "[csharp]": {
    "editor.formatOnSave": true
  },
  "[rust]": {
    "editor.formatOnSave": true,
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  },
  "editor.formatOnSave": true
}
```

## WPF / XAML 設計環境（Visual Studio 2022）

XAML デザイナを使用する場合は Visual Studio 2022（Windows 側）を別途インストールする。「.NET デスクトップ開発」ワークロードを選択すること。

## 検収コマンド

```bash
code --list-extensions | grep -E "typescript-next|vscode-eslint|prettier|csharp|rust-analyzer|tauri-vscode|vscode-xml"
```

## 関連参照

- [05_主要OSS導入](05_主要OSS導入.md)
- [07_テスト検証環境](07_テスト検証環境.md)

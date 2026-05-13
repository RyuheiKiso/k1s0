---
id: env.tier1.tier1_editor_ide
axis: tier1
phase: env_setup
kind: enforcement
status: draft
depends_on:
  - env.tier1.tier1_oss_install
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# 開発エディタ / IDE 設定

## 一文方針

- VS Code を主エディタとし、rust-analyzer / OmniSharp / gopls / TypeScript language server の 4 拡張を全てインストールして全言語でインテリセンスが動作することを本ページの検収とする。

## VS Code 必須拡張

| 拡張 ID | 対象言語 | 役割 |
|---|---|---|
| `rust-lang.rust-analyzer` | Rust | LSP / cargo check 統合 |
| `ms-dotnettools.csharp` (OmniSharp) | C# | .NET 8 LSP / Roslyn 統合 |
| `golang.go` (gopls) | Go | LSP / go vet 統合 |
| `ms-vscode.vscode-typescript-next` | TypeScript | tsc / language server |
| `zxh404.vscode-proto3` | proto3 | 文法ハイライト / Buf 連携 |

### 一括インストールコマンド

```bash
code --install-extension rust-lang.rust-analyzer
code --install-extension ms-dotnettools.csharp
code --install-extension golang.go
code --install-extension ms-vscode.vscode-typescript-next
code --install-extension zxh404.vscode-proto3
```

## workspace settings.json（推奨）

プロジェクトルートに `.vscode/settings.json` を配置する（git 管理対象外にする場合は `.gitignore` に追加）。

```json
{
  "rust-analyzer.cargo.allFeatures": true,
  "go.toolsManagement.autoUpdate": true,
  "typescript.tsdk": "./node_modules/typescript/lib",
  "[rust]": {
    "editor.formatOnSave": true,
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  },
  "[go]": {
    "editor.formatOnSave": true,
    "editor.defaultFormatter": "golang.go"
  },
  "[csharp]": {
    "editor.formatOnSave": true
  },
  "[typescript]": {
    "editor.formatOnSave": true,
    "editor.defaultFormatter": "ms-vscode.vscode-typescript-next"
  }
}
```

## WSL2 Remote 接続

VS Code を Windows 側で起動し、WSL2 Remote extension（`ms-vscode-remote.remote-wsl`）で接続する。

```bash
# WSL2 shell から直接起動
code .
```

## 検収コマンド

```bash
code --list-extensions | grep -E "rust-analyzer|csharp|golang|typescript|proto3"
# 5 拡張が全て表示されること
```

## 関連参照

- [05_主要OSS導入](05_主要OSS導入.md)
- [07_テスト検証環境](07_テスト検証環境.md)

---
id: env.tier2.tier2_editor_ide
axis: tier2
phase: env_setup
kind: enforcement
status: draft
depends_on:
  - env.tier2.tier2_oss_install
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# 開発エディタ / IDE 設定

## 一文方針

- VS Code を主エディタとし、rust-analyzer / OmniSharp / gopls / TypeScript language server に加え、SQL / BPMN / Avro スキーマ編集拡張を導入して全業務資産が編集できることを本ページの検収とする。

## VS Code 必須拡張

| 拡張 ID | 対象 | 役割 |
|---|---|---|
| `rust-lang.rust-analyzer` | Rust | LSP / cargo check 統合 |
| `ms-dotnettools.csharp` | C# | .NET 8 LSP |
| `golang.go` | Go | LSP / go vet 統合 |
| `ms-vscode.vscode-typescript-next` | TypeScript | language server |
| `mtxr.sqltools` | SQL | PostgreSQL クエリ・スキーマ確認 |
| `mtxr.sqltools-driver-pg` | PostgreSQL | SQLTools PostgreSQL ドライバ |
| `redhat.vscode-yaml` | YAML | Avro / workflow YAML スキーマ検証 |
| `bpmnio.bpmn` | BPMN | workflow 図の編集 |

### 一括インストールコマンド

```bash
code --install-extension rust-lang.rust-analyzer
code --install-extension ms-dotnettools.csharp
code --install-extension golang.go
code --install-extension ms-vscode.vscode-typescript-next
code --install-extension mtxr.sqltools
code --install-extension mtxr.sqltools-driver-pg
code --install-extension redhat.vscode-yaml
code --install-extension bpmnio.bpmn
```

## SQLTools 接続設定

ローカル postgres（docker compose 起動）への接続設定を `.vscode/settings.json` に追加する。

```json
{
  "sqltools.connections": [
    {
      "name": "local-postgres",
      "driver": "PostgreSQL",
      "server": "localhost",
      "port": 5432,
      "database": "tier2_dev",
      "username": "postgres",
      "password": "postgres"
    }
  ]
}
```

## 検収コマンド

```bash
code --list-extensions | grep -E "rust-analyzer|csharp|golang|typescript|sqltools|yaml|bpmn"
```

## 関連参照

- [05_主要OSS導入](05_主要OSS導入.md)
- [07_テスト検証環境](07_テスト検証環境.md)

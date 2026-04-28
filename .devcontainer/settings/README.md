# `.devcontainer/settings/` — VS Code 設定共有レイヤ

本ディレクトリは [`docs/05_実装/50_開発者体験設計/10_DevContainer_10役/01_DevContainer_10役設計.md`](../../docs/05_実装/50_開発者体験設計/10_DevContainer_10役/01_DevContainer_10役設計.md) の §VS Code 設定共有（IMP-DEV-DC-013）に対応する設定ファイル群を保持する。役割別 Dev Container は本ディレクトリの common と役割別ファイルを merge して使用する。

## 構成

```text
settings/
├── README.md                          # 本ファイル
├── common.settings.json               # 全役割共通設定
├── extensions.common.json             # 全役割共通の VS Code 推奨拡張
├── tier1-rust-dev.settings.json       # 以下、役割別の上書き
├── tier1-rust-dev.extensions.json
├── tier1-go-dev.settings.json
├── tier1-go-dev.extensions.json
├── tier2-dev.settings.json
├── tier2-dev.extensions.json
├── tier3-web-dev.settings.json
├── tier3-web-dev.extensions.json
├── tier3-native-dev.settings.json
├── tier3-native-dev.extensions.json
├── platform-cli-dev.settings.json
├── platform-cli-dev.extensions.json
├── sdk-dev.settings.json
├── sdk-dev.extensions.json
├── infra-ops.settings.json
├── infra-ops.extensions.json
├── docs-writer.settings.json
├── docs-writer.extensions.json
├── full.settings.json
└── full.extensions.json
```

## merge 規則

各プロファイルの `devcontainer.json` は次のパターンで本ディレクトリを参照する。

```jsonc
{
    "customizations": {
        "vscode": {
            "settings": "${localWorkspaceFolder}/.devcontainer/settings/<role>.settings.json",
            "extensions": "${localWorkspaceFolder}/.devcontainer/settings/<role>.extensions.json"
        }
    }
}
```

実体としては VS Code Dev Containers 拡張がプロファイルの `customizations.vscode.settings` を起動時に読み込むため、本ディレクトリの JSON は `devcontainer.json` 側で `"customizations.vscode.settings"` フィールドへ展開する形で取り込む（`include` 構文が無いため、各プロファイルの `devcontainer.json` で実体を埋め込み、内容の出所を本ディレクトリに置く位置付け）。共通レイヤ（`common.settings.json` / `extensions.common.json`）は各プロファイルでも展開され、`<role>.settings.json` の値が衝突キーで勝つ。

個人開発者の上書きは `.vscode/settings.local.json`（gitignore 対象）に分離する。これにより共有設定が個人都合で汚染されない。

## 関連

- 設計書: [`01_DevContainer_10役設計.md`](../../docs/05_実装/50_開発者体験設計/10_DevContainer_10役/01_DevContainer_10役設計.md) §VS Code 設定共有
- IMP-DEV-DC-013: VS Code 設定共有（`.devcontainer/settings/` の common + role 分離）

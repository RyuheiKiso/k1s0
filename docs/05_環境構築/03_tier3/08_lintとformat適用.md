---
id: env.tier3.tier3_lint_format
axis: tier3
phase: env_setup
kind: enforcement
status: published
depends_on:
  - env.tier3.tier3_test_environment
covered_by:
  defense_in_depth_layers: [B]
  proof_classes: []
---

# lint と format 適用

## 一文方針

- `pnpm eslint + prettier` / `dotnet format` / `cargo clippy` の全ツールが exit 0 になることを push 前の必須確認事項とする。

## TypeScript / React: ESLint + Prettier

```bash
cd src/tier3/typescript/spa
pnpm eslint 'src/**/*.{ts,tsx}' --max-warnings 0
pnpm prettier --check 'src/**/*.{ts,tsx}'
# 自動修正
pnpm prettier --write 'src/**/*.{ts,tsx}'
```

`.eslintrc.json` に React / TypeScript ルールを設定する。

```json
{
  "extends": [
    "eslint:recommended",
    "plugin:@typescript-eslint/recommended",
    "plugin:react-hooks/recommended"
  ]
}
```

## C# / WPF: dotnet format

```bash
cd src/tier3/wpf
dotnet format --verify-no-changes
```

`.editorconfig` の設定に従ってフォーマットを適用する。

## Rust / Tauri: clippy + fmt

```bash
cd src/tier3/tauri
cargo clippy -- -D warnings
cargo fmt -- --check
```

Tauri 固有の lint 警告も `-D warnings` で全てエラーとして扱う。

## 設定ファイルの確認

| ツール | 設定ファイル |
|---|---|
| ESLint | `.eslintrc.json` または `eslint.config.js` |
| Prettier | `.prettierrc.json` |
| dotnet format | `.editorconfig` |
| rustfmt | `rustfmt.toml` |
| clippy | `.clippy.toml` |

## 検収コマンド

```bash
pnpm eslint 'src/**/*.{ts,tsx}' --max-warnings 0
pnpm prettier --check 'src/**/*.{ts,tsx}'
dotnet format --verify-no-changes
cargo clippy -- -D warnings
cargo fmt -- --check
```

## 関連参照

- [07_テスト検証環境](07_テスト検証環境.md)
- [09_docs_lint実行手順](09_docs_lint実行手順.md)

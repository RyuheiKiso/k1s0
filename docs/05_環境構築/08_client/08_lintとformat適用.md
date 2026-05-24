---
id: env.client.client_lint_format
axis: client
phase: env_setup
kind: enforcement
status: draft
depends_on:
  - env.client.client_test_environment
covered_by:
  defense_in_depth_layers: [B]
  proof_classes: []
---

# lint と format 適用

## 一文方針

- client 軸エンジニアは `pnpm eslint + prettier` / `cargo clippy + fmt` / `dotnet format` / Playwright lint config の 4 lint を手元で実行できることを本ページの検収条件とする。

> **pre-P0 注記**: `src/` は P10 deliverable（pre-P0 時点で実体ゼロ）。以下の lint / format 手順は P10 完了後に有効。

## pnpm ESLint + Prettier（TypeScript）

```bash
# ESLint インストール（TypeScript プロジェクト）
pnpm add -D eslint @typescript-eslint/parser @typescript-eslint/eslint-plugin prettier eslint-config-prettier

# lint 実行
pnpm eslint src/ --ext .ts,.tsx
pnpm prettier --check src/

# format 実行（自動修正）
pnpm prettier --write src/
pnpm eslint src/ --ext .ts,.tsx --fix
```

`.eslintrc.json` 最小構成:

```json
{
  "parser": "@typescript-eslint/parser",
  "extends": [
    "eslint:recommended",
    "plugin:@typescript-eslint/recommended",
    "prettier"
  ],
  "plugins": ["@typescript-eslint"],
  "parserOptions": { "ecmaVersion": 2022, "sourceType": "module" }
}
```

## cargo clippy + fmt（Rust / Tauri）

```bash
# clippy（lint）
cargo clippy --all-targets --all-features -- -D warnings

# fmt（format チェック）
cargo fmt -- --check

# format 実行（自動修正）
cargo fmt
```

`-D warnings` で warning を error に昇格させ、CI と同条件にする。

## dotnet format（C# / .NET 8）

```bash
# format チェック
dotnet format --verify-no-changes

# format 実行（自動修正）
dotnet format
```

.editorconfig で C# のコードスタイルを統一する。

```ini
[*.cs]
indent_size = 4
charset = utf-8-bom
end_of_line = crlf
```

## Playwright lint config

Playwright のテストファイルは TypeScript で書くため、ESLint で品質を維持する。

```bash
# Playwright 専用 ESLint rule
pnpm add -D eslint-plugin-playwright

# .eslintrc の extends に追加
# "plugin:playwright/recommended"
pnpm eslint tests/ --ext .ts
```

## 検収コマンド

```bash
# TypeScript
pnpm eslint --version
pnpm prettier --version

# Rust
cargo clippy --version
cargo fmt --version

# .NET
dotnet format --version
```

## 関連参照

- [07_テスト検証環境](07_テスト検証環境.md)
- [09_docs_lint実行手順](09_docs_lint実行手順.md)
- [11_軸固有環境設定](11_軸固有環境設定.md)

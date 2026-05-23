---
id: env.tier1.tier1_lint_format
axis: tier1
phase: env_setup
kind: enforcement
status: draft
depends_on:
  - env.tier1.tier1_test_environment
covered_by:
  defense_in_depth_layers: [B]
  proof_classes: []
---

# lint と format 適用

## 一文方針

- `cargo clippy` / `cargo fmt` / `dotnet format` / `go vet + gofmt` / `pnpm eslint + prettier` の全ツールが exit 0 になることを push 前の必須確認事項とする。

> **pre-P0 注記**: `src/` は P10 deliverable（pre-P0 時点で実体ゼロ）。以下の lint / format 手順は P10 完了後に有効。

## Rust: clippy + fmt

```bash
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
```

`-D warnings` により警告をエラーとして扱う。`--check` は差分がある場合に exit 1 を返す（自動修正は `cargo fmt --all`）。

## Go: vet + gofmt

```bash
go vet ./...
gofmt -l .
# 差分があるファイルが表示された場合は
gofmt -w .
```

`go vet` は静的解析。`gofmt` はフォーマット差分の確認と自動修正。

## C#: dotnet format

```bash
dotnet format --verify-no-changes
# 自動修正する場合
dotnet format
```

`.editorconfig` の設定に従ってフォーマットを適用する。

## TypeScript: ESLint + Prettier

```bash
pnpm eslint 'src/**/*.ts' --max-warnings 0
pnpm prettier --check 'src/**/*.ts'
# 自動修正
pnpm prettier --write 'src/**/*.ts'
```

`--max-warnings 0` により警告をエラーとして扱う。

## 設定ファイルの確認

| ツール | 設定ファイル |
|---|---|
| clippy | `.clippy.toml` または `Cargo.toml` の `[lints]` |
| rustfmt | `rustfmt.toml` |
| go vet | なし（デフォルト） |
| dotnet format | `.editorconfig` |
| ESLint | `.eslintrc.json` または `eslint.config.js` |
| Prettier | `.prettierrc.json` |

## CI との対応

CI lint job では上記コマンドを順次実行する。差分が出ないことを push 前に確認すること。

## 検収コマンド

```bash
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
go vet ./...
dotnet format --verify-no-changes
pnpm eslint 'src/**/*.ts' --max-warnings 0
```

全コマンドが exit 0 であることを確認する。

## 関連参照

- [07_テスト検証環境](07_テスト検証環境.md)
- [09_docs_lint実行手順](09_docs_lint実行手順.md)

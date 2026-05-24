---
id: env.tier2.tier2_lint_format
axis: tier2
phase: env_setup
kind: enforcement
status: draft
depends_on:
  - env.tier2.tier2_test_environment
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

## Go: vet + gofmt

```bash
go vet ./...
gofmt -l .
```

差分がある場合は `gofmt -w .` で自動修正する。

## C#: dotnet format

```bash
dotnet format --verify-no-changes
```

`.editorconfig` の設定に従ってフォーマットを適用する。

## TypeScript: ESLint + Prettier

```bash
pnpm eslint 'src/**/*.ts' --max-warnings 0
pnpm prettier --check 'src/**/*.ts'
```

## 決定表 lint

BPMN / DMN 決定表ファイルの lint を実行する。

```bash
bpmnlint src/tier2/**/*.bpmn
```

## SQL マイグレーションファイルの整合確認

```bash
# sqlx の場合: マイグレーションファイルのチェックサム検証
sqlx migrate info --database-url postgres://postgres:postgres@localhost:5432/tier2_dev
```

## 検収コマンド

```bash
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
go vet ./...
dotnet format --verify-no-changes
pnpm eslint 'src/**/*.ts' --max-warnings 0
```

## 関連参照

- [07_テスト検証環境](07_テスト検証環境.md)
- [09_docs_lint実行手順](09_docs_lint実行手順.md)

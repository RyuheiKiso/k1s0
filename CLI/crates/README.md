# CLI Crates

k1s0 CLI を構成する Rust crate 群。

## Crate 一覧

| Crate | 説明 |
|-------|------|
| `k1s0-cli` | 実行 CLI（clap ベース） |
| `k1s0-generator` | テンプレート展開・差分適用ロジック |

## 依存関係

```
k1s0-cli
  └── k1s0-generator
```

## ビルド

```bash
cd CLI
cargo build --workspace
```

# k1s0 CLI

k1s0 の雛形生成・導入・アップグレードを担う CLI ツール。

## ディレクトリ構成

```
CLI/
├── crates/
│   ├── k1s0-cli/           # 実行 CLI (clap)
│   │   └── src/
│   │       ├── commands/   # サブコマンド実装
│   │       ├── main.rs
│   │       └── lib.rs
│   └── k1s0-generator/     # テンプレ展開・差分適用ロジック
│       └── src/
│           ├── renderer/
│           ├── diff/
│           └── lib.rs
└── templates/              # 生成テンプレ群
    ├── backend-rust/
    ├── backend-go/
    ├── frontend-react/
    └── frontend-flutter/
```

## コマンド一覧

| コマンド | 説明 |
|---------|------|
| `k1s0 init` | リポジトリ初期化（`.k1s0/` 作成等） |
| `k1s0 new-feature` | 新規サービスの雛形生成 |
| `k1s0 lint` | 規約違反の検査 |
| `k1s0 upgrade --check` | 差分提示と衝突検知 |
| `k1s0 upgrade` | テンプレート更新の適用 |

## 開発

```bash
# ビルド
cd CLI
cargo build

# 実行
cargo run -- --help
```

## 関連ドキュメント

- [プラン.md](../work/プラン.md): CLI 実装計画（フェーズ3〜5, 11〜13, 32〜33）

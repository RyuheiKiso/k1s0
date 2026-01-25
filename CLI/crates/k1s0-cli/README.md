# k1s0-cli

k1s0 CLI の実行バイナリ。clap を使用したコマンドライン引数解析。

## サブコマンド

- `init`: リポジトリ初期化
- `new-feature`: 新規サービスの雛形生成
- `lint`: 規約違反の検査
- `upgrade`: テンプレート更新

## ディレクトリ構成

```
k1s0-cli/
└── src/
    ├── commands/
    │   ├── init.rs
    │   ├── new_feature.rs
    │   ├── lint.rs
    │   ├── upgrade.rs
    │   └── mod.rs
    ├── main.rs
    └── lib.rs
```

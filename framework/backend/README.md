# Framework Backend

バックエンド共通部品（crate/ライブラリ）および共通マイクロサービス。

## ディレクトリ構成

```
backend/
├── rust/
│   ├── crates/       # 共通 crate 群
│   └── services/     # 共通マイクロサービス
└── go/
    └── (同様の構成)
```

## Rust

- 共通 crate: `crates/` 配下
- 共通サービス: `services/` 配下

## Go

置き場のみ固定。実装は後続フェーズで行う。

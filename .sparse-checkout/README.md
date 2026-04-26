# `.sparse-checkout/` — 役割別 cone 定義

本ディレクトリは [`docs/05_実装/00_ディレクトリ設計/80_スパースチェックアウト運用/02_役割別cone定義.md`](../docs/05_実装/00_ディレクトリ設計/80_スパースチェックアウト運用/02_役割別cone定義.md) で定義された 10 役の cone 定義を `.txt` で保持する。`tools/sparse/checkout-role.sh <role>` から参照される。

## ファイル一覧

```
.sparse-checkout/
├── README.md           # 本ファイル
└── roles/
    ├── tier1-rust-dev.txt
    ├── tier1-go-dev.txt
    ├── tier2-dev.txt
    ├── tier3-web-dev.txt
    ├── tier3-native-dev.txt
    ├── platform-cli-dev.txt
    ├── sdk-dev.txt
    ├── infra-ops.txt
    ├── docs-writer.txt
    └── full.txt
```

## 利用

```bash
# 役割切替
./tools/sparse/checkout-role.sh tier1-rust-dev

# 兼任（マージ）
./tools/sparse/checkout-role.sh -m tier1-rust-dev,tier2-dev

# 検証のみ
./tools/sparse/checkout-role.sh tier1-rust-dev --verify
```

## 関連

- 設計: [`02_役割別cone定義.md`](../docs/05_実装/00_ディレクトリ設計/80_スパースチェックアウト運用/02_役割別cone定義.md)
- 切替運用: [`04_役割切替運用.md`](../docs/05_実装/00_ディレクトリ設計/80_スパースチェックアウト運用/04_役割切替運用.md)
- IMP-DIR-SPARSE-127

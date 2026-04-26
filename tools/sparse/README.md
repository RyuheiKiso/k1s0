# `tools/sparse/` — Sparse-checkout 役割切替ツール

本ディレクトリは [`docs/05_実装/00_ディレクトリ設計/80_スパースチェックアウト運用/`](../../docs/05_実装/00_ディレクトリ設計/80_スパースチェックアウト運用/) の運用設計を実装する。

## 配置

```
tools/sparse/
├── README.md            # 本ファイル
└── checkout-role.sh     # 役割切替の実体スクリプト
```

役割定義の cone は [`/.sparse-checkout/roles/<role>.txt`](../../.sparse-checkout/roles/) に保持される。

## 利用

```bash
# 役割切替（cone 設定 + .devcontainer/devcontainer.json symlink 張替え）
./tools/sparse/checkout-role.sh tier1-rust-dev

# 兼任（カンマ区切りで複数 role をマージ。symlink は先頭 role）
./tools/sparse/checkout-role.sh -m tier1-rust-dev,tier2-dev

# 検証（変更せず現在の整合のみ確認）
./tools/sparse/checkout-role.sh tier1-rust-dev --verify

# dry-run（変更内容を表示のみ）
./tools/sparse/checkout-role.sh tier1-rust-dev --dry-run

# 利用可能 role 一覧
./tools/sparse/checkout-role.sh --list
```

切替後に Dev Container を再起動（VS Code の "Rebuild Container"）すると、新しい役割の image でビルドされる。

## 関連

- 設計: [`docs/05_実装/00_ディレクトリ設計/80_スパースチェックアウト運用/02_役割別cone定義.md`](../../docs/05_実装/00_ディレクトリ設計/80_スパースチェックアウト運用/02_役割別cone定義.md)
- 切替運用: [`docs/05_実装/00_ディレクトリ設計/80_スパースチェックアウト運用/04_役割切替運用.md`](../../docs/05_実装/00_ディレクトリ設計/80_スパースチェックアウト運用/04_役割切替運用.md)
- 10 役 Dev Container: [`docs/05_実装/50_開発者体験設計/10_DevContainer_10役/01_DevContainer_10役設計.md`](../../docs/05_実装/50_開発者体験設計/10_DevContainer_10役/01_DevContainer_10役設計.md)
- IMP ID: IMP-DIR-SPARSE-127 / IMP-DEV-DC-010

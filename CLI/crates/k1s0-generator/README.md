# k1s0-generator

テンプレート展開・差分適用ロジックを提供する crate。

## 機能

- テンプレートのレンダリング（変数展開）
- ファイル生成・上書き・衝突検知
- 差分計算と表示
- fingerprint 計算

## ディレクトリ構成

```
k1s0-generator/
└── src/
    ├── renderer/     # テンプレートレンダリング
    ├── diff/         # 差分計算・表示
    ├── fingerprint/  # fingerprint 算出
    └── lib.rs
```
